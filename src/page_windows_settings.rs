use egui::Ui;
use std::process::Command;

use crate::app::WindowsSettings;

pub fn show_windows_settings(ui: &mut Ui, win_settings_struct: &mut WindowsSettings)
{
    ui.columns(3, |ui| {
        ui[0].heading("Open in Settings");
        egui::ScrollArea::vertical().id_source("SettingsLinks").show(&mut ui[0], |ui|{
            ui.label("General Settings");
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("About Machine") ).clicked()
            {
                launch("ms-settings:about");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Windows Updates") ).clicked()
            {
                launch("ms-settings:windowsupdate");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Work Or School") ).clicked()
            {
                launch("ms-settings:workplace");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Date and Time") ).clicked()
            {
                launch("ms-settings:dateandtime");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Power and Sleep") ).clicked()
            {
                launch("ms-settings:powersleep");
            }
            ui.label("Apps");
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Installed Apps") ).clicked()
            {
                launch("ms-settings:appsfeatures");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Default Apps") ).clicked()
            {
                launch("ms-settings:defaultapps");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Startup Apps") ).clicked()
            {
                launch("ms-settings:startupapps");
            }

            ui.label("Network Settings");
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Airplane Mode") ).clicked()
            {
                launch("ms-settings:network-airplanemode");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Ethernet") ).clicked()
            {
                launch("ms-settings:network-ethernet");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Wi-Fi") ).clicked()
            {
                launch("ms-settings:network-wifi");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("VPN") ).clicked()
            {
                launch("ms-settings:network-vpn");
            }
            ui.label("Personalization");
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Taskbar") ).clicked()
            {
                launch("ms-settings:taskbar");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Themes") ).clicked()
            {
                launch("ms-settings:themes");
            }
            ui.label("Unknown");
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Workplace Repair Token") ).clicked()
            {
                launch("ms-settings:workplace-repairtoken");
            }
            



            ui.add_space(20.0);
        });

        ui[1].heading("Open in Control Panel");
        egui::ScrollArea::vertical().id_source("SettingsControlPanel").show(&mut ui[1], |ui|{
            ui.label("General");
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Add/Remove Programs") ).clicked()
            {
                launch("appwiz.cpl");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Sounds") ).clicked()
            {
                launch("mmsys.cpl");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Power") ).clicked()
            {
                launch("powercfg.cpl");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Network Adapters") ).clicked()
            {
                launch("Ncpa.cpl");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Scheduled Tasks") ).clicked()
            {
                launch("control schedtasks");
            }
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
            if win_settings_struct.vpn_add_as_admin
            {
                ui.columns(2, |ui| {
                    if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Add MSCHAPv22") ).clicked()
                    {
                        vpn_add_admin(win_settings_struct, "mschapv2");
                    }
                    if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Add PAP") ).clicked()
                    {
                        vpn_add_admin(win_settings_struct, "chap");
                    }
                });
            }
            else
            {
                ui.columns(2, |ui| {
                    if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Add MSCHAPv2") ).clicked()
                    {
                        vpn_add(win_settings_struct, "mschapv2");
                    }
                    if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Add PAP") ).clicked()
                    {
                        vpn_add(win_settings_struct, "chap");
                    }
                });
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Remove VPNs") ).clicked()
            {
                vpn_remove();
            }
            ui.small("Note: Removing VPNs only applies to current user unless running as admin");
            ui.add_space(8.0);
            ui.label("Rename PC");
            ui.text_edit_singleline(&mut win_settings_struct.pc_name);
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Rename PC") ).clicked()
            {
                rename_pc(&win_settings_struct.pc_name);
            }
            ui.small("Note: Renaming PC does note take effect until after reboot.");
            


            ui.add_space(20.0);
        });
        ui[2].heading("Run Commands");
        egui::ScrollArea::vertical().id_source("SettingsOther").show(&mut ui[2], |ui|{
            ui.label("Power and System");
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Shutdown") ).clicked()
            {
                shutdown();
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Restart") ).clicked()
            {
                restart();
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Restart to BIOS") ).clicked()
            {
                restart_bios();
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Enable High Performance") ).clicked()
            {
                enable_high_perf();
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Disable Fast Startup") ).clicked()
            {
                disable_fast_start();
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Enable Never Sleep") ).clicked()
            {
                never_sleep();
            }
        });
    });
}

#[tokio::main]
async fn launch(app: &str) {

    let app_vec: Vec<&str> = app.split(" ").collect();
    if app_vec.len()<2
    {
        Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg(app_vec[0])
        .spawn()
        .expect("failed to execute process");
    } else if app_vec.len() == 2 {
        Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg(app_vec[0])
        .arg(app_vec[1])
        .spawn()
        .expect("failed to execute process");
    }

}

#[tokio::main]
async fn shutdown() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["shutdown","/s","/t","1"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn restart() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["shutdown","/r","/t","1"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn restart_bios() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["shutdown","/r","/fw","/t","1"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn enable_high_perf() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powercfg","-SETACTIVE","8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn disable_fast_start() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powercfg","/h","off"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn never_sleep() {
    let vec = ["-disk-timeout-ac", "-standby-timeout-ac","-hibernate-timeout-ac"];
    for x in 0..(vec.len()-1)
    {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .args(["powercfg","-x",vec[x],"0"])
            .spawn()
            .expect("failed to execute process");
    }

}


#[tokio::main]
async fn vpn_add(win_settings_struct: &mut WindowsSettings, function: &str) {

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

    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powershell","-window","hidden","add-VpnConnection","-Name", win_settings_struct.vpn_name.as_str(), "-ServerAddress", win_settings_struct.vpn_addr.as_str(), "-tunneltype", "L2tp","-EncryptionLevel", "Optional", "-L2tpPsk", win_settings_struct.vpn_secret_key.replace("$(", "").as_str(), "-AuthenticationMethod", auth_method.as_str(),"-Force"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn vpn_add_admin(win_settings_struct: &mut WindowsSettings, function: &str) {

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

    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powershell","-window","hidden","add-VpnConnection","-AllUserConnection","-Name", win_settings_struct.vpn_name.as_str(), "-ServerAddress", win_settings_struct.vpn_addr.as_str(), "-tunneltype", "L2tp","-EncryptionLevel", "Optional", "-L2tpPsk", win_settings_struct.vpn_secret_key.replace("$(", "").as_str(), "-AuthenticationMethod", auth_method.as_str(),"-Force"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn vpn_remove() {

    Command::new("powershell")
    .args(["Get-VpnConnection","|", "Remove-VpnConnection", "-Force"])
    .spawn()
    .expect("failed to execute process");

    Command::new("powershell")
    .args(["Get-VpnConnection", "-AllUserConnection", "|", "Remove-VpnConnection", "-Force"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn rename_pc(pc_name: &String) {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powershell","-window","minimized","rename-computer","-NewName",pc_name.as_str()])
    .spawn()
    .expect("failed to execute process");
}