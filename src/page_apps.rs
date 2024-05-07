use egui::Ui;
use ini::Ini;
use std::process::Command;

pub fn page_apps(ui: &mut Ui, config: &Ini)
{


    ui.columns(3, |ui|{
        ui[0].heading("Click to Install Apps");
        egui::ScrollArea::vertical().id_source("InstallApps").show(&mut ui[0], |ui|{

            for (key, _) in config
            {
                if key != None
                {
                    if key.unwrap().contains("winget")
                    {
                        display_section(ui, key.unwrap(), config, "winget-")
                    }
                }  
            }
            ui.add_space(20.0);
        });
        

        ui[1].heading("Config Specified Installs");
        egui::ScrollArea::vertical().id_source("ConfigApps").show(&mut ui[1], |ui|{
            for (key, _) in config
            {
                if key != None
                {
                    if key.unwrap().contains("EXE")
                    {
                        if key.unwrap().contains("Local")
                        {
                            display_section(ui, key.unwrap(), config, "-")
                        }
                        if key.unwrap().contains("Online")
                        {
                            display_section(ui, key.unwrap(), config, "-")
                        }
                    }
                    if key.unwrap().contains("MSI")
                    {
                        if key.unwrap().contains("Local")
                        {
                            display_section(ui, key.unwrap(), config, "-")
                        }
                        if key.unwrap().contains("Online")
                        {
                            display_section(ui, key.unwrap(), config, "-")
                        }
                    }






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

#[tokio::main]
async fn launch(app: &str) {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .arg(app)
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn winget_install(app: &str) {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powershell", "-window", "minimized","winget","install",app,"-h","--accept-package-agreements","--accept-source-agreements","--force"])
    .spawn()
    .expect("failed to execute process");
}

#[tokio::main]
async fn winget_update_all() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["powershell", "-window", "minimized","winget","upgrade","--all","--include-unknown","-h","--accept-package-agreements","--accept-source-agreements","--force"])
    .spawn()
    .expect("failed to execute process");
}

fn exe_local_install(path: &str)
{
    Command::new("cmd")
    .arg("/C")
    .arg(path)
    .spawn()
    .expect("failed to execute process");  
}

fn exe_online_install(name: &str, path: &str)
{
    let mut new_name = name.to_string();
    new_name += ".exe";
    Command::new("curl.exe")
    .args(["-L", path, "-o",new_name.as_str()])
    .output()
    .expect("failed to execute process");

    Command::new("cmd")
    .arg("/C")
    .arg(new_name)
    .spawn()
    .expect("failed to execute process");
}

fn msi_local_install(path: &str)
{
    Command::new("cmd")
    .arg("/c")
    .arg("msiexec")
    .arg("/i")
    .arg(path)
    .spawn()
    .expect("failed to execute process");  
}

fn msi_online_install(name: &str, path: &str)
{
    let mut new_name = name.to_string();
    new_name += ".msi";
    Command::new("curl.exe")
    .args(["-L", path, "-o",new_name.as_str()])
    .output()
    .expect("failed to execute process");

    Command::new("cmd")
    .arg("/c")
    .arg("msiexec")
    .arg("/i")
    .arg(new_name)
    .spawn()
    .expect("failed to execute process"); 
}

fn display_section(ui: &mut Ui, section: &str, config: &Ini, replace: &str)
{
    ui.label(section.replace(replace, " "));
        
    let properties = config.section(Some(section)).unwrap();
    for (key, value) in properties.iter()
    {
        if ui.add_sized([100.0, 40.0], egui::widgets::Button::new(key)).clicked()
        {

            if replace.eq("winget-")
            {
                winget_install(value);
            } else if replace.eq("-")
            {
                if section.contains("EXE")
                {
                    if section.contains("Online")
                    {
                        exe_online_install(key, value);
                        println!("{key} {value}");
                    }
                    if section.contains("Local")
                    {
                        exe_local_install(value);
                        println!("le {value}");
                    }
                }

                if section.contains("MSI")
                {
                    if section.contains("Online")
                    {
                        msi_online_install(key, value);
                        println!("om {value}");
                    }
                    if section.contains("Local")
                    {
                        msi_local_install(value);
                        println!("lm {value}");
                    }
                }
            }
            
        }
    }
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