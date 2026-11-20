use eframe::egui::{self, RichText, Ui};

use crate::app::{Configurator, WingetStatus};
use crate::config::{Config, State};
use crate::platform::{self, Window};
use crate::ui;
use crate::{download, paths, winget};

impl Configurator {
    pub(crate) fn page_settings(&mut self, ui: &mut Ui) {
        ui.columns_const(|[config, pc, app]| {
            egui::ScrollArea::vertical()
                .id_salt("settings_config")
                .show(config, |ui| self.config_column(ui));
            egui::ScrollArea::vertical()
                .id_salt("settings_pc")
                .show(pc, |ui| self.pc_column(ui));
            egui::ScrollArea::vertical()
                .id_salt("settings_app")
                .show(app, |ui| self.app_column(ui));
        });
    }

    fn config_column(&mut self, ui: &mut Ui) {
        ui::heading(ui, "Configuration");
        let path = self.config.path.clone();
        match &self.config.state {
            State::Loaded => {
                ui.colored_label(ui::GREEN, "config.ini loaded");
            }
            State::Missing => {
                ui.label(
                    RichText::new("No config file found")
                        .color(ui::RED)
                        .strong(),
                );
                ui.label("Favourite apps, scripts, installers and tokens come from config.ini next to the program.");
                if ui.button("Create default config.ini").clicked() {
                    let result = Config::create_default(&path);
                    match result {
                        // The watcher would pick it up within 2 s; load now for instant feedback.
                        Ok(()) => self.config = Config::load(&path),
                        Err(e) => self.status(format!("Could not create {}: {e}", path.display())),
                    }
                }
            }
            State::Invalid(e) => {
                ui.label(
                    RichText::new("config.ini could not be read")
                        .color(ui::RED)
                        .strong(),
                );
                ui.label(e.as_str());
            }
        }
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new(path.display().to_string())
                    .monospace()
                    .size(12.0),
            )
            .selectable(true),
        );
        ui.weak("Changes to the file are picked up automatically.");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(path.exists(), egui::Button::new("Edit config"))
                .clicked()
            {
                let result = std::process::Command::new("notepad.exe").arg(&path).spawn();
                self.report("Could not open Notepad", result);
            }
            if ui.button("Open folder").clicked() {
                self.report(
                    "Could not open the folder",
                    platform::open(paths::exe_dir()),
                );
            }
        });
    }

    fn pc_column(&mut self, ui: &mut Ui) {
        ui::heading(ui, "This PC");

        ui.label(RichText::new("Rename PC").strong());
        let valid = is_valid_pc_name(self.pc_name.trim());
        ui.add_enabled_ui(self.elevated, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.pc_name).hint_text("new name").desired_width(160.0));
                if ui.add_enabled(valid, egui::Button::new("Rename")).on_hover_text("Takes effect after a restart").clicked() {
                    let script = format!(
                        "Rename-Computer -NewName '{}' -Force; Write-Host 'Restart the PC to apply the new name.'; Read-Host 'Press Enter to close'",
                        self.pc_name.trim()
                    );
                    let result = platform::powershell(&script, Window::Visible);
                    self.report("Could not start PowerShell", result);
                }
            });
        });
        if !self.elevated {
            ui.weak("Requires administrator rights.");
        } else if !self.pc_name.is_empty() && !valid {
            ui.colored_label(
                ui::ORANGE,
                "1–15 letters, digits or hyphens; not only digits.",
            );
        }
        ui.separator();

        ui.label(RichText::new("winget").strong());
        let (text, color) = match &self.winget.status {
            WingetStatus::Available(v) => (format!("Installed ({v})"), ui::GREEN),
            WingetStatus::Checking => ("Checking…".to_owned(), ui::ORANGE),
            WingetStatus::Installing => ("Installing…".to_owned(), ui::ORANGE),
            WingetStatus::Missing => ("Not found".to_owned(), ui::RED),
        };
        ui.colored_label(color, text);
        let idle = !matches!(
            self.winget.status,
            WingetStatus::Checking | WingetStatus::Installing
        );
        ui.horizontal(|ui| {
            let label = if self.winget.available() { "Repair / update winget" } else { "Install winget" };
            if ui
                .add_enabled(idle, egui::Button::new(label))
                .on_hover_text("Installs NuGet + Microsoft.WinGet.Client from PSGallery, then runs Repair-WinGetPackageManager -AllUsers (UAC prompt)")
                .clicked()
            {
                self.install_winget();
            }
            if ui.add_enabled(idle, egui::Button::new("Check again")).clicked() {
                self.winget.status = WingetStatus::Checking;
                winget::refresh(&self.bus);
            }
        });
    }

    fn app_column(&mut self, ui: &mut Ui) {
        ui::heading(ui, "Application");
        egui::Grid::new("app_info")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label("Version");
                ui.label(crate::VERSION);
                ui.end_row();
                ui.label("Administrator");
                ui.label(if self.elevated { "Yes" } else { "No" });
                ui.end_row();
                ui.label("Renderer");
                ui.add(egui::Label::new(self.renderer.as_str()).wrap());
                ui.end_row();
            });
        ui.add_space(8.0);

        if !self.elevated && ui.button("Restart as administrator").clicked() {
            self.restart_elevated(ui.ctx());
        }
        if ui
            .button("Update to latest version")
            .on_hover_text("Downloads the latest nightly build, replaces this exe and restarts")
            .clicked()
        {
            download::self_update(&self.bus);
        }
    }
}

/// NetBIOS-compatible computer name.
fn is_valid_pc_name(name: &str) -> bool {
    (1..=15).contains(&name.len())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !name.chars().all(|c| c.is_ascii_digit())
        && !name.starts_with('-')
}

#[cfg(test)]
mod tests {
    use super::is_valid_pc_name;

    #[test]
    fn pc_names() {
        assert!(is_valid_pc_name("DESKTOP-01"));
        assert!(!is_valid_pc_name("12345"));
        assert!(!is_valid_pc_name("has space"));
        assert!(!is_valid_pc_name("x';calc;'"));
        assert!(!is_valid_pc_name("ABCDEFGHIJKLMNOP"));
    }
}
