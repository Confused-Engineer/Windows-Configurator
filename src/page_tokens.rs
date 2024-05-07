use egui::Ui;
use ini::Ini;
pub fn show_page_tokens(ui: &mut Ui, config: &Ini)
{
    ui.heading("Tokens");
    ui.label("Here are any listed tokens in the config file. Note that if '\\' is present in a token, you need to make it a double back slash '\\\\' to show up properly as it is an escape character.");
                for (key, _) in config
            {
                if key != None
                {
                    if key.unwrap().eq("Tokens")
                    {
                        
        
                        let properties = config.section(Some(key.unwrap())).unwrap();
                        for (key, value) in properties.iter()
                        {
                            ui.horizontal(|ui|{
                                ui.label(format!("{key}: "));
                                ui.small(value);
                            });
                        }
                    }
                }  
            }
            ui.add_space(20.0);

}


