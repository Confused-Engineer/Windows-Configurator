// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use windows_configurator::{APP_NAME, Configurator, cpu, paths, platform};

fn viewport() -> egui::ViewportBuilder {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_app_id("WindowsConfigurator")
        .with_inner_size([1200.0, 650.0])
        // Small enough for an 800x600 / 1024x768 VM console.
        .with_min_inner_size([760.0, 420.0]);
    // Title bar / taskbar icon; the exe icon comes from build.rs.
    if let Ok(png) = image::load_from_memory(include_bytes!("../assets/icon-256.png")) {
        let rgba = png.to_rgba8();
        viewport = viewport.with_icon(egui::IconData {
            width: rgba.width(),
            height: rgba.height(),
            rgba: rgba.into_raw(),
        });
    }
    viewport
}

fn main() -> ExitCode {
    paths::init();
    match cpu::run(viewport(), Box::new(Configurator::new)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            platform::message_box(
                APP_NAME,
                &format!("The window could not be created.\n\n{e}"),
            );
            ExitCode::FAILURE
        }
    }
}
