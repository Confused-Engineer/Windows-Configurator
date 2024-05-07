use std::process::Command;
use egui::Ui;
//use std::sync::Arc;

pub fn page_troubleshoot(ui: &mut Ui, sys_info: &Vec<String>, sys_commands: &mut Vec<String>)
{
    
    ui.columns(2, |ui|{
        ui[0].heading("Options");
        ui[0].separator();
        egui::ScrollArea::vertical().id_source("TroubleshootOptions").show(&mut ui[0], |ui|{
            ui.columns(2, |ui|{

                /*
                ui[1].style_mut().text_styles.insert(
                    egui::TextStyle::Body, 
                    egui::FontId::new(14.0, eframe::epaint::FontFamily::Proportional),
                );
                 */


                ui[0].heading("Ping");
                ui[0].label("Enter IP");
                ui[0].text_edit_singleline(&mut sys_commands[0]);
                if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Ping") ).clicked()
                {
                    ping(sys_commands[0].to_string());
                }

                ui[1].heading("Trace Route");
                ui[1].label("Enter Domain");
                ui[1].text_edit_singleline(&mut sys_commands[1]);
                if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Trace Route") ).clicked()
                {
                    tracert(sys_commands[1].to_string());
                }

                ui[0].horizontal(|ui|{
                    ui.heading("SFC");
                    ui.small("(Need to be Administrator)")
                });
                
                if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Run SFC") ).clicked()
                {
                    sfc();
                }

                ui[1].horizontal(|ui|{
                    ui.heading("DISM");
                    ui.small("(Need to be Administrator)")
                });
                if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Run DISM") ).clicked()
                {
                    dism();
                }
                if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Restart") ).clicked()
                {
                    restart();
                }
                if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Restart to BIOS") ).clicked()
                {
                    restart_bios();
                }


                
                

                
            });

            ui.add_space(20.0);
        });
        ui[1].heading("System Info");
        ui[1].separator();
        egui::ScrollArea::vertical().id_source("TroubleshootInfo").show(&mut ui[1], |ui|{
            ui.heading("General");
            ui.label(sys_info[0].to_string());
            ui.separator();
            ui.heading("IPconfig");
            ui.label(sys_info[1].to_string());
            ui.add_space(20.0);
        });
    
    

});

}



fn ping(ip: String)
{

    Command::new("ping.exe")
    .arg(ip)
    .arg("-t")
    .spawn()
    .expect("failed to execute process");

}


fn tracert(ip: String)
{
    Command::new("cmd.exe")
    .args(["/C", "tracert",ip.as_str(),"&","timeout","60"])
    .spawn()
    .expect("failed to execute process");
}

fn sfc()
{
    Command::new("cmd.exe")
    .args(["/C", "SFC","/scannow","&","timeout","60"])
    .spawn()
    .expect("failed to execute process");
}

fn dism()
{
    Command::new("cmd.exe")
    .args(["/C", "DISM","/online","/cleanup-image","/restorehealth","&","timeout","60"])
    .spawn()
    .expect("failed to execute process");
}


fn restart() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["shutdown","/r","/t","1"])
    .spawn()
    .expect("failed to execute process");
}

fn restart_bios() {
    Command::new("cmd")
    .arg("/C")
    .arg("start")
    .args(["shutdown","/r","/fw","/t","1"])
    .spawn()
    .expect("failed to execute process");
}