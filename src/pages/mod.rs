//! One module per side-bar page; each adds a `page_*` method to the app.

mod apps;
mod home;
mod install_dialog;
mod launch;
mod settings;
mod sysinfo;
mod tokens;
mod troubleshoot;

use egui::Ui;

pub(crate) use install_dialog::InstallDialog;

use crate::app::{Configurator, WingetStatus};
use crate::ui;

impl Configurator {
    /// Banner offering to restart elevated; returns nothing, just draws.
    pub(crate) fn admin_notice(&mut self, ui: &mut Ui, what: &str) {
        if self.elevated {
            return;
        }
        let mut clicked = false;
        ui::notice(ui, ui::ORANGE, |ui| {
            ui.label(format!("{what} require administrator rights."));
            clicked = ui.button("Restart as administrator").clicked();
        });
        if clicked {
            self.restart_elevated(ui.ctx());
        }
    }

    pub(crate) fn restart_elevated(&mut self, ctx: &egui::Context) {
        match crate::platform::relaunch_elevated() {
            Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Err(e) => self.status(format!("Could not restart as administrator: {e}")),
        }
    }

    /// Banner shown when winget is not available, with an install button.
    pub(crate) fn winget_notice(&mut self, ui: &mut Ui) {
        let mut install = false;
        match &self.winget.status {
            WingetStatus::Available(_) => return,
            WingetStatus::Checking => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Looking for winget…");
                });
                return;
            }
            WingetStatus::Installing => ui::notice(ui, ui::ORANGE, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Installing winget. Follow the PowerShell window; this can take a few minutes.");
                });
            }),
            WingetStatus::Missing => ui::notice(ui, ui::RED, |ui| {
                ui.label(egui::RichText::new("winget was not found on this PC").strong());
                ui.label(
                    "It is usually missing on debloated or Store-less Windows installs. It can be \
                     installed from the PowerShell Gallery (NuGet → Microsoft.WinGet.Client → \
                     Repair-WinGetPackageManager). Requires administrator rights and internet access.",
                );
                install = ui.button("Install winget").clicked();
            }),
        }
        if install {
            self.install_winget();
        }
    }
}

/// Appends `& pause` so a visible console stays open to show the result.
pub(crate) fn with_pause(command: &str) -> String {
    if command.to_ascii_lowercase().contains("pause") {
        command.to_owned()
    } else {
        format!("{command} & pause")
    }
}
