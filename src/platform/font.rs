//! Embedded font system bundling FiraCode Nerd Font Mono and Inter, with dynamic system font discovery.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::collections::HashMap;
#[cfg(not(windows))]
use std::process::Command;
use std::sync::{Mutex, OnceLock};

/// Embedded high-legibility UI sans-serif font (Inter).
pub const EMBEDDED_UI_FONT: &[u8] = include_bytes!("../../assets/fonts/Inter.ttf");

/// Embedded developer coding, ligature, and symbol Nerd Font (FiraCode Nerd Font Mono).
pub const EMBEDDED_MONO_FONT: &[u8] = include_bytes!("../../assets/fonts/FiraCodeNerdFontMono.ttf");

static CACHED_SYSTEM_FONTS: OnceLock<Vec<String>> = OnceLock::new();
static CACHED_MONOSPACE_FONTS: OnceLock<Vec<String>> = OnceLock::new();
static CACHED_FONT_PATHS: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

/// Discovers installed system font family names on Linux (via `fc-list`) or Windows (via Fonts dir).
/// Results are cached in a OnceLock to eliminate UI thread blocking.
pub fn get_installed_system_fonts() -> Vec<String> {
    CACHED_SYSTEM_FONTS
        .get_or_init(|| {
            let mut fonts = vec!["Default".to_string()];

            #[cfg(target_os = "linux")]
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

                for f in detected {
                    if !fonts.contains(&f) {
                        fonts.push(f);
                    }
                }
            }

            #[cfg(windows)]
            {
                let candidates = &[
                    "Segoe UI",
                    "Aptos",
                    "Calibri",
                    "Arial",
                    "Inter",
                    "Roboto",
                    "Noto Sans",
                ];
                for p in candidates {
                    if resolve_font_path(p).is_some() && !fonts.contains(&p.to_string()) {
                        fonts.push(p.to_string());
                    }
                }
            }

            fonts
        })
        .clone()
}

/// Discovers installed monospace and coding font family names on Linux or Windows.
/// Results are cached in a OnceLock to eliminate UI thread blocking.
pub fn get_installed_monospace_fonts() -> Vec<String> {
    CACHED_MONOSPACE_FONTS
        .get_or_init(|| {
            let mut fonts = vec!["Default".to_string()];

            #[cfg(target_os = "linux")]
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

                for f in detected {
                    if !fonts.contains(&f) {
                        fonts.push(f);
                    }
                }
            }

            #[cfg(windows)]
            {
                let candidates = &[
                    "Cascadia Code",
                    "Cascadia Mono",
                    "Consolas",
                    "Fira Code",
                    "JetBrains Mono",
                    "Lucida Console",
                    "Courier New",
                ];
                for p in candidates {
                    if resolve_font_path(p).is_some() && !fonts.contains(&p.to_string()) {
                        fonts.push(p.to_string());
                    }
                }
            }

            fonts
        })
        .clone()
}

#[cfg(windows)]
fn query_platform_font_path(pattern: &str) -> Option<String> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let fonts_dir = std::path::PathBuf::from(windir).join("Fonts");
    if !fonts_dir.exists() {
        return None;
    }

    let p_lower = pattern.to_lowercase();
    let direct_candidates: &[(&str, &[&str])] = &[
        ("segoe ui emoji", &["seguiemj.ttf"]),
        ("segoe ui symbol", &["seguisym.ttf"]),
        ("segoe ui", &["segoeui.ttf", "segoeuib.ttf"]),
        (
            "cascadia code",
            &["CascadiaCode.ttf", "cascadiacode.ttf", "Cascadia.ttf"],
        ),
        ("cascadia mono", &["CascadiaMono.ttf", "cascadiamono.ttf"]),
        ("consolas", &["consola.ttf", "consolab.ttf"]),
        ("lucida console", &["lucon.ttf"]),
        ("courier new", &["cour.ttf"]),
        ("calibri", &["calibri.ttf"]),
        ("arial", &["arial.ttf"]),
        ("aptos mono", &["aptos-mono.ttf", "AptosMono.ttf"]),
        ("aptos", &["aptos.ttf", "Aptos.ttf"]),
    ];

    for (name, files) in direct_candidates {
        if p_lower.contains(name) {
            for f in *files {
                let candidate_path = fonts_dir.join(f);
                if candidate_path.exists() {
                    return Some(candidate_path.to_string_lossy().to_string());
                }
            }
        }
    }

    let sanitized_name = pattern.replace(' ', "");
    for ext in &["ttf", "otf", "ttc"] {
        let direct_file = fonts_dir.join(format!("{}.{}", sanitized_name, ext));
        if direct_file.exists() {
            return Some(direct_file.to_string_lossy().to_string());
        }
        let lower_file = fonts_dir.join(format!("{}.{}", sanitized_name.to_lowercase(), ext));
        if lower_file.exists() {
            return Some(lower_file.to_string_lossy().to_string());
        }
    }

    None
}

#[cfg(not(windows))]
fn query_platform_font_path(pattern: &str) -> Option<String> {
    if let Ok(output) = Command::new("fc-match")
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
    }
}

/// Queries system font provider (fontconfig on Linux, %WINDIR%/Fonts on Windows) to resolve font paths.
/// Results are cached in a thread-safe `Mutex<HashMap>` to eliminate repeated queries.
pub fn resolve_font_path(pattern: &str) -> Option<String> {
    let cache_map = CACHED_FONT_PATHS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache_map.lock()
        && let Some(cached) = guard.get(pattern)
    {
        return cached.clone();
    }

    let resolved = query_platform_font_path(pattern);

    if let Ok(mut guard) = cache_map.lock() {
        guard.insert(pattern.to_string(), resolved.clone());
    }

    resolved
}

/// Builds font definitions with separate System UI (Proportional) and Buffer (Monospace) fonts,
/// using embedded Inter and FiraCode Nerd Font Mono as guaranteed high-legibility defaults.
pub fn build_font_definitions(system_font: &str, editor_font: &str) -> FontDefinitions {
    let mut font_defs = FontDefinitions::default();

    // 1. Always install embedded FiraCode Nerd Font Mono as universal symbols/nerd icons provider
    font_defs.font_data.insert(
        "embedded_nerd_symbols".to_owned(),
        FontData::from_static(EMBEDDED_MONO_FONT).into(),
    );

    // 2. Configure System UI Font (Proportional)
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
    } else {
        font_defs.font_data.insert(
            "embedded_ui_inter".to_owned(),
            FontData::from_static(EMBEDDED_UI_FONT).into(),
        );
        font_defs
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "embedded_ui_inter".to_owned());
    }

    // 3. Configure Buffer / Coding Editor Font (Monospace)
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
    } else {
        font_defs.font_data.insert(
            "embedded_mono_firacode".to_owned(),
            FontData::from_static(EMBEDDED_MONO_FONT).into(),
        );
        font_defs
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "embedded_mono_firacode".to_owned());
    }

    // 4. Append Nerd Font symbols to both proportional UI and monospace buffers
    font_defs
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push("embedded_nerd_symbols".to_owned());
    font_defs
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("embedded_nerd_symbols".to_owned());

    font_defs
}

/// Applies both System UI and Buffer fonts to egui.
pub fn apply_system_fonts(ctx: &egui::Context, system_font: &str, editor_font: &str) {
    let font_defs = build_font_definitions(system_font, editor_font);
    ctx.set_fonts(font_defs);
}

/// Convenience single-font fallback for backward compatibility.
pub fn apply_system_font(ctx: &egui::Context, font_name: &str) {
    apply_system_fonts(ctx, font_name, "Default");
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
