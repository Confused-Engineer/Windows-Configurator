use std::os::windows::process::CommandExt;

use serde::{Deserialize, Serialize};


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Configurator {
    //#[serde(skip)]
    page: AppPage,
    #[serde(skip)]
    pub running_as_admin: bool,

    #[serde(skip)]
    pub close_app: bool,

    #[serde(skip)]
    multithreading_started: bool,

    pub winget_app_search: String,
    //#[serde(skip)]
    pub wingetlist: WingetList,
    #[serde(skip)]
    pub multithread_wingetlist: (std::sync::mpsc::Sender<WingetList>, std::sync::mpsc::Receiver<WingetList>),

    pub systeminfo: String,
    #[serde(skip)]
    pub multithread_systeminfo: (std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>),

    pub ipconfig: String,
    #[serde(skip)]
    pub multithread_ipconfig: (std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>),

    pub netstat: String,
    #[serde(skip)]
    pub multithread_netstat: (std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>),

    pub arp: String,
    #[serde(skip)]
    pub multithread_arp: (std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>),

    pub routetable: String,
    #[serde(skip)]
    pub multithread_routetable: (std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>),

    #[serde(skip)]
    pub config: ini::Ini,
    #[serde(skip)]
    pub multithread_config: (std::sync::mpsc::Sender<ini::Ini>, std::sync::mpsc::Receiver<ini::Ini>),

    pub ping: String,
    pub traceroute: String,
    pub nslookup: String,

    #[serde(skip)]
    pub vpn_name: String,
    #[serde(skip)]
    pub vpn_addr: String,
    #[serde(skip)]
    pub vpn_key: String,
    #[serde(skip)]
    pub vpn_view_key: bool,
    #[serde(skip)]
    pub pc_name: String,

}

impl Default for Configurator {
    fn default() -> Self {
        Self {

            page: AppPage::Main(SubPage::First),
            
            running_as_admin: is_admin(),
            close_app: false,
            multithreading_started: false,

            winget_app_search: String::new(),

            wingetlist: Vec::new(),
            multithread_wingetlist: std::sync::mpsc::channel(),

            systeminfo: String::new(),
            multithread_systeminfo: std::sync::mpsc::channel(),

            ipconfig: String::new(),
            multithread_ipconfig: std::sync::mpsc::channel(),

            netstat: String::new(),
            multithread_netstat: std::sync::mpsc::channel(),

            arp: String::new(),
            multithread_arp: std::sync::mpsc::channel(),

            routetable: String::new(),
            multithread_routetable: std::sync::mpsc::channel(),

            config: ini::Ini::load_from_file_noescape("config.ini").unwrap_or_default(),
            multithread_config: std::sync::mpsc::channel(),

            ping: String::new(),
            traceroute: String::new(),
            nslookup: String::new(),

            vpn_name: String::new(),
            vpn_addr: String::new(),
            vpn_key: String::new(),
            vpn_view_key: false,

            pc_name: String::new(),
        } 
    }
}

impl Configurator {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let visuals = egui::Visuals
        {
            dark_mode: true,
            //override_text_color: todo!(),
            //widgets: todo!(),
            //faint_bg_color: todo!(),
            //extreme_bg_color: todo!(),/
            //window_fill: todo!(),
            //panel_fill: todo!(),
            image_loading_spinners: true,
            ..Default::default()
        };

        cc.egui_ctx.set_visuals(visuals);

        //cc.egui_ctx.set_pixels_per_point(1.2);
        cc.egui_ctx.style_mut(|style| {
            style.text_styles.insert(
                egui::TextStyle::Heading, 
                egui::FontId::new(20.0, eframe::epaint::FontFamily::Proportional)
            );

            style.text_styles.insert(
                egui::TextStyle::Body, 
                egui::FontId::new(14.0, eframe::epaint::FontFamily::Proportional)
            );

            style.text_styles.insert(
                egui::TextStyle::Button, 
                egui::FontId::new(15.0, eframe::epaint::FontFamily::Proportional)
            );
        }); 

                
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

}

impl eframe::App for Configurator {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.multithreading_started
        {
            self.start_multithread();
            self.multithreading_started = true;
        }

        if self.close_app
        {
            std::process::Command::new("cmd")
                .args(["/C","msg", "%username%","Updating to latest version"])
                .creation_flags(0x08000000)
                .spawn()
                .expect("failed to execute process");

            let _ = std::process::Command::new("cmd")
                .args(["/C","timeout", "1","&","curl.exe","-L","https://github.com/Confused-Engineer/Windows-Configurator/releases/download/nightly/Windows_Configurator.exe","-o","Windows Configurator.exe","&","timeout","1","&","Windows Configurator.exe"])
                .creation_flags(0x08000000)
                .spawn()
                .expect("failed to execute process");

            
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            self.close_app = false;
        }

        self.unpack_multithread();
        egui::SidePanel::left("left_panel").resizable(false).exact_width(SIDE_PANEL_WIDTH).show(ctx, |ui| {
            
            let main = ui.add_sized(SIDE_BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../assets/resources/images/svg/home-outline.svg"))
                .frame(false)
                .selected(self.page == AppPage::Main(SubPage::First))
                .rounding(10.0)
                .tint(SIDE_BUTTON_COLOR))
                .on_hover_text("Main");

            if main.clicked()
            {
                self.page = AppPage::Main(SubPage::First);
            }

            let app = ui.add_sized(SIDE_BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../assets/resources/images/svg/arrow-circle-down-outline.svg"))
                .frame(false)
                .selected(self.page == AppPage::Applications(SubPage::First))
                .rounding(10.0)
                .tint(SIDE_BUTTON_COLOR))
                .on_hover_text("Install Applications");

            if app.clicked()
            {
                self.page = AppPage::Applications(SubPage::First);
            }

            let token = ui.add_sized(SIDE_BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../assets/resources/images/svg/edit-2-outline.svg"))
                .frame(false)
                .selected(self.page == AppPage::Applications(SubPage::Second))
                .rounding(10.0)
                .tint(SIDE_BUTTON_COLOR))
                .on_hover_text("Application Tokens");

            if token.clicked()
            {
                self.page = AppPage::Applications(SubPage::Second);
            }

            let launch = ui.add_sized(SIDE_BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../assets/resources/images/svg/external-link-outline.svg"))
                .frame(false)
                .selected(self.page == AppPage::Settings(SubPage::Second))
                .rounding(10.0)
                .tint(SIDE_BUTTON_COLOR))
                .on_hover_text("Launch Windows Settings");

            if launch.clicked()
            {
                self.page = AppPage::Settings(SubPage::Second);
            }

            let troubleshoot = ui.add_sized(SIDE_BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../assets/resources/images/svg/printer-outline.svg"))
                .frame(false)
                .selected(self.page == AppPage::TroubleShooting(SubPage::First))
                .rounding(10.0)
                .tint(SIDE_BUTTON_COLOR))
                .on_hover_text("Troubleshooting Options");

            if troubleshoot.clicked()
            {
                self.page = AppPage::TroubleShooting(SubPage::First);
            }

            let info = ui.add_sized(SIDE_BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../assets/resources/images/svg/info-outline.svg"))
                .frame(false)
                .selected(self.page == AppPage::TroubleShooting(SubPage::Second))
                .rounding(10.0)
                .tint(SIDE_BUTTON_COLOR))
                .on_hover_text("System Information");

            if info.clicked()
            {
                self.page = AppPage::TroubleShooting(SubPage::Second);
            }

            let config = ui.add_sized(SIDE_BUTTON_SIZE, egui::ImageButton::new(egui::include_image!("../assets/resources/images/svg/settings-2-outline.svg"))
                .frame(false)
                .selected(self.page == AppPage::Settings(SubPage::First))
                .rounding(10.0)
                .tint(if self.config.len() == 1 { egui::Color32::RED } else { SIDE_BUTTON_COLOR }))
                .on_hover_text("Settings and Options");

            if config.clicked()
            {
                self.page = AppPage::Settings(SubPage::First);
            }

            
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.page {
                AppPage::Main(sub_page) => {
                    ui.heading("Welcome");
                    ui.separator();

                    match sub_page {
                        SubPage::First => self.page_main_first(ui),
                        SubPage::Second => todo!(),
                        SubPage::Third => todo!(),
                    }
                },
                AppPage::Applications(sub_page) => {
                    ui.heading("Applications");
                    ui.separator();
                    match sub_page {
                        SubPage::First => self.page_applications_winget(ui),
                        SubPage::Second => self.page_applications_tokens(ui),
                        SubPage::Third => todo!(),
                    }
                },
                AppPage::Settings(sub_page) => {
                    ui.heading("Settings");
                    ui.separator();
                    match sub_page {
                        SubPage::First => self.page_settings_first(ui),
                        SubPage::Second => self.page_settings_second(ui),
                        SubPage::Third => todo!(),
                    }
                },
                AppPage::TroubleShooting(sub_page) => {
                    ui.heading("Troubleshooting");
                    ui.separator();
                    match sub_page {
                        SubPage::First => self.page_troubleshooting_first(ui),
                        SubPage::Second => self.page_troubleshooting_second(ui),
                        SubPage::Third => todo!(),
                    }
                },
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                source(ui);
            });
        });
        
    }
}


fn source(ui: &mut egui::Ui)
{
    ui.horizontal(|ui|{
        ui.hyperlink_to("Source Code",
        "https://github.com/Confused-Engineer/Windows-Configurator/");
        ui.label("Brought to you by a confused engineer!");
        egui::warn_if_debug_build(ui);
    });
}

const SIDE_BUTTON_SIZE: [f32; 2] = [50.0,50.0];
const SIDE_BUTTON_COLOR: egui::Color32 = egui::Color32::GOLD;
const SIDE_PANEL_WIDTH: f32 = 65.0;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
enum AppPage {
    Main(SubPage),
    Applications(SubPage),
    Settings(SubPage),
    TroubleShooting(SubPage),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
enum SubPage {
    First,
    Second,
    Third
}


pub type WingetList = Vec<Winget>;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Winget
{
    pub name: String,
    pub id: String,
    pub version: String
}

fn is_admin() -> bool
{
    if let Ok(_) = get_perms()
    {
        return true
    }
    false
}

use windows::{
    core::*, Win32::Foundation::*, Win32::Security::*, Win32::System::Memory::*,
    Win32::System::Threading::*,
};

fn get_perms() -> Result<()>
{
    let mut perm_vec: Vec<String> = Vec::new();
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        let mut bytes_required = 0;
        _ = GetTokenInformation(token, TokenPrivileges, None, 0, &mut bytes_required);

        let buffer = Owned::new(LocalAlloc(LPTR, bytes_required as usize)?);

        GetTokenInformation(
            token,
            TokenPrivileges,
            Some(buffer.0 as *mut _),
            bytes_required,
            &mut bytes_required,
        )?;

        let header = &*(buffer.0 as *const TOKEN_PRIVILEGES);

        let privileges =
            std::slice::from_raw_parts(header.Privileges.as_ptr(), header.PrivilegeCount as usize);

        for privilege in privileges {
            let mut name_len = 0;
            _ = LookupPrivilegeNameW(None, &privilege.Luid, PWSTR::null(), &mut name_len);

            let mut name = vec![0u16; (name_len + 1) as usize];
            let name = PWSTR(name.as_mut_ptr());
            LookupPrivilegeNameW(None, &privilege.Luid, name, &mut name_len)?;

            

            perm_vec.push(name.display().to_string());
        }

        for entry in perm_vec
        {
            if (entry == "SeSystemProfilePrivilege".to_owned()) || (entry == "SeSecurityPrivilege".to_owned())
            {
                return Ok(())
            }
        }
        Err(Error::empty())
    }
}