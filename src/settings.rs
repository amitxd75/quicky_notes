//! Application settings model and presets.

use crate::theme::ThemeMode;
use serde::{Deserialize, Serialize};

/// Pre-defined window dimension presets for Quicky Notes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowSizePreset {
    pub label: &'static str,
    pub width: f32,
    pub height: f32,
}

impl WindowSizePreset {
    pub const COMPACT: Self = Self {
        label: "Compact",
        width: 780.0,
        height: 520.0,
    };
    pub const STANDARD: Self = Self {
        label: "Standard",
        width: 880.0,
        height: 580.0,
    };
    pub const WIDE: Self = Self {
        label: "Wide",
        width: 1024.0,
        height: 640.0,
    };
    pub const LARGE: Self = Self {
        label: "Large",
        width: 1180.0,
        height: 720.0,
    };

    /// Returns list of all available window size presets.
    pub fn all() -> &'static [Self] {
        &[Self::COMPACT, Self::STANDARD, Self::WIDE, Self::LARGE]
    }
}

/// User settings configuration, serialized to JSON disk storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Glass transparency opacity (0.30 ..= 1.00).
    pub opacity: f32,

    /// Text editor font size in points.
    pub font_size: f32,

    /// Whether to use monospace font family for code editing.
    pub monospace_font: bool,

    /// Whether the window remains pinned on top of other windows.
    pub always_on_top: bool,

    /// Dark mode flag.
    pub dark_mode: bool,

    /// Background auto-save interval in seconds.
    pub auto_save_seconds: u32,

    /// Persistent window width in pixels.
    pub window_width: f32,

    /// Persistent window height in pixels.
    pub window_height: f32,

    /// Selected theme mode (Wallpaper sync or preset palette).
    #[serde(default)]
    pub theme_mode: ThemeMode,

    /// Selected system font family name.
    #[serde(default)]
    pub selected_font: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            opacity: 0.85,
            font_size: 16.0,
            monospace_font: true,
            always_on_top: true,
            dark_mode: true,
            auto_save_seconds: 2,
            window_width: WindowSizePreset::STANDARD.width,
            window_height: WindowSizePreset::STANDARD.height,
            theme_mode: ThemeMode::WallpaperSync,
            selected_font: "Default".to_string(),
        }
    }
}
