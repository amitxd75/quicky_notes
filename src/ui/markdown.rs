//! Markdown preview renderer using pulldown-cmark, syntect syntax highlighting, egui LayoutJob, and embedded image rendering.

use crate::models::Note;
use crate::theme::Palette;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, Ui, text::LayoutJob,
    text::TextFormat,
};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

/// Markdown preview mode for note editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkdownViewMode {
    #[default]
    Edit,
    Split,
    Preview,
}

impl MarkdownViewMode {
    /// Returns the next view mode in the cycle: Edit -> Split -> Preview -> Edit.
    pub fn next(self) -> Self {
        match self {
            Self::Edit => Self::Split,
            Self::Split => Self::Preview,
            Self::Preview => Self::Edit,
        }
    }

    /// Display name for view mode.
    pub fn label(self) -> &'static str {
        match self {
            Self::Edit => "Edit",
            Self::Split => "Split",
            Self::Preview => "Preview",
        }
    }

    /// Display icon for mode switcher.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Edit => "📝",
            Self::Split => "◫",
            Self::Preview => "👁",
        }
    }

    /// Tooltip label for mode switcher using dynamic shortcut string.
    pub fn tooltip_with_shortcut(self, shortcut: &str) -> String {
        match self {
            Self::Edit => format!("Edit Mode ({})", shortcut),
            Self::Split => format!("Split Mode ({})", shortcut),
            Self::Preview => format!("Preview Mode ({})", shortcut),
        }
    }
}

/// Cached intermediate representation of parsed Markdown blocks.
#[derive(Clone)]
pub enum MarkdownBlock {
    /// Section heading with level, pre-formatted layout job, and source byte range.
    Heading(HeadingLevel, LayoutJob, std::ops::Range<usize>),
    /// Text paragraph layout job with source byte range.
    Paragraph(LayoutJob, std::ops::Range<usize>),
    /// Blockquote layout job with source byte range.
    BlockQuote(LayoutJob, std::ops::Range<usize>),
    /// Code block with language tag, code text, and source byte range.
    CodeBlock {
        lang: String,
        content: String,
        range: std::ops::Range<usize>,
    },
    /// GFM Table with column headers, row cells, and source byte range.
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        range: std::ops::Range<usize>,
    },
    /// Embedded image or diagram with alt text, URL/path, and source byte range.
    Image {
        alt: String,
        url: String,
        range: std::ops::Range<usize>,
    },
    /// Horizontal thematic break divider rule with source byte range.
    Rule(std::ops::Range<usize>),
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct MarkdownCacheKey {
    text_hash_a: u64,
    text_hash_b: u64,
    text_len: usize,
    font_size_bits: u32,
    is_monospace: bool,
    accent_rgba: u32,
}

type MarkdownAstCache = Vec<(MarkdownCacheKey, Vec<MarkdownBlock>)>;

static MARKDOWN_AST_CACHE: OnceLock<Mutex<MarkdownAstCache>> = OnceLock::new();

fn hash_text_pair(text: &str) -> (u64, u64) {
    let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher1);
    let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
    let xored: Vec<u8> = text.as_bytes().iter().map(|&b| b ^ 0x55).collect();
    xored.hash(&mut hasher2);
    (hasher1.finish(), hasher2.finish())
}

/// Parses Markdown text into structured blocks for fast repeated rendering.
fn parse_markdown_blocks(
    text: &str,
    font_size: f32,
    is_monospace: bool,
    palette: &Palette,
) -> Vec<MarkdownBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(text, options).into_offset_iter();

    let base_font = if is_monospace {
        FontId::monospace(font_size)
    } else {
        FontId::proportional(font_size)
    };

    let mut blocks = Vec::new();
    let mut current_job: Option<LayoutJob> = None;
    let mut job_start_offset: usize = 0;
    let mut last_event_end_offset: usize = 0;
    let mut in_heading: Option<HeadingLevel> = None;
    let mut in_blockquote = false;
    let mut in_code_block = false;
    let mut code_block_start: usize = 0;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();
    let mut in_image: Option<(String, usize)> = None;
    let mut image_alt = String::new();
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_strikethrough = false;
    let mut in_link: Option<String> = None;
    let mut list_index: Option<u64> = None;
    let mut in_table = false;
    let mut in_table_head = false;
    let mut table_start_offset = 0;
    let mut table_headers: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_table_row: Vec<String> = Vec::new();
    let mut current_cell_text = String::new();

    let flush_job = |job: &mut Option<LayoutJob>,
                     blocks: &mut Vec<MarkdownBlock>,
                     heading: Option<HeadingLevel>,
                     bq: bool,
                     start_off: usize,
                     end_off: usize| {
        if let Some(j) = job.take()
            && !j.is_empty()
        {
            let safe_end = end_off.max(start_off + 1);
            let range = start_off..safe_end;
            if let Some(level) = heading {
                blocks.push(MarkdownBlock::Heading(level, j, range));
            } else if bq {
                blocks.push(MarkdownBlock::BlockQuote(j, range));
            } else {
                blocks.push(MarkdownBlock::Paragraph(j, range));
            }
        }
    };

    for (event, range) in parser {
        last_event_end_offset = range.end;
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                in_heading = Some(level);
                job_start_offset = range.start;
                current_job = Some(LayoutJob::default());
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.end,
                );
                in_heading = None;
            }
            Event::Start(Tag::Paragraph) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                job_start_offset = range.start;
                current_job = Some(LayoutJob::default());
            }
            Event::End(TagEnd::Paragraph) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.end,
                );
            }
            Event::Start(Tag::BlockQuote(_)) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                in_blockquote = true;
                job_start_offset = range.start;
                current_job = Some(LayoutJob::default());
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.end,
                );
                in_blockquote = false;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                in_code_block = true;
                code_block_start = range.start;
                code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code_block_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                blocks.push(MarkdownBlock::CodeBlock {
                    lang: code_block_lang.clone(),
                    content: code_block_content.trim_end().to_string(),
                    range: code_block_start..range.end,
                });
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                in_image = Some((dest_url.to_string(), range.start));
                image_alt.clear();
            }
            Event::End(TagEnd::Image) => {
                if let Some((url, img_start)) = in_image.take() {
                    blocks.push(MarkdownBlock::Image {
                        alt: image_alt.clone(),
                        url,
                        range: img_start..range.end,
                    });
                }
            }
            Event::Start(Tag::Table(_)) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                in_table = true;
                table_start_offset = range.start;
                table_headers.clear();
                table_rows.clear();
                current_table_row.clear();
                current_cell_text.clear();
            }
            Event::End(TagEnd::Table) => {
                in_table = false;
                blocks.push(MarkdownBlock::Table {
                    headers: std::mem::take(&mut table_headers),
                    rows: std::mem::take(&mut table_rows),
                    range: table_start_offset..range.end,
                });
            }
            Event::Start(Tag::TableHead) => {
                in_table_head = true;
            }
            Event::End(TagEnd::TableHead) => {
                in_table_head = false;
            }
            Event::Start(Tag::TableRow) => {
                current_table_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                if !in_table_head {
                    table_rows.push(std::mem::take(&mut current_table_row));
                }
            }
            Event::Start(Tag::TableCell) => {
                current_cell_text.clear();
            }
            Event::End(TagEnd::TableCell) => {
                let cell_text = std::mem::take(&mut current_cell_text).trim().to_string();
                if in_table_head {
                    table_headers.push(cell_text);
                } else {
                    current_table_row.push(cell_text);
                }
            }
            Event::Start(Tag::List(first_num)) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                list_index = first_num;
            }
            Event::End(TagEnd::List(_)) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.end,
                );
                list_index = None;
            }
            Event::Start(Tag::Item) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                job_start_offset = range.start;
                let mut job = LayoutJob::default();
                if let Some(ref mut idx) = list_index {
                    job.append(
                        &format!("{}. ", idx),
                        0.0,
                        TextFormat {
                            font_id: base_font.clone(),
                            color: palette.accent,
                            ..Default::default()
                        },
                    );
                    *idx += 1;
                } else {
                    job.append(
                        "• ",
                        0.0,
                        TextFormat {
                            font_id: base_font.clone(),
                            color: palette.accent,
                            ..Default::default()
                        },
                    );
                }
                current_job = Some(job);
            }
            Event::End(TagEnd::Item) => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.end,
                );
            }
            Event::Start(Tag::Strong) => {
                is_bold = true;
            }
            Event::End(TagEnd::Strong) => {
                is_bold = false;
            }
            Event::Start(Tag::Emphasis) => {
                is_italic = true;
            }
            Event::End(TagEnd::Emphasis) => {
                is_italic = false;
            }
            Event::Start(Tag::Strikethrough) => {
                is_strikethrough = true;
            }
            Event::End(TagEnd::Strikethrough) => {
                is_strikethrough = false;
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                in_link = Some(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => {
                in_link = None;
            }
            Event::Code(code) => {
                if in_code_block {
                    code_block_content.push_str(&code);
                } else if in_table {
                    current_cell_text.push_str(&format!("`{}`", code));
                } else {
                    let job = current_job.get_or_insert_with(LayoutJob::default);
                    let inline_font = FontId::monospace((font_size - 1.0).max(10.0));
                    job.append(
                        &format!(" `{}` ", code),
                        0.0,
                        TextFormat {
                            font_id: inline_font,
                            color: palette.accent,
                            background: Color32::from_rgba_unmultiplied(
                                palette.card.r(),
                                palette.card.g(),
                                palette.card.b(),
                                200,
                            ),
                            ..Default::default()
                        },
                    );
                }
            }
            Event::Text(t) => {
                if in_code_block {
                    code_block_content.push_str(&t);
                    continue;
                }
                if in_image.is_some() {
                    image_alt.push_str(&t);
                    continue;
                }
                if in_table {
                    current_cell_text.push_str(&t);
                    continue;
                }

                let job = current_job.get_or_insert_with(LayoutJob::default);

                let (font, color) = if let Some(level) = in_heading {
                    let (scale, h_color) = match level {
                        HeadingLevel::H1 => (1.75, palette.accent),
                        HeadingLevel::H2 => (1.45, Color32::WHITE),
                        HeadingLevel::H3 => (1.25, Color32::from_gray(230)),
                        _ => (1.10, Color32::from_gray(210)),
                    };
                    (FontId::proportional(font_size * scale), h_color)
                } else if is_bold {
                    (FontId::proportional(font_size), Color32::WHITE)
                } else if in_link.is_some() {
                    (base_font.clone(), palette.accent)
                } else {
                    (base_font.clone(), Color32::from_gray(230))
                };

                job.append(
                    &t,
                    0.0,
                    TextFormat {
                        font_id: font,
                        color,
                        italics: is_italic || in_blockquote,
                        strikethrough: if is_strikethrough {
                            Stroke::new(1.0, color)
                        } else {
                            Stroke::NONE
                        },
                        underline: if in_link.is_some() {
                            Stroke::new(1.0, palette.accent)
                        } else {
                            Stroke::NONE
                        },
                        ..Default::default()
                    },
                );
            }
            Event::TaskListMarker(checked) => {
                let symbol = if checked { "☑ " } else { "☐ " };
                let job = current_job.get_or_insert_with(LayoutJob::default);
                job.append(
                    symbol,
                    0.0,
                    TextFormat {
                        font_id: base_font.clone(),
                        color: if checked {
                            palette.accent
                        } else {
                            Color32::from_gray(160)
                        },
                        ..Default::default()
                    },
                );
            }
            Event::Rule => {
                flush_job(
                    &mut current_job,
                    &mut blocks,
                    in_heading,
                    in_blockquote,
                    job_start_offset,
                    range.start,
                );
                blocks.push(MarkdownBlock::Rule(range.clone()));
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(job) = current_job.as_mut() {
                    job.append(
                        "\n",
                        0.0,
                        TextFormat {
                            font_id: base_font.clone(),
                            ..Default::default()
                        },
                    );
                }
            }
            _ => {}
        }
    }

    flush_job(
        &mut current_job,
        &mut blocks,
        in_heading,
        in_blockquote,
        job_start_offset,
        last_event_end_offset,
    );
    blocks
}

impl MarkdownBlock {
    /// Returns the character range in the source document corresponding to this block.
    pub fn range(&self) -> std::ops::Range<usize> {
        match self {
            Self::Heading(_, _, r) => r.clone(),
            Self::Paragraph(_, r) => r.clone(),
            Self::BlockQuote(_, r) => r.clone(),
            Self::CodeBlock { range, .. } => range.clone(),
            Self::Table { range, .. } => range.clone(),
            Self::Image { range, .. } => range.clone(),
            Self::Rule(r) => r.clone(),
        }
    }
}

/// Renders CommonMark formatted Markdown content with full syntax-highlighted code blocks, true-color image rendering, and AST caching.
pub fn render_markdown(
    ui: &mut Ui,
    text: &str,
    font_size: f32,
    is_monospace: bool,
    palette: &Palette,
    note: Option<&Note>,
) {
    render_markdown_with_navigation(
        ui,
        text,
        font_size,
        is_monospace,
        palette,
        note,
        &mut None,
        None,
    );
}

/// Renders Markdown content with interactive cursor jump navigation and glowing focus flash animation.
#[allow(clippy::too_many_arguments)]
pub fn render_markdown_with_navigation(
    ui: &mut Ui,
    text: &str,
    font_size: f32,
    is_monospace: bool,
    palette: &Palette,
    note: Option<&Note>,
    jump_to_char: &mut Option<usize>,
    flash_animation: Option<(usize, std::time::Instant)>,
) {
    let (hash_a, hash_b) = hash_text_pair(text);
    let key = MarkdownCacheKey {
        text_hash_a: hash_a,
        text_hash_b: hash_b,
        text_len: text.len(),
        font_size_bits: font_size.to_bits(),
        is_monospace,
        accent_rgba: palette
            .accent
            .to_array()
            .into_iter()
            .fold(0u32, |acc, b| (acc << 8) | b as u32),
    };

    let cache = MARKDOWN_AST_CACHE.get_or_init(|| Mutex::new(Vec::new()));
    let blocks = {
        let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(pos) = guard.iter().position(|(k, _)| k == &key) {
            // LRU hit: move to front
            let entry = guard.remove(pos);
            let blocks = entry.1.clone();
            guard.insert(0, entry);
            blocks
        } else {
            // Cache miss: parse, insert at front, evict oldest if over capacity
            let parsed = parse_markdown_blocks(text, font_size, is_monospace, palette);
            guard.insert(0, (key, parsed.clone()));
            if guard.len() > 32 {
                guard.truncate(32);
            }
            parsed
        }
    };

    ui.spacing_mut().item_spacing.y = 8.0;

    let mut code_block_idx: usize = 0;
    let code_theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

    for (block_idx, block) in blocks.iter().enumerate() {
        let (resp, range) = match block {
            MarkdownBlock::Heading(level, job, r) => {
                let mut cloned_job = job.clone();
                cloned_job.wrap.max_width = ui.available_width();
                let space = match level {
                    HeadingLevel::H1 => 6.0,
                    HeadingLevel::H2 => 4.0,
                    _ => 2.0,
                };
                ui.add_space(space);
                let resp = ui.label(cloned_job);
                ui.add_space(2.0);
                (resp, r.clone())
            }
            MarkdownBlock::Paragraph(job, r) => {
                let mut cloned_job = job.clone();
                cloned_job.wrap.max_width = ui.available_width();
                let resp = ui.label(cloned_job);
                (resp, r.clone())
            }
            MarkdownBlock::BlockQuote(job, r) => {
                let mut cloned_job = job.clone();
                cloned_job.wrap.max_width = (ui.available_width() - 16.0).max(50.0);
                let resp = ui
                    .horizontal(|ui| {
                        let (_, rect) = ui.allocate_space(egui::vec2(3.0, 18.0));
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(2), palette.accent);
                        ui.add_space(8.0);
                        ui.label(cloned_job)
                    })
                    .response;
                (resp, r.clone())
            }
            MarkdownBlock::Image { alt, url, range } => {
                render_markdown_image(ui, alt, url, palette, note, font_size);
                let dummy_resp = ui.allocate_response(egui::vec2(1.0, 1.0), egui::Sense::hover());
                (dummy_resp, range.clone())
            }
            MarkdownBlock::Table {
                headers,
                rows,
                range,
            } => {
                let table_frame = egui::Frame::NONE
                    .fill(Palette::with_alpha(palette.card, 160))
                    .stroke(Stroke::new(1.0, Palette::with_alpha(palette.border, 90)))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(12, 8));

                let frame_resp = table_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    egui::ScrollArea::horizontal()
                        .id_salt(format!("table_scroll_{}", block_idx))
                        .show(ui, |ui| {
                            egui::Grid::new(format!("markdown_table_{}", block_idx))
                                .striped(true)
                                .spacing(egui::vec2(20.0, 8.0))
                                .show(ui, |ui| {
                                    if !headers.is_empty() {
                                        for h in headers {
                                            ui.label(
                                                RichText::new(h)
                                                    .font(FontId::proportional(
                                                        (font_size - 0.5).max(11.0),
                                                    ))
                                                    .strong()
                                                    .color(palette.accent),
                                            );
                                        }
                                        ui.end_row();
                                    }

                                    for row in rows {
                                        for cell in row {
                                            ui.label(
                                                RichText::new(cell)
                                                    .font(FontId::proportional(
                                                        (font_size - 1.0).max(10.5),
                                                    ))
                                                    .color(Color32::from_gray(225)),
                                            );
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                });
                ui.add_space(4.0);
                (frame_resp.response, range.clone())
            }
            MarkdownBlock::CodeBlock {
                lang,
                content,
                range,
            } => {
                let clean_lang = lang.trim();
                let lang_id = crate::ui::syntax::normalize_language(clean_lang);
                let display_lang = if clean_lang.is_empty() {
                    "CODE"
                } else {
                    clean_lang
                };
                let code_font = FontId::monospace((font_size - 1.5).max(10.0));

                let frame_resp = egui::Frame::NONE
                    .fill(Palette::with_alpha(palette.card, 210))
                    .stroke(Stroke::new(1.0, Palette::with_alpha(palette.border, 110)))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let lang_pill = egui::Frame::NONE
                                .fill(Palette::with_alpha(palette.accent, 35))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(5, 1));
                            lang_pill.show(ui, |ui| {
                                ui.label(
                                    RichText::new(display_lang.to_ascii_uppercase())
                                        .font(FontId::monospace(10.0))
                                        .color(palette.accent)
                                        .strong(),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let copy_btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("📋 Copy")
                                                .font(FontId::proportional(11.0))
                                                .color(Color32::from_gray(190)),
                                        )
                                        .fill(Palette::with_alpha(palette.card, 190))
                                        .corner_radius(CornerRadius::same(4)),
                                    );
                                    if copy_btn.on_hover_text("Copy code to clipboard").clicked() {
                                        ui.ctx().copy_text(content.clone());
                                    }
                                },
                            );
                        });

                        ui.add_space(4.0);
                        egui::ScrollArea::horizontal()
                            .id_salt(format!("code_block_hscroll_{}", code_block_idx))
                            .show(ui, |ui| {
                                let opts = crate::ui::syntax::HighlightOptions {
                                    theme: &code_theme,
                                    language: lang_id,
                                    font_id: code_font,
                                    text_color: Color32::from_gray(235),
                                    wrap_width: f32::INFINITY,
                                };
                                let layout_job = crate::ui::syntax::highlight_text(
                                    ui.ctx(),
                                    ui.style(),
                                    content,
                                    opts,
                                );
                                ui.label(layout_job);
                            });
                    });
                ui.add_space(4.0);
                code_block_idx += 1;
                (frame_resp.response, range.clone())
            }
            MarkdownBlock::Rule(range) => {
                ui.add_space(4.0);
                let (rect, resp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 1.0),
                    egui::Sense::hover(),
                );
                ui.painter().line_segment(
                    [
                        egui::pos2(rect.min.x, rect.center().y),
                        egui::pos2(rect.max.x, rect.center().y),
                    ],
                    Stroke::new(1.0, Palette::with_alpha(palette.border, 90)),
                );
                ui.add_space(4.0);
                (resp, range.clone())
            }
        };

        // Scroll to target section if requested
        if let Some(target) = *jump_to_char {
            let is_match = (range.start..=range.end).contains(&target)
                || (block_idx == 0 && target <= range.start)
                || (target >= range.start
                    && (block_idx + 1 == blocks.len()
                        || target < blocks[block_idx + 1].range().start));
            if is_match {
                ui.scroll_to_rect(resp.rect, Some(egui::Align::Center));
                *jump_to_char = None;
            }
        }

        // Draw animated focus glow if navigation flash is active
        if let Some((flash_target, start_time)) = flash_animation {
            let elapsed = start_time.elapsed().as_secs_f32();
            if elapsed < 1.2 {
                let is_flash_match = (range.start..=range.end).contains(&flash_target)
                    || (flash_target >= range.start
                        && (block_idx + 1 == blocks.len()
                            || flash_target < blocks[block_idx + 1].range().start));
                if is_flash_match {
                    let progress = (elapsed / 1.2).clamp(0.0, 1.0);
                    let alpha = (1.0 - progress).powi(2);
                    let glow_rect = resp.rect.expand(6.0);
                    ui.painter().rect_filled(
                        glow_rect,
                        CornerRadius::same(8),
                        Palette::with_alpha(palette.accent, (55.0 * alpha) as u8),
                    );
                    ui.painter().rect_stroke(
                        glow_rect,
                        CornerRadius::same(8),
                        Stroke::new(
                            1.5,
                            Palette::with_alpha(palette.accent, (190.0 * alpha) as u8),
                        ),
                        egui::StrokeKind::Outside,
                    );
                    ui.ctx().request_repaint();
                }
            }
        }
    }
}

/// Helper to render an image block inside Markdown with true colors and responsive font scaling.
fn render_markdown_image(
    ui: &mut Ui,
    alt: &str,
    url: &str,
    palette: &Palette,
    note: Option<&Note>,
    font_size: f32,
) {
    let clean_url = url.trim();
    let decoded_url = crate::ui::drag_drop::url_decode(clean_url);

    let att_id_opt = if clean_url.starts_with("attachment:") {
        Some(clean_url.trim_start_matches("attachment:").trim())
    } else if clean_url.starts_with("qn://") {
        Some(clean_url.trim_start_matches("qn://").trim())
    } else {
        None
    };

    // 1. Try finding in note attachments (checking raw ID, decoded ID, raw name, decoded name)
    if let Some(n) = note {
        let attachment = if let Some(att_id) = att_id_opt {
            let decoded_id = crate::ui::drag_drop::url_decode(att_id);
            n.get_attachment(att_id)
                .or_else(|| n.get_attachment(&decoded_id))
                .or_else(|| n.get_attachment_by_name_or_id(att_id))
                .or_else(|| n.get_attachment_by_name_or_id(&decoded_id))
        } else {
            n.get_attachment_by_name_or_id(clean_url)
                .or_else(|| n.get_attachment_by_name_or_id(&decoded_url))
        };

        if let Some(att) = attachment
            && let Some(tex) =
                crate::ui::image_view::get_or_load_attachment_texture(ui.ctx(), &n.id, att)
        {
            ui.vertical(|ui| {
                crate::ui::image_view::render_true_color_image(
                    ui,
                    &tex,
                    ui.available_width(),
                    6,
                    font_size,
                );

                if !alt.is_empty() {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(alt)
                            .font(FontId::proportional((font_size - 3.0).max(9.0)))
                            .italics()
                            .color(Color32::from_gray(160)),
                    );
                }
            });
            return;
        }
    }

    // 2. Try loading as a local file on disk (absolute, file:// URI, or relative to linked note file path)
    let candidate_path = if let Some(stripped) = clean_url.strip_prefix("file://") {
        Some(std::path::PathBuf::from(crate::ui::drag_drop::url_decode(
            stripped,
        )))
    } else if std::path::Path::new(clean_url).is_absolute() {
        Some(std::path::PathBuf::from(clean_url))
    } else if std::path::Path::new(&decoded_url).is_absolute() {
        Some(std::path::PathBuf::from(&decoded_url))
    } else if let Some(n) = note
        && let Some(ref fp) = n.file_path
    {
        let base_dir = std::path::Path::new(fp).parent();
        base_dir.map(|d| d.join(&decoded_url))
    } else {
        None
    };

    if let Some(path) = candidate_path
        && path.exists()
        && path.is_file()
        && let Some(tex) = crate::ui::image_view::get_or_load_file_texture(ui.ctx(), &path)
    {
        ui.vertical(|ui| {
            crate::ui::image_view::render_true_color_image(
                ui,
                &tex,
                ui.available_width(),
                6,
                font_size,
            );

            if !alt.is_empty() {
                ui.add_space(2.0);
                ui.label(
                    RichText::new(alt)
                        .font(FontId::proportional((font_size - 3.0).max(9.0)))
                        .italics()
                        .color(Color32::from_gray(160)),
                );
            }
        });
        return;
    }

    // 3. Fallback placeholder for missing or unresolvable images
    egui::Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(
            palette.card.r(),
            palette.card.g(),
            palette.card.b(),
            120,
        ))
        .stroke(Stroke::new(
            1.0_f32,
            Color32::from_rgba_unmultiplied(
                palette.border.r(),
                palette.border.g(),
                palette.border.b(),
                100,
            ),
        ))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🖼")
                        .font(FontId::proportional(14.0))
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new(if alt.is_empty() { clean_url } else { alt })
                        .font(FontId::proportional(11.5))
                        .color(Color32::from_gray(190)),
                );
            });
        });
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_blocks_code_highlighting() {
        let md = "```rust\nfn main() {\n    let x = 42;\n}\n```";
        let palette = Palette::new(
            Color32::BLACK,
            Color32::BLACK,
            Color32::DARK_GRAY,
            Color32::WHITE,
        );
        let blocks = parse_markdown_blocks(md, 14.0, false, &palette);
        assert_eq!(blocks.len(), 1);
        if let MarkdownBlock::CodeBlock { lang, content, .. } = &blocks[0] {
            assert_eq!(lang, "rust");
            assert!(content.contains("let x = 42;"));
            assert_eq!(crate::ui::syntax::normalize_language(lang), "rs");
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn test_parse_markdown_blocks_table() {
        let md = "| Feature | Status |\n| :--- | :--- |\n| Outline | Done |\n| Preview | Fast |";
        let palette = Palette::new(
            Color32::BLACK,
            Color32::BLACK,
            Color32::DARK_GRAY,
            Color32::WHITE,
        );
        let blocks = parse_markdown_blocks(md, 14.0, false, &palette);
        assert_eq!(blocks.len(), 1);
        if let MarkdownBlock::Table { headers, rows, .. } = &blocks[0] {
            assert_eq!(headers, &["Feature", "Status"]);
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0], vec!["Outline", "Done"]);
            assert_eq!(rows[1], vec!["Preview", "Fast"]);
        } else {
            panic!("Expected Table block");
        }
    }
}
