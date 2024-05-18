use crate::app::Config;
use egui::Ui;

pub fn show_page_config_edit(ui: &mut Ui, config: &mut Config)
{
    ui.heading("Config Options");
    ui.label("Here a few options for things that can be done with the configuration file.");
    ui.add_space(8.0);
    ui.columns(3, |ui|{
        ui[0].small("Auto-discover programs and scripts in current directory (experimental)");
        ui[1].small("To assist with Auto-Discover, the Programs and Powershell sections can be cleared for you. This is to aid with keeping the sections filled with content that is present in the directory");
        ui[2].small("Save changes to config file through this page");
 
    });

    ui.columns(3, |ui|{

        if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Auto Discover") ).clicked()
        {
            config.auto_discover();
        }        
        if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Clear Programs and Powershell") ).clicked()
        {
            config.clear_pgr_scripts();
        }         
        if ui[2].add_sized([100.0, 40.0], egui::widgets::Button::new("Save Config") ).clicked()
        {
            config.save_config();
        } 
    });

    egui::ScrollArea::vertical().id_source("TroubleshootOptions").show(ui, |ui|{
        ui.columns(2, |ui|{
            
        });
    });
}