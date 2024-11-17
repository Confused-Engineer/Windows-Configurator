use std::os::windows::process::CommandExt;

use crate::Configurator;
use eframe::egui;

impl Configurator
{
    pub fn page_settings_first(&mut self, ui: &mut egui::Ui)
    {

        ui.columns(3, |ui| {
            egui::ScrollArea::vertical().id_salt("settings_first_column").show(&mut ui[0], |ui| {

                if self.config.len() == 1
                {
                    ui.label(egui::RichText::new("Warning: No Config Found").heading().color(egui::Color32::RED));
                    ui.separator();

                    ui.label("There was an error loading the config file or it is missing. Create a new file with the button below.");
                    

                    let reset = ui.add_sized([25.0, 25.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/refresh-outline.svg"))
                    .frame(false)
                    .rounding(10.0)
                    .tint(egui::Color32::GREEN))
                    .on_hover_text("Reset Config File");

                    if reset.clicked()
                    {
                        let _ = std::fs::remove_file("config.ini");
                        let filemake = std::fs::File::create("config.ini");
                        if let Ok(mut file) = filemake {
                            let _ = std::io::Write::write_all(&mut file, include_bytes!("../../../assets/config.ini"));
                        }
                        
                        let temp = ini::Ini::load_from_file_noescape("config.ini");
                        if let Ok(config) = temp 
                        {
                            self.config = config;
                        }

                    }

                    



                    ui.separator();
                }
                ui.heading("Configuartion");
                ui.label(format!("Admin: {}", self.running_as_admin));
                ui.label("· Install, Update, and Remove Applications");
                ui.label("· Configure Common Windows Settings");
                ui.label("· Conveniently Launch Scripts and Programs");
                ui.label("· View System and Network Information");
                ui.label("· Help Provide Basic Troubleshooting Tools");
            });
        });
        
        
    }

    pub fn page_settings_second(&mut self, ui: &mut egui::Ui)
    {
        ui.columns(3, |ui| {
            ui[0].heading("Launch Windows Settings");
            ui[0].separator();
            egui::ScrollArea::vertical().id_salt("settings_second_first").show(&mut ui[0], |ui| {
                if let Ok(config) = ini::Ini::load_from_str(include_str!("../../../assets/resources/commands/win_settings"))
                {
                    display_commands(config, ui);
                }
            });

            ui[1].heading("Launch Control Panel");
            ui[1].separator();
            egui::ScrollArea::vertical().id_salt("settings_second_second").show(&mut ui[1], |ui| {
                if let Ok(config) = ini::Ini::load_from_str(include_str!("../../../assets/resources/commands/control_panel_settings"))
                {
                    display_commands(config, ui);
                }
            });

            ui[2].heading("Run System Commands");
            ui[2].separator();
            egui::ScrollArea::vertical().id_salt("settings_second_third").show(&mut ui[2], |ui| {
                if let Ok(config) = ini::Ini::load_from_str(include_str!("../../../assets/resources/commands/commands"))
                {
                    display_commands(config, ui);
                }
            });
        });
    }
}

fn program_launch(command: &str) {

    let app_vec: Vec<&str> = command.split(" ").collect();
    let mut command = std::process::Command::new("cmd");
    command.args(["/C","start"]);
    for x in 0..app_vec.len()
    {
        command.arg(app_vec[x]);
    }
    command.creation_flags(0x08000000);
    let _ = command.spawn();
}

fn display_commands(config: ini::Ini, ui: &mut egui::Ui)
{
    for section in config.sections()
                    {
                        if section.is_none() { continue; }
                        let section = section.unwrap();

                        ui.heading(section);
                        ui.separator();

                        if let Some(properties) = config.section(Some(section))
                        {
                            for (key, val) in properties
                            {
                                ui.horizontal(|ui| {
                                    ui.label(key);
                                            
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {

                                        ui.add_space(10.0);
        
                                        if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                                        .frame(false)
                                        .tint(egui::Color32::GREEN))
                                        .on_hover_text("Go-To")
                                        .clicked()
                                        {
                                            program_launch(val);
                                        }
                                    });
                                });
                                ui.separator();
                            }
                        }
                    }
}