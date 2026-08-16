//! Application settings model, window size presets, and invariant clamping.

use crate::theme::ThemeMode;
use serde::{Deserialize, Serialize};

/// Minimum allowed window opacity.
pub const MIN_OPACITY: f32 = 0.30;
/// Maximum allowed window opacity.
pub const MAX_OPACITY: f32 = 1.00;

/// Minimum allowed font size in points.
pub const MIN_FONT_SIZE: f32 = 8.0;
/// Maximum allowed font size in points.
pub const MAX_FONT_SIZE: f32 = 36.0;

/// Minimum allowed window width in pixels.
pub const MIN_WINDOW_WIDTH: f32 = 480.0;
/// Minimum allowed window height in pixels.
pub const MIN_WINDOW_HEIGHT: f32 = 340.0;

/// Maximum allowed window width in pixels.
pub const MAX_WINDOW_WIDTH: f32 = 3840.0;
/// Maximum allowed window height in pixels.
pub const MAX_WINDOW_HEIGHT: f32 = 2160.0;

/// Minimum allowed auto-save interval in seconds.
pub const MIN_AUTO_SAVE_SECS: u32 = 1;
/// Maximum allowed auto-save interval in seconds.
pub const MAX_AUTO_SAVE_SECS: u32 = 60;

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
    pub const fn all() -> &'static [Self] {
        &[Self::COMPACT, Self::STANDARD, Self::WIDE, Self::LARGE]
    }
}

fn default_custom_bg() -> [u8; 3] {
    [18, 12, 28]
}
fn default_custom_card() -> [u8; 3] {
    [28, 20, 42]
}
fn default_custom_border() -> [u8; 3] {
    [90, 50, 130]
}
fn default_custom_accent() -> [u8; 3] {
    [168, 85, 247]
}

/// User settings configuration, serialized to JSON disk storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Glass transparency opacity (0.30 ..= 1.00).
    pub opacity: f32,

    /// Text editor font size in points (8.0 ..= 36.0).
    pub font_size: f32,

    /// Whether to use monospace font family for code editing.
    pub monospace_font: bool,

    /// Whether the window remains pinned on top of other windows.
    pub always_on_top: bool,

    /// Dark mode flag.
    pub dark_mode: bool,

    /// Background auto-save interval in seconds (1 ..= 60).
    pub auto_save_seconds: u32,

    /// Persistent window width in pixels.
    pub window_width: f32,

    /// Persistent window height in pixels.
    pub window_height: f32,

    /// Selected theme mode (Wallpaper sync, custom, or preset palette).
    #[serde(default)]
    pub theme_mode: ThemeMode,

    /// Custom RGB background color [R, G, B].
    #[serde(default = "default_custom_bg")]
    pub custom_bg_color: [u8; 3],

    /// Custom RGB card container color [R, G, B].
    #[serde(default = "default_custom_card")]
    pub custom_card_color: [u8; 3],

    /// Custom RGB border tint color [R, G, B].
    #[serde(default = "default_custom_border")]
    pub custom_border_color: [u8; 3],

    /// Custom RGB accent color [R, G, B].
    #[serde(default = "default_custom_accent")]
    pub custom_accent_color: [u8; 3],

    /// Selected system font family name.
    #[serde(default)]
    pub selected_font: String,

    /// Whether to display line numbers in the editor gutter.
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,

    /// Tab indentation width in spaces (2 ..= 8).
    #[serde(default = "default_tab_size")]
    pub tab_size: u32,

    /// Whether to display the bottom status bar.
    #[serde(default = "default_true")]
    pub show_status_bar: bool,

    /// Whether to prompt before closing a tab.
    #[serde(default = "default_true")]
    pub confirm_close_tab: bool,

    /// Whether to automatically trim trailing spaces on save.
    #[serde(default = "default_true")]
    pub trim_trailing_whitespace: bool,

    /// Glass window corner radius roundness in pixels (4.0 ..= 24.0).
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,

    /// Default file extension for new notes (.txt or .md).
    #[serde(default = "default_extension")]
    pub default_extension: String,

    /// User-customizable keyboard shortcut keybindings map.
    #[serde(default)]
    pub keybindings: crate::ui::shortcuts::KeyBindings,

    /// AI Copilot and assistant configuration.
    #[serde(default)]
    pub ai: crate::ai::AiSettings,

    /// Whether to enable real-time language syntax highlighting in the editor.
    #[serde(default = "default_true")]
    pub enable_syntax_highlighting: bool,
}

const fn default_true() -> bool {
    true
}
const fn default_tab_size() -> u32 {
    4
}
const fn default_corner_radius() -> f32 {
    14.0
}
fn default_extension() -> String {
    ".txt".to_string()
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
            custom_bg_color: default_custom_bg(),
            custom_card_color: default_custom_card(),
            custom_border_color: default_custom_border(),
            custom_accent_color: default_custom_accent(),
            selected_font: "Default".to_string(),
            show_line_numbers: true,
            tab_size: 4,
            show_status_bar: true,
            confirm_close_tab: true,
            trim_trailing_whitespace: true,
            corner_radius: 14.0,
            default_extension: ".txt".to_string(),
            keybindings: crate::ui::shortcuts::KeyBindings::default(),
            ai: crate::ai::AiSettings::default(),
            enable_syntax_highlighting: true,
        }
    }
}

impl AppSettings {
    /// Validates all setting invariants and clamps values to safe bounds.
    pub fn validate_and_clamp(&mut self) {
        self.keybindings.ensure_all_actions_present();

        if self.opacity.is_nan() {
            self.opacity = 0.85;
        } else {
            self.opacity = self.opacity.clamp(MIN_OPACITY, MAX_OPACITY);
        }

        if self.font_size.is_nan() {
            self.font_size = 16.0;
        } else {
            self.font_size = self.font_size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
        }

        if self.window_width.is_nan() {
            self.window_width = MIN_WINDOW_WIDTH;
        } else {
            self.window_width = self.window_width.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
        }

        if self.window_height.is_nan() {
            self.window_height = MIN_WINDOW_HEIGHT;
        } else {
            self.window_height = self
                .window_height
                .clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);
        }

        self.auto_save_seconds = self
            .auto_save_seconds
            .clamp(MIN_AUTO_SAVE_SECS, MAX_AUTO_SAVE_SECS);

        self.tab_size = self.tab_size.clamp(2, 8);

        if self.corner_radius.is_nan() {
            self.corner_radius = 14.0;
        } else {
            self.corner_radius = self.corner_radius.clamp(4.0, 24.0);
        }

        if self.default_extension != ".txt" && self.default_extension != ".md" {
            self.default_extension = ".txt".to_string();
        }

        if self.selected_font.trim().is_empty() {
            self.selected_font = "Default".to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_validation_and_clamping() {
        let mut settings = AppSettings {
            opacity: -5.0,
            font_size: 100.0,
            monospace_font: false,
            always_on_top: false,
            dark_mode: true,
            auto_save_seconds: 0,
            window_width: 100.0,
            window_height: 50.0,
            theme_mode: ThemeMode::Custom,
            custom_bg_color: [0, 0, 0],
            custom_card_color: [0, 0, 0],
            custom_border_color: [0, 0, 0],
            custom_accent_color: [0, 0, 0],
            selected_font: "   ".to_string(),
            tab_size: 100,
            corner_radius: 500.0,
            default_extension: ".invalid".to_string(),
            ..AppSettings::default()
        };

        settings.validate_and_clamp();

        assert_eq!(settings.opacity, MIN_OPACITY);
        assert_eq!(settings.font_size, MAX_FONT_SIZE);
        assert_eq!(settings.auto_save_seconds, MIN_AUTO_SAVE_SECS);
        assert_eq!(settings.window_width, MIN_WINDOW_WIDTH);
        assert_eq!(settings.window_height, MIN_WINDOW_HEIGHT);
        assert_eq!(settings.tab_size, 8);
        assert_eq!(settings.corner_radius, 24.0);
        assert_eq!(settings.default_extension, ".txt");
        assert_eq!(settings.selected_font, "Default");

        // Test max clamping
        let mut max_settings = AppSettings {
            window_width: 10000.0,
            window_height: 10000.0,
            ..AppSettings::default()
        };
        max_settings.validate_and_clamp();
        assert_eq!(max_settings.window_width, MAX_WINDOW_WIDTH);
        assert_eq!(max_settings.window_height, MAX_WINDOW_HEIGHT);
    }

    #[test]
    fn test_settings_keybindings_backward_compatibility() {
        // Minimal JSON without keybindings field
        let json = r#"{"opacity": 0.9, "font_size": 14.0, "monospace_font": true, "always_on_top": true, "dark_mode": true, "auto_save_seconds": 2, "window_width": 800.0, "window_height": 600.0}"#;
        let mut settings: AppSettings = serde_json::from_str(json).expect("Deserialization failed");
        settings.validate_and_clamp();

        assert_eq!(
            settings
                .keybindings
                .get(crate::ui::shortcuts::ShortcutAction::NewNote),
            crate::ui::shortcuts::KeyBinding::ctrl("N")
        );
        assert_eq!(
            settings
                .keybindings
                .get(crate::ui::shortcuts::ShortcutAction::SearchNotes),
            crate::ui::shortcuts::KeyBinding::ctrl("K")
        );
    }
}
