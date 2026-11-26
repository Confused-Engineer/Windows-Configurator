//! Windows Configurator: a portable toolbox for setting up, troubleshooting
//! and maintaining Windows PCs, meant to be run from a USB drive.

pub mod app;
pub mod commands;
pub mod config;
pub mod cpu;
pub mod download;
pub mod paths;
pub mod platform;
pub mod sysinfo;
pub mod tasks;
pub mod winget;

mod pages;
mod ui;

pub use app::Configurator;

pub const APP_NAME: &str = "Windows Configurator";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPO_URL: &str = "https://github.com/Confused-Engineer/Windows-Configurator";
pub const UPDATE_URL: &str = "https://github.com/Confused-Engineer/Windows-Configurator/releases/download/nightly/Windows_Configurator.exe";
