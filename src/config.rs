//! The user's `config.ini`, stored next to the executable (see [`crate::paths`]).
//!
//! Sections:
//! * `[winget-<Category>]` — `Friendly name = Winget.Id` favourites
//! * `[Programs]`          — `Name = relative\or\absolute\path.exe`
//! * `[Powershell]`        — `Name = script.ps1`
//! * `[<EXT>-Online]`      — `Name = https://url`, downloaded as `Name.<ext>` then opened
//! * `[Tokens]`            — `Name = value`, shown with a copy button

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use ini::{Ini, ParseOption};

use crate::tasks::{Bus, Event};

/// The template written by "Create default config".
pub const DEFAULT_CONFIG: &str = include_str!("../assets/config.ini");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Loaded,
    Missing,
    Invalid(String),
}

#[derive(Clone)]
pub struct Config {
    pub path: PathBuf,
    pub state: State,
    ini: Ini,
}

/// What a non-winget launcher entry does when clicked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchKind {
    Program,
    Powershell,
    /// Download, saving with this extension (lower-case, without dot).
    Online(String),
}

impl LaunchKind {
    pub fn from_section(section: &str) -> Option<Self> {
        if section.eq_ignore_ascii_case("Programs") {
            Some(Self::Program)
        } else if section.eq_ignore_ascii_case("Powershell") {
            Some(Self::Powershell)
        } else {
            let (ext, suffix) = section.rsplit_once('-')?;
            (suffix.eq_ignore_ascii_case("Online") && !ext.is_empty())
                .then(|| Self::Online(ext.to_ascii_lowercase()))
        }
    }
}

pub struct Section<'a> {
    pub name: &'a str,
    pub entries: Vec<(&'a str, &'a str)>,
}

impl Config {
    pub fn load(path: &Path) -> Self {
        let options = ParseOption {
            enabled_escape: false,
            ..Default::default()
        };
        let (state, ini) = match Ini::load_from_file_opt(path, options) {
            Ok(ini) => (State::Loaded, ini),
            Err(ini::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                (State::Missing, Ini::new())
            }
            Err(e) => (State::Invalid(e.to_string()), Ini::new()),
        };
        Self {
            path: path.to_path_buf(),
            state,
            ini,
        }
    }

    #[cfg(test)]
    pub fn parse_str(text: &str) -> Self {
        let options = ParseOption {
            enabled_escape: false,
            ..Default::default()
        };
        let ini = Ini::load_from_str_opt(text, options).expect("valid ini");
        Self {
            path: PathBuf::from("config.ini"),
            state: State::Loaded,
            ini,
        }
    }

    /// Writes the default template next to the executable (only if absent).
    pub fn create_default(path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(DEFAULT_CONFIG.as_bytes())
    }

    fn sections_where(&self, mut keep: impl FnMut(&str) -> bool) -> Vec<Section<'_>> {
        self.ini
            .iter()
            .filter_map(|(name, props)| {
                let name = name?;
                keep(name).then(|| Section {
                    name,
                    entries: props.iter().collect(),
                })
            })
            .collect()
    }

    /// `[winget-<Category>]` sections, with the prefix stripped from the name.
    pub fn winget_sections(&self) -> Vec<Section<'_>> {
        let mut sections = self.sections_where(|n| winget_category(n).is_some());
        for s in &mut sections {
            s.name = winget_category(s.name).unwrap_or(s.name);
        }
        sections
    }

    /// `[Programs]`, `[Powershell]` and `[<EXT>-Online]` sections.
    pub fn launcher_sections(&self) -> Vec<(LaunchKind, Section<'_>)> {
        self.sections_where(|n| LaunchKind::from_section(n).is_some())
            .into_iter()
            .filter_map(|s| Some((LaunchKind::from_section(s.name)?, s)))
            .collect()
    }

    pub fn tokens(&self) -> Vec<(&str, &str)> {
        self.sections_where(|n| n.eq_ignore_ascii_case("Tokens"))
            .into_iter()
            .flat_map(|s| s.entries)
            .collect()
    }
}

fn winget_category(section: &str) -> Option<&str> {
    let (prefix, rest) = section.split_once('-')?;
    prefix.eq_ignore_ascii_case("winget").then_some(rest)
}

/// Watches the config file and sends an [`Event::Config`] whenever it is
/// created, deleted or modified. Only a `stat` every couple of seconds; the
/// file is re-parsed only when its timestamp or size changes.
pub fn watch(bus: &Bus) {
    bus.spawn_daemon("config-watcher", |bus| {
        let path = crate::paths::config();
        let stamp = |p: &Path| {
            std::fs::metadata(p)
                .ok()
                .map(|m| (m.modified().ok(), m.len()))
        };
        let mut last: Option<Option<(Option<SystemTime>, u64)>> = None;
        loop {
            let now = stamp(&path);
            if last.as_ref() != Some(&now) {
                last = Some(now);
                bus.send(Event::Config(Config::load(&path)));
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_template_parses_into_expected_sections() {
        let config = Config::parse_str(DEFAULT_CONFIG);
        let winget: Vec<_> = config
            .winget_sections()
            .iter()
            .map(|s| s.name.to_owned())
            .collect();
        assert!(winget.contains(&"Browsers".to_owned()));
        assert!(winget.contains(&"Design and Development".to_owned()));

        let kinds: Vec<_> = config
            .launcher_sections()
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert!(kinds.contains(&LaunchKind::Program));
        assert!(kinds.contains(&LaunchKind::Powershell));
        assert!(kinds.contains(&LaunchKind::Online("msix".into())));

        // Backslashes in tokens must survive (escape processing is disabled).
        let tokens = config.tokens();
        assert_eq!(
            tokens[0],
            ("app1 token", r"bkjvpje\fgh/iijfefewddd\\ddfeegrht==")
        );
    }

    #[test]
    fn launch_kind_from_section() {
        assert_eq!(
            LaunchKind::from_section("Taco-Online"),
            Some(LaunchKind::Online("taco".into()))
        );
        assert_eq!(
            LaunchKind::from_section("programs"),
            Some(LaunchKind::Program)
        );
        assert_eq!(LaunchKind::from_section("-Online"), None);
        assert_eq!(LaunchKind::from_section("Tokens"), None);
        assert_eq!(winget_category("winget-Foo-Bar"), Some("Foo-Bar"));
    }
}
