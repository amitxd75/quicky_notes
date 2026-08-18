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

/// Minimum window corner radius in pixels (0.0 allows sharp corners for Hyprland/tiling WMs).
pub const MIN_CORNER_RADIUS: f32 = 0.0;
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
        default_font_size: 18.0,
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
            auto_title_from_first_line: false,
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
    /// Glass window corner radius roundness in pixels (0.0 ..= 24.0).
    pub corner_radius: f32,
    /// Whether to render an internal window border stroke (disable for tiling/Hyprland window managers).
    pub show_window_border: bool,
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
            show_window_border: true,
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
            enable_syntax_highlighting: false,
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

/// Advanced resource limits, caching sizes, plugin engine budgets, and tunables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AdvancedSettings {
    /// Maximum allowable .qn binary file container size in megabytes (1 ..= 200 MB).
    pub max_qn_file_size_mb: usize,
    /// Maximum allowable single attachment size in megabytes (1 ..= 100 MB).
    pub max_attachment_size_mb: usize,
    /// Maximum cumulative attachments size per note in megabytes (5 ..= 500 MB).
    pub max_total_attachments_size_mb: usize,
    /// Maximum number of attachments permitted per note (5 ..= 500).
    pub max_attachments_per_note: usize,
    /// Maximum length for note titles in characters (16 ..= 512).
    pub max_note_title_len: usize,

    /// Maximum GPU texture cache entry capacity before LRU eviction (8 ..= 512).
    pub max_texture_cache_entries: usize,
    /// Maximum allowable single image dimension width/height in pixels (1024 ..= 16384).
    pub max_image_dimension: u32,
    /// Maximum allowable image total pixel count (1_000_000 ..= 67_108_864).
    pub max_image_pixels: u64,
    /// Maximum popup width in pixels for image viewer overlays (200.0 ..= 1200.0).
    pub image_popup_max_width: f32,
    /// Whether GPU hardware acceleration is enabled (OpenGL / Vulkan / wgpu; requires app restart).
    pub hardware_acceleration: bool,

    /// Maximum execution operation quota for plugin scripts (10_000 ..= 10_000_000).
    pub max_script_operations: u64,
    /// Maximum recursive call stack depth for plugins (10 ..= 200).
    pub max_script_call_levels: usize,
    /// Maximum allocated string size in bytes in plugin scripts (100_000 ..= 50_000_000).
    pub max_script_string_size: usize,
    /// Maximum array element count in plugin scripts (1_000 ..= 1_000_000).
    pub max_script_array_size: usize,
    /// Maximum map entry count in plugin scripts (1_000 ..= 1_000_000).
    pub max_script_map_size: usize,
    /// Maximum subprocess synchronous execution timeout in milliseconds (500 ..= 30_000).
    pub exec_timeout_ms: u64,
    /// Maximum HTTP request timeout in seconds (1 ..= 30).
    pub http_timeout_secs: u64,
    /// Maximum captured stdout output in bytes for synchronous commands (64_000 ..= 10_485_760).
    pub max_exec_output_bytes: usize,
    /// Maximum HTTP response body size in bytes (100_000 ..= 50_000_000).
    pub max_http_response_bytes: usize,

    /// Duration in seconds that temporary status bar messages stay visible (1.0 ..= 10.0).
    pub status_msg_duration_secs: f32,
    /// Maximum recursive directory scan depth for folder workspaces (1 ..= 20).
    pub folder_max_scan_depth: usize,
    /// Maximum search depth for Trie autocomplete suggestions (3 ..= 30).
    pub suggest_max_search_depth: usize,
    /// User learned frequency weight multiplier for suggestions (1_000 ..= 1_000_000).
    pub suggest_user_weight_multiplier: u64,
    /// Base dictionary vocabulary capacity (10_000 ..= 333_304 words).
    pub suggest_vocab_size: usize,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            max_qn_file_size_mb: 25,
            max_attachment_size_mb: 10,
            max_total_attachments_size_mb: 50,
            max_attachments_per_note: 50,
            max_note_title_len: 128,

            max_texture_cache_entries: 64,
            max_image_dimension: 8192,
            max_image_pixels: 16_777_216,
            image_popup_max_width: 380.0,
            hardware_acceleration: true,

            max_script_operations: 500_000,
            max_script_call_levels: 50,
            max_script_string_size: 5_000_000,
            max_script_array_size: 100_000,
            max_script_map_size: 100_000,
            exec_timeout_ms: 2000,
            http_timeout_secs: 3,
            max_exec_output_bytes: 1_048_576,
            max_http_response_bytes: 5_242_880,

            status_msg_duration_secs: 3.5,
            folder_max_scan_depth: 6,
            suggest_max_search_depth: 12,
            suggest_user_weight_multiplier: 100_000,
            suggest_vocab_size: 50_000,
        }
    }
}

impl AdvancedSettings {
    /// Validates and clamps all advanced parameters to safe bounds.
    pub fn validate_and_clamp(&mut self) {
        self.max_qn_file_size_mb = self.max_qn_file_size_mb.clamp(1, 200);
        self.max_attachment_size_mb = self.max_attachment_size_mb.clamp(1, 100);
        self.max_total_attachments_size_mb = self.max_total_attachments_size_mb.clamp(5, 500);
        self.max_attachments_per_note = self.max_attachments_per_note.clamp(5, 500);
        self.max_note_title_len = self.max_note_title_len.clamp(16, 512);

        self.max_texture_cache_entries = self.max_texture_cache_entries.clamp(8, 512);
        self.max_image_dimension = self.max_image_dimension.clamp(1024, 16384);
        self.max_image_pixels = self.max_image_pixels.clamp(1_000_000, 67_108_864);
        if self.image_popup_max_width.is_nan() {
            self.image_popup_max_width = 380.0;
        } else {
            self.image_popup_max_width = self.image_popup_max_width.clamp(200.0, 1200.0);
        }

        self.max_script_operations = self.max_script_operations.clamp(10_000, 10_000_000);
        self.max_script_call_levels = self.max_script_call_levels.clamp(10, 200);
        self.max_script_string_size = self.max_script_string_size.clamp(100_000, 50_000_000);
        self.max_script_array_size = self.max_script_array_size.clamp(1_000, 1_000_000);
        self.max_script_map_size = self.max_script_map_size.clamp(1_000, 1_000_000);
        self.exec_timeout_ms = self.exec_timeout_ms.clamp(500, 30_000);
        self.http_timeout_secs = self.http_timeout_secs.clamp(1, 30);
        self.max_exec_output_bytes = self.max_exec_output_bytes.clamp(64_000, 10_485_760);
        self.max_http_response_bytes = self.max_http_response_bytes.clamp(100_000, 50_000_000);

        if self.status_msg_duration_secs.is_nan() {
            self.status_msg_duration_secs = 3.5;
        } else {
            self.status_msg_duration_secs = self.status_msg_duration_secs.clamp(1.0, 10.0);
        }
        self.folder_max_scan_depth = self.folder_max_scan_depth.clamp(1, 20);
        self.suggest_max_search_depth = self.suggest_max_search_depth.clamp(3, 30);
        self.suggest_user_weight_multiplier =
            self.suggest_user_weight_multiplier.clamp(1_000, 1_000_000);
        self.suggest_vocab_size = self.suggest_vocab_size.clamp(10_000, 333_304);
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
    /// Advanced resource quotas, memory bounds, and engine tunables.
    pub advanced: AdvancedSettings,
}

impl AppSettings {
    /// Validates all setting invariants across all sub-structs.
    pub fn validate_and_clamp(&mut self) {
        self.general.validate_and_clamp();
        self.appearance.validate_and_clamp();
        self.editor.validate_and_clamp();
        self.window.validate_and_clamp();
        self.plugins.validate_and_clamp();
        self.advanced.validate_and_clamp();
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

        // Test min clamping for UI font size and corner radius
        let mut min_settings = AppSettings::default();
        min_settings.appearance.ui_font_size = 1.0;
        min_settings.appearance.corner_radius = -10.0;
        min_settings.validate_and_clamp();
        assert_eq!(min_settings.appearance.ui_font_size, MIN_UI_FONT_SIZE);
        assert_eq!(min_settings.appearance.corner_radius, MIN_CORNER_RADIUS);
        assert!(min_settings.appearance.show_window_border);

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

    #[test]
    fn test_advanced_settings_defaults_and_validation() {
        let mut adv = AdvancedSettings::default();
        assert_eq!(adv.max_qn_file_size_mb, 25);
        assert_eq!(adv.max_attachment_size_mb, 10);
        assert_eq!(adv.max_total_attachments_size_mb, 50);
        assert_eq!(adv.max_attachments_per_note, 50);
        assert_eq!(adv.max_note_title_len, 128);
        assert_eq!(adv.max_texture_cache_entries, 64);
        assert_eq!(adv.max_image_dimension, 8192);
        assert_eq!(adv.max_image_pixels, 16_777_216);
        assert_eq!(adv.max_script_operations, 500_000);
        assert_eq!(adv.folder_max_scan_depth, 6);
        assert!(adv.hardware_acceleration);
        assert_eq!(adv.suggest_vocab_size, 50_000);

        // Test out-of-range values get clamped
        adv.max_qn_file_size_mb = 9999;
        adv.max_attachment_size_mb = 0;
        adv.max_total_attachments_size_mb = 0;
        adv.max_attachments_per_note = 0;
        adv.max_note_title_len = 5;
        adv.max_texture_cache_entries = 0;
        adv.max_image_dimension = 100;
        adv.max_script_operations = 50;
        adv.folder_max_scan_depth = 100;
        adv.status_msg_duration_secs = 50.0;

        adv.validate_and_clamp();

        assert_eq!(adv.max_qn_file_size_mb, 200);
        assert_eq!(adv.max_attachment_size_mb, 1);
        assert_eq!(adv.max_total_attachments_size_mb, 5);
        assert_eq!(adv.max_attachments_per_note, 5);
        assert_eq!(adv.max_note_title_len, 16);
        assert_eq!(adv.max_texture_cache_entries, 8);
        assert_eq!(adv.max_image_dimension, 1024);
        assert_eq!(adv.max_script_operations, 10_000);
        assert_eq!(adv.folder_max_scan_depth, 20);
        assert_eq!(adv.status_msg_duration_secs, 10.0);
    }
}
