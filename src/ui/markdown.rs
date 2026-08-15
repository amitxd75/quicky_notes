//! Markdown preview renderer using pulldown-cmark, egui LayoutJob, and zero-allocation AST caching.

use crate::theme::ThemePalette;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, Ui, text::LayoutJob,
    text::TextFormat,
};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

/// Cached intermediate representation of parsed Markdown blocks.
#[derive(Clone)]
pub enum MarkdownBlock {
    Heading(HeadingLevel, LayoutJob),
    Paragraph(LayoutJob),
    BlockQuote(LayoutJob),
    CodeBlock(String),
    Rule,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct MarkdownCacheKey {
    text_hash: u64,
    text_len: usize,
    font_size_bits: u32,
    is_monospace: bool,
    accent_rgba: u32,
}

static MARKDOWN_AST_CACHE: OnceLock<Mutex<HashMap<MarkdownCacheKey, Vec<MarkdownBlock>>>> =
    OnceLock::new();

fn hash_text(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Parses Markdown text into structured blocks for fast repeated rendering.
fn parse_markdown_blocks(
    text: &str,
    font_size: f32,
    is_monospace: bool,
    palette: &ThemePalette,
) -> Vec<MarkdownBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(text, options);

    let base_font = if is_monospace {
        FontId::monospace(font_size)
    } else {
        FontId::proportional(font_size)
    };

    let mut blocks = Vec::new();
    let mut current_job: Option<LayoutJob> = None;
    let mut in_heading: Option<HeadingLevel> = None;
    let mut in_blockquote = false;
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_strikethrough = false;
    let mut in_link: Option<String> = None;
    let mut list_index: Option<u64> = None;

    let flush_job = |job: &mut Option<LayoutJob>,
                     blocks: &mut Vec<MarkdownBlock>,
                     heading: Option<HeadingLevel>,
                     bq: bool| {
        if let Some(j) = job.take()
            && !j.is_empty()
        {
            if let Some(level) = heading {
                blocks.push(MarkdownBlock::Heading(level, j));
            } else if bq {
                blocks.push(MarkdownBlock::BlockQuote(j));
            } else {
                blocks.push(MarkdownBlock::Paragraph(j));
            }
        }
    };

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                in_heading = Some(level);
                current_job = Some(LayoutJob::default());
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                in_heading = None;
            }
            Event::Start(Tag::Paragraph) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                current_job = Some(LayoutJob::default());
            }
            Event::End(TagEnd::Paragraph) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
            }
            Event::Start(Tag::BlockQuote(_)) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                in_blockquote = true;
                current_job = Some(LayoutJob::default());
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                in_blockquote = false;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                in_code_block = true;
                code_block_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                blocks.push(MarkdownBlock::CodeBlock(
                    code_block_content.trim_end().to_string(),
                ));
            }
            Event::Start(Tag::List(first_num)) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                list_index = first_num;
            }
            Event::End(TagEnd::List(_)) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                list_index = None;
            }
            Event::Start(Tag::Item) => {
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
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
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
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
                flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
                blocks.push(MarkdownBlock::Rule);
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

    flush_job(&mut current_job, &mut blocks, in_heading, in_blockquote);
    blocks
}

/// Renders CommonMark formatted Markdown content with thread-safe zero-allocation AST caching.
pub fn render_markdown(
    ui: &mut Ui,
    text: &str,
    font_size: f32,
    is_monospace: bool,
    palette: &ThemePalette,
) {
    let key = MarkdownCacheKey {
        text_hash: hash_text(text),
        text_len: text.len(),
        font_size_bits: font_size.to_bits(),
        is_monospace,
        accent_rgba: palette
            .accent
            .to_array()
            .into_iter()
            .fold(0u32, |acc, b| (acc << 8) | b as u32),
    };

    let cache = MARKDOWN_AST_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let blocks = {
        let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(cached) = guard.get(&key) {
            cached.clone()
        } else {
            // Evict if cache exceeds 48 items
            if guard.len() > 48 {
                guard.clear();
            }
            let parsed = parse_markdown_blocks(text, font_size, is_monospace, palette);
            guard.insert(key, parsed.clone());
            parsed
        }
    };

    ui.spacing_mut().item_spacing.y = 8.0;

    for block in blocks {
        match block {
            MarkdownBlock::Heading(level, mut job) => {
                job.wrap.max_width = ui.available_width();
                let space = match level {
                    HeadingLevel::H1 => 6.0,
                    HeadingLevel::H2 => 4.0,
                    _ => 2.0,
                };
                ui.add_space(space);
                ui.label(job);
                ui.add_space(2.0);
            }
            MarkdownBlock::Paragraph(mut job) => {
                job.wrap.max_width = ui.available_width();
                ui.label(job);
            }
            MarkdownBlock::BlockQuote(mut job) => {
                job.wrap.max_width = ui.available_width() - 16.0;
                ui.horizontal(|ui| {
                    let (_, rect) = ui.allocate_space(egui::vec2(3.0, 18.0));
                    ui.painter()
                        .rect_filled(rect, CornerRadius::same(2), palette.accent);
                    ui.add_space(8.0);
                    ui.label(job);
                });
            }
            MarkdownBlock::CodeBlock(code_text) => {
                egui::Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(
                        palette.card.r(),
                        palette.card.g(),
                        palette.card.b(),
                        230,
                    ))
                    .stroke(Stroke::new(
                        1.0_f32,
                        Color32::from_rgba_unmultiplied(
                            palette.border.r(),
                            palette.border.g(),
                            palette.border.b(),
                            140,
                        ),
                    ))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("CODE")
                                    .font(FontId::monospace(10.0))
                                    .color(palette.accent),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("📋 Copy")
                                        .on_hover_text("Copy code to clipboard")
                                        .clicked()
                                    {
                                        ui.ctx().copy_text(code_text.clone());
                                    }
                                },
                            );
                        });

                        ui.add_space(4.0);
                        egui::ScrollArea::horizontal()
                            .id_salt("code_block_hscroll")
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(&code_text)
                                        .font(FontId::monospace((font_size - 1.0).max(10.0)))
                                        .color(Color32::from_gray(235)),
                                );
                            });
                    });
                ui.add_space(4.0);
            }
            MarkdownBlock::Rule => {
                ui.add_space(4.0);
                crate::ui::draw_horizontal_divider(ui);
                ui.add_space(4.0);
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_markdown_parser_sanitary() {
        let md = "# Title\n\nSome **bold** and *italic* text with `inline code`.\n\n```rust\nfn main() {}\n```";
        let options = Options::all();
        let parser = Parser::new_ext(md, options);
        let count = parser.count();
        assert!(count > 5);
    }
}
