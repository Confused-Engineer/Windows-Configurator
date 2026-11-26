use egui::Ui;

use crate::app::Configurator;
use crate::commands::{CONTROL_PANEL, Group, SYSTEM_COMMANDS, WINDOWS_SETTINGS};
use crate::platform;
use crate::tasks::Bus;
use crate::ui::{self, icons};

impl Configurator {
    pub(crate) fn page_launch(&mut self, ui: &mut Ui) {
        let bus = self.bus.clone();
        ui.columns_const(|[settings, control, system]| {
            for (ui, title, groups) in [
                (settings, "Windows Settings", &*WINDOWS_SETTINGS),
                (control, "Control Panel", &*CONTROL_PANEL),
                (system, "System Commands", &*SYSTEM_COMMANDS),
            ] {
                ui::heading(ui, title);
                egui::ScrollArea::vertical()
                    .id_salt(title)
                    .show(ui, |ui| start_list(ui, &bus, groups));
            }
        });
    }
}

fn start_list(ui: &mut Ui, bus: &Bus, groups: &[Group]) {
    for group in groups {
        ui::group_heading(ui, &group.title);
        for entry in &group.entries {
            ui::row(ui, entry.label.as_str(), |ui| {
                if ui::icon_button(ui, icons::launch(), ui::GREEN, &entry.command).clicked()
                    && let Err(e) = platform::start(&entry.command)
                {
                    bus.status(format!("Could not start {}: {e}", entry.label));
                }
            });
        }
    }
}
