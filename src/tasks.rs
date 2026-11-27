//! Background work. Threads report back through a channel and wake the UI
//! with `request_repaint`, so the UI only redraws when something changed
//! instead of spinning every frame.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::config::Config;
use crate::sysinfo::SysInfo;
use crate::winget;

/// Results delivered from background threads to the UI thread.
pub enum Event {
    /// A short message for the status bar.
    Status(String),
    /// The config file changed on disk (or appeared / disappeared).
    Config(Config),
    /// Outcome of probing for winget: its version, or `None` if missing.
    Winget(Option<String>),
    /// The full winget package catalog.
    Catalog(Result<Vec<winget::Package>, String>),
    SysInfo(SysInfo),
    /// Result of checking which version an install will pick, tagged with
    /// the dialog check it answers.
    Resolved(u64, Result<winget::Resolution, String>),
    /// Close the app (used by the self-updater after swapping executables).
    Quit,
}

/// Cloneable handle background threads use to talk to the UI.
#[derive(Clone)]
pub struct Bus {
    tx: Sender<Event>,
    ctx: egui::Context,
    running: Arc<AtomicUsize>,
}

impl Bus {
    pub fn new(ctx: egui::Context) -> (Self, Receiver<Event>) {
        let (tx, rx) = channel();
        (
            Self {
                tx,
                ctx,
                running: Arc::default(),
            },
            rx,
        )
    }

    pub fn send(&self, event: Event) {
        let _ = self.tx.send(event);
        self.ctx.request_repaint();
    }

    pub fn status(&self, message: impl Into<String>) {
        self.send(Event::Status(message.into()));
    }

    /// Number of tracked background jobs currently running.
    pub fn busy(&self) -> usize {
        self.running.load(Ordering::Relaxed)
    }

    /// Runs `job` on a new thread, counted in [`Self::busy`].
    pub fn spawn(&self, name: &str, job: impl FnOnce(&Bus) + Send + 'static) {
        struct Guard(Bus);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.running.fetch_sub(1, Ordering::Relaxed);
                self.0.ctx.request_repaint();
            }
        }

        self.running.fetch_add(1, Ordering::Relaxed);
        let guard = Guard(self.clone());
        let spawned = std::thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || {
                job(&guard.0);
                drop(guard);
            });
        if let Err(err) = spawned {
            self.status(format!("Could not start {name}: {err}"));
        }
    }

    /// Runs `job` on a new thread that is not shown as "busy" (watchers).
    pub fn spawn_daemon(&self, name: &str, job: impl FnOnce(Bus) + Send + 'static) {
        let bus = self.clone();
        let _ = std::thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || job(bus));
    }
}
