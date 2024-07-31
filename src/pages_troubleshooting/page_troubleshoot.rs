use std::{os::windows::process::CommandExt, process::Command};
use egui::Ui;

use crate::app::TroubleshootInfo;
//use std::sync::Arc;

pub fn page_troubleshoot(ui: &mut Ui,sys_struct: &mut TroubleshootInfo)
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
                ui[0].text_edit_singleline(&mut sys_struct.ping);
                if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Ping") ).clicked()
                {
                    launch(&format!("ping {} -t", sys_struct.ping.to_string()), false);
                }

                ui[1].heading("Trace Route");
                ui[1].label("Enter Domain");
                ui[1].text_edit_singleline(&mut sys_struct.tracert);
                if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Trace Route") ).clicked()
                {
                    launch(&format!("tracert {} & timeout 60", sys_struct.tracert.to_string()), false);
                }

                ui[0].horizontal(|ui|{
                    ui.heading("SFC");
                    ui.small("(Need to be Administrator)")
                });
                
                if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Run SFC") ).clicked()
                {
                    launch("sfc /scannow & timeout 60", false);
                }

                ui[1].horizontal(|ui|{
                    ui.heading("DISM");
                    ui.small("(Need to be Administrator)")
                });
                if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Run DISM") ).clicked()
                {
                    launch("DISM /online /cleanup-image /restorehealth & timeout 60", false);
                }
                if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Restart") ).clicked()
                {
                    launch("shutdown /r /t 1", true);
                }
                if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Restart to BIOS") ).clicked()
                {
                    launch("shutdown /r /fw /t 1", true);
                }
                ui[0].horizontal(|ui|{
                    ui.heading("Clear RAM");
                    ui.small("(Should be Admin)")
                });
                
                if ui[0].add_sized([100.0, 40.0], egui::widgets::Button::new("Clear RAM") ).clicked()
                {
                    launch("powershell -windowstyle hidden -EncodedCommand PAAjAA0ACgAgACAAIAAgAC4AUwBZAE4ATwBQAFMASQBTAA0ACgAgACAAIAAgAFQAaABpAHMAIAB0AG8AbwBsACAAdQBzAGUAcwAgAGEAIABwAHIAbwBnAHIAYQBtACAAbwBmAGYAZQByAGUAZAAgAGIAeQAgAG0AaQBjAHIAbwBzAG8AZgB0ACAAdABvACAAYwBsAGUAYQByACAAUgBBAE0AIAB3AGkAdABoAG8AdQB0ACAAYQAgAHIAZQBiAG8AbwB0ACAAbwByACAAdQBzAGUAcgAgAGkAbgB0AGUAcgBhAGMAdABpAG8AbgANAAoADQAKAA0ACgAgACAAIAAgAC4ARABFAFMAQwBSAEkAUABUAEkATwBOAA0ACgAgACAAIAAgAFIAQQBNAE0AYQBwAC4AZQB4AGUAIABpAHMAIABkAG8AdwBuAGwAbwBhAGQAZQBkACAAZgByAG8AbQAgAGgAdAB0AHAAcwA6AC8ALwBsAGkAdgBlAC4AcwB5AHMAaQBuAHQAZQByAG4AYQBsAHMALgBjAG8AbQAvAFIAQQBNAE0AYQBwAC4AZQB4AGUAIAB3AGgAaQBjAGgAIABpAHMAIABvAHcAbgBlAGQAIABhAG4AZAAgAG8AcABlAHIAYQB0AGUAZAAgAGIAeQAgAE0AaQBjAHIAbwBzAG8AZgB0ACAAYQBuAGQAIABvAGYAZgBlAHIAcwAgAGMAbwBtAG0AYQBuAGQAIABsAGkAbgBlACAAbwBwAHQAaQBvAG4AcwAgAHQAbwAgAGMAbABlAGEAcgAgAFIAQQBNAC4ADQAKACAAIAAgACAAQgB5ACAAZABvAHcAbgBsAG8AYQBkAGkAbgBnACAAaQB0ACAAdABvACAAYQAgAGQAaQByAGUAYwB0AG8AcgB5ACAAYQBuAGQAIAByAHUAbgBuAGkAbgBnACAAaQB0ACAAYQBzACAAYQBkAG0AaQBuAGkAcwB0AHIAYQB0AG8AcgAgAHcAZQAgAGEAcgBlACAAYQBiAGwAZQAgAHQAbwAgAGMAbABlAGEAcgAgAG8AdQB0ACAAUgBBAE0AIAB3AGkAdABoAG8AdQB0ACAAYQBuAHkAIAB1AHMAZQByACAAaQBuAHQAZQByAGEAYwB0AGkAbwBuACAAbwByACAAaQBuAHQAZQByAHUAcAB0AGkAbwBuAHMALgANAAoADQAKACMAPgANAAoADQAKACMAVgBhAHIAaQBhAGIAbABlAHMADQAKAA0ACgAkAFcAZQBiAEYAaQBsAGUAIAA9ACAAJwBoAHQAdABwAHMAOgAvAC8AbABpAHYAZQAuAHMAeQBzAGkAbgB0AGUAcgBuAGEAbABzAC4AYwBvAG0ALwBSAEEATQBNAGEAcAAuAGUAeABlACcADQAKACQATABvAGMAYQBsAEYAaQBsAGUAIAA9ACAAJwBDADoAXAB0AGUAbQBwAFwAUgBBAE0ATQBhAHAALgBlAHgAZQAnAA0ACgAkAEEAcgBnAEwAaQBzAHQAIAA9ACAAQAAoACIALQBFAHcAIgAsACIALQBFAHMAIgAsACIALQBFAG0AIgAsACIALQBFAHQAIgAsACIALQBFADAAIgApAA0ACgANAAoAJABSAGUAZwBQAGEAdABoACAAPQAgACIASABLAEMAVQA6AFwAUwBvAGYAdAB3AGEAcgBlAFwAUwB5AHMAaQBuAHQAZQByAG4AYQBsAHMAXABSAEEATQBNAGEAcAAiAA0ACgAkAFIAZQBnAEkAdABlAG0AIAA9ACAAIgBFAHUAbABhAEEAYwBjAGUAcAB0AGUAZAAiAA0ACgAkAFIAZQBnAFYAYQBsAHUAZQAgAD0AIAAxAA0ACgANAAoADQAKACMATQBhAGsAZQAgAFQAZQBtAHAAIABmAG8AbABkAGUAcgAgAGkAZgAgAGkAdAAgAGQAbwBlAHMAbgB0ACAAZQB4AGkAcwB0AA0ACgANAAoATgBlAHcALQBJAHQAZQBtACAALQBJAHQAZQBtAFQAeQBwAGUAIABEAGkAcgBlAGMAdABvAHIAeQAgAC0AUABhAHQAaAAgACIAQwA6AFwAdABlAG0AcAAiACAALQBFAHIAcgBvAHIAQQBjAHQAaQBvAG4AIABTAGkAbABlAG4AdABsAHkAQwBvAG4AdABpAG4AdQBlAA0ACgANAAoAIwBkAG8AdwBuAGwAbwBhAGQAIABuAGUAZQBkAGUAZAAgAHAAcgBvAGcAcgBhAG0ADQAKAA0ACgBJAG4AdgBvAGsAZQAtAFcAZQBiAFIAZQBxAHUAZQBzAHQAIAAtAFUAcgBpACAAJABXAGUAYgBGAGkAbABlACAALQBPAHUAdABGAGkAbABlACAAJABMAG8AYwBhAGwARgBpAGwAZQANAAoADQAKACMAQQBkAGQAIAByAGUAZwBpAHMAdAByAHkAIAB2AGEAbAB1AGUAIABzAG8AIAB0AGgAZQAgAEUAVQBMAEEAIABkAG8AZQBzACAAbgBvAHQAIABwAG8AcAAgAHUAcAAgAGYAbwByACAAdQBzAGUAcgAsACAAbgBlAGUAZABlAGQAIAB0AGgAZQAgAHMAbABlAGUAcAAgAHMAbwAgAHQAaABlACAAcgBlAGcAaQBzAHQAcgB5ACAAYwBhAG4AIABjAGwAbwBzAGUAIABiAGUAZgBvAHIAZQAgAFIAQQBNAE0AYQBwACAAdAByAGkAZQBzACAAdABvACAAYwBoAGUAYwBrACAAaQB0ACAAKABJACAAdABoAGkAbgBrACkADQAKAGkAZgAoACgAKABHAGUAdAAtAEkAdABlAG0AUAByAG8AcABlAHIAdAB5ACAALQBQAGEAdABoACAAJABSAGUAZwBQAGEAdABoACAALQBOAGEAbQBlACAAJABSAGUAZwBJAHQAZQBtACAALQBFAHIAcgBvAHIAQQBjAHQAaQBvAG4AIABTAGkAbABlAG4AdABsAHkAQwBvAG4AdABpAG4AdQBlACkALgAkAFIAZQBnAEkAdABlAG0AKQAgAC0AbgBlACAAJABSAGUAZwBWAGEAbAB1AGUAKQAgACAAewANAAoADQAKACAAIAAgACAATgBlAHcALQBJAHQAZQBtACAALQBQAGEAdABoACAAJABSAGUAZwBQAGEAdABoACAALQBGAG8AcgBjAGUAIAB8ACAATwB1AHQALQBOAHUAbABsAA0ACgAgACAAIAAgAE4AZQB3AC0ASQB0AGUAbQBQAHIAbwBwAGUAcgB0AHkAIAAtAFAAYQB0AGgAIAAkAFIAZQBnAFAAYQB0AGgAIAAtAE4AYQBtAGUAIAAkAFIAZQBnAEkAdABlAG0AIAAtAFYAYQBsAHUAZQAgACQAUgBlAGcAVgBhAGwAdQBlACAALQBQAHIAbwBwAGUAcgB0AHkAVAB5AHAAZQAgAEQAVwBPAFIARAAgAC0ARgBvAHIAYwBlACAAfAAgAE8AdQB0AC0ATgB1AGwAbAANAAoADQAKAH0ADQAKAA0ACgBTAHQAYQByAHQALQBTAGwAZQBlAHAAIAAtAFMAZQBjAG8AbgBkAHMAIAAxAA0ACgANAAoADQAKACMAUwB0AGEAcgB0ACAAZQBhAGMAaAAgAGEAbgBkACAAdABoAGUAbgAgAHMAbABlAGUAcAAgAGYAbwByACAAWAAgAHMAZQBjAG8AbgBkAHMAIAB0AG8AIABhAHYAbwBpAGQAIAAiAHIAcABvAGMAZQBzAHMAIABiAGUAaQBuAGcAIAB1AHMAZQBkACAAYgB5ACAAYQBuAG8AdABoAGUAcgAgAHAAcgBvAGcAcgBhAG0AIgAgAGUAcgByAG8AcgANAAoAZgBvAHIAZQBhAGMAaAAgACgAJABBAHIAZwAgAGkAbgAgACQAQQByAGcATABpAHMAdAApAHsADQAKACAAIAAgACAAJgAgACIAJABMAG8AYwBhAGwARgBpAGwAZQAiACAAQAAoACIAJABBAHIAZwAiACkADQAKACAAIAAgACAAUwB0AGEAcgB0AC0AUwBsAGUAZQBwACAAMgAwAA0ACgB9AA0ACgANAAoADQAKACMAYwBvAG0AbQBlAG4AdABlAGQAIABvAHUAdAAgAGIAdQB0ACAAdABoAGkAcwAgAGkAcwAgAHcAaABhAHQAIABpAHMAIABuAGUAZQBkAGUAZAAgAHQAbwAgAHMAaABvAHcAIABjAG8AbQBtAGEAbgBkACAAbABpAG4AZQAgAGEAcgBnAHUAbQBlAG4AdABzAA0ACgAjACYAIAAiACQATABvAGMAYQBsAEYAaQBsAGUAIgAgAEAAKAAiAC0ALQBoAGUAbABwACIAKQA=", true);
                }

                ui[1].horizontal(|ui|{
                    ui.heading("Set Time to EST");
                    ui.small("(Need Admin)")
                });
                
                if ui[1].add_sized([100.0, 40.0], egui::widgets::Button::new("Set and Sync Time") ).clicked()
                {
                    launch("powershell -windowstyle hidden -EncodedCommand UwBlAHQALQBUAGkAbQBlAFoAbwBuAGUAIAAtAEkAZAAgACIARQBhAHMAdABlAHIAbgAgAFMAdABhAG4AZABhAHIAZAAgAFQAaQBtAGUAIgANAAoAUwB0AGEAcgB0AC0AUAByAG8AYwBlAHMAcwAgACIAdwAzADIAdABtAC4AZQB4AGUAIgAgAC0AQQByAGcAdQBtAGUAbgB0AEwAaQBzAHQAIABAACgAIgAvAHUAbgByAGUAZwBpAHMAdABlAHIAIgApACAALQBXAGEAaQB0ACAALQBOAG8ATgBlAHcAVwBpAG4AZABvAHcADQAKAFMAdABhAHIAdAAtAFAAcgBvAGMAZQBzAHMAIAAiAHcAMwAyAHQAbQAuAGUAeABlACIAIAAtAEEAcgBnAHUAbQBlAG4AdABMAGkAcwB0ACAAQAAoACIALwByAGUAZwBpAHMAdABlAHIAIgApACAALQBXAGEAaQB0ACAALQBOAG8ATgBlAHcAVwBpAG4AZABvAHcADQAKAFMAdABhAHIAdAAtAFMAZQByAHYAaQBjAGUAIABXADMAMgBUAGkAbQBlAA0ACgANAAoAUwB0AGEAcgB0AC0AUAByAG8AYwBlAHMAcwAgACIAdwAzADIAdABtAC4AZQB4AGUAIgAgAC0AQQByAGcAdQBtAGUAbgB0AEwAaQBzAHQAIABAACgAIgAvAHIAZQBzAHkAbgBjACIAKQAgAC0AVwBhAGkAdAAgAC0ATgBvAE4AZQB3AFcAaQBuAGQAbwB3AA0ACgBTAHQAbwBwAC0AUwBlAHIAdgBpAGMAZQAgAFcAMwAyAFQAaQBtAGUA", true);
                }


                
                

                
            });

            ui.add_space(20.0);
        });
        ui[1].heading("System Info");
        ui[1].separator();
        egui::ScrollArea::vertical().id_source("TroubleshootInfo").show(&mut ui[1], |ui|{
            ui.heading("General");


            ui.heading("Sysinfo");
            if !sys_struct.sys_info_receiver.is_empty()
            {
                sys_struct.systeminfo = sys_struct.sys_info_receiver.try_recv().unwrap();
            }
            ui.label(&sys_struct.systeminfo);

            ui.separator();

            ui.heading("IP Config Info");
            if !sys_struct.ipconfig_info_receiver.is_empty()
            {
                sys_struct.ipconfig = sys_struct.ipconfig_info_receiver.try_recv().unwrap();
            }
            ui.label(&sys_struct.ipconfig);
            ui.add_space(20.0);


        });
    
    

    });

}


fn launch(command: &str, hidden: bool) {

    let app_vec: Vec<&str> = command.split(" ").collect();
    let mut command = Command::new("cmd");
    command.args(["/C"]);
    for x in 0..app_vec.len()
    {
        command.arg(app_vec[x]);
    }
    if hidden
    {
       command.creation_flags(0x08000000); 
    }
    
    let _ = command.spawn().expect("failed to execute process");
}