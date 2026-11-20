//! Filesystem locations. The app is portable: everything it owns (the config
//! file in particular) lives next to the executable, never in the profile.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const CONFIG_FILE: &str = "config.ini";

static EXE: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("Windows_Configurator.exe"))
});

/// Full path of the running executable.
pub fn exe() -> &'static Path {
    &EXE
}

/// Directory containing the executable (e.g. the root of the USB drive).
pub fn exe_dir() -> &'static Path {
    EXE.parent().unwrap_or(Path::new("."))
}

/// `config.ini` next to the executable.
pub fn config() -> PathBuf {
    exe_dir().join(CONFIG_FILE)
}

/// Scratch directory for downloaded installers. Kept on the local disk so the
/// USB drive is not filled up (and may even be write-protected).
pub fn downloads() -> PathBuf {
    let dir = std::env::temp_dir().join("WindowsConfigurator");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Where the previous executable is parked during a self-update.
pub fn old_exe() -> PathBuf {
    EXE.with_extension("exe.old")
}

/// Where a self-update is downloaded before being swapped in.
pub fn new_exe() -> PathBuf {
    EXE.with_extension("exe.new")
}

/// Process-wide setup: make relative paths in the config (e.g. `[Programs]`
/// entries) resolve against the executable's folder and remove leftovers of a
/// previous self-update.
pub fn init() {
    let _ = std::env::set_current_dir(exe_dir());
    // Right after a self-update the previous process may still be exiting,
    // so retry for a few seconds before giving up until the next start.
    if old_exe().exists() || new_exe().exists() {
        std::thread::spawn(|| {
            for _ in 0..20 {
                let old = std::fs::remove_file(old_exe());
                let new = std::fs::remove_file(new_exe());
                if !old_exe().exists() && !new_exe().exists() || (old.is_ok() && new.is_ok()) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
    }
}

/// Resolves a path from the config: absolute paths are kept, relative ones are
/// taken relative to the executable's folder.
pub fn resolve(path: &str) -> PathBuf {
    let path = Path::new(path.trim().trim_matches('"'));
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        exe_dir().join(path)
    }
}
