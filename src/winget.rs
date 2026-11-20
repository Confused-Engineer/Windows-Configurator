//! winget (Windows Package Manager): detection, catalog, installs, and
//! bootstrapping winget itself on machines without the Microsoft Store.

use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::platform::{self, CREATE_NO_WINDOW, Window};
use crate::tasks::{Bus, Event};

/// Flags that keep winget from stopping at a license/agreement prompt.
const ACCEPT: &str = "--accept-source-agreements --accept-package-agreements";

/// PowerShell that installs winget from PSGallery, for debloated Windows
/// installs without the Store or a browser. Runs elevated.
pub const BOOTSTRAP_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$Host.UI.RawUI.WindowTitle = 'Installing winget'
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    Write-Host 'Installing the NuGet package provider...'
    Install-PackageProvider -Name NuGet -MinimumVersion 2.8.5.201 -Force | Out-Null
    Write-Host 'Installing the Microsoft.WinGet.Client module from PSGallery...'
    Install-Module -Name Microsoft.WinGet.Client -Force -Repository PSGallery -Scope AllUsers -AllowClobber | Out-Null
    Import-Module Microsoft.WinGet.Client -Force
    Write-Host 'Installing / repairing winget for all users (this can take a few minutes)...'
    Repair-WinGetPackageManager -AllUsers -Latest -Force
    Write-Host ''
    Write-Host 'winget is installed.' -ForegroundColor Green
    Start-Sleep -Seconds 3
} catch {
    Write-Host ''
    Write-Host "Installing winget failed: $_" -ForegroundColor Red
    Read-Host 'Press Enter to close'
    exit 1
}
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub id: String,
    pub version: String,
    /// Lower-case "name id", precomputed for searching.
    search_key: String,
}

impl Package {
    pub fn new(name: &str, id: &str, version: &str) -> Self {
        Self {
            search_key: format!("{name} {id}").to_lowercase(),
            name: name.to_owned(),
            id: id.to_owned(),
            version: version.to_owned(),
        }
    }

    /// `needle` must already be lower-case.
    pub fn matches(&self, needle: &str) -> bool {
        self.search_key.contains(needle)
    }
}

/// Finds `winget.exe`. Besides `PATH`, checks the per-user App Execution
/// Alias folder directly, since it is missing from `PATH` in some sessions
/// (elevated shells, freshly bootstrapped installs).
pub fn locate() -> Option<PathBuf> {
    // App execution aliases are reparse points that `metadata()` cannot
    // follow, so test for existence with `symlink_metadata`.
    let exists = |p: &PathBuf| std::fs::symlink_metadata(p).is_ok();
    let from_path = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .map(|dir| dir.join("winget.exe"));
    let alias = std::env::var_os("LOCALAPPDATA")
        .map(|d| PathBuf::from(d).join(r"Microsoft\WindowsApps\winget.exe"));
    from_path.chain(alias).find(exists)
}

fn command() -> Command {
    let mut cmd = Command::new(locate().unwrap_or_else(|| "winget.exe".into()));
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// `winget --version`, or `None` when winget is not usable.
pub fn version() -> Option<String> {
    let out = command().arg("--version").output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// Downloads the full catalog (`winget search` with an empty query).
pub fn fetch_catalog() -> Result<Vec<Package>, String> {
    let out = command()
        .args(["search", "--query", "", "--source", "winget"])
        .args(["--accept-source-agreements", "--disable-interactivity"])
        .output()
        .map_err(|e| format!("could not run winget: {e}"))?;
    let packages = parse_catalog(&String::from_utf8_lossy(&out.stdout));
    if packages.is_empty() {
        Err(format!(
            "winget returned no packages (exit code {:?})",
            out.status.code()
        ))
    } else {
        Ok(packages)
    }
}

/// Parses `winget search` table output.
///
/// The Name column may contain spaces but Id, Version and Source never do,
/// so rows are split from the right. Column widths are not used because
/// winget pads by display width, which breaks char-index slicing for CJK names.
pub fn parse_catalog(text: &str) -> Vec<Package> {
    let mut lines = text.lines().map(|l| l.rsplit('\r').next().unwrap_or(l));
    // Everything before the dashed rule (spinner, header) is skipped.
    let Some(header) = lines.by_ref().take_while(|l| !l.starts_with("---")).last() else {
        return Vec::new();
    };
    let columns = header.split_whitespace().count();

    lines
        .filter_map(|line| {
            // Peel `columns - 1` whitespace-free fields off the right:
            // Source, [Match,] Version, Id. What is left is the Name.
            let mut rest = line.trim_end();
            let mut fields = Vec::with_capacity(columns);
            for _ in 1..columns {
                let (left, right) = rest.rsplit_once(char::is_whitespace)?;
                fields.push(right);
                rest = left.trim_end();
            }
            let [.., version, id] = fields[..] else {
                return None;
            };
            let name = rest.trim();
            (!name.is_empty()).then(|| Package::new(name, id, version))
        })
        .collect()
}

/// What to install and with which extra `winget install` arguments
/// (e.g. `--scope machine --installer-type wix`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstallRequest {
    pub name: String,
    pub id: String,
    pub args: Vec<String>,
}

impl InstallRequest {
    pub fn new(name: &str, id: &str) -> Self {
        Self {
            name: name.to_owned(),
            id: id.to_owned(),
            args: Vec::new(),
        }
    }

    /// Parses a favourite from config: `Name = Winget.Id [extra args…]`.
    pub fn from_config(name: &str, value: &str) -> Result<Self, String> {
        let mut words = split_args(value)?.into_iter();
        let id = words.next().ok_or("missing winget id")?;
        Ok(Self {
            name: name.to_owned(),
            id,
            args: words.collect(),
        })
    }

    /// The `winget install …` arguments after the executable.
    fn arguments(&self) -> String {
        let mut line = format!("install --exact --id {} {ACCEPT} --silent", quote(&self.id));
        for arg in &self.args {
            line.push(' ');
            line.push_str(&quote(arg));
        }
        line
    }

    /// Human-readable command, for tooltips and the options dialog.
    pub fn display(&self) -> String {
        format!("winget {}", self.arguments())
    }
}

/// Characters that cannot be passed safely through `cmd.exe`, even quoted.
const FORBIDDEN: &[char] = &['"', '%', '\r', '\n'];

/// Splits an argument string on whitespace, honouring "double quotes".
/// Rejects characters that could break out of the `cmd.exe` line.
pub fn split_args(text: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut has_token = false;
    for c in text.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                has_token = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if has_token {
                    args.push(std::mem::take(&mut current));
                    has_token = false;
                }
            }
            '%' => return Err("'%' is not allowed in winget arguments".into()),
            c => {
                current.push(c);
                has_token = true;
            }
        }
    }
    if in_quotes {
        return Err("unbalanced quote".into());
    }
    if has_token {
        args.push(current);
    }
    Ok(args)
}

fn quote(arg: &str) -> String {
    debug_assert!(!arg.contains(FORBIDDEN));
    if !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.:/\\=+".contains(c))
    {
        return arg.to_owned();
    }
    // Trailing backslashes would escape the closing quote (`"C:\dir\"`).
    let trailing = arg.len() - arg.trim_end_matches('\\').len();
    format!("\"{arg}{}\"", "\\".repeat(trailing))
}

fn valid(request: &InstallRequest) -> Result<(), String> {
    std::iter::once(&request.id)
        .chain(&request.args)
        .find(|a| a.contains(FORBIDDEN) || a.is_empty())
        .map_or(Ok(()), |a| Err(format!("invalid argument {a:?}")))
}

/// Installs a package by exact id. `Window::Visible` shows winget's progress
/// in a console (kept open on failure); `Window::Hidden` reports the result
/// in the status bar.
pub fn install(bus: &Bus, request: &InstallRequest, window: Window) {
    if let Err(e) = valid(request) {
        bus.status(format!("Not installing {}: {e}", request.name));
        return;
    }
    let exe = locate().unwrap_or_else(|| "winget.exe".into());
    let line = format!("\"{}\" {}", exe.display(), request.arguments());
    match window {
        Window::Visible => {
            if let Err(e) = platform::cmd(&format!("{line} || pause"), Window::Visible) {
                bus.status(format!("Could not start winget: {e}"));
            }
        }
        Window::Hidden => {
            let (name, line) = (
                request.name.clone(),
                format!("{line} --disable-interactivity"),
            );
            bus.status(format!("Installing {name}…"));
            bus.spawn("winget-install", move |bus| {
                let result = platform::cmd(&line, Window::Hidden).and_then(|mut c| c.wait());
                bus.status(match result {
                    Ok(s) if s.success() => format!("Installed {name}"),
                    Ok(s) => format!(
                        "Installing {name} failed (winget exit code {})",
                        s.code().unwrap_or(-1)
                    ),
                    Err(e) => format!("Installing {name} failed: {e}"),
                });
            });
        }
    }
}

/// Which version/installer `winget install` will actually pick for a request.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resolution {
    pub version: String,
    pub installer_type: Option<String>,
    pub url: Option<String>,
    /// The newest version, when it differs from `version` (it had no
    /// installer matching the selected scope / type / architecture).
    pub newer_skipped: Option<String>,
}

/// How many versions (newest first) to try before giving up.
const RESOLVE_DEPTH: usize = 25;

/// `winget install` arguments that also narrow `winget show`.
const SELECTORS: [&str; 6] = [
    "--scope",
    "--installer-type",
    "--architecture",
    "-a",
    "--locale",
    "--version",
];

/// Works out which version and installer `winget install` will use for
/// `request`. winget silently falls back to the newest version that has a
/// matching installer, which `winget show` alone does not do, so versions are
/// tried newest-first.
pub fn resolve(request: &InstallRequest) -> Result<Resolution, String> {
    let mut selectors = Vec::new();
    let mut pinned = None;
    let mut args = request.args.iter();
    while let Some(arg) = args.next() {
        if SELECTORS.contains(&arg.as_str())
            && let Some(value) = args.next()
        {
            if arg == "--version" {
                pinned = Some(value.clone());
            } else {
                selectors.extend([arg.clone(), value.clone()]);
            }
        }
    }

    let versions = match pinned {
        Some(v) => vec![v],
        None => list_versions(&request.id)?,
    };
    let newest = versions.first().cloned();
    for version in versions.iter().take(RESOLVE_DEPTH) {
        if let Some(mut found) = show(&request.id, version, &selectors)? {
            found.newer_skipped = newest.filter(|n| *n != found.version);
            return Ok(found);
        }
    }
    Err(if versions.len() == 1 {
        "that version has no installer matching these options".to_owned()
    } else {
        format!(
            "none of the newest {} versions has an installer matching these options",
            versions.len().min(RESOLVE_DEPTH)
        )
    })
}

fn list_versions(id: &str) -> Result<Vec<String>, String> {
    let out = command()
        .args(["show", "--exact", "--id", id, "--versions"])
        .args(["--accept-source-agreements", "--disable-interactivity"])
        .output()
        .map_err(|e| format!("could not run winget: {e}"))?;
    let versions = parse_versions(&String::from_utf8_lossy(&out.stdout));
    if versions.is_empty() {
        Err(format!("no package with id {id}"))
    } else {
        Ok(versions)
    }
}

fn parse_versions(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .skip_while(|l| !l.starts_with("---"))
        .skip(1)
        .take_while(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

fn show(id: &str, version: &str, selectors: &[String]) -> Result<Option<Resolution>, String> {
    let out = command()
        .args(["show", "--exact", "--id", id, "--version", version])
        .args(selectors)
        .args(["--accept-source-agreements", "--disable-interactivity"])
        .output()
        .map_err(|e| format!("could not run winget: {e}"))?;
    Ok(parse_show(&String::from_utf8_lossy(&out.stdout)))
}

fn parse_show(text: &str) -> Option<Resolution> {
    if text.contains("No applicable installer") {
        return None;
    }
    let field = |name: &str| {
        text.lines()
            .find_map(|l| l.trim().strip_prefix(name).map(|v| v.trim().to_owned()))
    };
    Some(Resolution {
        version: field("Version:")?,
        installer_type: field("Installer Type:"),
        url: field("Installer Url:"),
        newer_skipped: None,
    })
}

/// Upgrades everything winget can, in a visible console.
pub fn upgrade_all(bus: &Bus) {
    let exe = locate().unwrap_or_else(|| "winget.exe".into());
    let line = format!(
        "\"{}\" upgrade --all --include-unknown {ACCEPT} --silent || pause",
        exe.display()
    );
    if let Err(e) = platform::cmd(&line, Window::Visible) {
        bus.status(format!("Could not start winget: {e}"));
    }
}

/// Probes for winget and, when present, loads the catalog.
pub fn refresh(bus: &Bus) {
    bus.spawn("winget-probe", |bus| {
        let version = version();
        let found = version.is_some();
        bus.send(Event::Winget(version));
        if found {
            bus.send(Event::Catalog(fetch_catalog()));
        }
    });
}

/// Installs winget via [`BOOTSTRAP_SCRIPT`] (UAC prompt), then re-probes.
pub fn bootstrap(bus: &Bus) {
    bus.status("Installing winget — follow the PowerShell window…");
    bus.spawn("winget-bootstrap", |bus| {
        match platform::powershell_elevated(BOOTSTRAP_SCRIPT, true) {
            Ok(Some(0) | None) => bus.status("winget install finished; checking…"),
            Ok(Some(code)) => {
                bus.status(format!("winget install script failed (exit code {code})"))
            }
            Err(e) => bus.status(format!("winget install was not started: {e}")),
        }
        refresh(bus);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\r   - \r   \\ \rName                       Id                       Version        Source\r\n\
---------------------------------------------------------------------------------\r\n\
Clippy                     0-don.clippy             1.7.14         winget\n\
Batch Picture Resizer      SoftOrbits.BatchPictureR… 14.2          winget\r\n\
Microsoft Teams classic    Microsoft.Teams.Classic  1.7.00.13456   winget\n\
\n";

    #[test]
    fn parses_rows_with_spaces_in_names() {
        let pkgs = parse_catalog(SAMPLE);
        assert_eq!(pkgs.len(), 3);
        assert_eq!(pkgs[0], Package::new("Clippy", "0-don.clippy", "1.7.14"));
        assert_eq!(pkgs[1].name, "Batch Picture Resizer");
        assert_eq!(pkgs[1].version, "14.2");
        assert_eq!(pkgs[2].id, "Microsoft.Teams.Classic");
    }

    #[test]
    fn match_column_does_not_panic() {
        let text = "Name   Id          Version Match       Source\n\
-----------------------------------------------\n\
7-Zip  7zip.7zip   24.09   Tag: zip    winget\n\
x\n";
        let _ = parse_catalog(text);
    }

    #[test]
    fn search_is_case_insensitive_on_name_and_id() {
        let p = Package::new("Mozilla Firefox", "Mozilla.Firefox", "1");
        assert!(p.matches("firefox"));
        assert!(p.matches("mozilla.fire"));
        assert!(!p.matches("chrome"));
    }

    /// Runs the real `winget search`: `cargo test -- --ignored live_catalog`.
    #[test]
    #[ignore = "needs winget and network"]
    fn live_catalog() {
        let pkgs = fetch_catalog().expect("catalog");
        assert!(pkgs.len() > 1000, "only {} packages", pkgs.len());
        let firefox = pkgs
            .iter()
            .find(|p| p.id == "Mozilla.Firefox")
            .expect("Mozilla.Firefox");
        assert_eq!(firefox.name, "Mozilla Firefox (en-US)");
        assert!(
            pkgs.iter()
                .all(|p| !p.id.contains(' ') && !p.version.is_empty())
        );
    }

    #[test]
    fn config_entries_with_extra_arguments() {
        let plain = InstallRequest::from_config("7-zip", "7zip.7zip").unwrap();
        assert_eq!(plain.args, Vec::<String>::new());

        let pwsh = InstallRequest::from_config(
            "PowerShell",
            " Microsoft.PowerShell  --scope machine --installer-type wix ",
        )
        .unwrap();
        assert_eq!(pwsh.id, "Microsoft.PowerShell");
        assert_eq!(pwsh.args, ["--scope", "machine", "--installer-type", "wix"]);
        assert_eq!(
            pwsh.display(),
            "winget install --exact --id Microsoft.PowerShell --accept-source-agreements \
             --accept-package-agreements --silent --scope machine --installer-type wix"
        );

        let quoted =
            InstallRequest::from_config("x", r#"Id --location "C:\Program Files\X""#).unwrap();
        assert_eq!(quoted.args, ["--location", r"C:\Program Files\X"]);
        assert!(
            quoted
                .display()
                .ends_with(r#"--location "C:\Program Files\X""#)
        );
        assert_eq!(quote(r"C:\Dir Name\"), r#""C:\Dir Name\\""#);
    }

    #[test]
    fn unsafe_arguments_are_rejected() {
        assert!(split_args("--override %PATH%").is_err());
        assert!(split_args(r#"--log "unterminated"#).is_err());
        assert!(InstallRequest::from_config("x", "").is_err());
        // cmd metacharacters are fine once quoted.
        let r = InstallRequest::from_config("x", "Id --custom a&b|c").unwrap();
        assert!(r.display().ends_with(r#""a&b|c""#));
    }

    #[test]
    fn parses_show_and_versions_output() {
        let show = "Found PowerShell [Microsoft.PowerShell]\r\nVersion: 7.6.10.0\r\nPublisher: Microsoft\r\n\
Installer:\r\n  Installer Type: wix\r\n  Installer Url: https://x/PowerShell-7.6.10-win-x64.msi\r\n";
        let r = parse_show(show).unwrap();
        assert_eq!(r.version, "7.6.10.0");
        assert_eq!(r.installer_type.as_deref(), Some("wix"));
        assert_eq!(
            r.url.as_deref(),
            Some("https://x/PowerShell-7.6.10-win-x64.msi")
        );
        assert_eq!(
            parse_show("Found X\nVersion: 1\n  No applicable installer found; see logs"),
            None
        );

        let versions = "Found PowerShell [Microsoft.PowerShell]\r\nVersion\r\n--------\r\n7.7.0.0\r\n7.6.10.0\r\n";
        assert_eq!(parse_versions(versions), ["7.7.0.0", "7.6.10.0"]);
    }

    /// `cargo test -- --ignored live_resolve`
    #[test]
    #[ignore = "needs winget and network"]
    fn live_resolve_falls_back_to_older_version() {
        let request = InstallRequest::from_config(
            "pwsh",
            "Microsoft.PowerShell --scope machine --installer-type wix",
        )
        .unwrap();
        let r = resolve(&request).unwrap();
        assert_eq!(r.installer_type.as_deref(), Some("wix"));
        println!("{r:?}");
    }

    #[test]
    fn empty_or_garbage_output() {
        assert!(parse_catalog("").is_empty());
        assert!(parse_catalog("No package found matching input criteria.").is_empty());
    }
}
