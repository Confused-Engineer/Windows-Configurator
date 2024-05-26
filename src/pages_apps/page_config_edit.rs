use crate::app::Config;
use egui::Ui;

pub fn show_page_config_edit(ui: &mut Ui, config: &mut Config)
{
    ui.heading("Config Options");
    ui.label("Here a few options for things that can be done with the configuration file.");
    ui.add_space(8.0);
    ui.columns(4, |ui|{
        ui[0].small("Auto-discover programs and scripts in current directory (experimental)");
        ui[1].small("To assist with Auto-Discover, the Programs and Powershell sections can be cleared for you. This is to aid with keeping the sections filled with content that is present in the directory");
        ui[2].small("Save changes to config file through this page");
 
    });

    ui.columns(4, |ui|{

        if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Auto Discover") ).clicked()
        {
            config.auto_discover();
        }        
        if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Clear Programs and Powershell") ).clicked()
        {
            config.clear_pgr_scripts();
        }       
        if ui[2].add_sized([100.0, 40.0], egui::widgets::Button::new("Save Changes") ).clicked()
        {
            config.save_config();
        }
        if ui[3].add_sized([100.0, 40.0], egui::widgets::Button::new("Revert Changes") ).clicked()
        {
            config.validate();
        } 
    });
    ui.add_space(10.0);
    egui::ScrollArea::vertical().id_source("configoptions").show(ui, |ui|{
        ui.columns(2, |ui|{
            ui[0].heading("Pending Config");
            ui[0].add_space(10.0);
            ui[0].heading("Programs");
            for (key, value) in config.config.section(Some("Programs")).unwrap().clone().iter()
            {
                ui[0].horizontal(|ui|{
                    ui.label(key);
                    ui.label("=");
                    ui.label(value);
                });
            }
            ui[0].add_space(10.0);
            ui[0].heading("Powershell");
            for (key, value) in config.config.section(Some("Powershell")).unwrap().clone().iter()
            {
                ui[0].horizontal(|ui|{
                    ui.label(key);
                    ui.label("=");
                    ui.label(value);
                });
            }

            ui[1].heading("Current Config");
            ui[1].add_space(10.0);
            ui[1].heading("Programs");
            for (key, value) in config.config_copy.section(Some("Programs")).unwrap().clone().iter()
            {
                ui[1].horizontal(|ui|{
                    ui.label(key);
                    ui.label("=");
                    ui.label(value);
                });
            }
            ui[1].add_space(10.0);
            ui[1].heading("Powershell");
            for (key, value) in config.config_copy.section(Some("Powershell")).unwrap().clone().iter()
            {
                ui[1].horizontal(|ui|{
                    ui.label(key);
                    ui.label("=");
                    ui.label(value);
                });
            }
        });
    });
}