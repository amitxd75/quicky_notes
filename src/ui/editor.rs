//! Main editor workspace, line numbers gutter, status bar, and modal overlays.

use crate::app::QuickyNotesApp;
use crate::components::{card, modal};
use crate::theme::{self, ACCENT_EMERALD, ACCENT_PURPLE};
use crate::ui;
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, Ui};
use std::fmt::Write as _;

/// Dedicated reserved height for the bottom status bar in pixels.
pub const STATUS_BAR_HEIGHT: f32 = 32.0;

/// Transient status notification message duration in seconds.
pub const STATUS_MSG_DURATION_SECS: f32 = 3.5;

/// Chunk size for virtualized line numbers gutter rendering.
pub const GUTTER_CHUNK_SIZE: usize = 100;

/// Minimum vertical height for multiline text editor canvas in pixels.
pub const MIN_EDITOR_HEIGHT: f32 = 300.0;

/// Renders line numbers gutter in small culling-friendly chunks to support arbitrarily large files without vertex limits.
pub fn render_line_numbers_gutter(ui: &mut Ui, line_count: usize, line_font: &FontId) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut buffer = String::with_capacity(GUTTER_CHUNK_SIZE * 6);

        for start in (1..=line_count).step_by(GUTTER_CHUNK_SIZE) {
            let end = (start + GUTTER_CHUNK_SIZE - 1).min(line_count);
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
            ui::draw_horizontal_divider(ui);

            // 2. Exact calculation of body vs status bar heights
            let status_bar_height = if !app.show_options && app.data.settings.show_status_bar {
                STATUS_BAR_HEIGHT
            } else {
                0.0
            };
            let body_height = (ui.available_height() - status_bar_height - 6.0).max(40.0);

            // 3. Body Area with explicit height allocation
            egui::Frame::NONE
                .inner_margin(Margin::symmetric(14, 4))
                .show(ui, |ui| {
                    ui.set_height(body_height);
                    ui.set_max_height(body_height);

                    if app.show_options {
                        ui::options_drawer::render_options_drawer(app, ctx, ui);
                    } else if app.show_search {
                        ui::search_drawer::render_search_drawer(app, ctx, ui);
                    } else {
                        render_editor_workspace(app, ctx, ui);
                    }
                });

            // 4. Status Bar at the bottom
            if !app.show_options && app.data.settings.show_status_bar {
                ui::draw_horizontal_divider(ui);
                render_status_bar(app, ctx, ui);
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
    // Ctrl + Mouse Wheel font zooming
    if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_y.abs() > 0.5 {
            let delta = if scroll_y > 0.0 { 0.5 } else { -0.5 };
            let new_size = (app.data.settings.font_size + delta).clamp(
                crate::models::settings::MIN_FONT_SIZE,
                crate::models::settings::MAX_FONT_SIZE,
            );
            if (new_size - app.data.settings.font_size).abs() > 0.01 {
                app.data.settings.font_size = new_size;
                app.set_status(format!("Zoom: {:.1}pt", new_size));
                app.is_dirty = true;
                let _ = crate::storage::AppData::save_settings_to_path(
                    &app.data.settings,
                    &crate::storage::AppData::config_path(),
                );
            }
        }
    }

    let font_size = app.data.settings.font_size;
    let is_monospace = app.data.settings.monospace_font;
    let preview_mode = app.preview_mode;
    let should_focus = app.focus_editor;
    app.focus_editor = false;

    let palette = theme::get_palette(&app.data.settings);
    let enable_ghost_text = app.data.settings.enable_ghost_text;
    let show_line_numbers = app.data.settings.show_line_numbers;
    let enable_syntax = app.data.settings.enable_syntax_highlighting;
    let line_count = app.cached_active_stats.2;
    let mut content_changed = false;
    let mut context_menu_action: Option<crate::ui::context_menu::ContextMenuAction> = None;
    let current_cursor_range = app.last_cursor_range;

    let suggestion_engine = &mut app.suggestion_engine;
    let active_ghost_suffix = &mut app.active_ghost_suffix;
    let last_cursor_range = &mut app.last_cursor_range;

    if let Some(active_id) = app.data.active_note_id.as_deref()
        && let Some(note) = app.data.notes.iter_mut().find(|n| n.id == active_id)
    {
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

        let language = if enable_syntax {
            crate::ui::syntax::detect_language(&note.title, note.file_path.as_deref())
        } else {
            ""
        };
        let code_theme =
            egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

        match effective_mode {
            crate::app::MarkdownViewMode::Edit => {
                egui::ScrollArea::vertical()
                    .id_salt("editor_scroll_area")
                    .auto_shrink([false, false])
                    .max_height(editor_height)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal_top(|ui| {
                                if show_line_numbers {
                                    // Line numbers gutter (chunked for unlimited line counts)
                                    render_line_numbers_gutter(ui, line_count, &line_font);
                                    ui.add_space(6.0);
                                    ui.separator();
                                    ui.add_space(6.0);
                                }

                                render_multiline_editor_pane(
                                    ui,
                                    note,
                                    enable_ghost_text,
                                    active_ghost_suffix,
                                    suggestion_engine,
                                    last_cursor_range,
                                    current_cursor_range,
                                    &palette,
                                    &line_font,
                                    &code_theme,
                                    language,
                                    is_monospace,
                                    font_size,
                                    "Type your notes here...",
                                    should_focus,
                                    &mut content_changed,
                                    &mut context_menu_action,
                                );
                            });
                        });
                    });
            }

            crate::app::MarkdownViewMode::Preview => {
                let preview_resp = egui::ScrollArea::vertical()
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
                                Some(note),
                            );
                        }
                    });

                let r = ui.interact(
                    preview_resp.inner_rect,
                    egui::Id::new("preview_ctx_menu"),
                    egui::Sense::click(),
                );

                r.context_menu(|ui| {
                    crate::ui::context_menu::render_editor_context_menu(
                        ui,
                        note,
                        *last_cursor_range,
                        &palette,
                        &mut context_menu_action,
                    );
                });
            }

            crate::app::MarkdownViewMode::Split => {
                let total_width = ui.available_width();
                let sep_width = 12.0;
                let usable_width = (total_width - sep_width).max(100.0);
                app.split_ratio = app.split_ratio.clamp(0.15, 0.85);

                let left_width = usable_width * app.split_ratio;
                let right_width = (usable_width - left_width).max(50.0);

                ui.horizontal(|ui| {
                    // Left column: Editor
                    ui.vertical(|ui| {
                        ui.set_width(left_width);
                        ui.set_height(editor_height);
                        egui::ScrollArea::vertical()
                            .id_salt("split_editor_scroll_area")
                            .auto_shrink([false, false])
                            .max_height(editor_height)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal_top(|ui| {
                                        if app.data.settings.show_line_numbers {
                                            render_line_numbers_gutter(ui, line_count, &line_font);
                                            ui.add_space(6.0);
                                            ui.separator();
                                            ui.add_space(6.0);
                                        }

                                        render_multiline_editor_pane(
                                            ui,
                                            note,
                                            enable_ghost_text,
                                            active_ghost_suffix,
                                            suggestion_engine,
                                            last_cursor_range,
                                            current_cursor_range,
                                            &palette,
                                            &line_font,
                                            &code_theme,
                                            language,
                                            is_monospace,
                                            font_size,
                                            "Type markdown here...",
                                            should_focus,
                                            &mut content_changed,
                                            &mut context_menu_action,
                                        );
                                    });
                                });
                            });
                    });

                    // Middle draggable vertical divider & grip handle
                    let (sep_rect, sep_resp) = ui.allocate_exact_size(
                        egui::vec2(sep_width, editor_height),
                        egui::Sense::click_and_drag(),
                    );

                    let is_active = sep_resp.hovered() || sep_resp.dragged();

                    if sep_resp.dragged() {
                        let delta_x = sep_resp.drag_delta().x;
                        let ratio_delta = delta_x / usable_width;
                        app.split_ratio = (app.split_ratio + ratio_delta).clamp(0.15, 0.85);
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    } else if sep_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    }

                    if sep_resp.double_clicked() {
                        app.split_ratio = 0.5;
                    }

                    // 1. Divider line from top to bottom
                    let center_x = sep_rect.center().x;
                    let line_color = if is_active {
                        palette.accent
                    } else {
                        crate::theme::Palette::with_alpha(palette.border, 180)
                    };
                    let line_stroke =
                        egui::Stroke::new(if is_active { 1.5 } else { 1.0 }, line_color);
                    ui.painter().line_segment(
                        [
                            egui::pos2(center_x, sep_rect.min.y),
                            egui::pos2(center_x, sep_rect.max.y),
                        ],
                        line_stroke,
                    );

                    // 2. Centered draggable grip pill handle
                    let pill_h = 36.0;
                    let pill_w = if is_active { 5.0 } else { 3.5 };
                    let pill_rect = egui::Rect::from_center_size(
                        egui::pos2(center_x, sep_rect.center().y),
                        egui::vec2(pill_w, pill_h),
                    );
                    let pill_bg = if is_active {
                        palette.accent
                    } else {
                        Color32::from_gray(140)
                    };

                    ui.painter()
                        .rect_filled(pill_rect, egui::CornerRadius::same(2), pill_bg);

                    // Right column: Live Markdown Preview
                    ui.vertical(|ui| {
                        ui.set_width(right_width);
                        ui.set_height(editor_height);
                        let split_prev_resp = egui::ScrollArea::vertical()
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
                                        Some(note),
                                    );
                                });
                            });

                        let r = ui.interact(
                            split_prev_resp.inner_rect,
                            egui::Id::new("split_preview_ctx_menu"),
                            egui::Sense::click(),
                        );
                        r.context_menu(|ui| {
                            crate::ui::context_menu::render_editor_context_menu(
                                ui,
                                note,
                                *last_cursor_range,
                                &palette,
                                &mut context_menu_action,
                            );
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

    // Execute actions triggered from the right-click context menu
    if let Some(action) = context_menu_action {
        let cursor_range = app.last_cursor_range;
        match action {
            crate::ui::context_menu::ContextMenuAction::LaunchAi => {
                if let Some((start, end)) = cursor_range
                    && start < end
                {
                    app.last_cursor_range = Some((start, end));
                }
                app.trigger_ai_assist();
            }

            crate::ui::context_menu::ContextMenuAction::Cut => {
                if let Some((start, end)) = cursor_range
                    && start < end
                    && let Some(note) = app.active_note_mut()
                {
                    let text = note.char_slice(start, end);
                    crate::ui::context_menu::set_clipboard_text(&text);
                    crate::ui::context_menu::delete_selection(note, start, end);
                    app.last_cursor_range = Some((start, start));
                    app.is_dirty = true;
                    app.show_toast("Cut to clipboard", crate::ui::toast::ToastKind::Success);
                }
            }
            crate::ui::context_menu::ContextMenuAction::Copy => {
                if let Some(note) = app.active_note() {
                    let text = if let Some((start, end)) = cursor_range
                        && start < end
                    {
                        note.char_slice(start, end)
                    } else {
                        note.content.clone()
                    };
                    crate::ui::context_menu::set_clipboard_text(&text);
                    app.show_toast("Copied to clipboard", crate::ui::toast::ToastKind::Success);
                }
            }
            crate::ui::context_menu::ContextMenuAction::Paste => {
                if let Some((name, mime, bytes)) = crate::ui::context_menu::get_clipboard_image() {
                    crate::ui::drag_drop::attach_image_to_active_note_at_cursor(
                        app,
                        &name,
                        mime,
                        bytes,
                        cursor_range,
                    );
                } else {
                    let clip = crate::ui::context_menu::get_clipboard_text();
                    if !clip.is_empty()
                        && let Some(note) = app.active_note_mut()
                    {
                        let s = cursor_range.map_or(note.char_len(), |(st, _)| st);
                        crate::ui::context_menu::insert_or_replace_text(note, &clip, cursor_range);
                        let new_pos = s + clip.chars().count();
                        app.last_cursor_range = Some((new_pos, new_pos));
                        app.is_dirty = true;
                        app.show_toast(
                            "Pasted from clipboard",
                            crate::ui::toast::ToastKind::Success,
                        );
                    }
                }
            }

            crate::ui::context_menu::ContextMenuAction::AttachImage => {
                crate::ui::drag_drop::open_image_dialog(app);
            }
            crate::ui::context_menu::ContextMenuAction::Delete => {
                if let Some((start, end)) = cursor_range
                    && start < end
                    && let Some(note) = app.active_note_mut()
                {
                    crate::ui::context_menu::delete_selection(note, start, end);
                    app.last_cursor_range = Some((start, start));
                    app.is_dirty = true;
                }
            }
            crate::ui::context_menu::ContextMenuAction::SelectAll => {
                if let Some(note) = app.active_note() {
                    app.last_cursor_range = Some((0, note.content.len()));
                }
            }
            crate::ui::context_menu::ContextMenuAction::SearchNotes => {
                app.show_search = true;
                app.focus_search = true;
            }
            crate::ui::context_menu::ContextMenuAction::SaveNotes => {
                app.save_notes_to_disk();
                app.show_toast(
                    "Notes saved to disk ✓",
                    crate::ui::toast::ToastKind::Success,
                );
            }
        }
        _ctx.request_repaint();
    }
}

/// Renders the bottom status bar with clean, unboxed typography, subtle dots, and generous padding.
fn render_status_bar(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = theme::get_palette(&app.data.settings);
    let mut attachment_changed = false;

    egui::Frame::NONE
        .inner_margin(Margin {
            left: 14,
            right: 14,
            top: 5,
            bottom: 9,
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

                // 1. Language / Monospace indicator
                if app.data.settings.monospace_font {
                    ui.label(
                        RichText::new("Mono")
                            .font(FontId::monospace(11.0))
                            .color(Color32::from_gray(210)),
                    );
                    ui.label(
                        RichText::new("•")
                            .font(FontId::proportional(10.0))
                            .color(Color32::from_gray(100)),
                    );
                }

                // 2. Linked File indicator or Mode
                if let Some(note) = app.active_note() {
                    if let Some(ref path_str) = note.file_path {
                        let is_status_active = app.status_msg.as_ref().is_some_and(|(_, created)| {
                            created.elapsed().as_secs_f32() < STATUS_MSG_DURATION_SECS
                        });
                        let max_path_len = if is_status_active { 20 } else { 34 };
                        let display_path =
                            crate::storage::format_display_path(path_str, max_path_len);
                        let path_resp = ui.add(
                            egui::Label::new(
                                RichText::new(format!("🔗 {}", display_path))
                                    .font(FontId::proportional(11.5))
                                    .color(palette.accent),
                            )
                            .sense(egui::Sense::click())
                            .truncate(),
                        );

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
                        ui.label(
                            RichText::new("Markdown")
                                .font(FontId::proportional(11.5))
                                .color(Color32::from_gray(190)),
                        );
                    } else {
                        ui.label(
                            RichText::new("Plain Text")
                                .font(FontId::proportional(11.5))
                                .color(Color32::from_gray(190)),
                        );
                    }
                }

                // 3. Attached Images minimal indicator
                if let Some(note) = app.active_note_mut()
                    && !note.attachments.is_empty()
                {
                    ui.label(
                        RichText::new("•")
                            .font(FontId::proportional(10.0))
                            .color(Color32::from_gray(100)),
                    );
                    crate::ui::image_view::render_attachment_popup_button(
                        ui,
                        ctx,
                        note,
                        &palette,
                        &mut attachment_changed,
                    );
                }

                // 4. Transient Status Message Notification with Smooth Fade Animation
                if let Some((msg, created)) = &app.status_msg {
                    let elapsed = created.elapsed().as_secs_f32();
                    if elapsed < STATUS_MSG_DURATION_SECS {
                        let fade = if elapsed < 0.2 {
                            (elapsed / 0.2).clamp(0.0, 1.0)
                        } else if elapsed > STATUS_MSG_DURATION_SECS - 0.7 {
                            ((STATUS_MSG_DURATION_SECS - elapsed) / 0.7).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };
                        let alpha = (255.0 * fade) as u8;
                        if alpha > 10 {
                            ui.label(
                                RichText::new("•")
                                    .font(FontId::proportional(10.0))
                                    .color(Color32::from_rgba_unmultiplied(100, 100, 100, alpha)),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(format!("⚡ {}", msg))
                                        .font(FontId::proportional(11.5))
                                        .color(Color32::from_rgba_unmultiplied(
                                            palette.accent.r(),
                                            palette.accent.g(),
                                            palette.accent.b(),
                                            alpha,
                                        )),
                                )
                                .truncate(),
                            );
                        }
                    }
                }


                // --- Right Side Stats ---
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

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
                    let (words, chars, lines) = app.cached_active_stats;
                    ui.label(
                        RichText::new(format!(
                            "Lines: {}  •  Words: {}  •  Chars: {}",
                            lines, words, chars
                        ))
                        .font(FontId::proportional(11.5))
                        .color(Color32::from_gray(190)),
                    );
                });
            });
        });

    if attachment_changed {
        if let Some(note) = app.active_note_mut() {
            note.update_timestamp();
        }
        app.is_dirty = true;
    }
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
        egui::vec2(380.0, 140.0),
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                // Header with title and compact close button
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Close Note Tab?")
                            .font(FontId::proportional(15.0))
                            .strong()
                            .color(Color32::WHITE),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close_x = crate::components::button::icon_button(
                            ui,
                            "✕",
                            false,
                            &palette,
                            12.0,
                            egui::vec2(24.0, 24.0),
                        );
                        if close_x.on_hover_text("Cancel (Esc)").clicked() {
                            app.confirm_close_id = None;
                        }
                    });
                });

                ui.label(
                    RichText::new(format!(
                        "Are you sure you want to close '{}'? Unsaved changes will be discarded.",
                        note_title
                    ))
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(195)),
                );

                ui.add_space(8.0);

                // Action buttons
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;

                        let confirm_btn = crate::components::button::animated_danger_button(
                            ui,
                            "Close Note",
                            egui::vec2(100.0, 30.0),
                        );

                        if confirm_btn.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            ctx.input_mut(|i| {
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                            });
                            app.close_note(&close_id);
                            app.confirm_close_id = None;
                            ctx.request_repaint();
                        }

                        let cancel_btn = crate::components::button::animated_action_button(
                            ui,
                            "Cancel",
                            &palette,
                            egui::vec2(80.0, 30.0),
                        );

                        if cancel_btn.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            ctx.input_mut(|i| {
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                            });
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

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("drop_overlay_layer"),
    ));
    let rect = ctx.content_rect();

    painter.rect_filled(
        rect,
        CornerRadius::same(12),
        Color32::from_rgba_unmultiplied(18, 12, 28, 210),
    );
    painter.rect_stroke(
        rect.shrink(8.0),
        CornerRadius::same(10),
        Stroke::new(2.0_f32, ACCENT_PURPLE),
        egui::StrokeKind::Outside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "📥 Drop image to attach or file to open",
        FontId::proportional(17.0),
        Color32::WHITE,
    );
}

pub use crate::ui::toast::render_floating_toast;

/// Renders a single multiline syntax-highlighted editor pane with ghost text, context menu, and auto-learning.
#[allow(clippy::too_many_arguments)]
fn render_multiline_editor_pane(
    ui: &mut Ui,
    note: &mut crate::note::Note,
    enable_ghost_text: bool,
    active_ghost_suffix: &mut Option<String>,
    suggestion_engine: &mut crate::suggest::SuggestionEngine,
    last_cursor_range: &mut Option<(usize, usize)>,
    current_cursor_range: Option<(usize, usize)>,
    palette: &theme::Palette,
    line_font: &FontId,
    code_theme: &egui_extras::syntax_highlighting::CodeTheme,
    language: &'static str,
    is_monospace: bool,
    font_size: f32,
    hint_text: &str,
    should_focus: bool,
    content_changed: &mut bool,
    context_menu_action: &mut Option<crate::ui::context_menu::ContextMenuAction>,
) {
    debug_assert!(
        (8.0..=48.0).contains(&font_size),
        "Font size must be within valid bounds"
    );

    let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, wrap_width: f32| {
        let text = buffer.as_str();
        let font_id = if is_monospace {
            FontId::monospace(font_size)
        } else {
            FontId::proportional(font_size)
        };
        let opts = crate::ui::syntax::HighlightOptions {
            theme: code_theme,
            language,
            font_id,
            text_color: Color32::WHITE,
            wrap_width,
        };
        let layout_job = crate::ui::syntax::highlight_text(ui.ctx(), ui.style(), text, opts);
        ui.fonts_mut(|f| f.layout_job(layout_job))
    };

    let tab_pressed_for_ghost = enable_ghost_text
        && active_ghost_suffix.is_some()
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));

    // Check for Ctrl+V image pasting while editor is focused
    let is_paste_image =
        ui.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::V));
    if is_paste_image
        && let Some((name, mime, bytes)) = crate::ui::context_menu::get_clipboard_image()
    {
        ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL, egui::Key::V);
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::V);
        });
        let id = note.add_attachment(&name, mime, bytes);
        let tag = format!("![{}](attachment:{})", name, id);
        let (s, e) = last_cursor_range.unwrap_or((note.char_len(), note.char_len()));
        crate::ui::context_menu::insert_or_replace_text(note, &tag, Some((s, e)));
        let new_pos = s + tag.chars().count();
        *last_cursor_range = Some((new_pos, new_pos));
        *content_changed = true;
    }

    let min_editor_h = (ui.available_height() - 10.0).max(300.0);
    let text_edit = egui::TextEdit::multiline(&mut note.content)
        .frame(egui::Frame::NONE)
        .hint_text(hint_text)
        .desired_width(ui.available_width())
        .min_size(egui::vec2(ui.available_width(), min_editor_h))
        .lock_focus(true)
        .layouter(&mut layouter);

    let output = text_edit.show(ui);
    let resp = &output.response;

    let accepted = handle_ghost_text(
        enable_ghost_text,
        tab_pressed_for_ghost,
        suggestion_engine,
        active_ghost_suffix,
        last_cursor_range,
        palette,
        ui,
        note,
        &output,
        line_font,
        content_changed,
    );

    let is_lmb_clicked = resp.clicked();

    if !accepted && let Some(range) = output.state.cursor.char_range() {
        let p: usize = range.primary.index.into();
        let s: usize = range.secondary.index.into();
        if p != s {
            // Active non-empty text selection: always record it
            *last_cursor_range = Some((p.min(s), p.max(s)));
        } else if is_lmb_clicked || last_cursor_range.is_none() {
            // User explicitly left-clicked at a point, clearing the selection
            *last_cursor_range = Some((p, p));
        } else if let Some((start, end)) = current_cursor_range
            && start < end
        {
            // Preserve existing selection when right-clicking or opening context menu
            *last_cursor_range = Some((start, end));
            let mut state = egui::text_edit::TextEditState::load(ui.ctx(), resp.id)
                .unwrap_or_else(|| output.state.clone());
            state
                .cursor
                .set_char_range(Some(egui::text_selection::CCursorRange::two(
                    egui::text::CCursor::new(start),
                    egui::text::CCursor::new(end),
                )));
            state.store(ui.ctx(), resp.id);
        }
    }

    resp.context_menu(|ui| {
        crate::ui::context_menu::render_editor_context_menu(
            ui,
            note,
            *last_cursor_range,
            palette,
            context_menu_action,
        );
    });

    if resp.changed() {
        *content_changed = true;
        if let Some(range) = output.state.cursor.char_range() {
            let p: usize = range.primary.index.into();
            let text_before = note.char_slice(0, p);
            let trimmed = text_before.trim_end();
            if trimmed.len() < text_before.len() {
                let (_, prev_w) = crate::engine::suggest::extract_preceding_words(&text_before);
                if let Some(completed_word) = prev_w {
                    suggestion_engine.learn_word(&completed_word);
                }
            }
        }
    }

    if resp.clicked() || should_focus {
        resp.request_focus();
    }
}

/// Extracts the active word prefix immediately preceding a character index (zero heap allocation).
fn extract_word_prefix_before_cursor(content: &str, cursor_char_idx: usize) -> &str {
    if cursor_char_idx == 0 {
        return "";
    }

    // Get the character immediately preceding cursor_char_idx
    let Some(prev_char) = content.chars().nth(cursor_char_idx - 1) else {
        return "";
    };

    // If character directly before cursor is not alphanumeric, identifier, or apostrophe, there is no prefix.
    if !prev_char.is_alphanumeric()
        && prev_char != '_'
        && prev_char != '-'
        && prev_char != '\''
        && prev_char != '’'
    {
        return "";
    }

    // Convert char index to byte index safely
    let mut cursor_byte = 0;
    for (i, (b_idx, c)) in content.char_indices().enumerate() {
        if i == cursor_char_idx {
            cursor_byte = b_idx;
            break;
        }
        cursor_byte = b_idx + c.len_utf8();
    }

    let text_before = &content[..cursor_byte.min(content.len())];
    let start = text_before
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '\'' && c != '’')
        .map_or(0, |pos| {
            let sub = &text_before[pos..];
            let first_char_len = sub.chars().next().map_or(1, |c| c.len_utf8());
            pos + first_char_len
        });

    &text_before[start..]
}

/// Checks if the character immediately following the cursor is not continuing an existing word.
fn is_cursor_at_end_of_word(content: &str, cursor_char_idx: usize) -> bool {
    let mut chars = content.chars().skip(cursor_char_idx);
    match chars.next() {
        Some(c) => !c.is_alphanumeric() && c != '_' && c != '-' && c != '\'' && c != '’',
        None => true,
    }
}

/// Handles ghost writing autocomplete display and Tab acceptance.
#[allow(clippy::too_many_arguments)]
fn handle_ghost_text(
    enable_ghost_text: bool,
    tab_accepted: bool,
    suggestion_engine: &mut crate::suggest::SuggestionEngine,
    active_ghost_suffix: &mut Option<String>,
    last_cursor_range: &mut Option<(usize, usize)>,
    palette: &theme::Palette,
    ui: &mut Ui,
    note: &mut crate::note::Note,
    output: &egui::text_edit::TextEditOutput,
    line_font: &FontId,
    content_changed: &mut bool,
) -> bool {
    if !enable_ghost_text {
        *active_ghost_suffix = None;
        return false;
    }

    let Some(cursor_range) = output.state.cursor.char_range() else {
        *active_ghost_suffix = None;
        return false;
    };

    if cursor_range.primary.index != cursor_range.secondary.index {
        *active_ghost_suffix = None;
        return false;
    }

    let char_idx: usize = cursor_range.primary.index.into();

    if !is_cursor_at_end_of_word(&note.content, char_idx) {
        *active_ghost_suffix = None;
        return false;
    }

    let prefix = extract_word_prefix_before_cursor(&note.content, char_idx);
    if prefix.is_empty() {
        *active_ghost_suffix = None;
        return false;
    }

    let prefix_str = prefix.to_string();
    let prefix_char_count = prefix_str.chars().count();
    let context_char_end = char_idx.saturating_sub(prefix_char_count);
    let context_before = note.char_slice(0, context_char_end);

    let suggestion = suggestion_engine.suggest_with_context(&context_before, &prefix_str);
    *active_ghost_suffix = suggestion.clone();

    let Some(suffix) = suggestion else {
        return false;
    };

    if tab_accepted {
        // Always append a trailing space when accepting completion (unless following char is already space)
        let next_char = note.content.chars().nth(char_idx);
        let should_add_space = next_char != Some(' ');
        let inserted_text = if should_add_space {
            format!("{} ", suffix)
        } else {
            suffix.clone()
        };

        note.replace_char_range(char_idx, char_idx, &inserted_text);
        let new_char_idx = char_idx + inserted_text.chars().count();

        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), output.response.id)
            .unwrap_or_else(|| output.state.clone());
        state
            .cursor
            .set_char_range(Some(egui::text_selection::CCursorRange::one(
                egui::text::CCursor::new(new_char_idx),
            )));
        state.store(ui.ctx(), output.response.id);

        *last_cursor_range = Some((new_char_idx, new_char_idx));
        *active_ghost_suffix = None;
        *content_changed = true;

        let full_word = format!("{}{}", prefix_str, suffix);
        suggestion_engine.learn_word(&full_word);
        let (_, prev_w) = crate::engine::suggest::extract_preceding_words(&context_before);
        if let Some(w1) = prev_w {
            suggestion_engine.learn_bigram(&w1, &full_word);
        }

        ui.ctx().request_repaint();
        return true;
    }

    // Render faded ghost text at the exact cursor screen position
    let cursor_cpos = cursor_range.primary;
    let cursor_rect = output.galley.pos_from_cursor(cursor_cpos);
    let ghost_screen_pos = output.galley_pos + egui::vec2(cursor_rect.max.x, cursor_rect.min.y);

    let ghost_color = Color32::from_rgba_unmultiplied(
        palette.accent.r(),
        palette.accent.g(),
        palette.accent.b(),
        140,
    );

    ui.painter().text(
        ghost_screen_pos,
        egui::Align2::LEFT_TOP,
        &suffix,
        line_font.clone(),
        ghost_color,
    );

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_word_prefix_stops_on_space_and_punctuation() {
        assert_eq!(extract_word_prefix_before_cursor("hello", 5), "hello");
        assert_eq!(extract_word_prefix_before_cursor("hello ", 6), "");
        assert_eq!(extract_word_prefix_before_cursor("hello world", 6), "");
        assert_eq!(
            extract_word_prefix_before_cursor("hello world", 11),
            "world"
        );
        assert_eq!(extract_word_prefix_before_cursor("hello\n", 6), "");
        assert_eq!(
            extract_word_prefix_before_cursor("let x = somet", 13),
            "somet"
        );
        assert_eq!(extract_word_prefix_before_cursor("let x = somet ", 14), "");
        assert_eq!(extract_word_prefix_before_cursor("hello.", 6), "");
        assert_eq!(extract_word_prefix_before_cursor("", 0), "");

        // Exact typing flow with spaces
        assert_eq!(extract_word_prefix_before_cursor("fast", 4), "fast");
        assert_eq!(extract_word_prefix_before_cursor("fast ", 5), "");
        assert_eq!(extract_word_prefix_before_cursor("fast c", 6), "c");
        assert_eq!(extract_word_prefix_before_cursor("fast cl", 7), "cl");
    }

    #[test]
    fn test_is_cursor_at_end_of_word() {
        assert!(is_cursor_at_end_of_word("hello", 5));
        assert!(is_cursor_at_end_of_word("hello world", 5));
        assert!(!is_cursor_at_end_of_word("hello world", 3));
    }
}
