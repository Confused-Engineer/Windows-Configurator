use std::collections::VecDeque;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use egui::ImageSource;

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
    pub fn new(ctx: &egui::Context) -> Self {
        ui::apply_theme(ctx);
        egui_extras::install_image_loaders(ctx);

        let (bus, events) = Bus::new(ctx.clone());
        config::watch(&bus);
        winget::refresh(&bus);

        Self {
            page: Page::Home,
            elevated: platform::is_elevated(),
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

/// Called by the window loop (`cpu.rs`) every frame: `logic` first, then `ui`.
impl Configurator {
    pub fn logic(&mut self, ctx: &egui::Context) {
        self.handle_events(ctx);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
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
    use crate::cpu::Offscreen;

    /// Renders every page offscreen with the window's own CPU renderer. Set
    /// `WINCONFIG_SNAPSHOT_DIR` to also save a PNG per page.
    #[test]
    fn every_page_renders() {
        let mut screen = Offscreen::new(1200, 650);
        let mut app = Configurator::new(&screen.ctx.clone());
        let snapshots = std::env::var_os("WINCONFIG_SNAPSHOT_DIR").map(std::path::PathBuf::from);
        let frames = |screen: &mut Offscreen, app: &mut Configurator, rounds: usize, pause: u64| {
            for _ in 0..rounds {
                for _ in 0..3 {
                    screen.frame(app);
                }
                std::thread::sleep(Duration::from_millis(pause)); // image loaders, background jobs
            }
            screen.frame(app)
        };
        let save = |image: &image::RgbaImage, name: &str| {
            if let Some(dir) = &snapshots {
                std::fs::create_dir_all(dir).unwrap();
                image.save(dir.join(format!("{name}.png"))).unwrap();
            }
        };

        for page in Page::ALL {
            app.page = page;
            let image = frames(&mut screen, &mut app, 4, 100);
            // Drawn, and dark: the panel background covers much of the frame.
            let [r, g, b, _] = egui::Rgba::from(ui::PANEL_FILL).to_srgba_unmultiplied();
            let bg = image.pixels().filter(|p| p.0[..3] == [r, g, b]).count();
            assert!(
                bg * 3 > image.pixels().len(),
                "{page:?}: only {bg} background pixels"
            );
            save(&image, &format!("{page:?}"));
        }

        // The install options dialog, pre-filled from a config-style entry.
        let request = winget::InstallRequest::from_config(
            "PowerShell 7 (all users)",
            "Microsoft.PowerShell --scope machine --installer-type wix",
        )
        .unwrap();
        app.page = Page::Apps;
        app.install_dialog = Some(crate::pages::InstallDialog::new(&request));
        // With snapshots, give the live "which version?" lookup time to finish.
        let rounds = if snapshots.is_some() { 60 } else { 2 };
        let image = frames(&mut screen, &mut app, rounds, 300);
        save(&image, "InstallDialog");
        assert!(app.install_dialog.is_some(), "dialog closed by itself");
    }
}
