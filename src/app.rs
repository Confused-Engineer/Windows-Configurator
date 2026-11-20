use std::collections::VecDeque;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use eframe::egui::{self, ImageSource};

use crate::config::{self, Config};
use crate::sysinfo::SysInfo;
use crate::tasks::{Bus, Event};
use crate::ui::{self, icons};
use crate::{platform, winget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Home,
    Apps,
    Tokens,
    Launch,
    Troubleshoot,
    SysInfo,
    Settings,
}

impl Page {
    const ALL: [Self; 7] = [
        Self::Home,
        Self::Apps,
        Self::Tokens,
        Self::Launch,
        Self::Troubleshoot,
        Self::SysInfo,
        Self::Settings,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Home => "Welcome",
            Self::Apps => "Install Applications",
            Self::Tokens => "Application Tokens",
            Self::Launch => "Launch Windows Settings",
            Self::Troubleshoot => "Troubleshooting",
            Self::SysInfo => "System Information",
            Self::Settings => "Settings and Options",
        }
    }

    fn icon(self) -> ImageSource<'static> {
        match self {
            Self::Home => icons::home(),
            Self::Apps => icons::download(),
            Self::Tokens => icons::key(),
            Self::Launch => icons::launch(),
            Self::Troubleshoot => icons::tools(),
            Self::SysInfo => icons::info(),
            Self::Settings => icons::settings(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WingetStatus {
    Checking,
    Missing,
    Installing,
    Available(String),
}

/// winget availability, catalog and search state.
pub struct WingetState {
    pub status: WingetStatus,
    pub catalog: Vec<winget::Package>,
    pub catalog_error: Option<String>,
    pub search: String,
    /// Indices into `catalog` matching `search`, recomputed only when the
    /// query (`filtered_for`) or the catalog changes.
    pub filtered: Vec<usize>,
    filtered_for: Option<String>,
}

impl WingetState {
    pub fn available(&self) -> bool {
        matches!(self.status, WingetStatus::Available(_))
    }

    pub fn update_filter(&mut self) {
        let needle = self.search.trim().to_lowercase();
        if self.filtered_for.as_deref() == Some(needle.as_str()) {
            return;
        }
        self.filtered = if needle.chars().count() < 2 {
            Vec::new()
        } else {
            (0..self.catalog.len())
                .filter(|&i| self.catalog[i].matches(&needle))
                .collect()
        };
        self.filtered_for = Some(needle);
    }
}

pub struct SysInfoState {
    pub data: SysInfo,
    pub loading: bool,
    pub updated: Option<Instant>,
}

const SYSINFO_REFRESH: Duration = Duration::from_secs(30);
const STATUS_HISTORY: usize = 30;

pub struct Configurator {
    pub(crate) page: Page,
    pub(crate) elevated: bool,
    pub(crate) renderer: String,
    pub(crate) bus: Bus,
    events: Receiver<Event>,
    pub(crate) config: Config,
    pub(crate) winget: WingetState,
    pub(crate) sysinfo: SysInfoState,
    pub(crate) net_target: String,
    pub(crate) pc_name: String,
    pub(crate) install_dialog: Option<crate::pages::InstallDialog>,
    status: VecDeque<String>,
}

impl Configurator {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::apply_theme(&cc.egui_ctx);

        let renderer = renderer_description(cc);
        eprintln!("renderer: {renderer}"); // visible in debug builds / when run from a console

        let (bus, events) = Bus::new(cc.egui_ctx.clone());
        config::watch(&bus);
        winget::refresh(&bus);

        Self {
            page: Page::Home,
            elevated: platform::is_elevated(),
            renderer,
            bus,
            events,
            config: Config::load(&crate::paths::config()),
            winget: WingetState {
                status: WingetStatus::Checking,
                catalog: Vec::new(),
                catalog_error: None,
                search: String::new(),
                filtered: Vec::new(),
                filtered_for: None,
            },
            sysinfo: SysInfoState {
                data: SysInfo::default(),
                loading: false,
                updated: None,
            },
            net_target: String::new(),
            pc_name: String::new(),
            install_dialog: None,
            status: VecDeque::new(),
        }
    }

    pub(crate) fn status(&mut self, message: impl Into<String>) {
        if self.status.len() == STATUS_HISTORY {
            self.status.pop_front();
        }
        self.status.push_back(message.into());
    }

    /// Reports the error of a fire-and-forget launch in the status bar.
    pub(crate) fn report<T>(&mut self, what: &str, result: std::io::Result<T>) {
        if let Err(e) = result {
            self.status(format!("{what}: {e}"));
        }
    }

    fn handle_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Status(msg) => self.status(msg),
                Event::Config(config) => self.config = config,
                Event::Winget(version) => {
                    self.winget.status = match version {
                        Some(v) => WingetStatus::Available(v),
                        None => WingetStatus::Missing,
                    };
                }
                Event::Catalog(result) => {
                    match result {
                        Ok(catalog) => {
                            self.winget.catalog = catalog;
                            self.winget.catalog_error = None;
                        }
                        Err(e) => self.winget.catalog_error = Some(e),
                    }
                    self.winget.filtered_for = None;
                }
                Event::SysInfo(data) => {
                    self.sysinfo = SysInfoState {
                        data,
                        loading: false,
                        updated: Some(Instant::now()),
                    };
                }
                Event::Resolved(check, result) => {
                    if let Some(dialog) = &mut self.install_dialog {
                        dialog.resolved(check, result);
                    }
                }
                Event::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }
    }

    pub(crate) fn install_winget(&mut self) {
        self.winget.status = WingetStatus::Installing;
        winget::bootstrap(&self.bus);
    }

    /// Refreshes system info when it is stale; only called while the page
    /// is visible, so the (slow) tools never run in the background otherwise.
    pub(crate) fn refresh_sysinfo(&mut self, ctx: &egui::Context, force: bool) {
        if self.sysinfo.loading {
            return;
        }
        let age = self.sysinfo.updated.map(|t| t.elapsed());
        match age {
            Some(age) if !force && age < SYSINFO_REFRESH => {
                ctx.request_repaint_after(SYSINFO_REFRESH - age);
            }
            _ => {
                self.sysinfo.loading = true;
                self.bus.spawn("sysinfo", |bus| {
                    bus.send(Event::SysInfo(crate::sysinfo::collect()))
                });
            }
        }
    }

    fn nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        for page in Page::ALL {
            let alert = page == Page::Settings && self.config.state != config::State::Loaded
                || page == Page::Apps && self.winget.status == WingetStatus::Missing;
            let tint = if alert { ui::RED } else { ui::NAV_TINT };
            let image = egui::Image::new(page.icon())
                .tint(tint)
                .fit_to_exact_size(egui::vec2(32.0, 32.0));
            let selected = self.page == page;
            let button = egui::Button::image(image)
                .selected(selected)
                .frame_when_inactive(selected)
                .corner_radius(10.0);
            if ui
                .add_sized([50.0, 50.0], button)
                .on_hover_text(page.title())
                .clicked()
            {
                self.page = page;
            }
        }
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.hyperlink_to("Source Code", crate::REPO_URL);
            ui.label("Brought to you by a confused engineer!");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.bus.busy() > 0 {
                    ui.spinner();
                }
                if let Some(last) = self.status.back() {
                    let history = self
                        .status
                        .iter()
                        .rev()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n");
                    ui.add(egui::Label::new(last.as_str()).truncate())
                        .on_hover_text(history);
                }
            });
        });
    }
}

fn renderer_description(cc: &eframe::CreationContext<'_>) -> String {
    if let Some(state) = &cc.wgpu_render_state {
        let info = state.adapter.get_info();
        format!(
            "wgpu · {:?} · {} ({:?})",
            info.backend, info.name, info.device_type
        )
    } else if cc.gl.is_some() {
        "OpenGL (glow)".to_owned()
    } else {
        "unknown".to_owned()
    }
}

impl eframe::App for Configurator {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_events(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("nav")
            .resizable(false)
            .exact_size(66.0)
            .show(ui, |ui| self.nav(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            ui::heading(ui, self.page.title());
            match self.page {
                Page::Home => self.page_home(ui),
                Page::Apps => self.page_apps(ui),
                Page::Tokens => self.page_tokens(ui),
                Page::Launch => self.page_launch(ui),
                Page::Troubleshoot => self.page_troubleshoot(ui),
                Page::SysInfo => self.page_sysinfo(ui),
                Page::Settings => self.page_settings(ui),
            }
        });
        let ctx = ui.ctx().clone();
        self.show_install_dialog(&ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    /// Renders every page offscreen. Set `WINCONFIG_SNAPSHOT_DIR` to also
    /// save a PNG per page.
    #[test]
    fn every_page_renders() {
        let mut harness = Harness::builder()
            .with_size([1200.0, 650.0])
            .wgpu()
            .build_eframe(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                Configurator::new(cc)
            });
        let snapshots = std::env::var_os("WINCONFIG_SNAPSHOT_DIR").map(std::path::PathBuf::from);
        for page in Page::ALL {
            harness.state_mut().page = page;
            // Fixed steps: spinners keep requesting repaints, so `run()` would not settle.
            for _ in 0..4 {
                harness.run_steps(3);
                std::thread::sleep(Duration::from_millis(100)); // let image loaders finish
            }
            let image = harness.render().expect("render");
            if let Some(dir) = &snapshots {
                std::fs::create_dir_all(dir).unwrap();
                image.save(dir.join(format!("{page:?}.png"))).unwrap();
            }
        }

        // The install options dialog, pre-filled from a config-style entry.
        let request = winget::InstallRequest::from_config(
            "PowerShell 7 (all users)",
            "Microsoft.PowerShell --scope machine --installer-type wix",
        )
        .unwrap();
        harness.state_mut().page = Page::Apps;
        harness.state_mut().install_dialog = Some(crate::pages::InstallDialog::new(&request));
        harness.run_steps(5);
        if snapshots.is_some() {
            // Give the live "which version?" lookup time to finish for the PNG.
            for _ in 0..60 {
                std::thread::sleep(Duration::from_millis(300));
                harness.run_steps(2);
            }
        }
        let image = harness.render().expect("render dialog");
        if let Some(dir) = &snapshots {
            image.save(dir.join("InstallDialog.png")).unwrap();
        }
        assert!(
            harness.state().install_dialog.is_some(),
            "dialog closed by itself"
        );
    }
}
