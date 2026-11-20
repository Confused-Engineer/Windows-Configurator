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

/// Installs a package by exact id. `Window::Visible` shows winget's progress
/// in a console (kept open on failure); `Window::Hidden` reports the result
/// in the status bar.
pub fn install(bus: &Bus, name: &str, id: &str, window: Window) {
    let exe = locate().unwrap_or_else(|| "winget.exe".into());
    let line = format!(
        "\"{}\" install --exact --id \"{id}\" {ACCEPT} --silent",
        exe.display()
    );
    match window {
        Window::Visible => {
            if let Err(e) = platform::cmd(&format!("{line} || pause"), Window::Visible) {
                bus.status(format!("Could not start winget: {e}"));
            }
        }
        Window::Hidden => {
            let (name, line) = (name.to_owned(), format!("{line} --disable-interactivity"));
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
    fn empty_or_garbage_output() {
        assert!(parse_catalog("").is_empty());
        assert!(parse_catalog("No package found matching input criteria.").is_empty());
    }
}
