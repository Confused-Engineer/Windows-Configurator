use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::env;
use std::time::Duration;
use ini::{Error, Ini};
use crossbeam_channel::{bounded, Receiver, Sender};

use crate::pages_other::*;
use crate::pages_settings::*;
use crate::pages_troubleshooting::*;
use crate::pages_apps::*;

//use std::sync::{Arc, RwLock};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    
    page_main: bool,
    page_apps: bool,
    page_apps_winget: bool,
    page_apps_tokens: bool,
    page_troubleshoot: bool,
    page_microsoft_settings: bool,
    page_debug: bool,
    
    #[serde(skip)]
    win_settings_struct: WindowsSettings,

    #[serde(skip)]
    config: Config,
    #[serde(skip)]
    sys_struct: TroubleshootInfo,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            page_main: true,
            page_apps: false,
            page_apps_winget: false,
            page_apps_tokens: false,
            page_microsoft_settings: false,
            page_troubleshoot: false,
            page_debug: false,
            config: Config::default(),
            sys_struct: TroubleshootInfo::default(),
            win_settings_struct: WindowsSettings::default(),
            
        } 
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn set_blank(&mut self)
    {
        self.page_main = false;
        self.page_apps = false;
        self.page_apps_winget = false;
        self.page_apps_tokens = false;
        self.page_microsoft_settings = false;
        self.page_troubleshoot = false;
        self.page_debug = false;
    }  

    fn restart_admin(&mut self)
    {
        Command::new("powershell")
            .args(["start-process",env::current_exe().unwrap().as_path().display().to_string().as_str(), "-verb", "runas"])
            .creation_flags(0x08000000)
            .spawn()
            .expect("failed to execute process");
    }


}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui|{

                    if ui.button("Main").clicked()
                    {
                        self.set_blank();
                        self.page_main = true;
                        ui.close_menu();
                    }


                    ui.menu_button("Apps", |ui|{
                        if ui.button("Apps").clicked()
                        {
                            self.set_blank();
                            self.page_apps = true;
                            self.page_apps_winget = true;
                            ui.close_menu();
                        }

                        if ui.button("Tokens").clicked()
                        {
                            self.set_blank();
                            self.page_apps = true;
                            self.page_apps_tokens = true;
                            ui.close_menu();
                        }
                    });

                    if ui.button("Windows Settings").clicked()
                    {
                        self.set_blank();
                        self.page_microsoft_settings = true;
                        ui.close_menu();
                    }

                    ui.menu_button("Troubleshooting", |ui|{
                        if ui.button("General").clicked()
                        {
                            self.set_blank();
                            self.page_troubleshoot = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Options", |ui|{
                    if ui.button("Restart as Admin").clicked()
                    {
                        ui.close_menu();
                        self.restart_admin();
                    }

                    if ui.button("Reload Config").clicked()
                    {
                        ui.close_menu();
                        self.config.validate();
                    }
                });

                #[cfg(debug_assertions)]
                ui.menu_button("Debug", |ui|{
                    if ui.button("debug").clicked()
                    {
                        self.set_blank();
                        self.page_debug = true;
                        ui.close_menu();
                    }
                });
                ui.add_space(8.0);
                
                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });

        

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(20.0, eframe::epaint::FontFamily::Proportional),
            );
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Body, 
                egui::FontId::new(16.0, eframe::epaint::FontFamily::Proportional),
            );
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Heading, 
                egui::FontId::new(20.0, eframe::epaint::FontFamily::Proportional),
            );
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Small, 
                egui::FontId::new(15.0, eframe::epaint::FontFamily::Proportional),
            );      
                     
            if self.page_main
            {
                ui.heading("Main");
                ui.separator();
                page_main::show_page_main(ui);
            } 

            if self.page_apps
            {
                ui.heading("Apps");
                ui.label("The Apps Section is where you will find tools for installing, updating, and uninstalling certain applications.");
                ui.separator();

                if self.config.config_check.is_ok()
                {
                    
                    if self.page_apps_winget
                    {
                        page_apps::page_apps(ui, &self.config.config);
                    }
    
                    if self.page_apps_tokens
                    {
                        page_tokens::show_page_tokens(ui, &self.config.config);
                    }
                } else if self.config.config_check.is_err() {
                    page_config_error::show_page_config_error(ui, &mut self.config)
                }

            }
            
            if self.page_microsoft_settings
            {
                ui.heading("Windows Settings");
                ui.label("Change and Open Common Windows Settings");
                ui.separator();
                page_windows_settings::show_windows_settings(ui, &mut self.win_settings_struct);
            }

            if self.page_troubleshoot
            {
                page_troubleshoot::page_troubleshoot(ui, &mut self.sys_struct);
            }

            if self.page_debug 
            {
                page_debug::show_page_debug(ui, &self.config.config);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui|{
                    ui.hyperlink_to("Source Code",
                    "https://github.com/Confused-Engineer/Windows-Configurator/");
                    ui.label("Brought to you by a confused engineer!");
                    egui::warn_if_debug_build(ui);
                });
            });
        });
    }
}


pub struct Config
{
    config: Ini,
    config_check: Result<Ini, Error>
}

impl Default for Config
{
    fn default() -> Self {
        Self {
            config: Config::config_load(),
            config_check: Ini::load_from_file("config.ini")
        }
    }
}

impl Config
{
    pub fn validate(&mut self)
    {
        
        self.config_check = Ini::load_from_file("config.ini");
        if self.config_check.is_ok()
        {
            self.config = self.config_check.as_ref().unwrap().clone();
        }
    }

    fn config_load() -> Ini
    {
        let temp_config = Ini::load_from_file("config.ini");
        if temp_config.is_ok()
        {
            return temp_config.unwrap();
        } else {
            return Ini::new();
        }
    }
}


pub struct WindowsSettings {
    pub vpn_name: String,
    pub vpn_addr: String,
    pub vpn_secret_key: String,
    pub vpn_view_secret_key: bool,
    pub vpn_add_as_admin: bool,
    pub pc_name: String,
}

impl Default for WindowsSettings {
    fn default() -> Self {
        Self { vpn_name: String::new(),
            vpn_addr: String::new(),
            vpn_secret_key: String::new(),
            vpn_view_secret_key: false,
            vpn_add_as_admin: false,
            pc_name: String::new(),
        }
    }
}

pub struct TroubleshootInfo {
    pub ping: String,
    pub tracert: String,
    pub systeminfo: String,
    pub ipconfig: String,
    pub sys_info_receiver: crossbeam_channel::Receiver<std::string::String>,
    pub ipconfig_info_receiver: crossbeam_channel::Receiver<std::string::String>,
}


impl Default for TroubleshootInfo {
    fn default() -> Self {
        Self { 
            ping: String::new(),
            tracert: String::new(), 
            systeminfo: String::from("Loading"), 
            ipconfig: String::from("Loading"),
            sys_info_receiver: Self::set_sys_info(),
            ipconfig_info_receiver: Self::set_ipconfig(),
        }
    }
}

impl TroubleshootInfo {
    fn set_sys_info() -> crossbeam_channel::Receiver<std::string::String>
    {
        let tx2: Sender<String>;
        let rx2: Receiver<String>;
        (tx2, rx2) = bounded(1);

        std::thread::spawn(move || {
            loop {
                let stdout_string = String::from_utf8(
                    Command::new("systeminfo.exe")
                        .creation_flags(0x08000000)
                        .stdout(Stdio::piped())
                        .output()
                        .unwrap()
                        .stdout)
                .unwrap();
                let _ = tx2.send(stdout_string);
                std::thread::sleep(Duration::from_secs(10));
            }
        });
        rx2
    }

    fn set_ipconfig() -> crossbeam_channel::Receiver<std::string::String>
    {
        let tx: Sender<String>;
        let rx: Receiver<String>;
        (tx, rx) = bounded(1);
        
        std::thread::spawn(move || {
            loop {
                let stdout_string = String::from_utf8(
                    Command::new("ipconfig.exe")
                        .arg("/all")
                        .creation_flags(0x08000000)
                        .stdout(Stdio::piped())
                        .output()
                        .unwrap()
                        .stdout)
                .unwrap();
                let _ = tx.send(stdout_string);
                std::thread::sleep(Duration::from_secs(10));
            }
        });
        rx 
    }
}