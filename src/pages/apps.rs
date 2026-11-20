use eframe::egui::{self, Ui};

use crate::app::Configurator;
use crate::config::LaunchKind;
use crate::platform::{self, Window};
use crate::ui::{self, icons};
use crate::winget::InstallRequest;
use crate::{download, paths, winget};

use super::InstallDialog;

/// Search results are virtualised, but capping them keeps the scrollbar sane.
const MAX_RESULTS: usize = 500;

impl Configurator {
    pub(crate) fn page_apps(&mut self, ui: &mut Ui) {
        self.winget_notice(ui);
        let bus = self.bus.clone();
        let winget_ok = self.winget.available();

        let mut dialog = None;
        ui.columns_const(|[favorites, search, others]| {
            // Favourites from [winget-*] config sections.
            ui::heading(favorites, "Favorites");
            egui::ScrollArea::vertical().id_salt("apps_favorites").show(favorites, |ui| {
                ui.add_enabled_ui(winget_ok, |ui| {
                    let sections = self.config.winget_sections();
                    if sections.is_empty() {
                        ui.weak("Add [winget-Category] sections to config.ini to list favourite apps here.");
                    }
                    for section in sections {
                        ui::group_heading(ui, section.name);
                        for (name, value) in section.entries {
                            let request = InstallRequest::from_config(name, value);
                            install_row(ui, &bus, name, request, &mut dialog);
                        }
                    }
                });
            });

            // Search of the whole winget catalog.
            ui::heading(search, "Search All");
            search.add_enabled_ui(winget_ok, |ui| self.catalog_search(ui));

            // Everything else.
            ui::heading(others, "Others");
            egui::ScrollArea::vertical().id_salt("apps_others").show(others, |ui| {
                ui::group_heading(ui, "Winget");
                ui.add_enabled_ui(winget_ok, |ui| {
                    ui::row(ui, "Update all applications", |ui| {
                        if ui::icon_button(
                            ui,
                            icons::upload(),
                            ui::GREEN,
                            "Upgrade every app winget can, even ones it did not install",
                        )
                        .clicked()
                        {
                            winget::upgrade_all(&bus);
                        }
                    });
                });
                for (kind, section) in self.config.launcher_sections() {
                    ui::group_heading(ui, section.name);
                    for (name, target) in section.entries {
                        ui::row(ui, name, |ui| {
                            let tip = match &kind {
                                LaunchKind::Online(_) => format!("Download and run {target}"),
                                _ => format!("Run {target}"),
                            };
                            if ui::icon_button(ui, icons::download(), ui::GREEN, &tip).clicked() {
                                launch(&bus, &kind, name, target);
                            }
                        });
                    }
                }
            });
        });
        if dialog.is_some() {
            self.install_dialog = dialog;
        }
    }

    fn catalog_search(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(
                egui::TextEdit::singleline(&mut self.winget.search)
                    .hint_text("name or id")
                    .desired_width(f32::INFINITY),
            );
        });
        ui.separator();

        if self.winget.catalog.is_empty() {
            match &self.winget.catalog_error {
                Some(e) => {
                    ui.colored_label(ui::RED, format!("Could not load the winget catalog: {e}"))
                }
                None if self.winget.available() => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading the winget catalog…");
                    })
                    .response
                }
                None => ui.weak("winget is not available."),
            };
            return;
        }

        self.winget.update_filter();
        let total = self.winget.filtered.len();
        if self.winget.search.trim().chars().count() < 2 {
            ui.weak(format!(
                "Type at least 2 characters to search {} packages.",
                self.winget.catalog.len()
            ));
            return;
        }
        if total == 0 {
            ui.weak("No matches.");
            return;
        }
        if total > MAX_RESULTS {
            ui.weak(format!(
                "Showing {MAX_RESULTS} of {total} matches; refine the search."
            ));
        }

        let bus = self.bus.clone();
        let row_height = ui::ROW_BUTTON.y + ui.spacing().item_spacing.y * 2.0 + 1.0;
        egui::ScrollArea::vertical()
            .id_salt("apps_search")
            .auto_shrink([false; 2])
            .show_rows(ui, row_height, total.min(MAX_RESULTS), |ui, range| {
                for &i in &self.winget.filtered[range] {
                    let pkg = &self.winget.catalog[i];
                    let label = format!("{}  |  {}", pkg.name, pkg.version);
                    let request = Ok(InstallRequest::new(&pkg.name, &pkg.id));
                    install_row(ui, &bus, &label, request, &mut self.install_dialog);
                }
            });
    }
}

fn install_row(
    ui: &mut Ui,
    bus: &crate::tasks::Bus,
    label: &str,
    request: Result<InstallRequest, String>,
    dialog: &mut Option<InstallDialog>,
) {
    let request = match request {
        Ok(request) => request,
        Err(e) => {
            ui::row(ui, egui::RichText::new(label).color(ui::RED), |ui| {
                ui.colored_label(ui::RED, "invalid")
                    .on_hover_text(format!("Invalid config entry: {e}"));
            });
            return;
        }
    };
    let command = request.display();
    ui::row(ui, label, |ui| {
        if ui::icon_button(ui, icons::options(), ui::NAV_TINT, "Install options: scope (user / all users), installer type, architecture, extra arguments…").clicked() {
            *dialog = Some(InstallDialog::new(&request));
        }
        if ui::icon_button(
            ui,
            icons::silent(),
            ui::ORANGE,
            &format!("Install in the background\n{command}"),
        )
        .clicked()
        {
            winget::install(bus, &request, Window::Hidden);
        }
        if ui::icon_button(
            ui,
            icons::download(),
            ui::GREEN,
            &format!("Install (shows progress)\n{command}"),
        )
        .clicked()
        {
            winget::install(bus, &request, Window::Visible);
        }
    });
}

fn launch(bus: &crate::tasks::Bus, kind: &LaunchKind, name: &str, target: &str) {
    let result = match kind {
        LaunchKind::Program => platform::open(paths::resolve(target)),
        LaunchKind::Powershell => platform::powershell_file(&paths::resolve(target)).map(drop),
        LaunchKind::Online(ext) => {
            download::download_and_open(bus, name, ext, target);
            Ok(())
        }
    };
    if let Err(e) = result {
        bus.status(format!("Could not start {name}: {e}"));
    }
}
