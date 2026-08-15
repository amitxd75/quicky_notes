//! System font discovery, dynamic TTF loading via `fontconfig`, and startup font setup.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};

static CACHED_SYSTEM_FONTS: OnceLock<Vec<String>> = OnceLock::new();
static CACHED_FONT_PATHS: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
static CACHED_EMOJI_BYTES: OnceLock<Option<Arc<Vec<u8>>>> = OnceLock::new();
static CACHED_MONO_BYTES: OnceLock<Option<Arc<Vec<u8>>>> = OnceLock::new();

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

                let priority_fonts = [
                    "FiraCode Nerd Font",
                    "CaskaydiaCove Nerd Font",
                    "JetBrains Mono",
                    "Hack",
                    "DejaVu Sans Mono",
                    "Inter",
                    "Adwaita Sans",
                    "Adwaita Mono",
                    "Roboto",
                    "Noto Sans",
                    "Ubuntu",
                ];

                for p in priority_fonts {
                    if detected.iter().any(|f| f.eq_ignore_ascii_case(p))
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
/// Results are cached in a thread-safe Mutex<HashMap> to eliminate repeated subprocess spawning.
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

/// Dynamically discovers and loads system color emoji font bytes via fontconfig.
/// Result is cached in a OnceLock to read the TTF file only once.
/// Returns an Arc-wrapped byte vector for cheap cloning.
pub fn get_system_emoji_font_bytes() -> Option<Arc<Vec<u8>>> {
    CACHED_EMOJI_BYTES
        .get_or_init(|| {
            if let Some(path) = resolve_font_path("emoji")
                && let Ok(bytes) = fs::read(&path)
            {
                return Some(Arc::new(bytes));
            }
            None
        })
        .clone()
}

/// Dynamically discovers and loads the system monospace / nerd font via fontconfig.
/// Result is cached in a OnceLock to read the TTF file only once.
/// Returns an Arc-wrapped byte vector for cheap cloning.
pub fn get_system_monospace_font_bytes() -> Option<Arc<Vec<u8>>> {
    CACHED_MONO_BYTES
        .get_or_init(|| {
            for query in ["FiraCode Nerd Font", "JetBrainsMono Nerd Font", "monospace"] {
                if let Some(path) = resolve_font_path(query)
                    && let Ok(bytes) = fs::read(&path)
                {
                    return Some(Arc::new(bytes));
                }
            }
            None
        })
        .clone()
}

/// Dynamically injects system monospace and color emoji fonts on startup.
pub fn setup_default_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    if let Some(emoji_bytes) = get_system_emoji_font_bytes() {
        fonts.font_data.insert(
            "system_emoji".to_owned(),
            FontData::from_owned((*emoji_bytes).clone()).into(),
        );
    }

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

    if fonts.font_data.contains_key("system_emoji") {
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

    ctx.set_fonts(fonts);
}

/// Resolves font file path via fontconfig and applies the TTF font family dynamically to egui with color emoji fallback.
pub fn apply_system_font(ctx: &egui::Context, font_name: &str) {
    if font_name == "Default" || font_name.trim().is_empty() {
        setup_default_fonts(ctx);
        return;
    }

    if let Some(file_path) = resolve_font_path(font_name)
        && let Ok(bytes) = std::fs::read(&file_path)
    {
        let mut font_defs = FontDefinitions::default();
        let font_key = format!("sys_font_{}", font_name);

        if let Some(emoji_bytes) = get_system_emoji_font_bytes() {
            font_defs.font_data.insert(
                "system_emoji".to_owned(),
                FontData::from_owned((*emoji_bytes).clone()).into(),
            );
        }

        font_defs
            .font_data
            .insert(font_key.clone(), FontData::from_owned(bytes).into());

        font_defs
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, font_key.clone());

        font_defs
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, font_key);

        if font_defs.font_data.contains_key("system_emoji") {
            font_defs
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push("system_emoji".to_owned());
            font_defs
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("system_emoji".to_owned());
        }

        ctx.set_fonts(font_defs);
    }
}

/// Builds font definitions without applying them, for use in async font loading.
fn build_font_definitions(font_name: &str) -> FontDefinitions {
    if font_name == "Default" || font_name.trim().is_empty() {
        let mut fonts = FontDefinitions::default();

        if let Some(emoji_bytes) = get_system_emoji_font_bytes() {
            fonts.font_data.insert(
                "system_emoji".to_owned(),
                FontData::from_owned((*emoji_bytes).clone()).into(),
            );
        }

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

        if fonts.font_data.contains_key("system_emoji") {
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

        fonts
    } else if let Some(file_path) = resolve_font_path(font_name)
        && let Ok(bytes) = std::fs::read(&file_path)
    {
        let mut font_defs = FontDefinitions::default();
        let font_key = format!("sys_font_{}", font_name);

        if let Some(emoji_bytes) = get_system_emoji_font_bytes() {
            font_defs.font_data.insert(
                "system_emoji".to_owned(),
                FontData::from_owned((*emoji_bytes).clone()).into(),
            );
        }

        font_defs
            .font_data
            .insert(font_key.clone(), FontData::from_owned(bytes).into());

        font_defs
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, font_key.clone());

        font_defs
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, font_key);

        if font_defs.font_data.contains_key("system_emoji") {
            font_defs
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push("system_emoji".to_owned());
            font_defs
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("system_emoji".to_owned());
        }

        font_defs
    } else {
        FontDefinitions::default()
    }
}

/// Asynchronously loads system fonts on a background thread.
///
/// Returns a `Receiver` that yields the built `FontDefinitions` when loading completes.
/// The caller should poll `try_recv()` each frame and call `ctx.set_fonts()` when ready.
/// During loading, egui uses its built-in default fonts.
pub fn setup_fonts_async(
    ctx: &egui::Context,
    font_name: &str,
) -> std::sync::mpsc::Receiver<FontDefinitions> {
    let (tx, rx) = std::sync::mpsc::channel();
    let ctx_clone = ctx.clone();
    let font_name = font_name.to_string();

    std::thread::spawn(move || {
        let font_defs = build_font_definitions(&font_name);
        let _ = tx.send(font_defs);
        ctx_clone.request_repaint();
    });

    rx
}
