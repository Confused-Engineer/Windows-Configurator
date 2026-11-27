//! Built-in command lists (Windows settings pages, control panel applets,
//! troubleshooting commands). Parsed once from embedded INI files.

use std::sync::LazyLock;

use ini::{Ini, ParseOption};

pub struct Entry {
    pub label: String,
    pub command: String,
}

pub struct Group {
    pub title: String,
    pub entries: Vec<Entry>,
}

fn parse(text: &str) -> Vec<Group> {
    let options = ParseOption {
        enabled_escape: false,
        ..Default::default()
    };
    let Ok(ini) = Ini::load_from_str_opt(text, options) else {
        return Vec::new();
    };
    ini.iter()
        .filter_map(|(title, props)| {
            Some(Group {
                title: title?.to_owned(),
                entries: props
                    .iter()
                    .map(|(k, v)| Entry {
                        label: k.to_owned(),
                        command: v.to_owned(),
                    })
                    .collect(),
            })
        })
        .collect()
}

macro_rules! command_list {
    ($name:ident, $file:literal) => {
        pub static $name: LazyLock<Vec<Group>> = LazyLock::new(|| {
            parse(include_str!(concat!(
                "../assets/resources/commands/",
                $file
            )))
        });
    };
}

command_list!(WINDOWS_SETTINGS, "win_settings.ini");
command_list!(CONTROL_PANEL, "control_panel_settings.ini");
command_list!(SYSTEM_COMMANDS, "commands.ini");
command_list!(NETWORK_ADMIN, "network_commands_admin.ini");
command_list!(TROUBLESHOOTING, "troubleshooting_commands.ini");
command_list!(
    TROUBLESHOOTING_HIDDEN,
    "troubleshooting_commands_hidden.ini"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_lists_parse() {
        for list in [
            &WINDOWS_SETTINGS,
            &CONTROL_PANEL,
            &SYSTEM_COMMANDS,
            &NETWORK_ADMIN,
            &TROUBLESHOOTING,
            &TROUBLESHOOTING_HIDDEN,
        ] {
            assert!(!list.is_empty());
            assert!(list.iter().all(|g| !g.entries.is_empty()));
        }
        let features = CONTROL_PANEL[0]
            .entries
            .iter()
            .find(|e| e.label == "Windows features")
            .unwrap();
        assert_eq!(features.command, r"%windir%\System32\OptionalFeatures.exe");
    }
}
