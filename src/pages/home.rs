use eframe::egui::{RichText, Ui};

use crate::app::{Configurator, WingetStatus};
use crate::config::State;
use crate::ui;

impl Configurator {
    pub(crate) fn page_home(&mut self, ui: &mut Ui) {
        ui.label("A portable toolbox for setting up, troubleshooting and maintaining Windows PCs:");
        for line in [
            "Install, update and remove applications (winget)",
            "Launch common Windows settings and control panel applets",
            "Conveniently launch your own scripts, programs and installers",
            "View system and network information",
            "Run basic troubleshooting tools",
        ] {
            ui.label(format!("  •  {line}"));
        }

        ui.add_space(12.0);
        ui::heading(ui, "This PC");
        egui_status_row(
            ui,
            "Running as administrator",
            if self.elevated { "Yes" } else { "No" },
            self.elevated,
        );
        let (winget, ok) = match &self.winget.status {
            WingetStatus::Available(v) => (format!("Installed ({v})"), true),
            WingetStatus::Checking => ("Checking…".to_owned(), true),
            WingetStatus::Installing => ("Installing…".to_owned(), true),
            WingetStatus::Missing => ("Not found".to_owned(), false),
        };
        egui_status_row(ui, "winget", &winget, ok);
        let config = match &self.config.state {
            State::Loaded => "Loaded".to_owned(),
            State::Missing => "Not found (create one under Settings)".to_owned(),
            State::Invalid(e) => format!("Could not be read: {e}"),
        };
        egui_status_row(
            ui,
            "config.ini",
            &config,
            self.config.state == State::Loaded,
        );

        ui.add_space(8.0);
        if !self.elevated {
            self.admin_notice(ui, "Some tools");
        }
        self.winget_notice(ui);
    }
}

fn egui_status_row(ui: &mut Ui, label: &str, value: &str, ok: bool) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.label(RichText::new(value).color(if ok { ui::GREEN } else { ui::RED }));
    });
}
