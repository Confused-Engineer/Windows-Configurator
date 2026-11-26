use egui::Ui;

use crate::app::Configurator;
use crate::ui::{self, icons};

impl Configurator {
    pub(crate) fn page_sysinfo(&mut self, ui: &mut Ui) {
        let mut force = false;
        ui.horizontal(|ui| {
            force = ui::icon_button(ui, icons::refresh(), ui::GREEN, "Refresh now").clicked();
            if self.sysinfo.loading {
                ui.spinner();
                ui.label("Collecting…");
            } else {
                ui.weak("Refreshes automatically every 30 seconds while this page is open.");
            }
        });
        ui.separator();
        self.refresh_sysinfo(ui.ctx(), force);

        let data = &self.sysinfo.data;
        ui.columns_const(|[left, right]| {
            egui::ScrollArea::both()
                .id_salt("sysinfo_left")
                .auto_shrink([false; 2])
                .show(left, |ui| {
                    ui::heading(ui, "System Info");
                    ui::output_block(ui, &data.systeminfo);
                    ui.add_space(8.0);
                    ui::heading(ui, "Netstat");
                    ui::output_block(ui, &data.netstat);
                });
            egui::ScrollArea::both()
                .id_salt("sysinfo_right")
                .auto_shrink([false; 2])
                .show(right, |ui| {
                    ui::heading(ui, "IP Config");
                    ui::output_block(ui, &data.ipconfig);
                    ui.add_space(8.0);
                    ui::heading(ui, "ARP");
                    ui::output_block(ui, &data.arp);
                    ui.add_space(8.0);
                    ui::heading(ui, "Route Table");
                    ui::output_block(ui, &data.routes);
                });
        });
    }
}
