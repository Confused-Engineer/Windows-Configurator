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
                    ui.heading("Admin Options:");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("IP Re-Register DNS Names");     
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            ui.add_space(10.0);
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                            .frame(false)
                            .tint(egui::Color32::GREEN))
                            .clicked()
                            {
                                program_launch("ipconfig /registerdns");
                            }
                        });
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("IP Config Release");     
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            ui.add_space(10.0);
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                            .frame(false)
                            .tint(egui::Color32::GREEN))
                            .clicked()
                            {
                                program_launch("ipconfig /release");
                            }
                        });
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("IP Config Renew");     
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            ui.add_space(10.0);
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                            .frame(false)
                            .tint(egui::Color32::GREEN))
                            .clicked()
                            {
                                program_launch("ipconfig /renew");
                            }
                        });
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Winsock Reset (Requires Manual Restart)");     
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            ui.add_space(10.0);
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                            .frame(false)
                            .tint(egui::Color32::GREEN))
                            .clicked()
                            {
                                program_launch("netsh winsock reset");
                            }
                        });
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Turn On Windows Firewall");     
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            ui.add_space(10.0);
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                            .frame(false)
                            .tint(egui::Color32::GREEN))
                            .clicked()
                            {
                                program_launch("powershell Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True");
                            }
                        });
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Turn Off Windows Firewall");     
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            ui.add_space(10.0);
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                            .frame(false)
                            .tint(egui::Color32::GREEN))
                            .clicked()
                            {
                                program_launch("powershell Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled False");
                            }
                        });
                    });
                    ui.separator();
                }
            });

            egui::ScrollArea::vertical().id_salt("troubleshooting_first_second").show(&mut ui[1], |ui| {
                ui.heading("System");
                ui.separator();
                
                if self.running_as_admin | cfg!(debug_assertions)
                {
                    ui.heading("Admin Options");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Free Excess RAM");     
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    
                            ui.add_space(10.0);
    
                            if ui.add_sized([20.0, 20.0], egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/external-link-outline.svg"))
                            .frame(false)
                            .tint(egui::Color32::GREEN))
                            .on_hover_text("Free RAM")
                            .clicked()
                            {
                                program_launch_hidden("powershell -windowstyle hidden -EncodedCommand PAAjAA0ACgAgACAAIAAgAC4AUwBZAE4ATwBQAFMASQBTAA0ACgAgACAAIAAgAFQAaABpAHMAIAB0AG8AbwBsACAAdQBzAGUAcwAgAGEAIABwAHIAbwBnAHIAYQBtACAAbwBmAGYAZQByAGUAZAAgAGIAeQAgAG0AaQBjAHIAbwBzAG8AZgB0ACAAdABvACAAYwBsAGUAYQByACAAUgBBAE0AIAB3AGkAdABoAG8AdQB0ACAAYQAgAHIAZQBiAG8AbwB0ACAAbwByACAAdQBzAGUAcgAgAGkAbgB0AGUAcgBhAGMAdABpAG8AbgANAAoADQAKAA0ACgAgACAAIAAgAC4ARABFAFMAQwBSAEkAUABUAEkATwBOAA0ACgAgACAAIAAgAFIAQQBNAE0AYQBwAC4AZQB4AGUAIABpAHMAIABkAG8AdwBuAGwAbwBhAGQAZQBkACAAZgByAG8AbQAgAGgAdAB0AHAAcwA6AC8ALwBsAGkAdgBlAC4AcwB5AHMAaQBuAHQAZQByAG4AYQBsAHMALgBjAG8AbQAvAFIAQQBNAE0AYQBwAC4AZQB4AGUAIAB3AGgAaQBjAGgAIABpAHMAIABvAHcAbgBlAGQAIABhAG4AZAAgAG8AcABlAHIAYQB0AGUAZAAgAGIAeQAgAE0AaQBjAHIAbwBzAG8AZgB0ACAAYQBuAGQAIABvAGYAZgBlAHIAcwAgAGMAbwBtAG0AYQBuAGQAIABsAGkAbgBlACAAbwBwAHQAaQBvAG4AcwAgAHQAbwAgAGMAbABlAGEAcgAgAFIAQQBNAC4ADQAKACAAIAAgACAAQgB5ACAAZABvAHcAbgBsAG8AYQBkAGkAbgBnACAAaQB0ACAAdABvACAAYQAgAGQAaQByAGUAYwB0AG8AcgB5ACAAYQBuAGQAIAByAHUAbgBuAGkAbgBnACAAaQB0ACAAYQBzACAAYQBkAG0AaQBuAGkAcwB0AHIAYQB0AG8AcgAgAHcAZQAgAGEAcgBlACAAYQBiAGwAZQAgAHQAbwAgAGMAbABlAGEAcgAgAG8AdQB0ACAAUgBBAE0AIAB3AGkAdABoAG8AdQB0ACAAYQBuAHkAIAB1AHMAZQByACAAaQBuAHQAZQByAGEAYwB0AGkAbwBuACAAbwByACAAaQBuAHQAZQByAHUAcAB0AGkAbwBuAHMALgANAAoADQAKACMAPgANAAoADQAKACMAVgBhAHIAaQBhAGIAbABlAHMADQAKAA0ACgAkAFcAZQBiAEYAaQBsAGUAIAA9ACAAJwBoAHQAdABwAHMAOgAvAC8AbABpAHYAZQAuAHMAeQBzAGkAbgB0AGUAcgBuAGEAbABzAC4AYwBvAG0ALwBSAEEATQBNAGEAcAAuAGUAeABlACcADQAKACQATABvAGMAYQBsAEYAaQBsAGUAIAA9ACAAJwBDADoAXAB0AGUAbQBwAFwAUgBBAE0ATQBhAHAALgBlAHgAZQAnAA0ACgAkAEEAcgBnAEwAaQBzAHQAIAA9ACAAQAAoACIALQBFAHcAIgAsACIALQBFAHMAIgAsACIALQBFAG0AIgAsACIALQBFAHQAIgAsACIALQBFADAAIgApAA0ACgANAAoAJABSAGUAZwBQAGEAdABoACAAPQAgACIASABLAEMAVQA6AFwAUwBvAGYAdAB3AGEAcgBlAFwAUwB5AHMAaQBuAHQAZQByAG4AYQBsAHMAXABSAEEATQBNAGEAcAAiAA0ACgAkAFIAZQBnAEkAdABlAG0AIAA9ACAAIgBFAHUAbABhAEEAYwBjAGUAcAB0AGUAZAAiAA0ACgAkAFIAZQBnAFYAYQBsAHUAZQAgAD0AIAAxAA0ACgANAAoADQAKACMATQBhAGsAZQAgAFQAZQBtAHAAIABmAG8AbABkAGUAcgAgAGkAZgAgAGkAdAAgAGQAbwBlAHMAbgB0ACAAZQB4AGkAcwB0AA0ACgANAAoATgBlAHcALQBJAHQAZQBtACAALQBJAHQAZQBtAFQAeQBwAGUAIABEAGkAcgBlAGMAdABvAHIAeQAgAC0AUABhAHQAaAAgACIAQwA6AFwAdABlAG0AcAAiACAALQBFAHIAcgBvAHIAQQBjAHQAaQBvAG4AIABTAGkAbABlAG4AdABsAHkAQwBvAG4AdABpAG4AdQBlAA0ACgANAAoAIwBkAG8AdwBuAGwAbwBhAGQAIABuAGUAZQBkAGUAZAAgAHAAcgBvAGcAcgBhAG0ADQAKAA0ACgBJAG4AdgBvAGsAZQAtAFcAZQBiAFIAZQBxAHUAZQBzAHQAIAAtAFUAcgBpACAAJABXAGUAYgBGAGkAbABlACAALQBPAHUAdABGAGkAbABlACAAJABMAG8AYwBhAGwARgBpAGwAZQANAAoADQAKACMAQQBkAGQAIAByAGUAZwBpAHMAdAByAHkAIAB2AGEAbAB1AGUAIABzAG8AIAB0AGgAZQAgAEUAVQBMAEEAIABkAG8AZQBzACAAbgBvAHQAIABwAG8AcAAgAHUAcAAgAGYAbwByACAAdQBzAGUAcgAsACAAbgBlAGUAZABlAGQAIAB0AGgAZQAgAHMAbABlAGUAcAAgAHMAbwAgAHQAaABlACAAcgBlAGcAaQBzAHQAcgB5ACAAYwBhAG4AIABjAGwAbwBzAGUAIABiAGUAZgBvAHIAZQAgAFIAQQBNAE0AYQBwACAAdAByAGkAZQBzACAAdABvACAAYwBoAGUAYwBrACAAaQB0ACAAKABJACAAdABoAGkAbgBrACkADQAKAGkAZgAoACgAKABHAGUAdAAtAEkAdABlAG0AUAByAG8AcABlAHIAdAB5ACAALQBQAGEAdABoACAAJABSAGUAZwBQAGEAdABoACAALQBOAGEAbQBlACAAJABSAGUAZwBJAHQAZQBtACAALQBFAHIAcgBvAHIAQQBjAHQAaQBvAG4AIABTAGkAbABlAG4AdABsAHkAQwBvAG4AdABpAG4AdQBlACkALgAkAFIAZQBnAEkAdABlAG0AKQAgAC0AbgBlACAAJABSAGUAZwBWAGEAbAB1AGUAKQAgACAAewANAAoADQAKACAAIAAgACAATgBlAHcALQBJAHQAZQBtACAALQBQAGEAdABoACAAJABSAGUAZwBQAGEAdABoACAALQBGAG8AcgBjAGUAIAB8ACAATwB1AHQALQBOAHUAbABsAA0ACgAgACAAIAAgAE4AZQB3AC0ASQB0AGUAbQBQAHIAbwBwAGUAcgB0AHkAIAAtAFAAYQB0AGgAIAAkAFIAZQBnAFAAYQB0AGgAIAAtAE4AYQBtAGUAIAAkAFIAZQBnAEkAdABlAG0AIAAtAFYAYQBsAHUAZQAgACQAUgBlAGcAVgBhAGwAdQBlACAALQBQAHIAbwBwAGUAcgB0AHkAVAB5AHAAZQAgAEQAVwBPAFIARAAgAC0ARgBvAHIAYwBlACAAfAAgAE8AdQB0AC0ATgB1AGwAbAANAAoADQAKAH0ADQAKAA0ACgBTAHQAYQByAHQALQBTAGwAZQBlAHAAIAAtAFMAZQBjAG8AbgBkAHMAIAAxAA0ACgANAAoADQAKACMAUwB0AGEAcgB0ACAAZQBhAGMAaAAgAGEAbgBkACAAdABoAGUAbgAgAHMAbABlAGUAcAAgAGYAbwByACAAWAAgAHMAZQBjAG8AbgBkAHMAIAB0AG8AIABhAHYAbwBpAGQAIAAiAHIAcABvAGMAZQBzAHMAIABiAGUAaQBuAGcAIAB1AHMAZQBkACAAYgB5ACAAYQBuAG8AdABoAGUAcgAgAHAAcgBvAGcAcgBhAG0AIgAgAGUAcgByAG8AcgANAAoAZgBvAHIAZQBhAGMAaAAgACgAJABBAHIAZwAgAGkAbgAgACQAQQByAGcATABpAHMAdAApAHsADQAKACAAIAAgACAAJgAgACIAJABMAG8AYwBhAGwARgBpAGwAZQAiACAAQAAoACIAJABBAHIAZwAiACkADQAKACAAIAAgACAAUwB0AGEAcgB0AC0AUwBsAGUAZQBwACAAMgAwAA0ACgB9AA0ACgANAAoADQAKACMAYwBvAG0AbQBlAG4AdABlAGQAIABvAHUAdAAgAGIAdQB0ACAAdABoAGkAcwAgAGkAcwAgAHcAaABhAHQAIABpAHMAIABuAGUAZQBkAGUAZAAgAHQAbwAgAHMAaABvAHcAIABjAG8AbQBtAGEAbgBkACAAbABpAG4AZQAgAGEAcgBnAHUAbQBlAG4AdABzAA0ACgAjACYAIAAiACQATABvAGMAYQBsAEYAaQBsAGUAIgAgAEAAKAAiAC0ALQBoAGUAbABwACIAKQA=");
                            }
                        });
                    });
                    ui.separator();
                }
                
                ui.label("commands in resources");
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
