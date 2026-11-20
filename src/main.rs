// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use eframe::{egui, egui_wgpu, wgpu};
use windows_configurator::{APP_NAME, Configurator, paths, platform};

/// Which renderer to start with. Selected automatically, or forced with
/// `--renderer <auto|wgpu|software|glow>` (or `WINCONFIG_RENDERER`) when
/// diagnosing a machine that will not draw the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Backend {
    /// DirectX 12 / Vulkan / OpenGL ES through wgpu, preferring real GPUs but
    /// accepting the software WARP adapter that every Windows install ships.
    Wgpu,
    /// wgpu forced onto a software (CPU) adapter.
    Software,
    /// Plain OpenGL through glow. Needs OpenGL 2+, which many VMs lack.
    Glow,
}

impl Backend {
    fn parse(s: &str) -> Option<Vec<Self>> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "" | "auto" => Self::AUTO.to_vec(),
            "wgpu" | "dx12" | "gpu" => vec![Self::Wgpu],
            "software" | "warp" | "cpu" => vec![Self::Software],
            "glow" | "opengl" | "gl" => vec![Self::Glow],
            _ => return None,
        })
    }

    /// Fallback order when nothing is forced.
    const AUTO: [Self; 3] = [Self::Wgpu, Self::Software, Self::Glow];
}

fn requested_backends() -> Vec<Backend> {
    let mut args = std::env::args().skip(1);
    let mut choice = std::env::var("WINCONFIG_RENDERER").ok();
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--renderer=") {
            choice = Some(value.to_owned());
        } else if arg == "--renderer" {
            choice = args.next();
        }
    }
    choice
        .and_then(|c| Backend::parse(&c))
        .unwrap_or_else(|| Backend::AUTO.to_vec())
}

fn native_options(backend: Backend) -> eframe::NativeOptions {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_app_id("WindowsConfigurator")
        .with_inner_size([1200.0, 650.0])
        // Small enough for an 800x600 / 1024x768 VM console.
        .with_min_inner_size([760.0, 420.0]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png")) {
        viewport = viewport.with_icon(icon);
    }

    let mut options = eframe::NativeOptions {
        viewport,
        renderer: match backend {
            Backend::Glow => eframe::Renderer::Glow,
            Backend::Wgpu | Backend::Software => eframe::Renderer::Wgpu,
        },
        // Lets us retry with another renderer on the same event loop.
        run_and_return: true,
        // Portable app: never write state to %APPDATA%.
        persistence_path: None,
        ..Default::default()
    };

    if let egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        // DX12 first: it is always present on Windows 10/11 and includes the
        // WARP software rasterizer used by Hyper-V / VMware / VirtualBox VMs
        // without 3D acceleration and by RDP sessions.
        setup.instance_descriptor.backends =
            wgpu::Backends::DX12 | wgpu::Backends::VULKAN | wgpu::Backends::GL;
        // Small allocation blocks: the default (performance) hint reserves
        // 256 MB chunks, which is wasteful for a few textured quads.
        let base_descriptor = setup.device_descriptor.clone();
        setup.device_descriptor = std::sync::Arc::new(move |adapter| wgpu::DeviceDescriptor {
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..base_descriptor(adapter)
        });
        let software_only = backend == Backend::Software;
        setup.native_adapter_selector = Some(std::sync::Arc::new(move |adapters, surface| {
            pick_adapter(adapters, surface, software_only)
        }));
    }
    options
}

/// Ranks adapters: hardware before software, DX12 before Vulkan before GL,
/// and only adapters that can present to the window.
fn pick_adapter(
    adapters: &[wgpu::Adapter],
    surface: Option<&wgpu::Surface<'_>>,
    software_only: bool,
) -> Result<wgpu::Adapter, String> {
    let rank = |a: &wgpu::Adapter| {
        let info = a.get_info();
        let software = info.device_type == wgpu::DeviceType::Cpu;
        let backend = match info.backend {
            wgpu::Backend::Dx12 => 0,
            wgpu::Backend::Vulkan => 1,
            _ => 2,
        };
        (software, backend)
    };
    adapters
        .iter()
        .filter(|a| surface.is_none_or(|s| a.is_surface_supported(s)))
        .filter(|a| !software_only || a.get_info().device_type == wgpu::DeviceType::Cpu)
        .min_by_key(|a| rank(a))
        .cloned()
        .ok_or_else(|| {
            let names: Vec<String> = adapters
                .iter()
                .map(|a| {
                    let i = a.get_info();
                    format!("{} ({:?}, {:?})", i.name, i.backend, i.device_type)
                })
                .collect();
            format!("no usable graphics adapter among: [{}]", names.join(", "))
        })
}

fn main() -> ExitCode {
    paths::init();

    let mut errors = Vec::new();
    for backend in requested_backends() {
        let result = eframe::run_native(
            APP_NAME,
            native_options(backend),
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                Ok(Box::new(Configurator::new(cc)))
            }),
        );
        match result {
            Ok(()) => return ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{backend:?} renderer failed: {err}");
                errors.push(format!("• {backend:?}: {err}"));
            }
        }
    }

    platform::message_box(
        APP_NAME,
        &format!(
            "The window could not be created with any renderer.\n\n{}\n\n\
             Try updating the graphics driver, or start the app with\n\
             --renderer software  or  --renderer glow",
            errors.join("\n")
        ),
    );
    ExitCode::FAILURE
}
