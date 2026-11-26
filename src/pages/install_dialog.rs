//! "Install options" dialog: pick scope, installer type, architecture and
//! extra `winget install` arguments before installing a package.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use egui::{RichText, Ui};

use crate::app::Configurator;
use crate::platform::Window;
use crate::tasks::{Bus, Event};
use crate::ui;
use crate::winget::{self, InstallRequest, Resolution};

/// Wait this long after the last edit before asking winget again.
const CHECK_DEBOUNCE: Duration = Duration::from_millis(700);

static NEXT_CHECK: AtomicU64 = AtomicU64::new(1);

/// The latest "which version will this install?" lookup.
struct Check {
    id: u64,
    args: Vec<String>,
    result: Option<Result<Resolution, String>>,
}

const SCOPES: [(&str, &str); 3] = [
    ("", "Default"),
    ("user", "Current user"),
    ("machine", "All users (machine)"),
];
const INSTALLER_TYPES: [&str; 11] = [
    "msi", "wix", "exe", "msix", "appx", "inno", "nullsoft", "burn", "zip", "portable", "pwa",
];
const ARCHITECTURES: [&str; 4] = ["x64", "x86", "arm64", "arm"];

pub struct InstallDialog {
    name: String,
    id: String,
    scope: String,
    installer_type: String,
    architecture: String,
    extra: String,
    check: Option<Check>,
    edited: Instant,
}

impl InstallDialog {
    /// Opens the dialog pre-filled from a request (e.g. a config favourite
    /// with `--scope machine --installer-type wix`).
    pub fn new(request: &InstallRequest) -> Self {
        let mut dialog = Self {
            name: request.name.clone(),
            id: request.id.clone(),
            scope: String::new(),
            installer_type: String::new(),
            architecture: String::new(),
            extra: String::new(),
            check: None,
            edited: Instant::now() - CHECK_DEBOUNCE,
        };
        let mut extra = Vec::new();
        let mut args = request.args.iter();
        while let Some(arg) = args.next() {
            let field = match arg.as_str() {
                "--scope" => Some(&mut dialog.scope),
                "--installer-type" => Some(&mut dialog.installer_type),
                "-a" | "--architecture" => Some(&mut dialog.architecture),
                _ => None,
            };
            match (field, args.clone().next()) {
                (Some(field), Some(value)) if field.is_empty() => {
                    *field = value.to_lowercase();
                    args.next();
                }
                _ => extra.push(if arg.contains(' ') {
                    format!("\"{arg}\"")
                } else {
                    arg.clone()
                }),
            }
        }
        dialog.extra = extra.join(" ");
        dialog
    }

    /// Delivers a lookup result (ignored if the options changed since).
    pub fn resolved(&mut self, id: u64, result: Result<Resolution, String>) {
        if let Some(check) = &mut self.check
            && check.id == id
        {
            check.result = Some(result);
        }
    }

    /// Starts a background lookup when the options changed and settled.
    fn refresh_check(&mut self, ctx: &egui::Context, bus: &Bus, request: &InstallRequest) {
        if self.check.as_ref().is_some_and(|c| c.args == request.args) {
            return;
        }
        let quiet = self.edited.elapsed();
        if quiet < CHECK_DEBOUNCE {
            ctx.request_repaint_after(CHECK_DEBOUNCE - quiet);
            return;
        }
        let id = NEXT_CHECK.fetch_add(1, Ordering::Relaxed);
        self.check = Some(Check {
            id,
            args: request.args.clone(),
            result: None,
        });
        let request = request.clone();
        bus.spawn("winget-resolve", move |bus| {
            bus.send(Event::Resolved(id, winget::resolve(&request)))
        });
    }

    /// Adds `--version <v>` to the extra arguments.
    fn pin(&mut self, version: &str) {
        self.extra = format!("{} --version {version}", self.extra.trim())
            .trim()
            .to_owned();
        self.edited = Instant::now() - CHECK_DEBOUNCE;
    }

    fn request(&self) -> Result<InstallRequest, String> {
        let mut args = Vec::new();
        for (flag, value) in [
            ("--scope", &self.scope),
            ("--installer-type", &self.installer_type),
            ("--architecture", &self.architecture),
        ] {
            if !value.is_empty() {
                args.extend([flag.to_owned(), value.clone()]);
            }
        }
        args.extend(winget::split_args(&self.extra)?);
        Ok(InstallRequest {
            name: self.name.clone(),
            id: self.id.clone(),
            args,
        })
    }
}

fn combo(ui: &mut Ui, id: &str, value: &mut String, options: &[&str]) {
    let selected = if value.is_empty() {
        "Default"
    } else {
        value.as_str()
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected.to_owned())
        .show_ui(ui, |ui| {
            ui.selectable_value(value, String::new(), "Default");
            for option in options {
                ui.selectable_value(value, (*option).to_owned(), *option);
            }
        });
}

/// Shows what the last lookup found; returns a version to pin if clicked.
fn check_result(ui: &mut Ui, check: Option<&Check>) -> Option<String> {
    ui.add_space(4.0);
    let Some(check) = check else {
        ui.weak("Checking which version will be installed…");
        return None;
    };
    match &check.result {
        None => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Checking which version matches these options…");
            });
            None
        }
        Some(Err(e)) => {
            ui.colored_label(
                ui::RED,
                format!("No match: {e}. winget will refuse to install with these options."),
            );
            None
        }
        Some(Ok(r)) => {
            let kind = r.installer_type.as_deref().unwrap_or("unknown type");
            ui.colored_label(
                ui::GREEN,
                format!("Will install version {} ({kind} installer)", r.version),
            );
            if let Some(url) = &r.url {
                ui.add(
                    egui::Label::new(RichText::new(url).weak().small())
                        .selectable(true)
                        .truncate(),
                );
            }
            let mut pin = None;
            if let Some(newest) = &r.newer_skipped {
                ui.colored_label(
                    ui::ORANGE,
                    format!("The newest version ({newest}) has no installer matching these options, so winget falls back to {}.", r.version),
                );
                let pinned = check.args.iter().any(|a| a == "--version");
                if !pinned
                    && ui
                        .button(format!("Pin version {}", r.version))
                        .on_hover_text("Adds --version so the install keeps using this version even after newer ones are published")
                        .clicked()
                {
                    pin = Some(r.version.clone());
                }
            }
            pin
        }
    }
}

impl Configurator {
    pub(crate) fn show_install_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = &mut self.install_dialog else {
            return;
        };
        let bus = self.bus.clone();
        let mut action = None;
        let response = egui::Modal::new(egui::Id::unique("install_dialog")).show(ctx, |ui| {
            ui.set_width(520.0);
            ui.heading(format!("Install {}", dialog.name));
            ui.weak(&dialog.id);
            ui.separator();

            egui::Grid::new("install_options")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Scope");
                    ui.horizontal(|ui| {
                        for (value, label) in SCOPES {
                            ui.radio_value(&mut dialog.scope, value.to_owned(), label);
                        }
                    });
                    ui.end_row();

                    ui.label("Installer type");
                    combo(
                        ui,
                        "installer_type",
                        &mut dialog.installer_type,
                        &INSTALLER_TYPES,
                    );
                    ui.end_row();

                    ui.label("Architecture");
                    combo(ui, "architecture", &mut dialog.architecture, &ARCHITECTURES);
                    ui.end_row();

                    ui.label("Extra arguments");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.extra)
                            .hint_text(r#"e.g. --version 7.4.6 --location "D:\Apps""#)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();
                });
            let before = (
                dialog.scope.clone(),
                dialog.installer_type.clone(),
                dialog.architecture.clone(),
                dialog.extra.clone(),
            );
            ui.separator();

            let request = dialog.request();
            if before
                != (
                    dialog.scope.clone(),
                    dialog.installer_type.clone(),
                    dialog.architecture.clone(),
                    dialog.extra.clone(),
                )
            {
                dialog.edited = Instant::now();
            }
            match &request {
                Ok(r) => {
                    ui.add(
                        egui::Label::new(RichText::new(r.display()).monospace().size(12.0))
                            .selectable(true),
                    );
                }
                Err(e) => {
                    ui.colored_label(ui::RED, format!("Extra arguments: {e}"));
                }
            }
            if let Ok(r) = &request {
                dialog.refresh_check(ui.ctx(), &bus, r);
                if let Some(pin) = check_result(ui, dialog.check.as_ref()) {
                    dialog.pin(&pin);
                }
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_enabled_ui(request.is_ok(), |ui| {
                    if ui.button("Install (show progress)").clicked() {
                        action = Some(Window::Visible);
                    }
                    if ui.button("Install silently").clicked() {
                        action = Some(Window::Hidden);
                    }
                });
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
            });
            request
        });

        if let (Some(window), Ok(request)) = (action, &response.inner) {
            winget::install(&self.bus, request, window);
            self.install_dialog = None;
        } else if response.should_close() {
            self.install_dialog = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_known_flags() {
        let request = InstallRequest::from_config(
            "PowerShell",
            r#"Microsoft.PowerShell --scope machine --installer-type WIX --location "C:\P S" -a x64"#,
        )
        .unwrap();
        let dialog = InstallDialog::new(&request);
        assert_eq!(dialog.scope, "machine");
        assert_eq!(dialog.installer_type, "wix");
        assert_eq!(dialog.architecture, "x64");
        assert_eq!(dialog.extra, r#"--location "C:\P S""#);
        assert_eq!(
            dialog.request().unwrap().args,
            [
                "--scope",
                "machine",
                "--installer-type",
                "wix",
                "--architecture",
                "x64",
                "--location",
                r"C:\P S"
            ]
        );
    }
}
