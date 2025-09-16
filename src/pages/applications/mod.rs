use std::os::windows::process::CommandExt;

use crate::Configurator;
use eframe::egui;

const BUTTON_SIZE: [f32; 2] = [25.0, 25.0];
impl Configurator
{
    pub fn page_applications_winget(&mut self, ui: &mut egui::Ui)
    {
        ui.columns(3, |ui| {
            ui[0].heading("Favorites");
            ui[0].separator();
            egui::ScrollArea::vertical().id_salt("app_winget_left").show(&mut ui[0], |ui|
            {

                for section in self.config.sections()
                {

                    if section.is_none() { continue; }

                    let section = section.unwrap();

                    if !section.contains("winget-"){ continue; }

                    if let Some(title) = section.split("-").last()
                    {
                        ui.vertical_centered(|ui| {
                            ui.heading(title);
                        });
                        
                        ui.separator();

                        
                        if let Some(properties) = self.config.section(Some(section))
                        {
                            for (item, id) in properties.iter()
                            {
                                ui.horizontal(|ui| {
                                    ui.label(item);
                                            
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {

                                        ui.add_space(10.0);
        
                                        if ui.add_sized(BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/volume-off-outline.svg"))
                                        //.frame(false)
                                        .corner_radius(5.0)
                                        .tint(egui::Color32::ORANGE))
                                        .on_hover_text("Install Silently")
                                        .clicked()
                                        {
                                            winget_install_silent(id.to_owned());
                                        }
        
                                        ui.add_space(7.0);
        
                                        if ui.add_sized(BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/arrow-circle-down-outline.svg"))
                                        //.frame(false)
                                        .corner_radius(5.0)
                                        .tint(egui::Color32::GREEN))
                                        .on_hover_text("Install")
                                        .clicked()
                                        {
                                            winget_install(id.to_owned());
                                        }
                                    });
                                });
                                ui.separator();
                            }
                        }
                    }
                }
            });

            ui[1].heading("Search All");
            ui[1].horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.winget_app_search);
            });
            ui[1].separator();
            egui::ScrollArea::vertical().id_salt("app_winget_mid").show(&mut ui[1], |ui|
            {
                for item in self.wingetlist.clone().into_iter()
                {
                    if item.name.to_ascii_lowercase().contains(&self.winget_app_search.clone().to_ascii_lowercase()) && !self.winget_app_search.clone().is_empty()
                    {
                        ui.horizontal(|ui| {
                            ui.label(&item.name);
                            ui.label("|");
                            ui.label(&item.version);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                ui.add_space(10.0);

                                if ui.add_sized(BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/volume-off-outline.svg"))
                                //.frame(false)
                                .corner_radius(5.0)
                                .tint(egui::Color32::ORANGE))
                                .on_hover_text("Install Silently")
                                .clicked()
                                {
                                    winget_install_silent(item.id.clone());
                                }

                                ui.add_space(7.0);

                                if ui.add_sized(BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/arrow-circle-down-outline.svg"))
                                //.frame(false)
                                .corner_radius(5.0)
                                .tint(egui::Color32::GREEN))
                                .on_hover_text("Install")
                                .clicked()
                                {
                                    winget_install(item.id);
                                }

                                
                            });

                            
                        });
                        ui.separator();
                    }
                }
            });
            
            
            ui[2].heading("Others");
            ui[2].separator();
            egui::ScrollArea::vertical().id_salt("app_winget_right").show(&mut ui[2], |ui| {
                ui.heading("Winget");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Update All Applications");
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.add_space(10.0);

                        if ui.add_sized(BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/upload-outline.svg"))
                        //.frame(false)
                        .corner_radius(5.0)
                        .tint(egui::Color32::GREEN))
                        .on_hover_text("Update any app winget can upgrade, regardless if it was installed with winget")
                        .clicked()
                        {
                            winget_update_all_silent();
                        }
                    });
                });

                ui.separator();
                
                for section in self.config.sections()
                {
                    if section.is_none() { continue; };
                    let section = section.unwrap();

                    if section.contains("winget-") || section.contains("Tokens") { continue; }

                    ui.heading(section);
                    ui.separator();

                    if let Some(properties) = self.config.section(Some(section))
                    {
                        for (item, name) in properties
                        {
                            ui.horizontal(|ui| {
                                ui.label(item);
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    ui.add_space(10.0);
    
                                    if ui.add_sized(BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../../../assets/resources/images/svg/arrow-circle-down-outline.svg"))
                                    //.frame(false)
                                    .corner_radius(5.0)
                                    .tint(egui::Color32::GREEN))
                                    .on_hover_text("Run or download and run")
                                    .clicked()
                                    {
                                        match section {
                                            "Programs" => {
                                                program_launch(name);
                                            },
                    
                                            "Powershell" => {
                                                powershell_launch(name);
                                            },

                                            "EXE-Online" => {
                                                download_and_run(Extensions::EXE, item, name);
                                            },

                                            "MSI-Online" => {
                                                download_and_run(Extensions::MSI, item, name);
                                            },

                                            "MSIX-Online" => {
                                                download_and_run(Extensions::MSIX, item, name);
                                            },
                    
                                            _ => {},
                                        }
                                    }
                                });
                            });
                            ui.separator();
                        }
                    }
                    

                }
            });
        });
        
    }

    pub fn page_applications_tokens(&mut self, ui: &mut egui::Ui)
    {
        ui.heading("Tokens");
        ui.separator();
        
        if let Some(properties) = self.config.section(Some("Tokens"))
        {
            for (name, token) in properties
            {
                ui.horizontal(|ui| {
                    ui.label(name.to_owned() + ":");
                    ui.label(token);
                });
                ui.separator();
            }
        }
        
        
    }
}

fn winget_install(app_id: String)
{
    let _ = std::process::Command::new("winget")
        .args(["install", &app_id,"--accept-source-agreements","--accept-package-agreements","--silent"])
        .spawn();
}

fn winget_install_silent(app_id: String)
{
    let _ = std::process::Command::new("winget")
        .args(["install", &app_id,"--accept-source-agreements","--accept-package-agreements","--silent"])
        .creation_flags(0x08000000)
        .spawn();
}

fn winget_update_all_silent()
{
    let _ = std::process::Command::new("winget")
        .args(["update", "--all","--accept-source-agreements","--accept-package-agreements","--silent", "--include-unknown"])
        .spawn();
}

fn program_launch(program: &str)
{
    let _ = std::process::Command::new(program)
        .spawn(); 
}

fn powershell_launch(powershell: &str)
{
    let _ = std::process::Command::new("powershell")
    .args(["-executionpolicy","bypass","-File", powershell])
    .spawn()
    .expect("failed to execute process");  
}

fn download_and_run(ext: Extensions, name: &str, link: &str)
{
    #[allow(unused_assignments)]
    let mut filename = String::new();
    match ext {
        Extensions::EXE => {
            filename = format!("{}.exe", name)
        },
        Extensions::MSI => {
            filename = format!("{}.msi", name)
        },
        Extensions::MSIX => {
            filename = format!("{}.exe", name)
        },
    }

        
    let uri = link.to_owned();
    std::thread::spawn(move || {

        if let Ok(_) = download(&filename, &uri)
        {
            program_launch(&filename);
        }
    });
}

fn download(filename: &str, uri: &str) -> std::io::Result<()>
{

    if let Ok(mut download) = reqwest::blocking::get(uri)
    {
        if std::path::Path::exists(&std::path::Path::new(filename))
        {
            let _ = std::fs::remove_file(filename);
        }

        // Create the file
        let mut out_file = std::fs::File::create(filename)?;

        // Write the bytes to the file
        let _ = std::io::copy(&mut download, &mut out_file)?;

    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Could Not Download File"));
    }

    Ok(())
}

enum Extensions {
    EXE,
    MSI,
    MSIX,
}