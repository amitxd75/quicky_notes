//! Glassmorphism visual theme system and dynamic Wayland/Pywal wallpaper synchronization.

use crate::settings::AppSettings;
use eframe::egui::{Color32, Context, CornerRadius, Stroke, Style, Visuals};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// Accent Emerald color constant.
pub const ACCENT_EMERALD: Color32 = Color32::from_rgb(46, 204, 113);
/// Accent Purple color constant.
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(168, 85, 247);
/// Accent Amber color constant.
pub const ACCENT_AMBER: Color32 = Color32::from_rgb(245, 158, 11);
/// Accent Rose color constant.
pub const ACCENT_ROSE: Color32 = Color32::from_rgb(244, 63, 94);

/// Available theme modes for quick customization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    /// Dynamically sync with active Hyprland wallpaper colors (Pywal & Caelestia).
    #[default]
    WallpaperSync,
    /// User-defined custom RGB/HEX color palette.
    Custom,
    /// Dark Violet glass palette.
    DarkViolet,
    /// Obsidian Emerald glass palette.
    ObsidianEmerald,
    /// Cyberpunk Cyan glass palette.
    CyberpunkCyan,
    /// Sunset Amber glass palette.
    SunsetAmber,
    /// Rose Pink glass palette.
    RosePink,
    /// Nordic Frost glass palette.
    NordicFrost,
    /// Pure OLED Dark palette.
    OledDark,
}

impl ThemeMode {
    /// Returns slice of all available theme modes.
    pub const fn all_modes() -> &'static [ThemeMode] {
        &[
            ThemeMode::WallpaperSync,
            ThemeMode::Custom,
            ThemeMode::DarkViolet,
            ThemeMode::ObsidianEmerald,
            ThemeMode::CyberpunkCyan,
            ThemeMode::SunsetAmber,
            ThemeMode::RosePink,
            ThemeMode::NordicFrost,
            ThemeMode::OledDark,
        ]
    }

    /// Display string for settings UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            ThemeMode::WallpaperSync => "Wallpaper Auto-Sync",
            ThemeMode::Custom => "Custom Colors 🎨",
            ThemeMode::DarkViolet => "Dark Violet Glass",
            ThemeMode::ObsidianEmerald => "Obsidian Emerald",
            ThemeMode::CyberpunkCyan => "Cyberpunk Cyan",
            ThemeMode::SunsetAmber => "Sunset Amber",
            ThemeMode::RosePink => "Rose Pink Glass",
            ThemeMode::NordicFrost => "Nordic Frost Glass",
            ThemeMode::OledDark => "OLED Dark",
        }
    }
}

/// Color palette container for UI rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteColors {
    pub bg: Color32,
    pub card: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub secondary_accent: Color32,
    pub text: Color32,
    pub muted_text: Color32,
    pub danger: Color32,
    pub warning: Color32,
    pub success: Color32,
}

impl Default for PaletteColors {
    fn default() -> Self {
        Self::new(
            Color32::from_rgb(18, 12, 28),
            Color32::from_rgb(28, 20, 42),
            Color32::from_rgb(90, 50, 130),
            ACCENT_PURPLE,
        )
    }
}

impl PaletteColors {
    /// Creates a palette from core theme colors with harmonious semantic defaults.
    pub fn new(bg: Color32, card: Color32, border: Color32, accent: Color32) -> Self {
        Self {
            bg,
            card,
            border,
            accent,
            secondary_accent: Color32::from_rgb(56, 189, 248),
            text: Color32::from_gray(240),
            muted_text: Color32::from_gray(160),
            danger: Color32::from_rgb(239, 68, 68),
            warning: Color32::from_rgb(251, 191, 36),
            success: ACCENT_EMERALD,
        }
    }

    /// Creates a fully custom palette with user-defined semantic colors.
    #[allow(clippy::too_many_arguments)]
    pub fn with_semantic(
        bg: Color32,
        card: Color32,
        border: Color32,
        accent: Color32,
        secondary_accent: Color32,
        text: Color32,
        muted_text: Color32,
        danger: Color32,
    ) -> Self {
        Self {
            bg,
            card,
            border,
            accent,
            secondary_accent,
            text,
            muted_text,
            danger,
            warning: Color32::from_rgb(251, 191, 36),
            success: ACCENT_EMERALD,
        }
    }

    /// Returns a lightened variant of this color with specified alpha.
    #[inline]
    pub fn lighten(color: Color32, amount: u8, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(
            (color.r() as u16 + amount as u16).min(255) as u8,
            (color.g() as u16 + amount as u16).min(255) as u8,
            (color.b() as u16 + amount as u16).min(255) as u8,
            alpha,
        )
    }

    /// Returns a semi-transparent variant of a color with custom alpha.
    #[inline]
    pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
    }

    /// Linearly interpolates between two colors with factor `t` in 0.0..=1.0.
    #[inline]
    pub fn interpolate_color(a: Color32, b: Color32, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        let r = (a.r() as f32 * (1.0 - t) + b.r() as f32 * t).round() as u8;
        let g = (a.g() as f32 * (1.0 - t) + b.g() as f32 * t).round() as u8;
        let b_c = (a.b() as f32 * (1.0 - t) + b.b() as f32 * t).round() as u8;
        let alpha = (a.a() as f32 * (1.0 - t) + b.a() as f32 * t).round() as u8;
        Color32::from_rgba_unmultiplied(r, g, b_c, alpha)
    }
}

/// Type alias for PaletteColors.
pub type Palette = PaletteColors;

/// Helper struct for deserializing Pywal `colors.json`.
#[derive(Deserialize)]
struct PywalColors {
    special: PywalSpecial,
    colors: PywalColorMap,
}

#[derive(Deserialize)]
struct PywalSpecial {
    background: String,
}

#[derive(Deserialize)]
struct PywalColorMap {
    color1: String,
    color4: String,
}

/// Parses hex color strings (e.g. `#1a1224`, `1a1224`, `#fff`, `fff`) into `Color32`.
pub fn hex_to_color(hex: &str) -> Color32 {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(20);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(15);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(30);
        Color32::from_rgb(r, g, b)
    } else if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(2) * 17;
        let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(1) * 17;
        let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(3) * 17;
        Color32::from_rgb(r, g, b)
    } else {
        Color32::from_rgb(20, 15, 30)
    }
}

/// Cached wallpaper file modification times to avoid redundant disk reads and parsing.
struct WallpaperCache {
    last_check: Option<std::time::Instant>,
    hypr_scheme_mtime: Option<SystemTime>,
    hypr_vars_mtime: Option<SystemTime>,
    caelestia_mtime: Option<SystemTime>,
    btop_mtime: Option<SystemTime>,
    pywal_mtime: Option<SystemTime>,
    colors: Option<PaletteColors>,
}

static WALLPAPER_CACHE: Mutex<WallpaperCache> = Mutex::new(WallpaperCache {
    last_check: None,
    hypr_scheme_mtime: None,
    hypr_vars_mtime: None,
    caelestia_mtime: None,
    btop_mtime: None,
    pywal_mtime: None,
    colors: None,
});

struct WallpaperFilePaths {
    hypr_scheme: PathBuf,
    hypr_vars: PathBuf,
    caelestia: PathBuf,
    btop_caelestia: PathBuf,
    pywal: PathBuf,
}

/// Gets wallpaper color file paths in user home directory.
fn get_wallpaper_paths() -> Option<WallpaperFilePaths> {
    let home = directories::UserDirs::new()?.home_dir().to_path_buf();
    let hypr_scheme = home.join(".config/hypr/scheme/current.lua");
    let hypr_vars = home.join(".config/hypr/variables.lua");
    let caelestia = home.join(".config/qtengine/caelestia.colors");
    let btop_caelestia = home.join(".config/btop/themes/caelestia.theme");
    let pywal = home.join(".cache/wal/colors.json");
    Some(WallpaperFilePaths {
        hypr_scheme,
        hypr_vars,
        caelestia,
        btop_caelestia,
        pywal,
    })
}

fn get_file_mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

/// Computes vibrancy score of an RGB color to find the most energetic wallpaper accent.
fn color_vibrancy(c: Color32) -> f32 {
    let r = c.r() as f32 / 255.0;
    let g = c.g() as f32 / 255.0;
    let b = c.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if max == 0.0 || delta == 0.0 {
        return 0.0;
    }

    let lightness = (max + min) / 2.0;
    let saturation = if lightness > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };

    // Strongly reward rich saturation with balanced luminance (45% - 75%)
    saturation * (1.0 - (lightness - 0.60).abs() * 1.5).max(0.1)
}

/// Enhances an accent color with rich saturation and luminous contrast on dark glass.
pub fn boost_accent_vibrancy(c: Color32) -> Color32 {
    let r = c.r() as f32 / 255.0;
    let g = c.g() as f32 / 255.0;
    let b = c.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta < 0.06 {
        // If near grayscale, use vibrant cyan/purple
        return Color32::from_rgb(34, 211, 238);
    }

    let lightness = ((max + min) / 2.0).clamp(0.55, 0.72);
    let saturation = 0.88_f32;

    let d = saturation
        * if lightness < 0.5 {
            lightness
        } else {
            1.0 - lightness
        };
    let norm_r = (r - min) / delta;
    let norm_g = (g - min) / delta;
    let norm_b = (b - min) / delta;

    let res_r = ((lightness + (norm_r - 0.5) * 2.0 * d) * 255.0).clamp(0.0, 255.0) as u8;
    let res_g = ((lightness + (norm_g - 0.5) * 2.0 * d) * 255.0).clamp(0.0, 255.0) as u8;
    let res_b = ((lightness + (norm_b - 0.5) * 2.0 * d) * 255.0).clamp(0.0, 255.0) as u8;

    Color32::from_rgb(res_r, res_g, res_b)
}

/// Parses Hyprland / Matugen dynamic scheme file (`~/.config/hypr/scheme/current.lua` and `variables.lua`).
fn parse_hypr_scheme_content(
    scheme_content: &str,
    vars_content: Option<&str>,
) -> Option<PaletteColors> {
    let mut bg = None;
    let mut surface_tint = None;
    let mut card = None;
    let mut primary = None;
    let mut tertiary = None;
    let mut on_surface = None;
    let mut on_surface_variant = None;
    let mut error = None;
    let mut accent_candidates: Vec<Color32> = Vec::new();

    // 1. Parse activeWindowBorderColour from variables.lua if present
    if let Some(vars) = vars_content {
        for line in vars.lines() {
            let line = line.trim();
            if line.contains("activeWindowBorderColour")
                && let Some((_, val)) = line.split_once('=')
            {
                let cleaned = val.trim().trim_matches('"').trim_matches('\'');
                if cleaned.starts_with("rgba(") || cleaned.starts_with("rgb(") {
                    let inner = cleaned
                        .trim_start_matches("rgba(")
                        .trim_start_matches("rgb(")
                        .trim_end_matches(')');
                    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 3
                        && let (Ok(r), Ok(g), Ok(b)) = (
                            parts[0].parse::<u8>(),
                            parts[1].parse::<u8>(),
                            parts[2].parse::<u8>(),
                        )
                    {
                        accent_candidates.push(Color32::from_rgb(r, g, b));
                    }
                } else if cleaned.len() >= 6 {
                    accent_candidates.push(hex_to_color(cleaned));
                }
            }
        }
    }

    // 2. Parse key-value pairs from scheme/current.lua
    for line in scheme_content.lines() {
        let line = line.trim().trim_end_matches(',');
        if line.is_empty()
            || line.starts_with("--")
            || line.starts_with("return")
            || line.starts_with('{')
            || line.starts_with('}')
        {
            continue;
        }
        if let Some((raw_key, raw_val)) = line.split_once('=') {
            let key = raw_key.trim();
            let val = raw_val.trim().trim_matches('"').trim_matches('\'');
            if val.len() < 3 {
                continue;
            }

            let color = hex_to_color(val);

            match key {
                "background" | "surface" if bg.is_none() => bg = Some(color),
                "surfaceContainer" | "surfaceContainerHigh" | "surfaceContainerHighest"
                    if card.is_none() =>
                {
                    card = Some(color);
                }
                "surfaceTint" => surface_tint = Some(color),
                "primary" => {
                    primary = Some(color);
                    accent_candidates.push(color);
                }
                "tertiary" => {
                    tertiary = Some(color);
                    accent_candidates.push(color);
                }
                "onBackground" | "onSurface" if on_surface.is_none() => on_surface = Some(color),
                "onSurfaceVariant" | "outline" if on_surface_variant.is_none() => {
                    on_surface_variant = Some(color)
                }
                "error" => error = Some(color),
                "term1" | "term2" | "term3" | "term4" | "term5" | "term6" | "term9" | "term10"
                | "term11" | "term12" | "term13" | "term14" | "mauve" | "peach" | "teal"
                | "sapphire" | "blue" | "sky" | "pink" | "lavender" => {
                    accent_candidates.push(color);
                }
                _ => {}
            }
        }
    }

    let best_accent = accent_candidates
        .into_iter()
        .max_by(|a, b| {
            color_vibrancy(*a)
                .partial_cmp(&color_vibrancy(*b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(boost_accent_vibrancy);

    if let (Some(b), Some(a)) = (bg, best_accent) {
        let tint = surface_tint.unwrap_or(a);
        let sec_accent = tertiary.or(primary).unwrap_or_else(|| {
            boost_accent_vibrancy(Palette::interpolate_color(
                a,
                Color32::from_rgb(168, 85, 247),
                0.5,
            ))
        });

        // Rich atmospheric glass background: blend background with surface tint and energetic accent
        let ambient_bg = Palette::interpolate_color(b, Palette::with_alpha(tint, 50), 0.24);
        let ambient_bg = Palette::interpolate_color(ambient_bg, Palette::with_alpha(a, 35), 0.16);

        // Rich card frame with glowing glass undertone
        let card_color = card
            .map(|c| Palette::interpolate_color(c, Palette::with_alpha(tint, 70), 0.28))
            .unwrap_or_else(|| {
                Palette::interpolate_color(Palette::lighten(b, 28, 255), tint, 0.25)
            });
        let card_color = Palette::interpolate_color(card_color, Palette::with_alpha(a, 50), 0.20);

        // Luminous border
        let border_color = Palette::interpolate_color(card_color, a, 0.60);

        let text_color = on_surface.unwrap_or(Color32::from_rgb(224, 230, 238));
        let muted_text_color = on_surface_variant.unwrap_or(Color32::from_rgb(166, 171, 180));
        let danger_color = error.unwrap_or(Color32::from_rgb(250, 116, 111));

        Some(PaletteColors::with_semantic(
            ambient_bg,
            card_color,
            border_color,
            a,
            sec_accent,
            text_color,
            muted_text_color,
            danger_color,
        ))
    } else {
        None
    }
}

/// Parses KDE / Caelestia INI format color files (`caelestia.colors` or `caelestia.theme`).
fn parse_caelestia_content(content: &str) -> Option<PaletteColors> {
    let mut bg = None;
    let mut card = None;
    let mut accent_candidates: Vec<Color32> = Vec::new();
    let mut border = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        if let Some((key, raw_val)) = line.split_once('=') {
            let key = key.trim();
            let val = raw_val.trim().trim_matches('"').trim_matches('\'');
            if val.len() < 4 {
                continue;
            }

            match key {
                "BackgroundNormal" if bg.is_none() => bg = Some(hex_to_color(val)),
                "theme[main_bg]" | "theme[selected_bg]" if bg.is_none() => {
                    bg = Some(hex_to_color(val))
                }
                "BackgroundAlternate" | "activeBackground" if card.is_none() => {
                    card = Some(hex_to_color(val))
                }
                "ForegroundNeutral"
                | "ForegroundPositive"
                | "ForegroundLink"
                | "ForegroundNegative"
                | "DecorationFocus"
                | "theme[hi_fg]"
                | "theme[cpu_box]"
                | "theme[available_start]"
                | "theme[process_start]" => {
                    accent_candidates.push(hex_to_color(val));
                }
                "DecorationHover" | "inactiveBackground" | "theme[div_line]"
                    if border.is_none() =>
                {
                    border = Some(hex_to_color(val))
                }
                _ => {}
            }
        }
    }

    // Pick the most vibrant accent candidate
    let best_accent = accent_candidates
        .into_iter()
        .max_by(|a, b| {
            color_vibrancy(*a)
                .partial_cmp(&color_vibrancy(*b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(boost_accent_vibrancy);

    if let (Some(b), Some(a)) = (bg, best_accent) {
        // Create atmospheric glass depth: tint background and card with subtle ambient tone of the accent
        let ambient_bg = Palette::interpolate_color(b, Palette::with_alpha(a, 40), 0.22);
        let card = card
            .map(|c| Palette::interpolate_color(c, Palette::with_alpha(a, 60), 0.26))
            .unwrap_or_else(|| Palette::interpolate_color(Palette::lighten(b, 26, 255), a, 0.24));
        let border = border
            .map(|br| Palette::interpolate_color(br, a, 0.55))
            .unwrap_or_else(|| Palette::interpolate_color(card, a, 0.55));

        Some(PaletteColors::new(ambient_bg, card, border, a))
    } else {
        None
    }
}

/// Reads active wallpaper colors from Hyprland Scheme, Caelestia, or Pywal cache with mtime check.
pub fn get_wallpaper_colors() -> Option<PaletteColors> {
    {
        let cache = WALLPAPER_CACHE.lock().unwrap_or_else(|e| {
            eprintln!("Warning: Wallpaper cache mutex poisoned, recovering");
            e.into_inner()
        });
        if cache.colors.is_some()
            && cache
                .last_check
                .is_some_and(|t| t.elapsed() < std::time::Duration::from_millis(1500))
        {
            return cache.colors;
        }
    }

    let paths = get_wallpaper_paths()?;

    let current_hypr_scheme_mtime = get_file_mtime(&paths.hypr_scheme);
    let current_hypr_vars_mtime = get_file_mtime(&paths.hypr_vars);
    let current_caelestia_mtime = get_file_mtime(&paths.caelestia);
    let current_btop_mtime = get_file_mtime(&paths.btop_caelestia);
    let current_pywal_mtime = get_file_mtime(&paths.pywal);

    {
        let mut cache = WALLPAPER_CACHE.lock().unwrap_or_else(|e| {
            eprintln!("Warning: Wallpaper cache mutex poisoned, recovering");
            e.into_inner()
        });
        cache.last_check = Some(std::time::Instant::now());
        if cache.colors.is_some()
            && cache.hypr_scheme_mtime == current_hypr_scheme_mtime
            && cache.hypr_vars_mtime == current_hypr_vars_mtime
            && cache.caelestia_mtime == current_caelestia_mtime
            && cache.btop_mtime == current_btop_mtime
            && cache.pywal_mtime == current_pywal_mtime
        {
            return cache.colors;
        }
    }

    // 1. Try parsing Hyprland Matugen dynamic scheme (`~/.config/hypr/scheme/current.lua`)
    if let Ok(scheme_content) = fs::read_to_string(&paths.hypr_scheme) {
        let vars_content = fs::read_to_string(&paths.hypr_vars).ok();
        if let Some(palette) = parse_hypr_scheme_content(&scheme_content, vars_content.as_deref()) {
            if let Ok(mut cache) = WALLPAPER_CACHE.lock() {
                cache.last_check = Some(std::time::Instant::now());
                cache.hypr_scheme_mtime = current_hypr_scheme_mtime;
                cache.hypr_vars_mtime = current_hypr_vars_mtime;
                cache.caelestia_mtime = current_caelestia_mtime;
                cache.btop_mtime = current_btop_mtime;
                cache.pywal_mtime = current_pywal_mtime;
                cache.colors = Some(palette);
            }
            return Some(palette);
        }
    }

    // 2. Try parsing Caelestia qtengine colors
    if let Ok(content) = fs::read_to_string(&paths.caelestia)
        && let Some(palette) = parse_caelestia_content(&content)
    {
        if let Ok(mut cache) = WALLPAPER_CACHE.lock() {
            cache.last_check = Some(std::time::Instant::now());
            cache.hypr_scheme_mtime = current_hypr_scheme_mtime;
            cache.hypr_vars_mtime = current_hypr_vars_mtime;
            cache.caelestia_mtime = current_caelestia_mtime;
            cache.btop_mtime = current_btop_mtime;
            cache.pywal_mtime = current_pywal_mtime;
            cache.colors = Some(palette);
        }
        return Some(palette);
    }

    // 3. Try parsing Caelestia btop theme
    if let Ok(content) = fs::read_to_string(&paths.btop_caelestia)
        && let Some(palette) = parse_caelestia_content(&content)
    {
        if let Ok(mut cache) = WALLPAPER_CACHE.lock() {
            cache.last_check = Some(std::time::Instant::now());
            cache.hypr_scheme_mtime = current_hypr_scheme_mtime;
            cache.hypr_vars_mtime = current_hypr_vars_mtime;
            cache.caelestia_mtime = current_caelestia_mtime;
            cache.btop_mtime = current_btop_mtime;
            cache.pywal_mtime = current_pywal_mtime;
            cache.colors = Some(palette);
        }
        return Some(palette);
    }

    // 4. Try parsing Pywal colors.json
    if let Ok(content) = fs::read_to_string(&paths.pywal)
        && let Ok(data) = serde_json::from_str::<PywalColors>(&content)
    {
        let bg = hex_to_color(&data.special.background);
        let border = hex_to_color(&data.colors.color1);
        let accent = hex_to_color(&data.colors.color4);
        let card = Palette::lighten(bg, 18, 255);
        let palette = PaletteColors::new(bg, card, border, accent);
        if let Ok(mut cache) = WALLPAPER_CACHE.lock() {
            cache.last_check = Some(std::time::Instant::now());
            cache.hypr_scheme_mtime = current_hypr_scheme_mtime;
            cache.hypr_vars_mtime = current_hypr_vars_mtime;
            cache.caelestia_mtime = current_caelestia_mtime;
            cache.btop_mtime = current_btop_mtime;
            cache.pywal_mtime = current_pywal_mtime;
            cache.colors = Some(palette);
        }
        return Some(palette);
    }

    None
}

/// Checks whether wallpaper colors have changed since the last check.
pub fn check_wallpaper_color_change(last_colors: &mut Option<PaletteColors>) -> bool {
    let current = get_wallpaper_colors();
    if current != *last_colors {
        *last_colors = current;
        true
    } else {
        false
    }
}

/// Resolves theme palette colors according to user settings.
pub fn get_palette(settings: &AppSettings) -> PaletteColors {
    match settings.appearance.theme_mode {
        ThemeMode::Custom => PaletteColors::with_semantic(
            Color32::from_rgb(
                settings.appearance.custom_colors.bg[0],
                settings.appearance.custom_colors.bg[1],
                settings.appearance.custom_colors.bg[2],
            ),
            Color32::from_rgb(
                settings.appearance.custom_colors.card[0],
                settings.appearance.custom_colors.card[1],
                settings.appearance.custom_colors.card[2],
            ),
            Color32::from_rgb(
                settings.appearance.custom_colors.border[0],
                settings.appearance.custom_colors.border[1],
                settings.appearance.custom_colors.border[2],
            ),
            Color32::from_rgb(
                settings.appearance.custom_colors.accent[0],
                settings.appearance.custom_colors.accent[1],
                settings.appearance.custom_colors.accent[2],
            ),
            Color32::from_rgb(
                settings.appearance.custom_colors.secondary_accent[0],
                settings.appearance.custom_colors.secondary_accent[1],
                settings.appearance.custom_colors.secondary_accent[2],
            ),
            Color32::from_rgb(
                settings.appearance.custom_colors.text[0],
                settings.appearance.custom_colors.text[1],
                settings.appearance.custom_colors.text[2],
            ),
            Color32::from_rgb(
                settings.appearance.custom_colors.muted_text[0],
                settings.appearance.custom_colors.muted_text[1],
                settings.appearance.custom_colors.muted_text[2],
            ),
            Color32::from_rgb(
                settings.appearance.custom_colors.danger[0],
                settings.appearance.custom_colors.danger[1],
                settings.appearance.custom_colors.danger[2],
            ),
        ),
        ThemeMode::WallpaperSync => {
            if let Some(palette) = get_wallpaper_colors() {
                palette
            } else {
                PaletteColors::new(
                    Color32::from_rgb(18, 12, 28),
                    Color32::from_rgb(28, 20, 42),
                    Color32::from_rgb(90, 50, 130),
                    ACCENT_PURPLE,
                )
            }
        }
        ThemeMode::DarkViolet => PaletteColors::new(
            Color32::from_rgb(18, 12, 28),
            Color32::from_rgb(28, 20, 42),
            Color32::from_rgb(90, 50, 130),
            ACCENT_PURPLE,
        ),
        ThemeMode::ObsidianEmerald => PaletteColors::new(
            Color32::from_rgb(12, 24, 18),
            Color32::from_rgb(20, 38, 30),
            Color32::from_rgb(46, 120, 80),
            ACCENT_EMERALD,
        ),
        ThemeMode::CyberpunkCyan => PaletteColors::new(
            Color32::from_rgb(10, 22, 32),
            Color32::from_rgb(18, 34, 48),
            Color32::from_rgb(40, 140, 190),
            Color32::from_rgb(6, 182, 212),
        ),
        ThemeMode::SunsetAmber => PaletteColors::new(
            Color32::from_rgb(28, 16, 12),
            Color32::from_rgb(42, 26, 20),
            Color32::from_rgb(140, 70, 30),
            ACCENT_AMBER,
        ),
        ThemeMode::RosePink => PaletteColors::new(
            Color32::from_rgb(28, 14, 22),
            Color32::from_rgb(42, 22, 34),
            Color32::from_rgb(150, 60, 110),
            Color32::from_rgb(236, 72, 153),
        ),
        ThemeMode::NordicFrost => PaletteColors::new(
            Color32::from_rgb(15, 23, 36),
            Color32::from_rgb(24, 34, 52),
            Color32::from_rgb(70, 100, 150),
            Color32::from_rgb(56, 189, 248),
        ),
        ThemeMode::OledDark => PaletteColors::new(
            Color32::from_rgb(8, 8, 12),
            Color32::from_rgb(18, 18, 24),
            Color32::from_rgb(60, 60, 80),
            Color32::from_rgb(147, 51, 234),
        ),
    }
}

/// Applies glassmorphism visual styles to egui context without touching font definitions.
pub fn setup_glassmorphism_theme(ctx: &Context, settings: &AppSettings) {
    let palette = get_palette(settings);
    let alpha = (settings.appearance.opacity * 255.0).clamp(40.0, 255.0) as u8;
    let radius = settings.appearance.corner_radius.round().clamp(0.0, 32.0) as u8;

    let mut visuals = Visuals::dark();

    visuals.panel_fill =
        Color32::from_rgba_unmultiplied(palette.bg.r(), palette.bg.g(), palette.bg.b(), alpha);

    // Context menu & popup background: 50% elevated opacity for crisp readability over editor text
    let menu_alpha = (alpha as f32 + (255.0 - alpha as f32) * 0.50)
        .round()
        .clamp(190.0, 255.0) as u8;
    visuals.window_fill =
        Color32::from_rgba_unmultiplied(palette.bg.r(), palette.bg.g(), palette.bg.b(), menu_alpha);
    visuals.window_stroke = Stroke::new(1.0, Palette::with_alpha(palette.border, 180));
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.faint_bg_color = Color32::from_rgba_unmultiplied(
        palette.card.r(),
        palette.card.g(),
        palette.card.b(),
        (alpha as f32 * 0.6) as u8,
    );

    // Smooth, translucent glass text selection highlight
    visuals.selection.bg_fill = Palette::with_alpha(palette.accent, 85);
    visuals.selection.stroke = Stroke::new(1.0_f32, Palette::with_alpha(palette.accent, 160));

    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.noninteractive.bg_fill =
        Color32::from_rgba_unmultiplied(palette.card.r(), palette.card.g(), palette.card.b(), 200);
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(radius.min(8));

    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_gray(230));
    visuals.widgets.inactive.bg_fill =
        Color32::from_rgba_unmultiplied(palette.card.r(), palette.card.g(), palette.card.b(), 220);
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.corner_radius = CornerRadius::same(radius.min(8));

    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(
        palette.accent.r(),
        palette.accent.g(),
        palette.accent.b(),
        200,
    );
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.corner_radius = CornerRadius::same(radius.min(8));

    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.bg_fill = palette.accent;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.corner_radius = CornerRadius::same(radius.min(8));

    visuals.window_corner_radius = CornerRadius::same(radius);

    ctx.set_visuals(visuals.clone());

    let ui_size = settings.appearance.ui_font_size;
    let scale = (ui_size / crate::models::settings::DEFAULT_UI_FONT_SIZE).clamp(0.65, 2.0);

    let mut style = Style {
        visuals,
        ..Style::default()
    };

    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::proportional(11.0)),
        (egui::TextStyle::Body, egui::FontId::proportional(13.5)),
        (egui::TextStyle::Button, egui::FontId::proportional(13.0)),
        (egui::TextStyle::Heading, egui::FontId::proportional(18.0)),
        (
            egui::TextStyle::Monospace,
            egui::FontId::monospace(settings.editor.font_size),
        ),
    ]
    .into();

    ctx.set_style_of(eframe::egui::Theme::Dark, style.clone());
    ctx.set_style_of(eframe::egui::Theme::Light, style);
    ctx.set_zoom_factor(scale);
}
