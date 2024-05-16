use egui::Ui;

pub fn show_page_main(ui: &mut Ui)
{
    ui.heading("Welcome");
    ui.heading("Navigation Tips");
    ui.add_space(8.0);
    ui.label("Explore the app by using the 'View' menu up top");
    ui.add_space(8.0);
    ui.label("The 'Options' menu can be used to reload information or restart as Administrator");
    ui.add_space(8.0);
    ui.label("This app is intended to configure Windows and centralize a lot of miscellaneous settings. Some application settings are customizable through a config file. A generic config should be provided.");
    ui.add_space(8.0);
    ui.label("This app does not come with a warranty and you (the user) assume any responsibility for actions taken by the application.");


}