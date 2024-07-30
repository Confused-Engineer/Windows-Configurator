use egui::Ui;

use crate::app::Config;

pub fn show_page_config_error(ui: &mut Ui, config: &mut Config)
{
    ui.heading("Uh Oh.....");
    ui.label("It appears that the config file is missing or misconfigured.");
    ui.horizontal(|ui|{
        if ui.add_sized([300.0, 40.0], egui::widgets::Button::new("Create New Config")).clicked()
        {
            let _ = config.remove_and_replace();
        }
        if ui.add_sized([300.0, 40.0], egui::widgets::Button::new("Reload Config")).clicked()
        {
            config.validate();
        }
    });

}



