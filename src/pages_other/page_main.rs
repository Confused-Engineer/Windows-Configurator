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
    ui.label("The intended way to use this application is by putting this exe in a folder with any installers or programs you may want to launch, adding their entries to the config file. Then put the folder either on a USB or computer and use it to help setup or troubleshoot as needed.");
    ui.add_space(8.0);
    ui.label("This app is intended to configure Windows and centralize a lot of miscellaneous settings. Some application settings are customizable through a config file. A generic config can be generated on any 'Apps' page.");
    ui.add_space(8.0);
    ui.label("This app does not come with a warranty and you (the user) assume any responsibility for actions taken by the application.");
}

