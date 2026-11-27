//! HTTP downloads: `[<EXT>-Online]` config entries and the self-updater.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;

use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

use crate::paths;
use crate::platform;
use crate::tasks::{Bus, Event};

static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .timeout_connect(Some(Duration::from_secs(20)))
        .user_agent(concat!("Windows-Configurator/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
});

/// Adds `https://` to scheme-less URLs such as `example.com/setup.exe`.
fn normalize_url(url: &str) -> String {
    let url = url.trim();
    if url.contains("://") {
        url.to_owned()
    } else {
        format!("https://{url}")
    }
}

/// Downloads `url` to `dest` via a temporary `.part` file, so a failed
/// download never leaves a truncated file behind under the final name.
pub fn download(url: &str, dest: &Path) -> io::Result<()> {
    let part = dest.with_extension("part");
    let result = (|| {
        let response = AGENT
            .get(&normalize_url(url))
            .call()
            .map_err(io::Error::other)?;
        let mut reader = response.into_body().into_reader();
        let mut file = std::fs::File::create(&part)?;
        io::copy(&mut reader, &mut file)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&part, dest)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

/// Makes a config entry name safe to use as a file name.
fn file_name(name: &str, ext: &str) -> String {
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_control() || r#"<>:"/\|?*"#.contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    format!("{}.{ext}", stem.trim().trim_end_matches('.'))
}

/// Downloads an installer to the temp folder and opens it.
pub fn download_and_open(bus: &Bus, name: &str, ext: &str, url: &str) {
    let dest: PathBuf = paths::downloads().join(file_name(name, ext));
    let (name, url) = (name.to_owned(), url.to_owned());
    bus.status(format!("Downloading {name}…"));
    bus.spawn("download", move |bus| {
        let result = download(&url, &dest).and_then(|()| platform::open(&dest));
        bus.status(match result {
            Ok(()) => format!("Downloaded and started {name}"),
            Err(e) => format!("Downloading {name} failed: {e}"),
        });
    });
}

/// Replaces the running executable with the latest nightly build and restarts.
///
/// Windows allows renaming (not overwriting) a running exe, so the current
/// file is moved aside to `*.exe.old` (deleted on next start, see
/// [`paths::init`]) and the download takes its place.
pub fn self_update(bus: &Bus) {
    bus.status("Downloading the latest version…");
    bus.spawn("self-update", |bus| {
        let (exe, old, new) = (paths::exe(), paths::old_exe(), paths::new_exe());
        let result = (|| {
            download(crate::UPDATE_URL, &new)?;
            let _ = std::fs::remove_file(&old);
            std::fs::rename(exe, &old)?;
            if let Err(e) = std::fs::rename(&new, exe) {
                let _ = std::fs::rename(&old, exe); // roll back
                return Err(e);
            }
            std::process::Command::new(exe)
                .current_dir(paths::exe_dir())
                .spawn()
        })();
        match result {
            Ok(_) => bus.send(Event::Quit),
            Err(e) => {
                let _ = std::fs::remove_file(&new);
                bus.status(format!("Update failed: {e}"));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_and_file_names() {
        assert_eq!(
            normalize_url(" example.com/a.exe "),
            "https://example.com/a.exe"
        );
        assert_eq!(normalize_url("http://x/y"), "http://x/y");
        assert_eq!(file_name("Live Mas", "taco"), "Live Mas.taco");
        assert_eq!(file_name(r"a/b:c?", "msi"), "a_b_c_.msi");
    }
}
