//! Windows-specific font path resolution and font directory indexer.

use std::path::PathBuf;

/// Direct candidate table mapping font family names to common Windows font filenames.
const DIRECT_CANDIDATES: &[(&str, &[&str])] = &[
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

/// Resolves a font name pattern against the Windows `%WINDIR%\Fonts\` directory.
pub fn query_platform_font_path(pattern: &str) -> Option<String> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let fonts_dir = PathBuf::from(windir).join("Fonts");
    if !fonts_dir.exists() {
        return None;
    }

    let p_lower = pattern.to_lowercase();

    for (name, files) in DIRECT_CANDIDATES {
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

/// Discovers common Windows UI font candidates that exist on the current machine.
pub fn discover_system_ui_fonts() -> Vec<String> {
    let candidates = &[
        "Segoe UI",
        "Aptos",
        "Calibri",
        "Arial",
        "Inter",
        "Roboto",
        "Noto Sans",
    ];
    let mut fonts = Vec::new();
    for p in candidates {
        if query_platform_font_path(p).is_some() {
            fonts.push(p.to_string());
        }
    }
    fonts
}

/// Discovers common Windows monospace and coding font candidates that exist on the current machine.
pub fn discover_monospace_fonts() -> Vec<String> {
    let candidates = &[
        "Cascadia Code",
        "Cascadia Mono",
        "Consolas",
        "Fira Code",
        "JetBrains Mono",
        "Lucida Console",
        "Courier New",
    ];
    let mut fonts = Vec::new();
    for p in candidates {
        if query_platform_font_path(p).is_some() {
            fonts.push(p.to_string());
        }
    }
    fonts
}
