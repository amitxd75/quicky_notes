//! Real-time language syntax highlighting engine for the text editor.
//!
//! Provides language detection and `egui::text::LayoutJob` syntax coloration
//! powered by `egui_extras::syntax_highlighting` for code and markdown notes.

use eframe::egui::{self, Color32, FontId, Style, text::LayoutJob};
use std::path::Path;

/// Detects syntax language identifier from note title or linked file path.
///
/// # Preconditions
/// * Invariant: Returns a recognized lowercase language identifier string, or empty string `""` for plain text.
pub fn detect_language(title: &str, file_path: Option<&str>) -> &'static str {
    let path_to_check = file_path.unwrap_or(title);
    let path = Path::new(path_to_check);

    let ext = match path.extension().and_then(|s| s.to_str()) {
        Some(e) => e.to_ascii_lowercase(),
        None => return "",
    };

    match ext.as_str() {
        "rs" => "rs",
        "py" | "pyw" => "py",
        "js" | "jsx" | "mjs" | "cjs" => "js",
        "ts" | "tsx" | "mts" | "cts" => "ts",
        "json" | "jsonc" | "json5" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" | "mdown" => "md",
        "sh" | "bash" | "zsh" | "fish" => "sh",
        "c" | "h" => "c",
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => "cpp",
        "html" | "htm" | "xml" | "svg" => "html",
        "css" | "scss" | "sass" | "less" => "css",
        "sql" => "sql",
        "lua" => "lua",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "rb" => "ruby",
        "php" => "php",
        "zig" => "zig",
        "diff" | "patch" => "diff",
        _ => "",
    }
}

/// Rendering parameters and style configuration for syntax highlighting.
#[derive(Clone)]
pub struct HighlightOptions<'a> {
    /// Active code coloration theme.
    pub theme: &'a egui_extras::syntax_highlighting::CodeTheme,
    /// Target language identifier (e.g. "rs", "py", "json"), or "" for plain text.
    pub language: &'a str,
    /// Desired typography font style and size.
    pub font_id: FontId,
    /// Fallback plain text color.
    pub text_color: Color32,
    /// Max line wrapping width in pixels.
    pub wrap_width: f32,
}

/// Creates a syntax-highlighted `LayoutJob` for the note content.
///
/// If language is empty or syntax highlighting fails, returns a default `LayoutJob`
/// styled according to the active theme font and colors.
pub fn highlight_text(
    ctx: &egui::Context,
    style: &Style,
    text: &str,
    opts: HighlightOptions<'_>,
) -> LayoutJob {
    if opts.language.is_empty() {
        let mut job = LayoutJob::simple(
            text.to_string(),
            opts.font_id,
            opts.text_color,
            opts.wrap_width,
        );
        job.wrap.max_width = opts.wrap_width;
        job
    } else {
        let mut job = egui_extras::syntax_highlighting::highlight(
            ctx,
            style,
            opts.theme,
            text,
            opts.language,
        );
        for section in &mut job.sections {
            section.format.font_id = opts.font_id.clone();
        }
        job.wrap.max_width = opts.wrap_width;
        job
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_from_extensions() {
        assert_eq!(detect_language("main.rs", None), "rs");
        assert_eq!(detect_language("script.py", None), "py");
        assert_eq!(detect_language("app.tsx", None), "ts");
        assert_eq!(detect_language("server.js", None), "js");
        assert_eq!(detect_language("config.toml", None), "toml");
        assert_eq!(detect_language("data.json", None), "json");
        assert_eq!(detect_language("docker-compose.yml", None), "yaml");
        assert_eq!(detect_language("README.md", None), "md");
        assert_eq!(detect_language("setup.sh", None), "sh");
        assert_eq!(detect_language("query.sql", None), "sql");
        assert_eq!(detect_language("index.html", None), "html");
        assert_eq!(detect_language("style.css", None), "css");
    }

    #[test]
    fn test_detect_language_case_insensitivity() {
        assert_eq!(detect_language("MAIN.RS", None), "rs");
        assert_eq!(detect_language("SCRIPT.PY", None), "py");
        assert_eq!(detect_language("NOTES.TXT", None), "");
    }

    #[test]
    fn test_detect_language_from_file_path() {
        assert_eq!(
            detect_language("untitled", Some("/home/user/project/src/lib.rs")),
            "rs"
        );
        assert_eq!(detect_language("untitled", Some("notes.txt")), "");
        assert_eq!(detect_language("no_extension", None), "");
    }
}
