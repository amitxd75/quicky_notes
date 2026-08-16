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
}

impl PaletteColors {
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
    caelestia_mtime: Option<SystemTime>,
    btop_mtime: Option<SystemTime>,
    pywal_mtime: Option<SystemTime>,
    colors: Option<PaletteColors>,
}

static WALLPAPER_CACHE: Mutex<WallpaperCache> = Mutex::new(WallpaperCache {
    caelestia_mtime: None,
    btop_mtime: None,
    pywal_mtime: None,
    colors: None,
});

/// Gets wallpaper color file paths in user home directory.
fn get_wallpaper_paths() -> Option<(PathBuf, PathBuf, PathBuf)> {
    let home = directories::UserDirs::new()?.home_dir().to_path_buf();
    let caelestia = home.join(".config/qtengine/caelestia.colors");
    let btop_caelestia = home.join(".config/btop/themes/caelestia.theme");
    let pywal = home.join(".cache/wal/colors.json");
    Some((caelestia, btop_caelestia, pywal))
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
        // If near grayscale, use vibrant purple
        return Color32::from_rgb(168, 85, 247);
    }

    let lightness = ((max + min) / 2.0).clamp(0.55, 0.72);
    let saturation = 0.85_f32;

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
        let ambient_bg = Palette::interpolate_color(b, Palette::with_alpha(a, 20), 0.15);
        let card = card
            .map(|c| Palette::interpolate_color(c, Palette::with_alpha(a, 35), 0.20))
            .unwrap_or_else(|| Palette::interpolate_color(Palette::lighten(b, 24, 255), a, 0.18));
        let border = border
            .map(|br| Palette::interpolate_color(br, a, 0.45))
            .unwrap_or_else(|| Palette::interpolate_color(card, a, 0.50));

        Some(PaletteColors {
            bg: ambient_bg,
            card,
            border,
            accent: a,
        })
    } else {
        None
    }
}

/// Reads active wallpaper colors from Caelestia or Pywal cache with mtime check.
pub fn get_wallpaper_colors() -> Option<PaletteColors> {
    let (caelestia_path, btop_path, pywal_path) = get_wallpaper_paths()?;

    let current_caelestia_mtime = get_file_mtime(&caelestia_path);
    let current_btop_mtime = get_file_mtime(&btop_path);
    let current_pywal_mtime = get_file_mtime(&pywal_path);

    {
        let cache = WALLPAPER_CACHE.lock().unwrap_or_else(|e| {
            eprintln!("Warning: Wallpaper cache mutex poisoned, recovering");
            e.into_inner()
        });
        if cache.colors.is_some()
            && cache.caelestia_mtime == current_caelestia_mtime
            && cache.btop_mtime == current_btop_mtime
            && cache.pywal_mtime == current_pywal_mtime
        {
            return cache.colors;
        }
    }

    // 1. Try parsing Caelestia qtengine colors
    if let Ok(content) = fs::read_to_string(&caelestia_path)
        && let Some(palette) = parse_caelestia_content(&content)
    {
        if let Ok(mut cache) = WALLPAPER_CACHE.lock() {
            cache.caelestia_mtime = current_caelestia_mtime;
            cache.btop_mtime = current_btop_mtime;
            cache.pywal_mtime = current_pywal_mtime;
            cache.colors = Some(palette);
        }
        return Some(palette);
    }

    // 2. Try parsing Caelestia btop theme
    if let Ok(content) = fs::read_to_string(&btop_path)
        && let Some(palette) = parse_caelestia_content(&content)
    {
        if let Ok(mut cache) = WALLPAPER_CACHE.lock() {
            cache.caelestia_mtime = current_caelestia_mtime;
            cache.btop_mtime = current_btop_mtime;
            cache.pywal_mtime = current_pywal_mtime;
            cache.colors = Some(palette);
        }
        return Some(palette);
    }

    // 3. Try parsing Pywal colors.json
    if let Ok(content) = fs::read_to_string(&pywal_path)
        && let Ok(data) = serde_json::from_str::<PywalColors>(&content)
    {
        let bg = hex_to_color(&data.special.background);
        let border = hex_to_color(&data.colors.color1);
        let accent = hex_to_color(&data.colors.color4);
        let card = Palette::lighten(bg, 18, 255);
        let palette = PaletteColors {
            bg,
            card,
            border,
            accent,
        };
        if let Ok(mut cache) = WALLPAPER_CACHE.lock() {
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
    match settings.theme_mode {
        ThemeMode::Custom => PaletteColors {
            bg: Color32::from_rgb(
                settings.custom_bg_color[0],
                settings.custom_bg_color[1],
                settings.custom_bg_color[2],
            ),
            card: Color32::from_rgb(
                settings.custom_card_color[0],
                settings.custom_card_color[1],
                settings.custom_card_color[2],
            ),
            border: Color32::from_rgb(
                settings.custom_border_color[0],
                settings.custom_border_color[1],
                settings.custom_border_color[2],
            ),
            accent: Color32::from_rgb(
                settings.custom_accent_color[0],
                settings.custom_accent_color[1],
                settings.custom_accent_color[2],
            ),
        },
        ThemeMode::WallpaperSync => {
            if let Some(palette) = get_wallpaper_colors() {
                palette
            } else {
                PaletteColors {
                    bg: Color32::from_rgb(18, 12, 28),
                    card: Color32::from_rgb(28, 20, 42),
                    border: Color32::from_rgb(90, 50, 130),
                    accent: ACCENT_PURPLE,
                }
            }
        }
        ThemeMode::DarkViolet => PaletteColors {
            bg: Color32::from_rgb(18, 12, 28),
            card: Color32::from_rgb(28, 20, 42),
            border: Color32::from_rgb(90, 50, 130),
            accent: ACCENT_PURPLE,
        },
        ThemeMode::ObsidianEmerald => PaletteColors {
            bg: Color32::from_rgb(12, 24, 18),
            card: Color32::from_rgb(20, 38, 30),
            border: Color32::from_rgb(46, 120, 80),
            accent: ACCENT_EMERALD,
        },
        ThemeMode::CyberpunkCyan => PaletteColors {
            bg: Color32::from_rgb(10, 22, 32),
            card: Color32::from_rgb(18, 34, 48),
            border: Color32::from_rgb(40, 140, 190),
            accent: Color32::from_rgb(6, 182, 212),
        },
        ThemeMode::SunsetAmber => PaletteColors {
            bg: Color32::from_rgb(28, 16, 12),
            card: Color32::from_rgb(42, 26, 20),
            border: Color32::from_rgb(140, 70, 30),
            accent: ACCENT_AMBER,
        },
        ThemeMode::RosePink => PaletteColors {
            bg: Color32::from_rgb(28, 14, 22),
            card: Color32::from_rgb(42, 22, 34),
            border: Color32::from_rgb(150, 60, 110),
            accent: Color32::from_rgb(236, 72, 153),
        },
        ThemeMode::NordicFrost => PaletteColors {
            bg: Color32::from_rgb(15, 23, 36),
            card: Color32::from_rgb(24, 34, 52),
            border: Color32::from_rgb(70, 100, 150),
            accent: Color32::from_rgb(56, 189, 248),
        },
        ThemeMode::OledDark => PaletteColors {
            bg: Color32::from_rgb(8, 8, 12),
            card: Color32::from_rgb(18, 18, 24),
            border: Color32::from_rgb(60, 60, 80),
            accent: Color32::from_rgb(147, 51, 234),
        },
    }
}

/// Applies glassmorphism visual styles to egui context without touching font definitions.
pub fn setup_glassmorphism_theme(ctx: &Context, settings: &AppSettings) {
    let palette = get_palette(settings);
    let alpha = (settings.opacity * 255.0).clamp(40.0, 255.0) as u8;
    let radius = settings.corner_radius.round().clamp(0.0, 32.0) as u8;

    let mut visuals = Visuals::dark();

    visuals.panel_fill =
        Color32::from_rgba_unmultiplied(palette.bg.r(), palette.bg.g(), palette.bg.b(), alpha);
    visuals.window_fill =
        Color32::from_rgba_unmultiplied(palette.bg.r(), palette.bg.g(), palette.bg.b(), alpha);
    visuals.faint_bg_color = Color32::from_rgba_unmultiplied(
        palette.card.r(),
        palette.card.g(),
        palette.card.b(),
        (alpha as f32 * 0.6) as u8,
    );

    visuals.selection.bg_fill = palette.accent;
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

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
    let style = Style {
        visuals,
        ..Style::default()
    };

    ctx.set_style_of(eframe::egui::Theme::Dark, style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_color() {
        assert_eq!(hex_to_color("#ff00aa"), Color32::from_rgb(255, 0, 170));
        assert_eq!(hex_to_color("#f0a"), Color32::from_rgb(255, 0, 170));
        assert_eq!(hex_to_color("invalid"), Color32::from_rgb(20, 15, 30));
    }
}
