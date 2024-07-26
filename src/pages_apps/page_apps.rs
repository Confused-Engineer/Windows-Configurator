use egui::Ui;
use ini::Ini;
use std::{os::windows::process::CommandExt, process::Command};

pub fn page_apps(ui: &mut Ui, config: &Ini)
{
    ui.columns(3, |ui|{
        ui[0].heading("Click to Install Apps");
        egui::ScrollArea::vertical().id_source("InstallApps").show(&mut ui[0], |ui|{

            for (key, _) in config
            {

                match key {
                    None => {},
                    section if section.unwrap().contains("winget-") => {
                        ui.heading(key.unwrap().split("-").last().unwrap());
                        display(ui, section.unwrap(), config)
                    }
                    _ => {},
                }
            }
            ui.add_space(20.0);
        });
        

        ui[1].heading("Config Specified Installs");
        egui::ScrollArea::vertical().id_source("ConfigApps").show(&mut ui[1], |ui|{
            for (key, _) in config
            {

                match key {
                    None =>{},
                    section if section.unwrap().eq("Programs") => {
                        ui.heading("Programs");
                        display(ui, section.unwrap(), config)
                    },
                    section if section.unwrap().eq("Powershell") => {
                        ui.heading("Powershell Scripts");
                        display(ui, section.unwrap(), config)
                    },
                    section if section.unwrap().contains("Online") => {
                        ui.heading(key.unwrap().replace("-", " "));
                        display(ui, section.unwrap(), config);
                    },
                    _ => {
                        //ui.label(key.unwrap());
                    },
                }

            }
        });

        ui[2].heading("Winget Options");
        egui::ScrollArea::vertical().id_source("WingetOption").show(&mut ui[2], |ui|{
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Update All Apps") ).clicked()
            {
                winget_update_all();
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Install Winget") ).clicked()
            {
                winget_auto_install();
                launch("ms-windows-store://pdp/?ProductId=9NBLGGH4NNS1");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Check MS Store Updates") ).clicked()
            {
                launch("ms-windows-store://downloadsandupdates");
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Install MS Store") ).clicked()
            {
                install_msstore();
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Install Work Teams") ).clicked()
            {
                launch("ms-windows-store://pdp/?ProductId=XP8BT8DW290MPQ");
            }         
            ui.heading("Uninstalls");
            ui.small("The actions below are destructive. Only use if you know what you are doing.");
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Personal Teams") ).clicked()
            {
                uninstall_teams();
            }
            if ui.add_sized([100.0, 40.0], egui::widgets::Button::new("Uninstall all defaults") ).clicked()
            {
                uninstall_defaults();
            }

            ui.add_space(20.0);
        });
    });
}

fn launch(app: &str) {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .arg(app)
    .spawn()
    .expect("failed to execute process");
}


fn winget_install(app: &str) {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powershell", "-window", "minimized","winget","install",app,"-h","--accept-package-agreements","--accept-source-agreements","--force"])
    .spawn()
    .expect("failed to execute process");
}

fn winget_update_all() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powershell", "-window", "minimized","winget","upgrade","--all","--include-unknown","-h","--accept-package-agreements","--accept-source-agreements","--force"])
    .spawn()
    .expect("failed to execute process");
}

fn display(ui: &mut Ui, section: &str, config: &Ini)
{
    let properties = config.section(Some(section)).unwrap();
    for (key, value) in properties.iter()
    {
        if ui.add_sized([100.0, 40.0], egui::widgets::Button::new(key)).clicked()
        {
            match section {

                section if section.eq("Programs") => {
                    program_launch(value);
                },
                section if section.eq("Powershell") => {
                    powershell_launch(value);
                },
                section if section.contains("-Online") => {
                    online_install(section, key, value);
                },
                section if section.contains("winget-") => {
                    winget_install(value);
                },
                _ => {},
            }
        }
    }
}


fn program_launch(path: &str)
{
    Command::new("cmd")
    .arg("/C")
    .arg(path)
    .creation_flags(0x08000000)
    .spawn()
    .expect("failed to execute process");  
}

fn powershell_launch(path: &str)
{
    Command::new("powershell")
    .args(["-executionpolicy","bypass","-File",path])
    .spawn()
    .expect("failed to execute process");  
}

fn online_install(section: &str, key: &str, value: &str)
{
    let sec = section.to_string();
    let name = format!("{}.{}",key, sec.split("-").next().unwrap().to_lowercase());
    let val = value.to_string();
    std::thread::spawn(move || {
        Command::new("curl.exe")
            .args(["-L", val.as_str(), "-o",name.as_str()])
            .output()
            .expect("failed to execute process");

        Command::new("cmd")
            .arg("/c")
            .arg(name)
            .spawn()
            .expect("failed to execute process"); 
    });

}


fn uninstall_defaults()
{
    Command::new("powershell")
    .arg("-windowstyle")
    .arg("minimized")
    .args(["{", "Get-AppxPackage", "|", "Remove-AppxPackage", "-ErrorAction", "SilentlyContinue", "-WarningAction", "SilentlyContinue", "-InformationAction", "SilentlyContinue","}"])
    .spawn()
    .expect("failed to execute process");  

    Command::new("powershell")
    .arg("-windowstyle")
    .arg("minimized")
    .args(["{", "Get-AppxPackage", "-AllUsers", "|", "Remove-AppxPackage", "-AllUsers", "-ErrorAction", "SilentlyContinue", "-WarningAction", "SilentlyContinue", "-InformationAction", "SilentlyContinue","}"])
    .spawn()
    .expect("failed to execute process");  
}

fn uninstall_teams()
{
    Command::new("powershell")
    .arg("-windowstyle")
    .arg("minimized")
    .args(["{", "Get-AppxPackage", "-AllUsers", "*teams*", "|", "Remove-AppxPackage", "-AllUsers", "-ErrorAction", "SilentlyContinue", "-WarningAction", "SilentlyContinue", "-InformationAction", "SilentlyContinue","}"])
    .spawn()
    .expect("failed to execute process");  
}

fn install_msstore()
{
    Command::new("powershell")
    .arg("-windowstyle")
    .arg("minimized")
    .args(["Add-AppxPackage", "-RegisterByFamilyName", "-MainPackage", "Microsoft.StorePurchaseApp_22403.1401.0.0_x64__8wekyb3d8bbwe"])
    .spawn()
    .expect("failed to execute process");

    Command::new("powershell")
    .arg("-windowstyle")
    .arg("minimized")
    .args(["Add-AppxPackage", "-RegisterByFamilyName", "-MainPackage", "Microsoft.WindowsStore_22403.1401.3.0_x64__8wekyb3d8bbwe"])
    .spawn()
    .expect("failed to execute process");  
    winget_auto_install();

 
}

fn winget_auto_install()
{
    Command::new("powershell")
    .arg("-windowstyle")
    .arg("minimized")
    .args(["Add-AppxPackage", "-RegisterByFamilyName", "-MainPackage", "Microsoft.DesktopAppInstaller_1.22.11132.0_x64__8wekyb3d8bbwe"])
    .spawn()
    .expect("failed to execute process"); 
}