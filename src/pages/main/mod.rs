use crate::Configurator;
use eframe::egui;

impl Configurator
{
    pub fn page_main_first(&mut self, ui: &mut egui::Ui)
    {
        ui.heading("Purpose");
        ui.label("· Install, Update, and Remove Applications");
        ui.label("· Configure Common Windows Settings");
        ui.label("· Conveniently Launch Scripts and Programs");
        ui.label("· View System and Network Information");
        ui.label("· Help Provide Basic Troubleshooting Tools");
        
    }
}