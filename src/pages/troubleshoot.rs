use egui::Ui;

use crate::app::Configurator;
use crate::commands::{Group, NETWORK_ADMIN, TROUBLESHOOTING, TROUBLESHOOTING_HIDDEN};
use crate::platform::{self, Window};
use crate::tasks::Bus;
use crate::ui::{self, icons};

use super::with_pause;

/// Network tools that take a host name / IP: (label, command template, tooltip).
const HOST_TOOLS: [(&str, &str, &str); 4] = [
    ("Ping", "ping {} -a -t", "Continuous ping (Ctrl+C to stop)"),
    (
        "PathPing",
        "pathping {} & pause",
        "Route with per-hop packet loss",
    ),
    (
        "Trace Route",
        "tracert {} & pause",
        "Show every hop to the host",
    ),
    (
        "NS Lookup",
        "nslookup {} & pause",
        "Resolve the name with DNS",
    ),
];

impl Configurator {
    pub(crate) fn page_troubleshoot(&mut self, ui: &mut Ui) {
        self.admin_notice(ui, "Most troubleshooting tools");
        let bus = self.bus.clone();
        let elevated = self.elevated;

        ui.columns_const(|[network, system]| {
            ui::heading(network, "Networking");
            egui::ScrollArea::vertical()
                .id_salt("trouble_network")
                .show(network, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Host or IP:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.net_target)
                                .hint_text("e.g. 8.8.8.8 or example.com")
                                .desired_width(f32::INFINITY),
                        );
                    });
                    ui.separator();
                    let target = self.net_target.trim();
                    let valid = is_valid_host(target);
                    for (label, template, tip) in HOST_TOOLS {
                        ui::row(ui, label, |ui| {
                            let button = ui.add_enabled_ui(valid, |ui| {
                                ui::icon_button(ui, icons::launch(), ui::GREEN, tip)
                            });
                            if button.inner.clicked() {
                                run(
                                    &bus,
                                    label,
                                    &template.replace("{}", target),
                                    Window::Visible,
                                );
                            }
                        });
                    }
                    ui::row(ui, "Flush DNS cache", |ui| {
                        if ui::icon_button(ui, icons::refresh(), ui::GREEN, "ipconfig /flushdns")
                            .clicked()
                        {
                            flush_dns(&bus);
                        }
                    });
                    ui.add_enabled_ui(elevated, |ui| {
                        command_list(ui, &bus, &NETWORK_ADMIN, Window::Visible)
                    });
                });

            ui::heading(system, "System");
            egui::ScrollArea::vertical()
                .id_salt("trouble_system")
                .show(system, |ui| {
                    ui.add_enabled_ui(elevated, |ui| {
                        command_list(ui, &bus, &TROUBLESHOOTING_HIDDEN, Window::Hidden);
                        command_list(ui, &bus, &TROUBLESHOOTING, Window::Visible);
                    });
                });
        });
    }
}

fn command_list(ui: &mut Ui, bus: &Bus, groups: &[Group], window: Window) {
    for group in groups {
        ui::group_heading(ui, &group.title);
        for entry in &group.entries {
            ui::row(ui, entry.label.as_str(), |ui| {
                if ui::icon_button(ui, icons::launch(), ui::GREEN, &entry.command).clicked() {
                    let command = match window {
                        Window::Visible => with_pause(&entry.command),
                        Window::Hidden => entry.command.clone(),
                    };
                    run(bus, &entry.label, &command, window);
                    if window == Window::Hidden {
                        bus.status(format!("Started “{}” in the background", entry.label));
                    }
                }
            });
        }
    }
}

fn run(bus: &Bus, label: &str, command: &str, window: Window) {
    if let Err(e) = platform::cmd(command, window) {
        bus.status(format!("Could not start {label}: {e}"));
    }
}

fn flush_dns(bus: &Bus) {
    bus.spawn("flushdns", |bus| {
        let result = platform::capture("ipconfig", &["/flushdns"]);
        bus.status(match result {
            Ok((status, _)) if status.success() => "DNS cache flushed".to_owned(),
            Ok((_, out)) => format!("Flushing DNS failed: {}", out.trim()),
            Err(e) => format!("Flushing DNS failed: {e}"),
        });
    });
}

/// Host names, IPv4/IPv6 addresses (with zone ids). Rejects anything that
/// could smuggle extra commands into the `cmd.exe` line.
fn is_valid_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 255
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-:_%".contains(c))
}

#[cfg(test)]
mod tests {
    use super::is_valid_host;

    #[test]
    fn host_validation() {
        for ok in ["8.8.8.8", "example.com", "fe80::1%12", "my_host-1"] {
            assert!(is_valid_host(ok), "{ok}");
        }
        for bad in ["", "a b", "x & calc", "a|b", "\"quoted\"", "a>b"] {
            assert!(!is_valid_host(bad), "{bad}");
        }
    }
}
