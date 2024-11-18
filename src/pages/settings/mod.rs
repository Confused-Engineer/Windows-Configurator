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
                ui.heading("Configuration");
                ui.label(format!("Admin: {}", self.running_as_admin));
                ui.label("· Install, Update, and Remove Applications");
                ui.label("· Configure Common Windows Settings");
                ui.label("· Conveniently Launch Scripts and Programs");
                ui.label("· View System and Network Information");
                ui.label("· Help Provide Basic Troubleshooting Tools");
            });

            egui::ScrollArea::vertical().id_salt("settings_second_column").show(&mut ui[1], |ui| {
                ui.heading("VPN");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.vpn_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Addr:");
                    ui.text_edit_singleline(&mut self.vpn_addr);
                });
                ui.horizontal(|ui| {
                    ui.label("Key:");
                    ui.add(egui::text_edit::TextEdit::singleline(&mut self.vpn_key).password(!self.vpn_view_key));
                    ui.checkbox(&mut self.vpn_view_key, "View Key");
                });

                ui.columns(3, |ui| {
                    if ui[0].button(egui::RichText::new("Add MSCHAPv2").color(egui::Color32::GREEN)).clicked()
                    {
                        if self.running_as_admin
                        {
                            powershell_launch(&format!("add-VpnConnection -AlluserConnection -Name '{}' -ServerAddress {} -tunneltype L2tp -EncryptionLevel Optional -L2tpPsk {} -AuthenticationMethod MSChapv2 -Force", self.vpn_name, self.vpn_addr, self.vpn_key));
                        } else {
                            powershell_launch(&format!("add-VpnConnection -Name '{}' -ServerAddress {} -tunneltype L2tp -EncryptionLevel Optional -L2tpPsk {} -AuthenticationMethod MSChapv2 -Force", self.vpn_name, self.vpn_addr, self.vpn_key));
                        }
                        
                    }

                    if ui[1].button(egui::RichText::new("Add PAP").color(egui::Color32::YELLOW)).clicked()
                    {
                        if self.running_as_admin
                        {
                            powershell_launch(&format!("add-VpnConnection -AlluserConnection -Name '{}' -ServerAddress {} -tunneltype L2tp -EncryptionLevel Optional -L2tpPsk {} -AuthenticationMethod Pap -Force", self.vpn_name, self.vpn_addr, self.vpn_key));
                        } else {
                            powershell_launch(&format!("add-VpnConnection -Name '{}' -ServerAddress {} -tunneltype L2tp -EncryptionLevel Optional -L2tpPsk {} -AuthenticationMethod Pap -Force", self.vpn_name, self.vpn_addr, self.vpn_key));
                        }
                        
                    }

                    if ui[2].button(egui::RichText::new("Remove VPN").color(egui::Color32::RED)).clicked()
                    {
                        if self.running_as_admin
                        {
                            powershell_launch(&format!("Get-VpnConnection -AlluserConnection | Remove-VpnConnection -Force"));
                        } else {
                            powershell_launch(&format!("Get-VpnConnection | Remove-VpnConnection -Force"));
                        }
                        
                    }
                });

                ui.label("Add MSCHAPv2. PAP, or Remove, respectfully. Removing VPNs will remove the VPN for everyone if running as admin, otherwise it will only effect the current user.");

                ui.separator();

                if self.running_as_admin | cfg!(debug_assertions)
                {
                    ui.heading("Rename PC");
                    ui.horizontal(|ui| {
                        //todo textbox
                        ui.text_edit_singleline(&mut self.pc_name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                                .frame(false)
                                .tint(egui::Color32::GREEN))
                                .on_hover_text("Rename PC. Restart to take effect.")
                                .clicked()
                            {
                                powershell_launch(&format!("rename-computer -NewName '{}'", self.pc_name));
                            }
    
                        });
                    });
                }

            });

            egui::ScrollArea::vertical().id_salt("settings_third_column").show(&mut ui[2], |ui| {
                ui.horizontal(|ui| {
                    //todo textbox
                    ui.heading("Perform Self Update");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {

                        if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                            .frame(false)
                            .tint(egui::Color32::GREEN))
                            .on_hover_text("Closes and updates the application")
                            .clicked()
                        {
                            self.close_app = true;
                        }

                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    //todo textbox
                    if !self.running_as_admin
                    {
                        ui.heading("Start as Admin");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                                .frame(false)
                                .tint(egui::Color32::GREEN))
                                .on_hover_text("Launches another instance as admin")
                                .clicked()
                            {
                                if let Ok(exe_path) = std::env::current_exe()
                                {
                                    let _ = std::process::Command::new("powershell")
                                    .args(["start-process", &format!("\"{}\"", exe_path.display()), "-verb", "runas"])
                                    .creation_flags(0x08000000)
                                    .spawn();
                                }
                                
                            }
    
                        });
                    } else {
                        ui.heading("Running as Admin");
                    }
                    
                });
            });
        });
        
        
    }

    pub fn page_settings_second(&mut self, ui: &mut egui::Ui)
    {
        ui.columns(3, |ui| {
            ui[0].heading("Launch Windows Settings");
            ui[0].separator();
            egui::ScrollArea::vertical().id_salt("settings_second_first").show(&mut ui[0], |ui| {
                if let Ok(config) = ini::Ini::load_from_str(include_str!("../../../assets/resources/commands/win_settings.ini"))
                {
                    display_commands(config, ui);
                }
            });

            ui[1].heading("Launch Control Panel");
            ui[1].separator();
            egui::ScrollArea::vertical().id_salt("settings_second_second").show(&mut ui[1], |ui| {
                if let Ok(config) = ini::Ini::load_from_str_noescape(include_str!("../../../assets/resources/commands/control_panel_settings.ini"))
                {
                    display_commands(config, ui);
                }
            });

            ui[2].heading("Run System Commands");
            ui[2].separator();
            egui::ScrollArea::vertical().id_salt("settings_second_third").show(&mut ui[2], |ui| {
                if let Ok(config) = ini::Ini::load_from_str(include_str!("../../../assets/resources/commands/commands.ini"))
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

fn powershell_launch(command: &str) {

    let app_vec: Vec<&str> = command.split(" ").collect();
    let mut command = std::process::Command::new("powershell");
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