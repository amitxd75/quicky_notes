//! Application settings model, window size presets, and invariant clamping.

use crate::theme::ThemeMode;
use serde::{Deserialize, Serialize};

/// Minimum allowed window opacity.
pub const MIN_OPACITY: f32 = 0.30;
/// Maximum allowed window opacity.
pub const MAX_OPACITY: f32 = 1.00;
/// Default window opacity.
pub const DEFAULT_OPACITY: f32 = 0.85;

/// Minimum allowed font size in points.
pub const MIN_FONT_SIZE: f32 = 8.0;
/// Maximum allowed font size in points.
pub const MAX_FONT_SIZE: f32 = 36.0;
/// Default editor font size in points.
pub const DEFAULT_FONT_SIZE: f32 = 16.0;

/// Minimum allowed system/UI font size in points.
pub const MIN_UI_FONT_SIZE: f32 = 10.0;
/// Maximum allowed system/UI font size in points.
pub const MAX_UI_FONT_SIZE: f32 = 24.0;
/// Default system/UI font size in points.
pub const DEFAULT_UI_FONT_SIZE: f32 = 13.5;

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
/// Default auto-save interval in seconds.
pub const DEFAULT_AUTO_SAVE_SECS: u32 = 2;

/// Minimum allowed glass blur strength / hardness.
pub const MIN_BLUR_STRENGTH: f32 = 0.0;
/// Maximum allowed glass blur strength / hardness.
pub const MAX_BLUR_STRENGTH: f32 = 1.0;
/// Default glass blur strength / hardness.
pub const DEFAULT_BLUR_STRENGTH: f32 = 0.75;

/// Minimum allowed tab indentation width in spaces.
pub const MIN_TAB_SIZE: u32 = 2;
/// Maximum allowed tab indentation width in spaces.
pub const MAX_TAB_SIZE: u32 = 8;
/// Default tab indentation width in spaces.
pub const DEFAULT_TAB_SIZE: u32 = 4;

/// Minimum window corner radius in pixels.
pub const MIN_CORNER_RADIUS: f32 = 4.0;
/// Maximum window corner radius in pixels.
pub const MAX_CORNER_RADIUS: f32 = 24.0;
/// Default window corner radius in pixels.
pub const DEFAULT_CORNER_RADIUS: f32 = 14.0;

/// Pre-defined window dimension presets for Quicky Notes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowSizePreset {
    pub label: &'static str,
    pub width: f32,
    pub height: f32,
    pub default_font_size: f32,
}

impl WindowSizePreset {
    pub const COMPACT: Self = Self {
        label: "Compact",
        width: 780.0,
        height: 520.0,
        default_font_size: 13.5,
    };
    pub const STANDARD: Self = Self {
        label: "Standard",
        width: 880.0,
        height: 580.0,
        default_font_size: 15.0,
    };
    pub const WIDE: Self = Self {
        label: "Wide",
        width: 1024.0,
        height: 640.0,
        default_font_size: 16.0,
    };
    pub const LARGE: Self = Self {
        label: "Large",
        width: 1180.0,
        height: 720.0,
        default_font_size: 17.5,
    };
    pub const XL: Self = Self {
        label: "XL",
        width: 1360.0,
        height: 820.0,
        default_font_size: 19.0,
    };
    pub const ULTRAWIDE: Self = Self {
        label: "Ultrawide",
        width: 1560.0,
        height: 900.0,
        default_font_size: 20.5,
    };
    pub const STUDIO: Self = Self {
        label: "Studio 2K",
        width: 1780.0,
        height: 1000.0,
        default_font_size: 22.0,
    };

    /// Returns list of all available window size presets.
    pub const fn all() -> &'static [Self] {
        &[
            Self::COMPACT,
            Self::STANDARD,
            Self::WIDE,
            Self::LARGE,
            Self::XL,
            Self::ULTRAWIDE,
            Self::STUDIO,
        ]
    }
}

/// Custom theme color palette for user-defined styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomThemeColors {
    /// Background surface color [R, G, B].
    pub bg: [u8; 3],
    /// Card container surface color [R, G, B].
    pub card: [u8; 3],
    /// Border stroke tint color [R, G, B].
    pub border: [u8; 3],
    /// Primary accent highlight color [R, G, B].
    pub accent: [u8; 3],
    /// Secondary accent / pill color [R, G, B].
    pub secondary_accent: [u8; 3],
    /// Primary text typography color [R, G, B].
    pub text: [u8; 3],
    /// Muted text / secondary label color [R, G, B].
    pub muted_text: [u8; 3],
    /// Danger / destructive action color [R, G, B].
    pub danger: [u8; 3],
}

impl Default for CustomThemeColors {
    fn default() -> Self {
        Self {
            bg: [18, 12, 28],
            card: [28, 20, 42],
            border: [90, 50, 130],
            accent: [168, 85, 247],
            secondary_accent: [56, 189, 248],
            text: [235, 240, 250],
            muted_text: [155, 165, 180],
            danger: [239, 68, 68],
        }
    }
}

/// General application, system startup, and workspace defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    /// Whether Quicky Notes is registered in ~/.config/autostart for system login.
    pub autostart: bool,
    /// Whether to restore the previous workspace folder and note tabs on startup.
    pub restore_session: bool,
    /// Whether to continuously watch and live-sync externally modified files on disk.
    pub live_disk_sync: bool,
    /// Whether to automatically sync theme colors with dynamic wallpaper tools (Pywal/Caelestia).
    pub auto_sync_wallpaper: bool,
    /// Whether to auto-derive note tab title from the first line of content.
    pub auto_title_from_first_line: bool,
    /// Default title naming prefix for new notes (e.g. "Note", "Scratchpad").
    pub default_title_prefix: String,
    /// Whether to automatically close brackets, quotes, and markdown backticks.
    pub auto_close_brackets: bool,
    /// Whether to strip ANSI terminal escape color codes when pasting text.
    pub strip_ansi_on_paste: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            restore_session: true,
            live_disk_sync: true,
            auto_sync_wallpaper: true,
            auto_title_from_first_line: true,
            default_title_prefix: "note".to_string(),
            auto_close_brackets: true,
            strip_ansi_on_paste: true,
        }
    }
}

impl GeneralSettings {
    /// Validates and ensures defaults for general settings.
    pub fn validate_and_clamp(&mut self) {
        if self.default_title_prefix.trim().is_empty() {
            self.default_title_prefix = "note".to_string();
        }
    }
}

/// Visual appearance, glassmorphism styling, and active palette configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceSettings {
    /// Selected theme mode (Wallpaper sync, custom, or preset palette).
    pub theme_mode: ThemeMode,
    /// Glass transparency opacity (0.30 ..= 1.00).
    pub opacity: f32,
    /// Glass blur hardness and surface specular strength (0.0 ..= 1.0).
    pub blur_strength: f32,
    /// Glass window corner radius roundness in pixels (4.0 ..= 24.0).
    pub corner_radius: f32,
    /// Selected system / UI font family name.
    pub selected_font: String,
    /// System / UI font size in points (10.0 ..= 24.0).
    pub ui_font_size: f32,
    /// Custom theme color palette tokens.
    pub custom_colors: CustomThemeColors,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::WallpaperSync,
            opacity: DEFAULT_OPACITY,
            blur_strength: DEFAULT_BLUR_STRENGTH,
            corner_radius: DEFAULT_CORNER_RADIUS,
            selected_font: "Default".to_string(),
            ui_font_size: DEFAULT_UI_FONT_SIZE,
            custom_colors: CustomThemeColors::default(),
        }
    }
}

impl AppearanceSettings {
    /// Validates and clamps appearance parameters to safe bounds.
    pub fn validate_and_clamp(&mut self) {
        if self.opacity.is_nan() {
            self.opacity = DEFAULT_OPACITY;
        } else {
            self.opacity = self.opacity.clamp(MIN_OPACITY, MAX_OPACITY);
        }

        if self.blur_strength.is_nan() {
            self.blur_strength = DEFAULT_BLUR_STRENGTH;
        } else {
            self.blur_strength = self
                .blur_strength
                .clamp(MIN_BLUR_STRENGTH, MAX_BLUR_STRENGTH);
        }

        if self.corner_radius.is_nan() {
            self.corner_radius = DEFAULT_CORNER_RADIUS;
        } else {
            self.corner_radius = self
                .corner_radius
                .clamp(MIN_CORNER_RADIUS, MAX_CORNER_RADIUS);
        }

        if self.ui_font_size.is_nan() {
            self.ui_font_size = DEFAULT_UI_FONT_SIZE;
        } else {
            self.ui_font_size = self.ui_font_size.clamp(MIN_UI_FONT_SIZE, MAX_UI_FONT_SIZE);
        }

        if self.selected_font.trim().is_empty() {
            self.selected_font = "Default".to_string();
        }
    }
}

/// Text editor typography, formatting, autocomplete, and view preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorSettings {
    /// Selected buffer / code editor font family name.
    pub editor_font: String,
    /// Text editor font size in points (8.0 ..= 36.0).
    pub font_size: f32,
    /// Whether to use monospace font family for code editing.
    pub monospace_font: bool,
    /// Tab indentation width in spaces (2 ..= 8).
    pub tab_size: u32,
    /// Whether to display line numbers in the editor gutter.
    pub show_line_numbers: bool,
    /// Whether to display the bottom status bar.
    pub show_status_bar: bool,
    /// Whether to prompt before closing a tab with unsaved changes.
    pub confirm_close_tab: bool,
    /// Whether to automatically trim trailing spaces on manual save.
    pub trim_trailing_whitespace: bool,
    /// Whether to display inline ghost autocomplete suggestions.
    pub enable_ghost_text: bool,
    /// Whether to enable real-time language syntax highlighting.
    pub enable_syntax_highlighting: bool,
    /// Background auto-save interval in seconds (1 ..= 60).
    pub auto_save_seconds: u32,
    /// Default file extension for new notes (.qn, .md, or .txt).
    pub default_extension: String,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            editor_font: "Default".to_string(),
            font_size: DEFAULT_FONT_SIZE,
            monospace_font: true,
            tab_size: DEFAULT_TAB_SIZE,
            show_line_numbers: true,
            show_status_bar: true,
            confirm_close_tab: true,
            trim_trailing_whitespace: false,
            enable_ghost_text: true,
            enable_syntax_highlighting: true,
            auto_save_seconds: DEFAULT_AUTO_SAVE_SECS,
            default_extension: ".qn".to_string(),
        }
    }
}

impl EditorSettings {
    /// Validates and clamps editor parameters to safe bounds.
    pub fn validate_and_clamp(&mut self) {
        if self.editor_font.trim().is_empty() {
            self.editor_font = "Default".to_string();
        }

        if self.font_size.is_nan() {
            self.font_size = DEFAULT_FONT_SIZE;
        } else {
            self.font_size = self.font_size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
        }

        self.tab_size = self.tab_size.clamp(MIN_TAB_SIZE, MAX_TAB_SIZE);
        self.auto_save_seconds = self
            .auto_save_seconds
            .clamp(MIN_AUTO_SAVE_SECS, MAX_AUTO_SAVE_SECS);

        let ext = self.default_extension.trim().to_lowercase();
        if ext == ".md" || ext == ".txt" || ext == ".qn" {
            self.default_extension = ext;
        } else {
            self.default_extension = ".qn".to_string();
        }
    }
}

/// Window dimensions and window-manager level properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowSettings {
    /// Persistent window width in pixels.
    pub width: f32,
    /// Persistent window height in pixels.
    pub height: f32,
    /// Whether the window remains pinned on top of other windows.
    pub always_on_top: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: WindowSizePreset::STANDARD.width,
            height: WindowSizePreset::STANDARD.height,
            always_on_top: true,
        }
    }
}

impl WindowSettings {
    /// Validates and clamps window size dimensions to desktop limits.
    pub fn validate_and_clamp(&mut self) {
        if self.width.is_nan() {
            self.width = MIN_WINDOW_WIDTH;
        } else {
            self.width = self.width.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
        }

        if self.height.is_nan() {
            self.height = MIN_WINDOW_HEIGHT;
        } else {
            self.height = self.height.clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);
        }
    }
}

/// Plugin system settings and enabled states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginSettings {
    /// Whether the plugin runtime is enabled.
    pub enabled: bool,
    /// List of plugin IDs that are explicitly disabled by the user.
    pub disabled_plugins: Vec<String>,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            disabled_plugins: Vec::new(),
        }
    }
}

impl PluginSettings {
    /// Validates and ensures plugin settings invariants.
    pub fn validate_and_clamp(&mut self) {
        self.disabled_plugins.retain(|id| !id.trim().is_empty());
        self.disabled_plugins.sort();
        self.disabled_plugins.dedup();
    }
}

/// Global user settings configuration container.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// General application behavior, startup, and workspace settings.
    pub general: GeneralSettings,
    /// Appearance, theme, and visual glass styling.
    pub appearance: AppearanceSettings,
    /// Text editor typography, formatting, and view toggles.
    pub editor: EditorSettings,
    /// Window dimensions and layout properties.
    pub window: WindowSettings,
    /// AI Copilot and assistant configuration.
    pub ai: crate::ai::AiSettings,
    /// User-customizable keyboard shortcut keybindings map.
    pub keybindings: crate::ui::shortcuts::KeyBindings,
    /// Plugin system and extension script settings.
    pub plugins: PluginSettings,
}

impl AppSettings {
    /// Validates all setting invariants across all sub-structs.
    pub fn validate_and_clamp(&mut self) {
        self.general.validate_and_clamp();
        self.appearance.validate_and_clamp();
        self.editor.validate_and_clamp();
        self.window.validate_and_clamp();
        self.plugins.validate_and_clamp();
        self.keybindings.ensure_all_actions_present();
    }

    /// Applies an AI-generated theme specification cleanly into custom settings.
    pub fn apply_generated_theme(&mut self, theme: &crate::engine::GeneratedTheme) {
        if let Some(bg) = crate::engine::parse_hex_color(&theme.bg) {
            self.appearance.custom_colors.bg = bg;
        }
        if let Some(card) = crate::engine::parse_hex_color(&theme.card) {
            self.appearance.custom_colors.card = card;
        }
        if let Some(border) = crate::engine::parse_hex_color(&theme.border) {
            self.appearance.custom_colors.border = border;
        }
        if let Some(accent) = crate::engine::parse_hex_color(&theme.accent) {
            self.appearance.custom_colors.accent = accent;
        }
        if let Some(sec) = crate::engine::parse_hex_color(&theme.secondary_accent) {
            self.appearance.custom_colors.secondary_accent = sec;
        }
        if let Some(text) = crate::engine::parse_hex_color(&theme.text) {
            self.appearance.custom_colors.text = text;
        }
        if let Some(muted) = crate::engine::parse_hex_color(&theme.muted_text) {
            self.appearance.custom_colors.muted_text = muted;
        }
        if let Some(danger) = crate::engine::parse_hex_color(&theme.danger) {
            self.appearance.custom_colors.danger = danger;
        }
        if let Some(op) = theme.opacity {
            self.appearance.opacity = op;
        }
        if let Some(blur) = theme.blur_strength {
            self.appearance.blur_strength = blur;
        }

        self.appearance.theme_mode = crate::theme::ThemeMode::Custom;
        self.validate_and_clamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_validation_and_clamping() {
        let mut settings = AppSettings::default();
        settings.appearance.opacity = -5.0;
        settings.editor.font_size = 100.0;
        settings.editor.editor_font = "   ".to_string();
        settings.appearance.ui_font_size = 100.0;
        settings.editor.auto_save_seconds = 0;
        settings.window.width = 100.0;
        settings.window.height = 50.0;
        settings.appearance.selected_font = "   ".to_string();
        settings.editor.tab_size = 100;
        settings.appearance.corner_radius = 500.0;
        settings.editor.default_extension = ".invalid".to_string();

        settings.validate_and_clamp();

        assert_eq!(settings.appearance.opacity, MIN_OPACITY);
        assert_eq!(settings.editor.font_size, MAX_FONT_SIZE);
        assert_eq!(settings.editor.editor_font, "Default");
        assert_eq!(settings.appearance.ui_font_size, MAX_UI_FONT_SIZE);
        assert_eq!(settings.editor.auto_save_seconds, MIN_AUTO_SAVE_SECS);
        assert_eq!(settings.window.width, MIN_WINDOW_WIDTH);
        assert_eq!(settings.window.height, MIN_WINDOW_HEIGHT);
        assert_eq!(settings.editor.tab_size, 8);
        assert_eq!(settings.appearance.corner_radius, 24.0);
        assert_eq!(settings.editor.default_extension, ".qn");
        assert_eq!(settings.appearance.selected_font, "Default");

        // Test min clamping for UI font size
        let mut min_settings = AppSettings::default();
        min_settings.appearance.ui_font_size = 1.0;
        min_settings.validate_and_clamp();
        assert_eq!(min_settings.appearance.ui_font_size, MIN_UI_FONT_SIZE);

        // Test max clamping
        let mut max_settings = AppSettings::default();
        max_settings.window.width = 10000.0;
        max_settings.window.height = 10000.0;
        max_settings.validate_and_clamp();
        assert_eq!(max_settings.window.width, MAX_WINDOW_WIDTH);
        assert_eq!(max_settings.window.height, MAX_WINDOW_HEIGHT);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let mut settings = AppSettings::default();
        settings.editor.font_size = 18.5;
        settings.editor.editor_font = "JetBrains Mono".to_string();
        settings.appearance.ui_font_size = 15.0;
        settings.appearance.theme_mode = ThemeMode::CyberpunkCyan;

        let json = serde_json::to_string(&settings).expect("Serialization failed");
        let loaded: AppSettings = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(loaded.editor.font_size, 18.5);
        assert_eq!(loaded.editor.editor_font, "JetBrains Mono");
        assert_eq!(loaded.appearance.ui_font_size, 15.0);
        assert_eq!(loaded.appearance.theme_mode, ThemeMode::CyberpunkCyan);
        assert!(loaded.plugins.enabled);
    }

    #[test]
    fn test_plugin_settings_deduplication_and_validation() {
        let mut plugin_settings = PluginSettings {
            enabled: true,
            disabled_plugins: vec![
                "terminal".to_string(),
                "   ".to_string(),
                "terminal".to_string(),
                "formatter".to_string(),
            ],
        };
        plugin_settings.validate_and_clamp();
        assert_eq!(
            plugin_settings.disabled_plugins,
            vec!["formatter".to_string(), "terminal".to_string()]
        );
    }
}
