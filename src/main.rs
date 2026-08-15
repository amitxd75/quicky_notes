//! Quicky Notes - Floating glassmorphism note widget for Hyprland & Wayland desktops.
//!
//! Entry point for initializing application settings, window options, eframe runner,
//! and Hyprland Wayland viewport configuration.

mod app;
mod components;
mod crash;
mod font;
mod note;
mod settings;
mod storage;
mod theme;
mod ui;

use app::QuickyNotesApp;
use eframe::egui;

/// Main application entry point.
/// Loads saved settings/notes from disk and starts the `eframe` event loop.
fn main() -> Result<(), eframe::Error> {
    crash::install_crash_handler();

    let data = storage::AppData::load();
    let width = data.settings.window_width;
    let height = data.settings.window_height;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Quicky Notes")
            .with_inner_size([width, height])
            .with_min_inner_size([480.0, 340.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_app_id("quicky_notes"),
        ..Default::default()
    };

    eframe::run_native(
        "Quicky Notes",
        options,
        Box::new(move |cc| Ok(Box::new(QuickyNotesApp::new_with_data(cc, data)))),
    )
}
