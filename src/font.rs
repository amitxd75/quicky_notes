//! System font discovery and dynamic TTF binary loading via `fontconfig`.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::process::Command;

use std::sync::OnceLock;

static CACHED_SYSTEM_FONTS: OnceLock<Vec<String>> = OnceLock::new();

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

/// Resolves font file path via `fc-match` and applies the TTF font family dynamically to egui.
pub fn apply_system_font(ctx: &egui::Context, font_name: &str) {
    if font_name == "Default" || font_name.is_empty() {
        return;
    }

    if let Ok(output) = Command::new("fc-match")
        .arg("-f")
        .arg("%{file}")
        .arg(font_name)
        .output()
        && output.status.success()
    {
        let file_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !file_path.is_empty()
            && std::path::Path::new(&file_path).exists()
            && let Ok(bytes) = std::fs::read(&file_path)
        {
            let mut font_defs = FontDefinitions::default();
            let font_key = format!("sys_font_{}", font_name);

            font_defs
                .font_data
                .insert(font_key.clone(), FontData::from_owned(bytes));

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

            ctx.set_fonts(font_defs);
        }
    }
}
