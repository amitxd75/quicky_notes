use eframe::egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, Frame, Margin, Stroke,
    Style, Visuals,
};
use serde::{Deserialize, Serialize};
use std::fs;

/// Accent Emerald color constant.
pub const ACCENT_EMERALD: Color32 = Color32::from_rgb(46, 204, 113);
/// Accent Purple color constant.
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(168, 85, 247);
/// Accent Amber color constant.
pub const ACCENT_AMBER: Color32 = Color32::from_rgb(245, 158, 11);

/// Available theme modes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    /// Dynamically sync with active Hyprland wallpaper colors (Pywal & Caelestia).
    #[default]
    WallpaperSync,
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
    pub fn all_modes() -> &'static [ThemeMode] {
        &[
            ThemeMode::WallpaperSync,
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
    pub fn display_name(&self) -> &'static str {
        match self {
            ThemeMode::WallpaperSync => "Wallpaper Auto-Sync",
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

/// Color palette container for theme modes.
#[derive(Debug, Clone, Copy)]
pub struct PaletteColors {
    pub bg: Color32,
    pub card: Color32,
    pub border: Color32,
    pub accent: Color32,
}

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

/// Parses color hex strings like `#1a1224` into egui `Color32`.
pub fn hex_to_color(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(20);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(15);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(30);
        Color32::from_rgb(r, g, b)
    } else {
        Color32::from_rgb(20, 15, 30)
    }
}

/// Reads active wallpaper colors from Caelestia or Pywal cache on Linux.
pub fn get_wallpaper_colors() -> Option<(Color32, Color32, Color32)> {
    if let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
        let caelestia_path = home.join(".config/qtengine/caelestia.colors");
        if let Ok(content) = fs::read_to_string(&caelestia_path) {
            let mut bg = None;
            let mut accent = None;
            let mut border = None;

            for line in content.lines() {
                let line = line.trim();
                if line.contains("background=") {
                    if let Some(val) = line.split('=').nth(1) {
                        bg = Some(hex_to_color(val));
                    }
                } else if (line.contains("color4=") || line.contains("primary="))
                    && let Some(val) = line.split('=').nth(1)
                {
                    accent = Some(hex_to_color(val));
                } else if line.contains("color1=")
                    && let Some(val) = line.split('=').nth(1)
                {
                    border = Some(hex_to_color(val));
                }
            }

            if let (Some(b), Some(a), Some(br)) = (bg, accent, border) {
                return Some((b, a, br));
            }
        }

        let pywal_path = home.join(".cache/wal/colors.json");
        if let Ok(content) = fs::read_to_string(&pywal_path)
            && let Ok(data) = serde_json::from_str::<PywalColors>(&content)
        {
            let bg = hex_to_color(&data.special.background);
            let border = hex_to_color(&data.colors.color1);
            let accent = hex_to_color(&data.colors.color4);
            return Some((bg, accent, border));
        }
    }
    None
}

/// Polls wallpaper colors and returns true if wallpaper colors changed.
pub fn check_wallpaper_color_change(last_colors: &mut Option<(Color32, Color32, Color32)>) -> bool {
    let current = get_wallpaper_colors();
    if current != *last_colors {
        *last_colors = current;
        true
    } else {
        false
    }
}

/// Resolves palette colors for a theme mode.
pub fn get_palette(mode: ThemeMode) -> PaletteColors {
    match mode {
        ThemeMode::WallpaperSync => {
            if let Some((bg, accent, border)) = get_wallpaper_colors() {
                let card = Color32::from_rgb(
                    ((bg.r() as u16 + 20).min(255)) as u8,
                    ((bg.g() as u16 + 18).min(255)) as u8,
                    ((bg.b() as u16 + 32).min(255)) as u8,
                );
                PaletteColors {
                    bg,
                    card,
                    border,
                    accent,
                }
            } else {
                get_palette(ThemeMode::DarkViolet)
            }
        }
        ThemeMode::DarkViolet => PaletteColors {
            bg: Color32::from_rgb(20, 14, 28),
            card: Color32::from_rgb(32, 22, 45),
            border: Color32::from_rgb(130, 80, 190),
            accent: ACCENT_PURPLE,
        },
        ThemeMode::ObsidianEmerald => PaletteColors {
            bg: Color32::from_rgb(12, 22, 18),
            card: Color32::from_rgb(20, 36, 28),
            border: Color32::from_rgb(46, 204, 113),
            accent: ACCENT_EMERALD,
        },
        ThemeMode::CyberpunkCyan => PaletteColors {
            bg: Color32::from_rgb(10, 20, 30),
            card: Color32::from_rgb(18, 32, 48),
            border: Color32::from_rgb(0, 210, 255),
            accent: Color32::from_rgb(0, 210, 255),
        },
        ThemeMode::SunsetAmber => PaletteColors {
            bg: Color32::from_rgb(26, 16, 14),
            card: Color32::from_rgb(42, 26, 22),
            border: Color32::from_rgb(245, 158, 11),
            accent: ACCENT_AMBER,
        },
        ThemeMode::RosePink => PaletteColors {
            bg: Color32::from_rgb(34, 18, 28),
            card: Color32::from_rgb(52, 26, 42),
            border: Color32::from_rgb(244, 63, 94),
            accent: Color32::from_rgb(236, 72, 153),
        },
        ThemeMode::NordicFrost => PaletteColors {
            bg: Color32::from_rgb(22, 32, 42),
            card: Color32::from_rgb(34, 48, 62),
            border: Color32::from_rgb(129, 161, 193),
            accent: Color32::from_rgb(136, 192, 208),
        },
        ThemeMode::OledDark => PaletteColors {
            bg: Color32::from_rgb(8, 8, 12),
            card: Color32::from_rgb(16, 16, 24),
            border: Color32::from_rgb(90, 90, 120),
            accent: Color32::from_rgb(160, 160, 220),
        },
    }
}

/// Setup custom fallback Nerd Font.
fn setup_custom_fonts(ctx: &Context) {
    let font_paths = [
        "/usr/share/fonts/TTF/FiraCodeNerdFont-Regular.ttf",
        "/usr/share/fonts/TTF/CaskaydiaCoveNerdFont-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    ];

    let mut fonts = FontDefinitions::default();
    for path in font_paths {
        if let Ok(bytes) = fs::read(path) {
            fonts
                .font_data
                .insert("nerd_font".to_owned(), FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "nerd_font".to_owned());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "nerd_font".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

/// Applies glassmorphism visual styles to egui context.
pub fn setup_glassmorphism_theme(ctx: &Context, opacity: f32, mode: ThemeMode) {
    setup_custom_fonts(ctx);

    let palette = get_palette(mode);
    let alpha = (opacity * 255.0).clamp(140.0, 245.0) as u8;

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
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);

    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_gray(230));
    visuals.widgets.inactive.bg_fill =
        Color32::from_rgba_unmultiplied(palette.card.r(), palette.card.g(), palette.card.b(), 220);
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(
        palette.accent.r(),
        palette.accent.g(),
        palette.accent.b(),
        200,
    );
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);

    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.bg_fill = palette.accent;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.corner_radius = CornerRadius::same(8);

    visuals.window_corner_radius = CornerRadius::same(12);

    let style = Style {
        visuals,
        ..Style::default()
    };

    ctx.set_style_of(egui::Theme::Dark, style);
}

/// Creates a glass editor frame with translucent background and accent border.
pub fn glass_editor_frame(opacity: f32, mode: ThemeMode) -> Frame {
    let palette = get_palette(mode);
    let alpha = (opacity * 255.0).clamp(140.0, 235.0) as u8;
    Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(
            palette.bg.r(),
            palette.bg.g(),
            palette.bg.b(),
            alpha,
        ))
        .stroke(Stroke::new(1.2_f32, palette.border))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(0))
}

/// Creates a glass card frame for settings cards.
pub fn glass_card_frame(opacity: f32, mode: ThemeMode) -> Frame {
    let palette = get_palette(mode);
    let alpha = (opacity * 255.0).clamp(160.0, 240.0) as u8;
    Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(
            palette.card.r(),
            palette.card.g(),
            palette.card.b(),
            alpha,
        ))
        .stroke(Stroke::new(1.0_f32, palette.border))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(12))
}
