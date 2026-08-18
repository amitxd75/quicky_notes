//! Cross-platform desktop integrations, system font loading, hardware diagnostics, and crash handling.

pub mod cli;
pub mod crash;
pub mod diagnostics;
pub mod font;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(windows)]
pub mod windows;

pub use cli::CliArgs;
pub use crash::install_crash_handler;
pub use font::{
    apply_system_font, apply_system_fonts, get_installed_monospace_fonts,
    get_installed_system_fonts, setup_fonts_async,
};

/// Synchronizes system autostart configuration for the current platform.
pub fn sync_autostart_desktop_entry(enabled: bool, custom_flags: &str) {
    #[cfg(target_os = "linux")]
    linux::sync_autostart_desktop_entry(enabled, custom_flags);

    #[cfg(windows)]
    windows::sync_autostart_desktop_entry(enabled, custom_flags);
}

/// Convenience autostart synchronizer without custom flags.
pub fn sync_autostart_desktop_file(enabled: bool) {
    sync_autostart_desktop_entry(enabled, "");
}
