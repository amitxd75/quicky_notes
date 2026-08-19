//! Interactive right-hand Document Outline & Symbol Navigator sidebar.
//!
//! Parses and displays hierarchical Markdown headings (H1–H6), numbered sections (1., 1.1),
//! and code symbols (functions, structs, classes) with active section tracking, real-time filtering,
//! and smooth jump-to-section navigation.

use crate::theme::Palette;
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, RichText, Sense, Stroke, Ui};

/// Default outline sidebar width in pixels.
pub const DEFAULT_OUTLINE_WIDTH: f32 = 220.0;
/// Minimum outline sidebar width in pixels.
pub const MIN_OUTLINE_WIDTH: f32 = 150.0;
/// Maximum outline sidebar width in pixels.
pub const MAX_OUTLINE_WIDTH: f32 = 400.0;

/// Maximum hierarchical depth level supported for outline indentation.
pub const MAX_HEADING_LEVEL: usize = 6;
/// Pixel indentation per hierarchy level in the symbol tree.
pub const INDENT_PER_LEVEL: f32 = 10.0;
/// Maximum total indentation allowed in the sidebar to preserve readability.
pub const MAX_INDENT_OFFSET: f32 = 40.0;
/// Maximum allowed heading length in characters for plain-text heuristics.
pub const MAX_PLAIN_TEXT_HEADER_LEN: usize = 60;

/// Represents a classified document symbol or section heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbol {
    /// Display text of the heading or symbol.
    pub title: String,
    /// 1-based hierarchy level (1 for H1 / top section, 2 for H2 / method, etc.).
    pub level: usize,
    /// 1-based line number in the source note.
    pub line_number: usize,
    /// 0-based character offset in the document buffer.
    pub char_offset: usize,
    /// Semantic classification of the symbol.
    pub kind: SymbolKind,
}

/// Semantic kind of document symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    NumberedSection,
    Function,
    StructOrClass,
    SectionHeader,
    TodoItem,
}

impl SymbolKind {
    /// Short 2-3 letter badge text for the symbol type.
    pub fn badge_label(&self) -> &'static str {
        match self {
            Self::Heading1 => "H1",
            Self::Heading2 => "H2",
            Self::Heading3 => "H3",
            Self::Heading4 => "H4",
            Self::Heading5 => "H5",
            Self::Heading6 => "H6",
            Self::NumberedSection => "§",
            Self::Function => "fn",
            Self::StructOrClass => "type",
            Self::SectionHeader => "sec",
            Self::TodoItem => "todo",
        }
    }
}

/// Extracts hierarchical outline symbols from document text buffer.
pub fn extract_document_symbols(
    content: &str,
    is_markdown: bool,
    file_path: Option<&str>,
) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    let mut in_fenced_code_block = false;

    let is_code_file = file_path.is_some_and(|fp| {
        let p = std::path::Path::new(fp);
        p.extension().is_some_and(|ext| {
            let ext_str = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_str.as_str(),
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "go"
                    | "c"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "java"
                    | "kt"
                    | "cs"
                    | "sh"
                    | "bash"
                    | "lua"
                    | "toml"
                    | "json"
                    | "yaml"
                    | "yml"
            )
        })
    });

    let mut char_offset = 0;
    for (line_idx, line) in content.split('\n').enumerate() {
        let line_number = line_idx + 1;
        let line_clean = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = line_clean.trim();

        // Handle fenced code blocks in Markdown so code lines aren't parsed as Markdown headings
        if is_markdown && (trimmed.starts_with("```") || trimmed.starts_with("~~~")) {
            in_fenced_code_block = !in_fenced_code_block;
            char_offset += line.chars().count() + 1;
            continue;
        }

        if !in_fenced_code_block && (is_markdown || !is_code_file) {
            // 1. Markdown Headings (# through ######)
            if let Some(symbol) = parse_markdown_heading(line_clean, line_number, char_offset) {
                symbols.push(symbol);
                char_offset += line.chars().count() + 1;
                continue;
            }

            // 2. Numbered Section Headings (e.g. "1. Getting Started", "1.1. Installation")
            if let Some(symbol) = parse_numbered_section(line_clean, line_number, char_offset) {
                symbols.push(symbol);
                char_offset += line.chars().count() + 1;
                continue;
            }

            // 3. Task / Todo Items
            if let Some(symbol) = parse_task_item(line_clean, line_number, char_offset) {
                symbols.push(symbol);
                char_offset += line.chars().count() + 1;
                continue;
            }
        }

        // 4. Code Symbols (Functions, Structs, Classes, Banners)
        if (is_code_file || in_fenced_code_block)
            && let Some(symbol) = parse_code_symbol(line_clean, line_number, char_offset)
        {
            symbols.push(symbol);
            char_offset += line.chars().count() + 1;
            continue;
        }

        // 5. Plain text section headers (e.g. lines ending in ":" or banner dividers)
        if !is_markdown
            && !is_code_file
            && let Some(symbol) = parse_plain_text_header(line_clean, line_number, char_offset)
        {
            symbols.push(symbol);
        }

        char_offset += line.chars().count() + 1;
    }

    symbols
}

/// Parses a Markdown heading line (`# Title` through `###### Title`).
fn parse_markdown_heading(
    line: &str,
    line_number: usize,
    char_offset: usize,
) -> Option<DocumentSymbol> {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return None;
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if hash_count == 0 || hash_count > 6 {
        return None;
    }

    let remainder = trimmed[hash_count..].trim();
    if remainder.is_empty() {
        return None;
    }

    // Must have a space after '#' or be valid markdown heading syntax
    if !trimmed[hash_count..].starts_with(' ') && !trimmed[hash_count..].starts_with('\t') {
        return None;
    }

    let kind = match hash_count {
        1 => SymbolKind::Heading1,
        2 => SymbolKind::Heading2,
        3 => SymbolKind::Heading3,
        4 => SymbolKind::Heading4,
        5 => SymbolKind::Heading5,
        _ => SymbolKind::Heading6,
    };

    Some(DocumentSymbol {
        title: remainder.to_string(),
        level: hash_count,
        line_number,
        char_offset,
        kind,
    })
}

/// Parses numbered section titles like "1. Introduction", "1.1 Installation", "2.3.4 Details".
fn parse_numbered_section(
    line: &str,
    line_number: usize,
    char_offset: usize,
) -> Option<DocumentSymbol> {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }

    let mut num_prefix_end = 0;

    for (idx, ch) in trimmed.char_indices() {
        if ch.is_ascii_digit() || ch == '.' {
            num_prefix_end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    if num_prefix_end == 0 || num_prefix_end >= trimmed.len() {
        return None;
    }

    let rest = trimmed[num_prefix_end..].trim();
    if rest.is_empty() {
        return None;
    }

    // Calculate level based on dot count (e.g. "1." -> level 1, "1.1" -> level 2, "1.1.1" -> level 3)
    let segments: Vec<&str> = trimmed[..num_prefix_end]
        .trim_matches('.')
        .split('.')
        .collect();

    let level = segments.len().clamp(1, MAX_HEADING_LEVEL);

    // Require title to start with capital letter or meaningful character
    if !rest.starts_with(|c: char| c.is_alphabetic() || c == '[' || c == '`') {
        return None;
    }

    Some(DocumentSymbol {
        title: trimmed.to_string(),
        level,
        line_number,
        char_offset,
        kind: SymbolKind::NumberedSection,
    })
}

/// Parses Markdown task items `- [ ]` or `- [x]`.
fn parse_task_item(line: &str, line_number: usize, char_offset: usize) -> Option<DocumentSymbol> {
    let trimmed = line.trim();
    if trimmed.starts_with("- [ ] ")
        || trimmed.starts_with("- [x] ")
        || trimmed.starts_with("* [ ] ")
        || trimmed.starts_with("* [x] ")
    {
        let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).count();
        let level = (leading_spaces / 2 + 2).clamp(1, 6);
        let task_text = trimmed[6..].trim();
        if !task_text.is_empty() {
            return Some(DocumentSymbol {
                title: task_text.to_string(),
                level,
                line_number,
                char_offset,
                kind: SymbolKind::TodoItem,
            });
        }
    }
    None
}

/// Parses code definitions (functions, structs, classes, enums, interfaces, banners).
fn parse_code_symbol(line: &str, line_number: usize, char_offset: usize) -> Option<DocumentSymbol> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).count();
    let indent_level = (leading_spaces / 4 + 1).clamp(1, 6);

    // Section banners in comments (e.g. "// --- SETUP ---", "# === CONFIG ===")
    if (trimmed.starts_with("// ---")
        || trimmed.starts_with("// ===")
        || trimmed.starts_with("# ===")
        || trimmed.starts_with("# ---"))
        && trimmed.len() > 7
    {
        let clean = trimmed
            .trim_start_matches(['/', '#', '-', '='])
            .trim_end_matches(['-', '='])
            .trim();
        if !clean.is_empty() {
            return Some(DocumentSymbol {
                title: clean.to_string(),
                level: 1,
                line_number,
                char_offset,
                kind: SymbolKind::SectionHeader,
            });
        }
    }

    // Rust / C / Go / Python / JS functions and types
    let is_fn = trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("async def ")
        || trimmed.starts_with("function ")
        || trimmed.starts_with("func ");

    if is_fn {
        let clean_title = trimmed
            .trim_start_matches("pub ")
            .trim_start_matches("async ")
            .split('{')
            .next()
            .unwrap_or(trimmed)
            .split(':')
            .next()
            .unwrap_or(trimmed)
            .trim();
        return Some(DocumentSymbol {
            title: clean_title.to_string(),
            level: indent_level + 1,
            line_number,
            char_offset,
            kind: SymbolKind::Function,
        });
    }

    let is_type = trimmed.starts_with("struct ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("pub type ");

    if is_type {
        let clean_title = trimmed
            .trim_start_matches("pub ")
            .split('{')
            .next()
            .unwrap_or(trimmed)
            .split('(')
            .next()
            .unwrap_or(trimmed)
            .trim();
        return Some(DocumentSymbol {
            title: clean_title.to_string(),
            level: indent_level,
            line_number,
            char_offset,
            kind: SymbolKind::StructOrClass,
        });
    }

    None
}

/// Parses plain text capitalized headers or lines ending with `:`.
fn parse_plain_text_header(
    line: &str,
    line_number: usize,
    char_offset: usize,
) -> Option<DocumentSymbol> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_PLAIN_TEXT_HEADER_LEN {
        return None;
    }

    if trimmed.ends_with(':') && !trimmed.contains("http") && trimmed.len() >= 3 {
        let title = trimmed.trim_end_matches(':').trim();
        if !title.is_empty() {
            return Some(DocumentSymbol {
                title: title.to_string(),
                level: 1,
                line_number,
                char_offset,
                kind: SymbolKind::SectionHeader,
            });
        }
    }

    None
}

/// Determines the index of the symbol that encompasses the current cursor position.
pub fn find_active_symbol_index(
    symbols: &[DocumentSymbol],
    cursor_char_offset: usize,
) -> Option<usize> {
    if symbols.is_empty() {
        return None;
    }

    let mut best_idx = None;
    for (idx, sym) in symbols.iter().enumerate() {
        if sym.char_offset <= cursor_char_offset {
            best_idx = Some(idx);
        } else {
            break;
        }
    }

    best_idx
}

/// Configuration and state inputs for rendering the Document Outline sidebar.
pub struct OutlineRenderConfig<'a> {
    /// Extracted document symbols to display in the outline tree.
    pub symbols: &'a [DocumentSymbol],
    /// Index of the symbol currently under the editor cursor, if any.
    pub active_symbol_idx: Option<usize>,
    /// Selected symbol index in filtered list for keyboard arrow navigation.
    pub selected_idx: &'a mut Option<usize>,
    /// Real-time search query for filtering headings.
    pub filter_query: &'a mut String,
    /// Active theme color palette.
    pub palette: &'a Palette,
    /// Total available vertical height for the sidebar.
    pub height: f32,
    /// Configured UI font size from user appearance settings.
    pub ui_font_size: f32,
}

/// Renders the interactive glass Document Outline sidebar.
/// Returns `Some((char_offset, line_number))` if the user clicks a symbol to navigate to.
pub fn render_outline_sidebar(
    config: &mut OutlineRenderConfig<'_>,
    ui: &mut Ui,
    on_close: &mut bool,
) -> Option<(usize, usize)> {
    let height = config.height;
    let ui_font_size = config.ui_font_size;
    let symbols = config.symbols;
    let active_symbol_idx = config.active_symbol_idx;
    let palette = config.palette;
    let filter_query = &mut *config.filter_query;
    let selected_idx = &mut *config.selected_idx;

    debug_assert!(height >= 0.0, "Sidebar height must be non-negative");
    debug_assert!(
        (crate::models::settings::MIN_UI_FONT_SIZE..=crate::models::settings::MAX_UI_FONT_SIZE)
            .contains(&ui_font_size),
        "UI font size must be within valid bounds"
    );
    let mut navigation_target = None;

    let header_font_size = ui_font_size.clamp(10.5, 22.0);
    let filter_font_size = (ui_font_size - 2.0).clamp(9.5, 18.0);
    let item_font_size = (ui_font_size - 2.5).clamp(9.0, 19.0);
    let h1_font_size = (ui_font_size - 2.0).clamp(9.5, 20.0);
    let badge_font_size = (ui_font_size - 3.5).clamp(8.5, 16.0);
    let line_font_size = (ui_font_size - 4.0).clamp(8.0, 15.0);

    let frame_margin = Margin::symmetric(8, 6);
    let inner_height = (height - (frame_margin.top + frame_margin.bottom) as f32).max(20.0);

    let outline_frame = egui::Frame::NONE
        .fill(Palette::with_alpha(palette.card, 190))
        .stroke(Stroke::new(1.0, Palette::with_alpha(palette.border, 90)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(frame_margin);

    outline_frame.show(ui, |ui| {
        ui.set_height(inner_height);
        ui.set_max_height(inner_height);

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

            // 1. Outline Header Bar
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                ui.label(
                    RichText::new("📑 Outline")
                        .font(FontId::proportional(header_font_size))
                        .strong()
                        .color(Color32::WHITE),
                );

                ui.add_space(4.0);

                // Symbol count badge
                let count_text = format!("{}", symbols.len());
                let badge_bg = Palette::with_alpha(palette.card, 220);
                let badge_stroke = Stroke::new(1.0, Palette::with_alpha(palette.border, 100));
                let badge_frame = egui::Frame::NONE
                    .fill(badge_bg)
                    .stroke(badge_stroke)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(6, 2));
                badge_frame.show(ui, |ui| {
                    ui.label(
                        RichText::new(count_text)
                            .font(FontId::proportional((badge_font_size + 0.5).max(10.0)))
                            .color(Color32::from_gray(210)),
                    );
                });

                // Close Button on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = crate::components::button::close_button(ui, palette);
                    if close_btn.on_hover_text("Close outline (Alt+O)").clicked() {
                        *on_close = true;
                    }
                });
            });

            // 2. Real-time Search / Filter Input
            if symbols.len() > 4 || !filter_query.is_empty() {
                let search_frame = egui::Frame::NONE
                    .fill(Palette::with_alpha(palette.card, 230))
                    .stroke(Stroke::new(1.0, Palette::with_alpha(palette.border, 100)))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(8, 4));

                search_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.label(
                            RichText::new("🔍")
                                .font(FontId::proportional(filter_font_size))
                                .color(Color32::from_gray(140)),
                        );
                        let has_filter = !filter_query.is_empty();
                        let desired_w = ui.available_width() - if has_filter { 24.0 } else { 8.0 };
                        let search_edit = egui::TextEdit::singleline(filter_query)
                            .hint_text("Filter headings...")
                            .font(FontId::proportional(filter_font_size))
                            .text_color(Color32::WHITE)
                            .frame(egui::Frame::NONE)
                            .desired_width(desired_w);
                        ui.add(search_edit);

                        if has_filter {
                            let clear_btn = ui.add(
                                egui::Button::new(
                                    RichText::new("×")
                                        .font(FontId::proportional(filter_font_size + 1.0))
                                        .color(Color32::from_gray(160)),
                                )
                                .frame(false),
                            );
                            if clear_btn.on_hover_text("Clear filter").clicked() {
                                filter_query.clear();
                            }
                        }
                    });
                });
            }

            ui.add_space(2.0);

            // 3. Scrollable Hierarchical Symbol Tree
            let query = filter_query.trim().to_lowercase();
            let filtered_symbols: Vec<(usize, &DocumentSymbol)> = symbols
                .iter()
                .enumerate()
                .filter(|(_, sym)| query.is_empty() || sym.title.to_lowercase().contains(&query))
                .collect();

            // Handle Keyboard Arrow & Enter Navigation
            let mut arrow_navigated = false;
            if !filtered_symbols.is_empty() {
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    let next_idx = match *selected_idx {
                        Some(cur) => (cur + 1).min(filtered_symbols.len().saturating_sub(1)),
                        None => 0,
                    };
                    *selected_idx = Some(next_idx);
                    arrow_navigated = true;
                    ui.ctx().request_repaint();
                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    let prev_idx = match *selected_idx {
                        Some(cur) => cur.saturating_sub(1),
                        None => 0,
                    };
                    *selected_idx = Some(prev_idx);
                    arrow_navigated = true;
                    ui.ctx().request_repaint();
                }

                if ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && let Some(sel_idx) = *selected_idx
                    && let Some((_, sym)) = filtered_symbols.get(sel_idx)
                {
                    navigation_target = Some((sym.char_offset, sym.line_number));
                }
            }

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                if !filter_query.is_empty() {
                    filter_query.clear();
                    *selected_idx = None;
                } else {
                    *on_close = true;
                }
            }

            if filtered_symbols.is_empty() {
                ui.add_space(16.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(if symbols.is_empty() {
                            "No headings or symbols found.\nUse # or 1. to structure notes."
                        } else {
                            "No matching headings"
                        })
                        .font(FontId::proportional(item_font_size))
                        .color(Color32::from_gray(140)),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .id_salt("outline_symbols_scroll_area")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;

                        for (filter_pos, (orig_idx, sym)) in
                            filtered_symbols.into_iter().enumerate()
                        {
                            let is_keyboard_sel = *selected_idx == Some(filter_pos);
                            let is_active = is_keyboard_sel
                                || (selected_idx.is_none() && active_symbol_idx == Some(orig_idx));
                            let indent = ((sym.level.saturating_sub(1)) as f32 * INDENT_PER_LEVEL)
                                .min(MAX_INDENT_OFFSET);

                            let item_id = egui::Id::new((orig_idx, "outline_item"));
                            let hov_id = item_id.with("hov");
                            let prev_hov: bool =
                                ui.ctx().data(|d| d.get_temp(hov_id)).unwrap_or(false);
                            let hov_anim = ui
                                .ctx()
                                .animate_bool_responsive(hov_id, prev_hov && !is_active);

                            let base_bg = Color32::TRANSPARENT;
                            let hov_bg = Palette::with_alpha(palette.card, 160);
                            let active_bg = Palette::interpolate_color(
                                Palette::with_alpha(palette.card, 220),
                                Palette::with_alpha(palette.accent, 55),
                                0.7,
                            );

                            let item_bg = if is_active {
                                active_bg
                            } else {
                                Palette::interpolate_color(base_bg, hov_bg, hov_anim)
                            };

                            let item_stroke = if is_active {
                                Stroke::new(1.0, Palette::with_alpha(palette.accent, 140))
                            } else if hov_anim > 0.01 {
                                Stroke::new(
                                    1.0,
                                    Palette::with_alpha(palette.border, (90.0 * hov_anim) as u8),
                                )
                            } else {
                                Stroke::NONE
                            };

                            let item_frame = egui::Frame::NONE
                                .fill(item_bg)
                                .stroke(item_stroke)
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::symmetric(6, 4));

                            let item_resp = item_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 5.0;

                                    // Indentation spacing
                                    if indent > 0.0 {
                                        ui.add_space(indent);
                                    }

                                    // Distinct rounded badge pill with color coding
                                    let (badge_bg_col, badge_text_col) = match sym.kind {
                                        SymbolKind::Heading1
                                        | SymbolKind::Heading2
                                        | SymbolKind::Heading3 => (
                                            Palette::with_alpha(
                                                palette.accent,
                                                if is_active { 60 } else { 30 },
                                            ),
                                            if is_active {
                                                Color32::WHITE
                                            } else {
                                                palette.accent
                                            },
                                        ),
                                        SymbolKind::NumberedSection => (
                                            Color32::from_rgba_unmultiplied(
                                                140,
                                                110,
                                                240,
                                                if is_active { 60 } else { 30 },
                                            ),
                                            Color32::from_rgb(180, 155, 255),
                                        ),
                                        SymbolKind::Function => (
                                            Color32::from_rgba_unmultiplied(
                                                40,
                                                190,
                                                130,
                                                if is_active { 60 } else { 30 },
                                            ),
                                            Color32::from_rgb(70, 220, 155),
                                        ),
                                        SymbolKind::StructOrClass => (
                                            Color32::from_rgba_unmultiplied(
                                                230,
                                                170,
                                                50,
                                                if is_active { 60 } else { 30 },
                                            ),
                                            Color32::from_rgb(245, 190, 70),
                                        ),
                                        SymbolKind::TodoItem => (
                                            Color32::from_rgba_unmultiplied(
                                                20,
                                                180,
                                                200,
                                                if is_active { 60 } else { 30 },
                                            ),
                                            Color32::from_rgb(56, 215, 235),
                                        ),
                                        SymbolKind::SectionHeader => (
                                            Color32::from_rgba_unmultiplied(
                                                160,
                                                160,
                                                190,
                                                if is_active { 50 } else { 25 },
                                            ),
                                            Color32::from_rgb(190, 190, 220),
                                        ),
                                        _ => (
                                            Palette::with_alpha(
                                                palette.border,
                                                if is_active { 50 } else { 25 },
                                            ),
                                            Color32::from_gray(160),
                                        ),
                                    };

                                    let badge_pill = egui::Frame::NONE
                                        .fill(badge_bg_col)
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::symmetric(4, 1));
                                    badge_pill.show(ui, |ui| {
                                        ui.label(
                                            RichText::new(sym.kind.badge_label())
                                                .font(FontId::monospace(badge_font_size))
                                                .color(badge_text_col),
                                        );
                                    });

                                    // Symbol Title
                                    let title_color = if is_active {
                                        Color32::WHITE
                                    } else if hov_anim > 0.01 {
                                        Palette::interpolate_color(
                                            Color32::from_gray(190),
                                            Color32::WHITE,
                                            hov_anim,
                                        )
                                    } else {
                                        Color32::from_gray(190)
                                    };

                                    let font = if sym.level == 1 {
                                        FontId::proportional(h1_font_size)
                                    } else {
                                        FontId::proportional(item_font_size)
                                    };

                                    let mut rich_title =
                                        RichText::new(&sym.title).font(font).color(title_color);
                                    if sym.level == 1 {
                                        rich_title = rich_title.strong();
                                    }

                                    let line_text = format!("L{}", sym.line_number);
                                    let line_galley = ui.painter().layout_no_wrap(
                                        line_text.clone(),
                                        FontId::monospace(line_font_size),
                                        Color32::from_gray(130),
                                    );
                                    let line_pill_w = line_galley.size().x + 8.0;

                                    let avail_for_title =
                                        (ui.available_width() - line_pill_w - 4.0).max(20.0);
                                    ui.add_sized(
                                        [avail_for_title, 18.0],
                                        egui::Label::new(rich_title).truncate(),
                                    );

                                    // Line number badge on right
                                    let line_pill = egui::Frame::NONE
                                        .fill(Palette::with_alpha(palette.card, 160))
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::symmetric(3, 1));
                                    line_pill.show(ui, |ui| {
                                        ui.label(
                                            RichText::new(line_text)
                                                .font(FontId::monospace(line_font_size))
                                                .color(Color32::from_gray(130)),
                                        );
                                    });
                                });
                            });

                            let is_hovered = item_resp.response.hovered();
                            ui.ctx().data_mut(|d| d.insert_temp(hov_id, is_hovered));

                            // Paint active section vertical accent bar on left edge (matching user design)
                            if is_active {
                                let rect = item_resp.response.rect;
                                let bar_rect = egui::Rect::from_min_max(
                                    egui::pos2(rect.min.x + 1.0, rect.min.y + 2.0),
                                    egui::pos2(rect.min.x + 3.5, rect.max.y - 2.0),
                                );
                                ui.painter().rect_filled(
                                    bar_rect,
                                    CornerRadius::same(2),
                                    palette.accent,
                                );
                            }

                            let interact = item_resp
                                .response
                                .interact(Sense::click())
                                .on_hover_text(format!(
                                    "Jump to '{}' (Line {})",
                                    sym.title, sym.line_number
                                ));

                            if is_keyboard_sel && arrow_navigated {
                                ui.scroll_to_rect(
                                    item_resp.response.rect,
                                    Some(egui::Align::Center),
                                );
                            }

                            if interact.clicked() {
                                navigation_target = Some((sym.char_offset, sym.line_number));
                            }
                        }
                    });
            }
        });
    });

    navigation_target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markdown_headings() {
        let md = "# Title\nSome text\n## Section 1\nContent\n### Subsection 1.1\nMore content\n```\n# Not a heading\n```\n## Section 2";
        let symbols = extract_document_symbols(md, true, None);

        assert_eq!(symbols.len(), 4);
        assert_eq!(symbols[0].title, "Title");
        assert_eq!(symbols[0].level, 1);
        assert_eq!(symbols[0].kind, SymbolKind::Heading1);

        assert_eq!(symbols[1].title, "Section 1");
        assert_eq!(symbols[1].level, 2);
        assert_eq!(symbols[1].kind, SymbolKind::Heading2);

        assert_eq!(symbols[2].title, "Subsection 1.1");
        assert_eq!(symbols[2].level, 3);
        assert_eq!(symbols[2].kind, SymbolKind::Heading3);

        assert_eq!(symbols[3].title, "Section 2");
        assert_eq!(symbols[3].level, 2);
    }

    #[test]
    fn test_extract_numbered_sections() {
        let doc = "1. Getting Started\nIntro text\n1.1. Installation\nCargo install\n1.2. Hello World\nPrintln\n2. Programming Concepts";
        let symbols = extract_document_symbols(doc, false, None);

        assert_eq!(symbols.len(), 4);
        assert_eq!(symbols[0].title, "1. Getting Started");
        assert_eq!(symbols[0].level, 1);
        assert_eq!(symbols[1].title, "1.1. Installation");
        assert_eq!(symbols[1].level, 2);
        assert_eq!(symbols[2].title, "1.2. Hello World");
        assert_eq!(symbols[2].level, 2);
        assert_eq!(symbols[3].title, "2. Programming Concepts");
        assert_eq!(symbols[3].level, 1);
    }

    #[test]
    fn test_extract_code_symbols() {
        let code = "pub struct MyStruct {\n}\n\npub fn calculate() -> u32 {\n    42\n}\n\nfn private_helper() {\n}";
        let symbols = extract_document_symbols(code, false, Some("main.rs"));

        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].kind, SymbolKind::StructOrClass);
        assert_eq!(symbols[1].kind, SymbolKind::Function);
        assert_eq!(symbols[2].kind, SymbolKind::Function);
    }

    #[test]
    fn test_find_active_symbol_index() {
        let doc = "# Heading 1\nLine 2\nLine 3\n## Heading 2\nLine 5";
        let symbols = extract_document_symbols(doc, true, None);
        assert_eq!(symbols.len(), 2);

        assert_eq!(find_active_symbol_index(&symbols, 0), Some(0));
        assert_eq!(find_active_symbol_index(&symbols, 5), Some(0));
        let h2_offset = symbols[1].char_offset;
        assert_eq!(find_active_symbol_index(&symbols, h2_offset), Some(1));
        assert_eq!(find_active_symbol_index(&symbols, h2_offset + 5), Some(1));
    }
}
