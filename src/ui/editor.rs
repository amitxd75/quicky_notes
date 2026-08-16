//! Main editor workspace, line numbers gutter, status bar, and modal overlays.

use crate::app::QuickyNotesApp;
use crate::components::{card, modal};
use crate::theme::{self, ACCENT_EMERALD, ACCENT_PURPLE};
use crate::ui;
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, Ui};
use std::fmt::Write as _;

/// Renders line numbers gutter in small culling-friendly chunks to support arbitrarily large files without vertex limits.
pub fn render_line_numbers_gutter(ui: &mut Ui, line_count: usize, line_font: &FontId) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let chunk_size = 100;
        let mut buffer = String::with_capacity(chunk_size * 6);

        for start in (1..=line_count).step_by(chunk_size) {
            let end = (start + chunk_size - 1).min(line_count);
            buffer.clear();
            for line in start..=end {
                let _ = writeln!(buffer, "{}", line);
            }
            if buffer.ends_with('\n') {
                buffer.pop();
            }
            ui.label(
                RichText::new(&buffer)
                    .font(line_font.clone())
                    .color(Color32::from_gray(100)),
            );
        }
    });
}

/// Renders the main glass editor container, header, drawers/editor, and status bar.
pub fn render_main_editor(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    card::glass_editor_frame(&app.data.settings).show(ui, |ui| {
        ui.vertical(|ui| {
            // 1. Sleek Header Bar
            ui::header::render_header(app, ctx, ui);

            // Divider below header
            ui::draw_horizontal_divider(ui);

            let status_bar_height = if app.data.settings.show_status_bar && !app.show_options {
                32.0
            } else {
                0.0
            };
            let body_height = (ui.available_height() - status_bar_height - 6.0).max(60.0);

            // 2. Body Area (Options Drawer vs Search Drawer vs Editor)
            egui::Frame::NONE
                .inner_margin(Margin::symmetric(14, 4))
                .show(ui, |ui| {
                    ui.set_height(body_height);
                    if app.show_options {
                        ui::options_drawer::render_options_drawer(app, ctx, ui);
                    } else if app.show_search {
                        ui::search_drawer::render_search_drawer(app, ctx, ui);
                    } else {
                        render_editor_workspace(app, ctx, ui);
                    }
                });

            // 3. Status Bar (Pinned to bottom when options drawer is not open and status bar is enabled)
            if !app.show_options && app.data.settings.show_status_bar {
                ui::draw_horizontal_divider(ui);
                render_status_bar(app, ui);
            }
        });
    });

    // Confirmation Modal for Tab Deletion
    render_close_confirmation_modal(app, ctx);

    // Floating Toast Notification Overlay
    render_floating_toast(app, ctx);
}

/// Renders the main note editing area with line numbers and preview toggling.
fn render_editor_workspace(app: &mut QuickyNotesApp, _ctx: &egui::Context, ui: &mut Ui) {
    let font_size = app.data.settings.font_size;
    let is_monospace = app.data.settings.monospace_font;
    let preview_mode = app.preview_mode;
    let should_focus = app.focus_editor;
    app.focus_editor = false;

    let palette = theme::get_palette(&app.data.settings);
    let mut content_changed = false;

    if let Some(active_id) = app.data.active_note_id.as_deref()
        && let Some(note) = app.data.notes.iter_mut().find(|n| n.id == active_id)
    {
        let line_count = app.cached_active_stats.2;
        let line_font = if is_monospace {
            FontId::monospace(font_size)
        } else {
            FontId::proportional(font_size)
        };

        let editor_height = ui.available_height();
        let effective_mode = if note.is_markdown() {
            preview_mode
        } else {
            crate::app::MarkdownViewMode::Edit
        };

        match effective_mode {
            crate::app::MarkdownViewMode::Edit => {
                egui::ScrollArea::vertical()
                    .id_salt("editor_scroll_area")
                    .auto_shrink([false, false])
                    .max_height(editor_height)
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            if app.data.settings.show_line_numbers {
                                // Line numbers gutter (chunked for unlimited line counts)
                                render_line_numbers_gutter(ui, line_count, &line_font);

                                ui.add_space(8.0);

                                // Vertical line separator
                                let (_, rect) =
                                    ui.allocate_space(egui::vec2(1.0, editor_height.max(300.0)));
                                ui.painter().line_segment(
                                    [rect.min, egui::pos2(rect.min.x, rect.max.y)],
                                    Stroke::new(
                                        1.0_f32,
                                        Color32::from_rgba_unmultiplied(80, 50, 110, 60),
                                    ),
                                );

                                ui.add_space(8.0);
                            }

                            // Multiline text editor
                            let font = if is_monospace {
                                FontId::monospace(font_size)
                            } else {
                                FontId::proportional(font_size)
                            };

                            let text_edit = egui::TextEdit::multiline(&mut note.content)
                                .font(font)
                                .text_color(Color32::WHITE)
                                .frame(egui::Frame::NONE)
                                .hint_text("Type your notes here...")
                                .desired_width(ui.available_width());

                            let resp = ui.add(text_edit);

                            if resp.changed() {
                                content_changed = true;
                            }

                            if should_focus {
                                resp.request_focus();
                            }
                        });
                    });
            }
            crate::app::MarkdownViewMode::Preview => {
                egui::ScrollArea::vertical()
                    .id_salt("preview_scroll_area")
                    .auto_shrink([false, false])
                    .max_height(editor_height)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        if note.content.trim().is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(
                                    RichText::new("Markdown preview is empty.")
                                        .font(FontId::proportional(14.0))
                                        .color(Color32::from_gray(120)),
                                );
                            });
                        } else {
                            crate::ui::markdown::render_markdown(
                                ui,
                                &note.content,
                                font_size,
                                is_monospace,
                                &palette,
                            );
                        }
                    });
            }
            crate::app::MarkdownViewMode::Split => {
                let half_width = (ui.available_width() - 16.0) * 0.5;

                ui.horizontal(|ui| {
                    // Left column: Editor
                    ui.vertical(|ui| {
                        ui.set_width(half_width);
                        ui.set_height(editor_height);
                        egui::ScrollArea::vertical()
                            .id_salt("split_editor_scroll_area")
                            .auto_shrink([false, false])
                            .max_height(editor_height)
                            .show(ui, |ui| {
                                ui.horizontal_top(|ui| {
                                    if app.data.settings.show_line_numbers {
                                        render_line_numbers_gutter(ui, line_count, &line_font);
                                        ui.add_space(6.0);
                                    }

                                    let font = if is_monospace {
                                        FontId::monospace(font_size)
                                    } else {
                                        FontId::proportional(font_size)
                                    };

                                    let text_edit = egui::TextEdit::multiline(&mut note.content)
                                        .font(font)
                                        .text_color(Color32::WHITE)
                                        .frame(egui::Frame::NONE)
                                        .hint_text("Type markdown here...")
                                        .desired_width(ui.available_width());

                                    let resp = ui.add(text_edit);

                                    if resp.changed() {
                                        content_changed = true;
                                    }

                                    if should_focus {
                                        resp.request_focus();
                                    }
                                });
                            });
                    });

                    // Middle vertical divider
                    let (_, rect) = ui.allocate_space(egui::vec2(1.0, editor_height));
                    ui.painter().line_segment(
                        [rect.min, egui::pos2(rect.min.x, rect.max.y)],
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(80, 50, 110, 80)),
                    );

                    ui.add_space(8.0);

                    // Right column: Live Markdown Preview
                    ui.vertical(|ui| {
                        ui.set_width(half_width - 8.0);
                        ui.set_height(editor_height);
                        egui::ScrollArea::vertical()
                            .id_salt("split_preview_scroll_area")
                            .auto_shrink([false, false])
                            .max_height(editor_height)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width());
                                    crate::ui::markdown::render_markdown(
                                        ui,
                                        &note.content,
                                        font_size,
                                        is_monospace,
                                        &palette,
                                    );
                                });
                            });
                    });
                });
            }
        }

        if content_changed {
            note.update_timestamp();
            app.is_dirty = true;
        }
    }
}

/// Renders the bottom status bar with clean, unboxed typography, subtle dots, and generous padding.
fn render_status_bar(app: &QuickyNotesApp, ui: &mut Ui) {
    let palette = theme::get_palette(&app.data.settings);

    egui::Frame::NONE
        .inner_margin(Margin {
            left: 14,
            right: 14,
            top: 5,
            bottom: 6,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // 1. Sync State / Save status dot & label
                if app.is_dirty {
                    ui.label(
                        RichText::new("●")
                            .font(FontId::proportional(11.0))
                            .color(Color32::from_rgb(251, 191, 36)),
                    );
                    ui.label(
                        RichText::new("Unsaved")
                            .font(FontId::proportional(11.5))
                            .color(Color32::from_rgb(251, 191, 36)),
                    );
                } else {
                    ui.label(
                        RichText::new("●")
                            .font(FontId::proportional(11.0))
                            .color(ACCENT_EMERALD),
                    );
                    ui.label(
                        RichText::new("Saved")
                            .font(FontId::proportional(11.5))
                            .color(Color32::from_gray(210)),
                    );
                }

                                ui.label(
                    RichText::new("|")
                        .font(FontId::proportional(11.0))
                        .color(Color32::from_gray(90)),
                );

                // 2. Linked File indicator or Mode
                if let Some(note) = app.active_note() {
                    if let Some(ref path_str) = note.file_path {
                        let is_status_active = app
                            .status_msg
                            .as_ref()
                            .is_some_and(|(_, created)| created.elapsed().as_secs_f32() < 3.5);
                        let max_path_len = if is_status_active { 22 } else { 38 };
                        let display_path =
                            crate::storage::format_display_path(path_str, max_path_len);
                        let path_resp = ui
                            .horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.label(
                                    RichText::new("🔗")
                                        .font(FontId::proportional(11.5))
                                        .color(Color32::WHITE),
                                );
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(display_path)
                                            .font(FontId::proportional(11.5))
                                            .color(palette.accent),
                                    )
                                    .sense(egui::Sense::click())
                                    .truncate(),
                                )
                            })
                            .inner;

                        let tooltip = format!(
                            "📁 Directly linked file on disk:\n{}\n\n• Click to copy full path\n• Right-click to reveal in file manager",
                            path_str
                        );
                        let path_resp = path_resp.on_hover_text(tooltip);

                        if path_resp.clicked() {
                            ui.ctx().copy_text(path_str.clone());
                        }

                        path_resp.context_menu(|ui| {
                            if ui.button("📋 Copy Full Path").clicked() {
                                ui.ctx().copy_text(path_str.clone());
                                ui.close();
                            }
                            if ui.button("📂 Reveal Folder in File Manager").clicked() {
                                if let Some(parent) = std::path::Path::new(path_str).parent() {
                                    crate::ui::drag_drop::safe_open_folder(parent);
                                }
                                ui.close();
                            }
                        });
                    } else if note.is_markdown() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.label(
                                RichText::new("📝")
                                    .font(FontId::proportional(11.5))
                                    .color(Color32::WHITE),
                            );
                            ui.label(
                                RichText::new("Markdown")
                                    .font(FontId::proportional(11.5))
                                    .color(Color32::from_gray(190)),
                            );
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.label(
                                RichText::new("📄")
                                    .font(FontId::proportional(11.5))
                                    .color(Color32::WHITE),
                            );
                            ui.label(
                                RichText::new("Plain Text")
                                    .font(FontId::proportional(11.5))
                                    .color(Color32::from_gray(190)),
                            );
                        });
                    }
                }

                // 3. Transient Status Message Notification with Smooth Fade Animation
                if let Some((msg, created)) = &app.status_msg {
                    let elapsed = created.elapsed().as_secs_f32();
                    if elapsed < 3.5 {
                        let fade = if elapsed < 0.2 {
                            (elapsed / 0.2).clamp(0.0, 1.0)
                        } else if elapsed > 2.8 {
                            ((3.5 - elapsed) / 0.7).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };
                        let alpha = (255.0 * fade) as u8;
                        if alpha > 10 {
                            ui.label(
                                RichText::new("|")
                                    .font(FontId::proportional(11.0))
                                    .color(Color32::from_rgba_unmultiplied(90, 90, 90, alpha)),
                            );
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.label(
                                    RichText::new("⚡")
                                        .font(FontId::proportional(11.5))
                                        .color(Color32::from_rgba_unmultiplied(255, 255, 255, alpha)),
                                );
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(msg)
                                            .font(FontId::proportional(11.5))
                                            .strong()
                                            .color(Color32::from_rgba_unmultiplied(
                                                palette.accent.r(),
                                                palette.accent.g(),
                                                palette.accent.b(),
                                                alpha,
                                            )),
                                    )
                                    .truncate(),
                                );
                            });
                        }
                    }
                }

                // --- Right Side Stats ---
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;

                    // UTF-8
                    ui.label(
                        RichText::new("UTF-8")
                            .font(FontId::monospace(11.0))
                            .color(Color32::from_gray(150)),
                    );

                    // Tab size
                    ui.label(
                        RichText::new(format!("Tab: {} sp", app.data.settings.tab_size))
                            .font(FontId::proportional(11.5))
                            .color(Color32::from_gray(170)),
                    );

                    // Word / Char statistics
                    {
                        let (words, chars, lines) = app.cached_active_stats;

                        ui.label(
                            RichText::new(format!(
                                "Lines: {}  •  Words: {}  •  Chars: {}",
                                lines, words, chars
                            ))
                            .font(FontId::proportional(11.5))
                            .color(Color32::from_gray(190)),
                        );
                    }
                });
            });
        });
}

/// Renders the close confirmation modal dialog if requested.
pub fn render_close_confirmation_modal(app: &mut QuickyNotesApp, ctx: &egui::Context) {
    let Some(close_id) = app.confirm_close_id.clone() else {
        return;
    };

    let note_title = app
        .data
        .notes
        .iter()
        .find(|n| n.id == close_id)
        .map(|n| {
            if n.title.trim().is_empty() {
                "untitled.txt".to_string()
            } else {
                n.title.clone()
            }
        })
        .unwrap_or_else(|| "note".to_string());

    let palette = theme::get_palette(&app.data.settings);
    let settings = app.data.settings.clone();

    modal::modal_overlay(
        ctx,
        "confirm_close_modal",
        &settings,
        egui::vec2(380.0, 150.0),
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 12.0;

                // Header with warning icon and title description
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    ui.label(
                        RichText::new("⚠️")
                            .font(FontId::proportional(22.0))
                            .color(Color32::WHITE),
                    );
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;
                        ui.label(
                            RichText::new("Close Note Tab?")
                                .font(FontId::proportional(15.0))
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.label(
                            RichText::new(format!(
                                "Are you sure you want to close '{}'?",
                                note_title
                            ))
                            .font(FontId::proportional(12.5))
                            .color(Color32::from_gray(190)),
                        );
                    });
                });

                ui.add_space(4.0);

                // Right-aligned action buttons
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;

                        let confirm_btn = crate::components::button::animated_danger_button(
                            ui,
                            "🗑  Close Note",
                            egui::vec2(105.0, 32.0),
                        );

                        if confirm_btn.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            app.close_note(&close_id);
                            app.confirm_close_id = None;
                            ctx.request_repaint();
                        }

                        let cancel_btn = crate::components::button::animated_action_button(
                            ui,
                            "Cancel (Esc)",
                            &palette,
                            egui::vec2(95.0, 32.0),
                        );

                        if cancel_btn.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            app.confirm_close_id = None;
                            ctx.request_repaint();
                        }
                    });
                });
            });
        },
    );
}

/// Renders drag-and-drop hover overlay when files are hovering over the window.
pub fn render_drop_hover_overlay(ctx: &egui::Context) {
    if ctx.input(|i| i.raw.hovered_files.is_empty()) {
        return;
    }

    egui::Area::new(egui::Id::new("drop_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::Pos2::ZERO)
        .show(ctx, |ui| {
            let rect = ui.clip_rect();
            ui.allocate_rect(rect, egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(12),
                Color32::from_rgba_unmultiplied(18, 12, 28, 220),
            );
            ui.painter().rect_stroke(
                rect.shrink(8.0),
                CornerRadius::same(10),
                Stroke::new(2.0_f32, ACCENT_PURPLE),
                egui::StrokeKind::Outside,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "📥 Drop text file to open as a new tab",
                FontId::proportional(18.0),
                Color32::WHITE,
            );
        });
}

pub use crate::ui::toast::render_floating_toast;
