//! Thin wrappers over Win32 and process spawning.

use std::ffi::{OsStr, c_void};
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, ExitStatus};

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, INFINITE, OpenProcessToken, WaitForSingleObject,
};
use windows::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW, SW_SHOWNORMAL};
use windows::core::PCWSTR;

/// `CREATE_NO_WINDOW`: run a console program without showing a console.
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// `CREATE_NEW_CONSOLE`: give a console program its own visible window.
pub const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

/// Whether a spawned console program gets a visible window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    Hidden,
    Visible,
}

impl Window {
    fn flags(self) -> u32 {
        match self {
            Self::Hidden => CREATE_NO_WINDOW,
            Self::Visible => CREATE_NEW_CONSOLE,
        }
    }
}

fn wide(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0)).collect()
}

/// True when the process runs with an elevated (administrator) token.
pub fn is_elevated() -> bool {
    // SAFETY: plain Win32 calls with valid out-pointers; the token handle is
    // closed before returning.
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut len = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(std::ptr::from_mut(&mut elevation).cast::<c_void>()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

/// Modal error box; used when no window could be created at all.
pub fn message_box(title: &str, text: &str) {
    let (title, text) = (wide(title), wide(text));
    // SAFETY: both strings are NUL-terminated and outlive the call.
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

/// `ShellExecuteEx` wrapper. `verb` is e.g. `"open"` or `"runas"` (elevate).
/// With `wait`, blocks until the started process exits and returns its exit
/// code (`None` when the shell did not start a process, e.g. a document
/// handed to an already-running app).
pub fn shell_execute(
    verb: &str,
    file: impl AsRef<OsStr>,
    params: Option<&str>,
    wait: bool,
) -> io::Result<Option<u32>> {
    let verb = wide(verb);
    let file = wide(file);
    let params = params.map(wide);
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: if wait {
            SEE_MASK_NOCLOSEPROCESS
        } else {
            Default::default()
        },
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: params
            .as_ref()
            .map_or(PCWSTR::null(), |p| PCWSTR(p.as_ptr())),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    // SAFETY: every string pointer in `info` points into a live, NUL-terminated
    // buffer owned by this frame.
    unsafe { ShellExecuteExW(&mut info) }
        .map_err(|e| io::Error::from_raw_os_error(e.code().0 & 0xFFFF))?;

    if !wait || info.hProcess.is_invalid() {
        return Ok(None);
    }
    // SAFETY: hProcess is a valid process handle we own (SEE_MASK_NOCLOSEPROCESS).
    unsafe {
        WaitForSingleObject(info.hProcess, INFINITE);
        let mut code = 0;
        let got = GetExitCodeProcess(info.hProcess, &mut code).is_ok();
        let _ = CloseHandle(info.hProcess);
        Ok(got.then_some(code))
    }
}

/// Opens a file, folder, URL or `ms-settings:` URI with its default handler.
pub fn open(target: impl AsRef<OsStr>) -> io::Result<()> {
    shell_execute("open", target, None, false).map(|_| ())
}

/// Starts another copy of this program elevated (UAC prompt).
pub fn relaunch_elevated() -> io::Result<()> {
    shell_execute("runas", crate::paths::exe(), None, false).map(|_| ())
}

/// Runs a `cmd.exe` command line exactly as written (pipes, `&`, `%VARS%`).
pub fn cmd(command_line: &str, window: Window) -> io::Result<Child> {
    Command::new("cmd.exe")
        // /D: skip AutoRun; /S + outer quotes: pass the line through verbatim.
        .raw_arg(format!("/D /S /C \"{command_line}\""))
        .creation_flags(window.flags())
        .spawn()
}

/// Hands `target` to `start`, i.e. launches it detached the way Explorer
/// would (`control schedtasks`, `appwiz.cpl`, `ms-settings:…`).
pub fn start(target: &str) -> io::Result<Child> {
    cmd(&format!("start \"\" {target}"), Window::Hidden)
}

/// Runs a PowerShell script (Windows PowerShell 5.1, always present) without
/// any quoting problems by passing it base64-encoded.
pub fn powershell(script: &str, window: Window) -> io::Result<Child> {
    Command::new("powershell.exe")
        .args(powershell_args(script))
        .creation_flags(window.flags())
        .spawn()
}

/// Same as [`powershell`] but elevated through UAC, in a visible window.
/// Blocks until the script finishes when `wait` is set.
pub fn powershell_elevated(script: &str, wait: bool) -> io::Result<Option<u32>> {
    let params = powershell_args(script).join(" ");
    shell_execute("runas", "powershell.exe", Some(&params), wait)
}

/// Runs a PowerShell `.ps1` file in a visible window.
pub fn powershell_file(path: &Path) -> io::Result<Child> {
    Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(path)
        .current_dir(path.parent().unwrap_or(crate::paths::exe_dir()))
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
}

fn powershell_args(script: &str) -> [String; 5] {
    [
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-EncodedCommand".into(),
        encode_powershell(script),
    ]
}

/// Encodes a script for `powershell -EncodedCommand` (base64 of UTF-16LE).
pub fn encode_powershell(script: &str) -> String {
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64(&bytes)
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = u32::from(chunk[0]) << 16
            | u32::from(*chunk.get(1).unwrap_or(&0)) << 8
            | u32::from(*chunk.get(2).unwrap_or(&0));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Runs a console program hidden and captures stdout (lossy UTF-8, since many
/// built-in tools print in the OEM code page).
pub fn capture(program: impl AsRef<OsStr>, args: &[&str]) -> io::Result<(ExitStatus, String)> {
    let out = Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;
    Ok((
        out.status,
        String::from_utf8_lossy(&out.stdout).into_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(input.as_bytes()), expected);
        }
    }

    #[test]
    fn cmd_passes_the_line_verbatim() {
        let out = std::env::temp_dir().join("wc_cmd_test.txt");
        let _ = std::fs::remove_file(&out);
        // (cmd keeps the space before `&` as part of the first echo.)
        let line = format!(
            r#"echo one>"{0}" & echo "two & three">>"{0}""#,
            out.display()
        );
        assert!(
            cmd(&line, Window::Hidden)
                .unwrap()
                .wait()
                .unwrap()
                .success()
        );
        assert_eq!(
            std::fs::read_to_string(&out).unwrap(),
            "one \r\n\"two & three\"\r\n"
        );
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn powershell_runs_encoded_scripts_with_quotes() {
        let out = std::env::temp_dir().join("wc_ps_test.txt");
        let _ = std::fs::remove_file(&out);
        let script = format!(
            "Set-Content -LiteralPath '{}' -Value \"it's `\"quoted`\"\" -NoNewline",
            out.display()
        );
        assert!(
            powershell(&script, Window::Hidden)
                .unwrap()
                .wait()
                .unwrap()
                .success()
        );
        assert_eq!(std::fs::read_to_string(&out).unwrap(), "it's \"quoted\"");
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn powershell_encoding_is_utf16le() {
        // `powershell -EncodedCommand ZABpAHIA` runs `dir`.
        assert_eq!(encode_powershell("dir"), "ZABpAHIA");
    }
}
