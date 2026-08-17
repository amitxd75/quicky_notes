//! Quicky Notes - Floating glassmorphism note widget and code scratchpad for Linux.
//!
//! Entry point for initializing application settings, window options, eframe runner,
//! and Wayland / X11 desktop viewport configuration.

mod app;
pub mod components;
pub mod engine;
pub mod models;
pub mod platform;
pub mod plugins;
pub mod storage;
pub mod theme;
pub mod ui;

// Top-level re-exports for clean ergonomic access across submodules
pub use engine::ai;
pub use engine::suggest;
pub use models::note;
pub use models::settings;
pub use platform::crash;
pub use platform::font;
pub use plugins::PluginManager;

use app::QuickyNotesApp;
use eframe::egui;

/// Loads the embedded application icon from `assets/icon.png`.
fn load_app_icon() -> Option<egui::IconData> {
    const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
    if let Ok(img) = image::load_from_memory(ICON_BYTES) {
        let resized = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

/// Main application entry point.
/// Parses CLI arguments, loads saved settings/notes from disk, and starts the `eframe` event loop.
fn main() -> Result<(), eframe::Error> {
    platform::install_crash_handler();

    let cli = platform::CliArgs::parse(std::env::args().skip(1));
    if cli.show_help {
        platform::CliArgs::print_help();
        return Ok(());
    }
    if cli.show_version {
        platform::CliArgs::print_version();
        return Ok(());
    }

    let data = storage::AppData::load();
    let width = data.settings.window.width;
    let height = data.settings.window.height;

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Quicky Notes")
        .with_inner_size([width, height])
        .with_min_inner_size([480.0, 340.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_app_id("quicky_notes");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Quicky Notes",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(QuickyNotesApp::new_with_data_and_cli(
                cc, data, cli,
            )))
        }),
    )
}
