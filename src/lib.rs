#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;
mod page_main;
mod page_apps;
mod page_tokens;
mod page_windows_settings;
mod page_troubleshoot;
mod page_debug;