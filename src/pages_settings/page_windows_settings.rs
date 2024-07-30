use egui::Ui;
use ini::Ini;
use std::{os::windows::process::CommandExt, process::Command};

use crate::app::WindowsSettings;

pub fn show_windows_settings(ui: &mut Ui, win_settings_struct: &mut WindowsSettings, uri_win_settings: &mut ini::Ini, control_panel_settings: &mut ini::Ini, commands: &mut ini::Ini)
{
    ui.columns(3, |ui| {
        ui[0].heading("Open in Settings");
        egui::ScrollArea::vertical().id_source("SettingsLinks").show(&mut ui[0], |ui|{
            display_column(uri_win_settings, ui);
            ui.add_space(20.0);
        });

        ui[1].heading("Open in Control Panel");
        egui::ScrollArea::vertical().id_source("SettingsControlPanel").show(&mut ui[1], |ui|{
            display_column(control_panel_settings, ui);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            ui.heading("Configure Settings");
            ui.label("VPN Setup");
            ui.add_space(8.0);
            ui.label("Name");
            ui.text_edit_singleline(&mut win_settings_struct.vpn_name);
            ui.label("Address");
            ui.text_edit_singleline(&mut win_settings_struct.vpn_addr);
            ui.label("Secret Key");
            ui.horizontal(|ui|{
                let view_password = !win_settings_struct.vpn_view_secret_key;
                egui::TextEdit::singleline(&mut win_settings_struct.vpn_secret_key).password(view_password).show(ui);
                ui.checkbox( &mut win_settings_struct.vpn_view_secret_key, "view")
            });
            
            ui.checkbox(&mut win_settings_struct.vpn_add_as_admin, "Run for all users? (Requires Admin Priv)");

            ui.columns(2, |ui| {
                if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Add MSCHAPv2") ).clicked()
                {
                    vpn_add(win_settings_struct, "mschapv2", win_settings_struct.vpn_add_as_admin);
                }
                if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Add PAP") ).clicked()
                {
                    vpn_add(win_settings_struct, "chap", win_settings_struct.vpn_add_as_admin);
                }
            });

            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Remove VPNs") ).clicked()
            {
                Command::new("powershell")
                .args(["Get-VpnConnection","|", "Remove-VpnConnection", "-Force"])
                .spawn()
                .expect("failed to execute process");
            
                Command::new("powershell")
                .args(["Get-VpnConnection", "-AllUserConnection", "|", "Remove-VpnConnection", "-Force"])
                .spawn()
                .expect("failed to execute process");
            }
            ui.small("Note: Removing VPNs only applies to current user unless running as admin");
            ui.add_space(8.0);
            ui.label("Rename PC");
            ui.text_edit_singleline(&mut win_settings_struct.pc_name);
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Rename PC") ).clicked()
            {
                launch(&format!("powershell -window hiddern rename-computer -NewName '{}'", &win_settings_struct.pc_name));
            }
            ui.small("Note: Renaming PC does note take effect until after reboot.");
            
            ui.add_space(20.0);
        });
        ui[2].heading("Run Commands");
        egui::ScrollArea::vertical().id_source("SettingsOther").show(&mut ui[2], |ui|{
            display_column(commands, ui);
        });
    });
}

fn launch(command: &str) {

    let app_vec: Vec<&str> = command.split(" ").collect();
    let mut command = Command::new("cmd");
    command.args(["/C","start"]);
    for x in 0..app_vec.len()
    {
        command.arg(app_vec[x]);
    }
    command.creation_flags(0x08000000);
    let _ = command.spawn().expect("failed to execute process");
    
}

fn vpn_add(win_settings_struct: &mut WindowsSettings, function: &str, as_admin: bool) {

    #[allow(unused_assignments)]
    let mut auth_method = String::new();
    match function {
        "mschapv2" => {
            auth_method = "MSChapv2".to_string();
        },
        "chap" => {
            auth_method = "Pap".to_string();
        },
        _ => {auth_method = "MSChapv2".to_string();},
    }

    if as_admin
    {
        launch(&format!("powershell -window hidden add-VpnConnection -AllUserConnection -Name '{}' -ServerAddress {} -tunneltype L2tp -EncryptionLevel Optional -L2tpPsk {} -AuthenticationMethod {} -Force",win_settings_struct.vpn_name, win_settings_struct.vpn_addr, win_settings_struct.vpn_secret_key.replace("$(", ""), auth_method));
    } else {
        launch(&format!("powershell -window hidden add-VpnConnection -Name '{}' -ServerAddress {} -tunneltype L2tp -EncryptionLevel Optional -L2tpPsk {} -AuthenticationMethod {} -Force",win_settings_struct.vpn_name, win_settings_struct.vpn_addr, win_settings_struct.vpn_secret_key.replace("$(", ""), auth_method));
    }
}

fn display_column(ini: &mut Ini, ui: &mut Ui)
{
    for (section, _) in ini.clone().into_iter()
    {
        if section.is_none()
        {
            continue;
        }
        let section = section.unwrap();
        ui.label(section);

        let ini_section = ini.section(Some(section));
        if ini_section.is_none()
        {
            continue;
        }

        let ini_section = ini_section.unwrap();
        for (key, val) in ini_section.iter()
        {
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new(key) ).clicked()
            {
                launch(val);
            }
        }
    }
}