use std::os::windows::process::CommandExt;

use crate::Configurator;
use eframe::egui;

impl Configurator
{
    pub fn page_troubleshooting_first(&mut self, ui: &mut egui::Ui)
    {
        ui.columns(3, |ui| {
            
            egui::ScrollArea::vertical().id_salt("troubleshooting_first_first").show(&mut ui[0], |ui| {
                ui.heading("Networking");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Ping:");
                    ui.text_edit_singleline(&mut self.ping);        
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {

                        ui.add_space(10.0);

                        if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                        .frame(false)
                        .tint(egui::Color32::GREEN))
                        .on_hover_text("Launch Ping")
                        .clicked()
                        {
                            program_launch(&format!("ping {} -a -t", self.ping));
                        }
                    });
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("PathPing:");
                    ui.text_edit_singleline(&mut self.ping);        
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {

                        ui.add_space(10.0);

                        if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                        .frame(false)
                        .tint(egui::Color32::GREEN))
                        .on_hover_text("Launch PathPing")
                        .clicked()
                        {
                            program_launch(&format!("pathping {} & pause", self.ping));
                        }
                    });
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Trace Route:");
                    ui.text_edit_singleline(&mut self.traceroute);        
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {

                        ui.add_space(10.0);

                        if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                        .frame(false)
                        .tint(egui::Color32::GREEN))
                        .on_hover_text("Launch TraceRoute")
                        .clicked()
                        {
                            program_launch(&format!("tracert {} & pause", self.traceroute));
                        }
                    });
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("NS Lookup:");
                    ui.text_edit_singleline(&mut self.nslookup);        
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {

                        ui.add_space(10.0);

                        if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                        .frame(false)
                        .tint(egui::Color32::GREEN))
                        .on_hover_text("Launch NSLookup")
                        .clicked()
                        {
                            program_launch(&format!("nslookup {} & pause", self.nslookup));
                        }
                    });
                });
                ui.separator();

                ui.horizontal(|ui| {
                    if self.running_as_admin | cfg!(debug_assertions)
                    {
                        ui.label("Flush System DNS");
                    } else {
                        ui.label("Flush User DNS");
                    }
                           
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {

                        ui.add_space(10.0);

                        if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                        .frame(false)
                        .tint(egui::Color32::GREEN))
                        .on_hover_text("Launch NSLookup")
                        .clicked()
                        {
                            program_launch_hidden("ipconfig /flushdns");
                        }
                    });
                });
                ui.separator();

                if self.running_as_admin | cfg!(debug_assertions)
                {
                    ui.add_space(5.0);
                    ui.heading("Admin Options:");
                    ui.separator();

                    if let Ok(config) = ini::Ini::load_from_str(include_str!("../../../assets/resources/commands/network_commands_admin.ini"))
                    {
                        display_config(config, ui);
                    }
                }
            });

            egui::ScrollArea::vertical().id_salt("troubleshooting_first_second").show(&mut ui[1], |ui| {
                ui.heading("System");
                ui.separator();
                
                if self.running_as_admin | cfg!(debug_assertions)
                {
                    ui.heading("Admin Options");
                    ui.separator();
                    if let Ok(config) = ini::Ini::load_from_str(include_str!("../../../assets/resources/commands/troubleshooting_commands_hidden.ini"))
                    {
                        display_config_hidden(config, ui);
                    }
                    //ui.separator();
                    if let Ok(config) = ini::Ini::load_from_str(include_str!("../../../assets/resources/commands/troubleshooting_commands.ini"))
                    {
                        display_config(config, ui);
                    }
                }
                

                
            });
        });
        
    }

    pub fn page_troubleshooting_second(&mut self, ui: &mut egui::Ui)
    {
        ui.columns(2, |ui| {
            
            egui::ScrollArea::vertical().id_salt("info_first_column").show(&mut ui[0], |ui| {
                ui.heading("System Info");
                ui.separator();
                ui.label(&self.systeminfo);
                ui.separator();
                ui.heading("NetStat");
                ui.separator();
                ui.label(&self.netstat);
                
            });

            egui::ScrollArea::vertical().id_salt("info_second_column").show(&mut ui[1], |ui| {
                ui.heading("IP Config");
                ui.separator();
                ui.label(&self.ipconfig);
                ui.separator();
                ui.heading("ARP");
                ui.separator();
                ui.label(&self.arp);
                ui.separator();
                ui.heading("Route Table");
                ui.separator();
                ui.label(&self.routetable);
                
            });
        });
    }
}

fn program_launch(command: &str) {

    let app_vec: Vec<&str> = command.split(" ").collect();
    let mut command = std::process::Command::new("cmd");
    command.arg("/C");
    for x in 0..app_vec.len()
    {
        command.arg(app_vec[x]);
    }
    //command.creation_flags(0x08000000);
    let _ = command.spawn();
}

fn program_launch_hidden(command: &str) {

    let app_vec: Vec<&str> = command.split(" ").collect();
    let mut command = std::process::Command::new("cmd");
    command.arg("/C");
    for x in 0..app_vec.len()
    {
        command.arg(app_vec[x]);
    }
    command.creation_flags(0x08000000);
    let _ = command.spawn();
}

fn display_config(config: ini::Ini, ui: &mut egui::Ui)
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

fn display_config_hidden(config: ini::Ini, ui: &mut egui::Ui)
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
                        .clicked()
                        {
                            program_launch_hidden(val);
                        }
                    });
                });
                ui.separator();
            }
        }


    }
}