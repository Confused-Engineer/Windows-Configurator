use egui::Ui;
use std::fs::{remove_file, File};
use std::io::Write;

use crate::app::Config;

pub fn show_page_config_error(ui: &mut Ui, config: &mut Config)
{
    ui.heading("Uh Oh.....");
    ui.label("It appears that the config file is missing or misconfigured.");
    ui.horizontal(|ui|{
        if ui.add_sized([300.0, 40.0], egui::widgets::Button::new("Create New Config")).clicked()
        {
            overwrite_config(config);
        }
        if ui.add_sized([300.0, 40.0], egui::widgets::Button::new("Reload Config")).clicked()
        {
            config.validate();
        }
    });

}



fn overwrite_config(config: &mut Config)
{
    let _ = remove_file("config.ini");
    let mut filemake = File::create("config.ini").unwrap();
    let _ = filemake.write_all(include_bytes!("../../config.ini"));
    config.validate();

}