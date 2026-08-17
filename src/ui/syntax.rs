//! Real-time language syntax highlighting engine for the text editor and markdown preview.
//!
//! Provides language detection, identifier normalization, and `egui::text::LayoutJob`
//! syntax coloration powered by `egui_extras::syntax_highlighting` for code and notes.

use eframe::egui::{self, Color32, FontId, Style, text::LayoutJob};
use std::path::Path;

/// Normalizes any language name, alias, or file extension into a canonical Syntect language identifier.
pub fn normalize_language(name_or_ext: &str) -> &'static str {
    let lower = name_or_ext.trim().to_ascii_lowercase();
    match lower.as_str() {
        "rs" | "rust" => "rs",
        "py" | "python" | "pyw" => "py",
        "js" | "javascript" | "jsx" | "mjs" | "cjs" | "node" => "js",
        "ts" | "typescript" | "tsx" | "mts" | "cts" => "ts",
        "json" | "jsonc" | "json5" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" | "mdown" | "qn" | "qnote" => "md",
        "sh" | "bash" | "zsh" | "fish" | "shell" => "sh",
        "c" | "h" => "c",
        "cpp" | "cxx" | "cc" | "c++" | "hpp" | "hxx" | "hh" => "cpp",
        "html" | "htm" | "xml" | "svg" => "html",
        "css" | "scss" | "sass" | "less" => "css",
        "sql" | "mysql" | "pgsql" | "sqlite" => "sql",
        "lua" => "lua",
        "go" | "golang" => "go",
        "java" => "java",
        "kt" | "kts" | "kotlin" => "kotlin",
        "swift" => "swift",
        "rb" | "ruby" => "ruby",
        "php" => "php",
        "zig" => "zig",
        "diff" | "patch" => "diff",
        _ => "",
    }
}

/// Detects syntax language identifier from note title or linked file path.
///
/// # Preconditions
/// * Invariant: Returns a recognized lowercase language identifier string, or empty string `""` for plain text.
pub fn detect_language(title: &str, file_path: Option<&str>) -> &'static str {
    let path_to_check = file_path.unwrap_or(title);
    let path = Path::new(path_to_check);

    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let normalized = normalize_language(ext);
        if !normalized.is_empty() {
            return normalized;
        }
    }

    normalize_language(path_to_check)
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

/// Creates a syntax-highlighted `LayoutJob` for Markdown documents with embedded code block highlighting.
pub fn highlight_markdown_editor(
    ctx: &egui::Context,
    style: &Style,
    text: &str,
    opts: HighlightOptions<'_>,
) -> LayoutJob {
    let mut job = LayoutJob {
        wrap: egui::text::TextWrapping {
            max_width: opts.wrap_width,
            ..Default::default()
        },
        ..Default::default()
    };

    if text.is_empty() {
        return job;
    }

    let mono_font = FontId::monospace(opts.font_id.size);
    let prop_font = opts.font_id.clone();
    let text_format = egui::TextFormat::simple(prop_font.clone(), opts.text_color);
    let fence_format =
        egui::TextFormat::simple(mono_font.clone(), Color32::from_rgb(140, 150, 190));
    let header_format = egui::TextFormat::simple(
        FontId::proportional(opts.font_id.size + 1.0),
        Color32::from_rgb(130, 200, 255),
    );
    let quote_format =
        egui::TextFormat::simple(prop_font.clone(), Color32::from_rgb(170, 185, 200));

    let mut in_code_block: Option<(String, Vec<String>)> = None;

    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();

        if let Some((lang, mut lines)) = in_code_block.take() {
            if trimmed.starts_with("```") {
                // 1. Highlight accumulated code block content
                let code_block_str = lines.concat();
                if !code_block_str.is_empty() {
                    let code_lang = normalize_language(&lang);
                    if !code_lang.is_empty() {
                        let sub_job = egui_extras::syntax_highlighting::highlight(
                            ctx,
                            style,
                            opts.theme,
                            &code_block_str,
                            code_lang,
                        );
                        for section in sub_job.sections {
                            let mut fmt = section.format;
                            fmt.font_id = mono_font.clone();
                            let start = section.byte_range.start.0;
                            let end = section.byte_range.end.0;
                            let slice = &sub_job.text[start..end];
                            job.append(slice, section.leading_space, fmt);
                        }
                    } else {
                        job.append(
                            &code_block_str,
                            0.0,
                            egui::TextFormat::simple(
                                mono_font.clone(),
                                Color32::from_rgb(220, 225, 235),
                            ),
                        );
                    }
                }

                // 2. Append closing fence line
                job.append(line, 0.0, fence_format.clone());
            } else {
                lines.push(line.to_string());
                in_code_block = Some((lang, lines));
            }
        } else if trimmed.starts_with("```") {
            // Opening fence line
            let lang_tag = trimmed.trim_start_matches('`').trim().to_string();
            job.append(line, 0.0, fence_format.clone());
            in_code_block = Some((lang_tag, Vec::new()));
        } else {
            // Regular Markdown line
            let line_format = if trimmed.starts_with('#') {
                header_format.clone()
            } else if trimmed.starts_with('>') {
                quote_format.clone()
            } else {
                text_format.clone()
            };

            job.append(line, 0.0, line_format);
        }
    }

    // Trailing unclosed code block
    if let Some((lang, lines)) = in_code_block {
        let code_block_str = lines.concat();
        if !code_block_str.is_empty() {
            let code_lang = normalize_language(&lang);
            if !code_lang.is_empty() {
                let sub_job = egui_extras::syntax_highlighting::highlight(
                    ctx,
                    style,
                    opts.theme,
                    &code_block_str,
                    code_lang,
                );
                for section in sub_job.sections {
                    let mut fmt = section.format;
                    fmt.font_id = mono_font.clone();
                    let start = section.byte_range.start.0;
                    let end = section.byte_range.end.0;
                    let slice = &sub_job.text[start..end];
                    job.append(slice, section.leading_space, fmt);
                }
            } else {
                job.append(
                    &code_block_str,
                    0.0,
                    egui::TextFormat::simple(mono_font.clone(), Color32::from_rgb(220, 225, 235)),
                );
            }
        }
    }

    job
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
    if opts.language == "md"
        || opts.language == "markdown"
        || (opts.language.is_empty() && text.contains("```"))
    {
        highlight_markdown_editor(ctx, style, text, opts)
    } else if opts.language.is_empty() {
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
        assert_eq!(detect_language("document.qn", None), "md");
        assert_eq!(detect_language("setup.sh", None), "sh");
        assert_eq!(detect_language("query.sql", None), "sql");
        assert_eq!(detect_language("index.html", None), "html");
        assert_eq!(detect_language("style.css", None), "css");
    }

    #[test]
    fn test_normalize_language_names() {
        assert_eq!(normalize_language("rust"), "rs");
        assert_eq!(normalize_language("python"), "py");
        assert_eq!(normalize_language("javascript"), "js");
        assert_eq!(normalize_language("typescript"), "ts");
        assert_eq!(normalize_language("bash"), "sh");
        assert_eq!(normalize_language("shell"), "sh");
        assert_eq!(normalize_language("c++"), "cpp");
        assert_eq!(normalize_language("golang"), "go");
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

    #[test]
    fn test_highlight_text_basic() {
        let ctx = egui::Context::default();
        let style = egui::Style::default();
        let theme = egui_extras::syntax_highlighting::CodeTheme::dark(13.0);
        let opts = HighlightOptions {
            theme: &theme,
            language: "rs",
            font_id: FontId::monospace(13.0),
            text_color: Color32::WHITE,
            wrap_width: 500.0,
        };
        let job = highlight_text(&ctx, &style, "fn main() { let x = 42; }", opts);
        assert!(!job.sections.is_empty());
    }

    #[test]
    fn test_highlight_markdown_with_fenced_code_block() {
        let ctx = egui::Context::default();
        let style = egui::Style::default();
        let theme = egui_extras::syntax_highlighting::CodeTheme::dark(13.0);
        let opts = HighlightOptions {
            theme: &theme,
            language: "md",
            font_id: FontId::proportional(14.0),
            text_color: Color32::WHITE,
            wrap_width: 600.0,
        };
        let md_text = "# Header\nSome regular note text\n```rust\nfn main() {\n    println!(\"Hello!\");\n}\n```\nAfter code block";
        let job = highlight_text(&ctx, &style, md_text, opts);
        assert!(!job.sections.is_empty());
        assert_eq!(job.text, md_text);
    }
}
