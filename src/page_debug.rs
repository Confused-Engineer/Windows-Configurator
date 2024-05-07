use egui::Ui;
use std::env;
use ini::Ini;
pub fn show_page_debug(ui: &mut Ui, config: &Ini)
{
    ui.heading("Debug");
    ui.label("Here are any listed tokens in the config file.");
    ui.label(get_file_path());
    //Properties::new();
    //let sections = config.section(Some("winget-browsers")).unwrap();
    if ui.button("print sections").clicked()
    {


    }

    for (key, _) in config
    {

        if key != None
        {
            
            if key.unwrap().contains("winget")
            {

                display_section(ui, key.unwrap(), config);

            }


        }
        
    }
    

}



fn get_file_path() -> String
{
    let exe_path = env::current_exe().unwrap().as_path().display().to_string();
    let split_exe_path: Vec<&str> = exe_path.split("\\").collect();
    split_exe_path[split_exe_path.len()-1].to_string()
    
    
}

#[allow(dead_code)]
fn show_config()
{
    let config: Ini = Ini::load_from_file("config.ini").unwrap();
    let exes = config.section(Some("exes")).unwrap();
    for (key, value) in exes.iter()
    {
        println!("{key} {value}");
    }
}


fn display_section(ui: &mut Ui, section: &str, config: &Ini)
{
    ui.label(section.replace("-", " "));
        
    let properties = config.section(Some(section)).unwrap();
    for (key, value) in properties.iter()
    {
        if ui.add_sized([100.0, 40.0], egui::widgets::Button::new(key)).clicked()
        {
            println!("{value}");
        }
    }
}