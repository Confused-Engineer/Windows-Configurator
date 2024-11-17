#![warn(clippy::all, rust_2018_idioms)]

mod tests;

mod app;
pub use app::Configurator;

pub mod multithread;
pub mod unpack_multi;

mod pages;
pub use pages::applications;
pub use pages::main;
pub use pages::settings;
pub use pages::troubleshooting;