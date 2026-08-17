//! System font discovery, dynamic TTF loading via `fontconfig`, and startup font setup.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};

/// Ordered candidate priority for discovering high-legibility desktop UI sans-serif typefaces.
pub const CANDIDATE_SYSTEM_UI_FONTS: &[&str] = &[
    "Inter",
    "Adwaita Sans",
    "Roboto",
    "Noto Sans",
    "Ubuntu",
    "Cantarell",
    "DejaVu Sans",
    "Liberation Sans",
    "FreeSans",
    "sans-serif",
];

/// Ordered candidate priority for discovering developer coding and monospace typefaces.
pub const CANDIDATE_MONOSPACE_FONTS: &[&str] = &[
    "JetBrains Mono",
    "JetBrainsMono Nerd Font",
    "FiraCode Nerd Font",
    "Fira Code",
    "CaskaydiaCove Nerd Font",
    "Hack",
    "DejaVu Sans Mono",
    "Adwaita Mono",
    "Cascadia Code",
    "Inconsolata",
    "Iosevka",
    "Source Code Pro",
    "Noto Sans Mono",
    "Liberation Mono",
    "FreeMono",
    "monospace",
];

/// Ordered candidate priority for discovering vector symbol glyphs.
pub const CANDIDATE_SYMBOLS_FONTS: &[&str] = &[
    "Noto Sans Symbols 2",
    "Noto Sans Symbols",
    "DejaVu Sans",
    "DejaVuSans",
    "Symbola",
    "Standard Symbols PS",
    "FreeSans",
];

/// Ordered candidate priority for discovering Nerd Font glyph sets.
pub const CANDIDATE_NERD_FONTS: &[&str] = &[
    "JetBrainsMono Nerd Font",
    "FiraCode Nerd Font",
    "CaskaydiaCove Nerd Font",
    "Hack Nerd Font",
];

/// Ordered candidate priority for discovering system color emojis.
pub const CANDIDATE_EMOJI_FONTS: &[&str] = &[
    "Noto Color Emoji",
    "NotoColorEmoji",
    "Noto Emoji",
    "Twitter Color Emoji",
    "Twemoji",
    "Apple Color Emoji",
    "JoyPixels",
    "Symbola",
    "DejaVu Sans",
    "emoji",
];

static CACHED_SYSTEM_FONTS: OnceLock<Vec<String>> = OnceLock::new();
static CACHED_MONOSPACE_FONTS: OnceLock<Vec<String>> = OnceLock::new();
static CACHED_FONT_PATHS: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
static CACHED_UI_DEFAULT_BYTES: OnceLock<Option<Arc<Vec<u8>>>> = OnceLock::new();
static CACHED_EMOJI_BYTES: OnceLock<Option<Arc<Vec<u8>>>> = OnceLock::new();
static CACHED_MONO_BYTES: OnceLock<Option<Arc<Vec<u8>>>> = OnceLock::new();
static CACHED_SYMBOLS_BYTES: OnceLock<Option<Arc<Vec<u8>>>> = OnceLock::new();
static CACHED_NERD_BYTES: OnceLock<Option<Arc<Vec<u8>>>> = OnceLock::new();

/// Discovers installed system font family names on Linux via `fc-list`.
/// Results are cached in a OnceLock to eliminate UI thread blocking.
pub fn get_installed_system_fonts() -> Vec<String> {
    CACHED_SYSTEM_FONTS
        .get_or_init(|| {
            let mut fonts = vec!["Default".to_string()];

            if let Ok(output) = Command::new("fc-list").arg(":").arg("family").output()
                && output.status.success()
            {
                let font_str = String::from_utf8_lossy(&output.stdout);
                let mut detected: Vec<String> = font_str
                    .lines()
                    .flat_map(|line| line.split(','))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && !s.contains(':'))
                    .collect();

                detected.sort();
                detected.dedup();

                for p in CANDIDATE_SYSTEM_UI_FONTS {
                    if *p != "sans-serif"
                        && detected.iter().any(|f| f.eq_ignore_ascii_case(p))
                        && !fonts.contains(&p.to_string())
                    {
                        fonts.push(p.to_string());
                    }
                }

                for f in detected {
                    if !fonts.contains(&f) {
                        fonts.push(f);
                    }
                }
            }

            fonts
        })
        .clone()
}

/// Discovers installed monospace and coding font family names on Linux via `fc-list :spacing=mono`.
/// Results are cached in a OnceLock to eliminate UI thread blocking.
pub fn get_installed_monospace_fonts() -> Vec<String> {
    CACHED_MONOSPACE_FONTS
        .get_or_init(|| {
            let mut fonts = vec!["Default".to_string()];

            if let Ok(output) = Command::new("fc-list")
                .arg(":spacing=mono")
                .arg("family")
                .output()
                && output.status.success()
            {
                let font_str = String::from_utf8_lossy(&output.stdout);
                let mut detected: Vec<String> = font_str
                    .lines()
                    .flat_map(|line| line.split(','))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && !s.contains(':'))
                    .collect();

                detected.sort();
                detected.dedup();

                for p in CANDIDATE_MONOSPACE_FONTS {
                    if *p != "monospace"
                        && detected.iter().any(|f| f.eq_ignore_ascii_case(p))
                        && !fonts.contains(&p.to_string())
                    {
                        fonts.push(p.to_string());
                    }
                }

                for f in detected {
                    if !fonts.contains(&f) {
                        fonts.push(f);
                    }
                }
            }

            fonts
        })
        .clone()
}

/// Queries fontconfig (`fc-match`) to resolve the exact file path of any font family or pattern on Linux.
/// Results are cached in a thread-safe `Mutex<HashMap>` to eliminate repeated subprocess spawning.
pub fn resolve_font_path(pattern: &str) -> Option<String> {
    let cache_map = CACHED_FONT_PATHS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache_map.lock()
        && let Some(cached) = guard.get(pattern)
    {
        return cached.clone();
    }

    let resolved = if let Ok(output) = Command::new("fc-match")
        .arg("-f")
        .arg("%{file}")
        .arg(pattern)
        .output()
        && output.status.success()
    {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() && std::path::Path::new(&path_str).exists() {
            Some(path_str)
        } else {
            None
        }
    } else {
        None
    };

    if let Ok(mut guard) = cache_map.lock() {
        guard.insert(pattern.to_string(), resolved.clone());
    }

    resolved
}

/// Helper to resolve and read the first matching font file from an ordered candidate priority list.
fn find_first_font_bytes(candidates: &[&str]) -> Option<Arc<Vec<u8>>> {
    for query in candidates {
        if let Some(path) = resolve_font_path(query)
            && let Ok(bytes) = fs::read(&path)
        {
            return Some(Arc::new(bytes));
        }
    }
    None
}

/// Discovers and loads the best installed modern system UI sans-serif font bytes.
pub fn get_default_system_ui_font_bytes() -> Option<Arc<Vec<u8>>> {
    CACHED_UI_DEFAULT_BYTES
        .get_or_init(|| find_first_font_bytes(CANDIDATE_SYSTEM_UI_FONTS))
        .clone()
}

/// Dynamically discovers and loads vector symbol font bytes (DejaVu Sans, Standard Symbols, Symbola).
pub fn get_system_symbols_font_bytes() -> Option<Arc<Vec<u8>>> {
    CACHED_SYMBOLS_BYTES
        .get_or_init(|| find_first_font_bytes(CANDIDATE_SYMBOLS_FONTS))
        .clone()
}

/// Dynamically discovers and loads Nerd Font icon glyphs.
pub fn get_system_nerd_font_bytes() -> Option<Arc<Vec<u8>>> {
    CACHED_NERD_BYTES
        .get_or_init(|| find_first_font_bytes(CANDIDATE_NERD_FONTS))
        .clone()
}

/// Dynamically discovers and loads system color emoji font bytes via fontconfig.
pub fn get_system_emoji_font_bytes() -> Option<Arc<Vec<u8>>> {
    CACHED_EMOJI_BYTES
        .get_or_init(|| find_first_font_bytes(CANDIDATE_EMOJI_FONTS))
        .clone()
}

/// Dynamically discovers and loads the system monospace / nerd font via fontconfig.
pub fn get_system_monospace_font_bytes() -> Option<Arc<Vec<u8>>> {
    CACHED_MONO_BYTES
        .get_or_init(|| find_first_font_bytes(CANDIDATE_MONOSPACE_FONTS))
        .clone()
}

/// Appends system symbols, nerd font icons, and emoji fonts into font definitions.
fn append_symbol_fallbacks(fonts: &mut FontDefinitions) {
    if let Some(symbols_bytes) = get_system_symbols_font_bytes() {
        fonts.font_data.insert(
            "system_symbols".to_owned(),
            FontData::from_owned((*symbols_bytes).clone()).into(),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("system_symbols".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("system_symbols".to_owned());
    }

    if let Some(nerd_bytes) = get_system_nerd_font_bytes() {
        fonts.font_data.insert(
            "system_nerd".to_owned(),
            FontData::from_owned((*nerd_bytes).clone()).into(),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("system_nerd".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("system_nerd".to_owned());
    }

    if let Some(emoji_bytes) = get_system_emoji_font_bytes() {
        fonts.font_data.insert(
            "system_emoji".to_owned(),
            FontData::from_owned((*emoji_bytes).clone()).into(),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("system_emoji".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("system_emoji".to_owned());
    }
}

/// Dynamically injects system monospace, vector symbols, nerd icons, and emoji fonts on startup.
pub fn setup_default_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    if let Some(mono_bytes) = get_system_monospace_font_bytes() {
        fonts.font_data.insert(
            "system_mono".to_owned(),
            FontData::from_owned((*mono_bytes).clone()).into(),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "system_mono".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "system_mono".to_owned());
    }

    append_symbol_fallbacks(&mut fonts);
    ctx.set_fonts(fonts);
}

/// Resolves font file paths via fontconfig and applies both System UI and Buffer fonts to egui.
pub fn apply_system_fonts(ctx: &egui::Context, system_font: &str, editor_font: &str) {
    let font_defs = build_font_definitions(system_font, editor_font);
    ctx.set_fonts(font_defs);
}

/// Convenience single-font fallback for backward compatibility.
pub fn apply_system_font(ctx: &egui::Context, font_name: &str) {
    apply_system_fonts(ctx, font_name, "Default");
}

/// Builds font definitions with separate System UI (Proportional) and Buffer (Monospace) fonts.
pub fn build_font_definitions(system_font: &str, editor_font: &str) -> FontDefinitions {
    let mut font_defs = FontDefinitions::default();

    // 1. Configure System UI Font (Proportional)
    if system_font != "Default"
        && !system_font.trim().is_empty()
        && let Some(file_path) = resolve_font_path(system_font)
        && let Ok(bytes) = std::fs::read(&file_path)
    {
        let font_key = format!("sys_ui_{}", system_font);
        font_defs
            .font_data
            .insert(font_key.clone(), FontData::from_owned(bytes).into());
        font_defs
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, font_key);
    } else if let Some(ui_bytes) = get_default_system_ui_font_bytes() {
        font_defs.font_data.insert(
            "system_ui_default".to_owned(),
            FontData::from_owned((*ui_bytes).clone()).into(),
        );
        font_defs
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "system_ui_default".to_owned());
    }

    // 2. Configure Buffer / Coding Editor Font (Monospace)
    if editor_font != "Default"
        && !editor_font.trim().is_empty()
        && let Some(file_path) = resolve_font_path(editor_font)
        && let Ok(bytes) = std::fs::read(&file_path)
    {
        let font_key = format!("sys_editor_{}", editor_font);
        font_defs
            .font_data
            .insert(font_key.clone(), FontData::from_owned(bytes).into());
        font_defs
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, font_key);
    } else if let Some(mono_bytes) = get_system_monospace_font_bytes() {
        font_defs.font_data.insert(
            "system_mono_default".to_owned(),
            FontData::from_owned((*mono_bytes).clone()).into(),
        );
        font_defs
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "system_mono_default".to_owned());
    } else if let Some(ui_bytes) = get_default_system_ui_font_bytes() {
        // Fall back to system UI font if no monospace font is present
        font_defs.font_data.insert(
            "system_ui_mono_fallback".to_owned(),
            FontData::from_owned((*ui_bytes).clone()).into(),
        );
        font_defs
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "system_ui_mono_fallback".to_owned());
    }

    // 3. Append universal fallbacks (symbols, nerd icons, emojis)
    append_symbol_fallbacks(&mut font_defs);
    font_defs
}

/// Asynchronously loads system and buffer fonts on a background thread.
///
/// Returns a `Receiver` that yields the built `FontDefinitions` when loading completes.
/// The caller should poll `try_recv()` each frame and call `ctx.set_fonts()` when ready.
pub fn setup_fonts_async(
    ctx: &egui::Context,
    system_font: &str,
    editor_font: &str,
) -> std::sync::mpsc::Receiver<FontDefinitions> {
    let (tx, rx) = std::sync::mpsc::channel();
    let ctx_clone = ctx.clone();
    let sys_font = system_font.to_string();
    let ed_font = editor_font.to_string();

    std::thread::spawn(move || {
        let font_defs = build_font_definitions(&sys_font, &ed_font);
        let _ = tx.send(font_defs);
        ctx_clone.request_repaint();
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_discovery_includes_default() {
        let sys_fonts = get_installed_system_fonts();
        assert!(!sys_fonts.is_empty());
        assert_eq!(sys_fonts[0], "Default");

        let mono_fonts = get_installed_monospace_fonts();
        assert!(!mono_fonts.is_empty());
        assert_eq!(mono_fonts[0], "Default");
    }

    #[test]
    fn test_build_font_definitions_defaults() {
        let defs = build_font_definitions("Default", "Default");
        assert!(defs.families.contains_key(&FontFamily::Proportional));
        assert!(defs.families.contains_key(&FontFamily::Monospace));
    }
}
