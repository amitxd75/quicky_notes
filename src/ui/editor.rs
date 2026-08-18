//! Main editor workspace, line numbers gutter, status bar, and modal overlays.

use crate::app::QuickyNotesApp;
use crate::components::{card, modal};
use crate::theme::{self, ACCENT_EMERALD, ACCENT_PURPLE, Palette};
use crate::ui;
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, Ui};

/// Dedicated reserved height for the bottom status bar in pixels.
pub const STATUS_BAR_HEIGHT: f32 = 26.0;

/// Status bar compact viewport breakpoint width in pixels.
pub const STATUS_BAR_COMPACT_BREAKPOINT: f32 = 600.0;
/// Status bar ultra-compact viewport breakpoint width in pixels.
pub const STATUS_BAR_ULTRA_COMPACT_BREAKPOINT: f32 = 450.0;
/// Status bar wide statistics column width in pixels.
pub const STATUS_BAR_WIDE_STATS_WIDTH: f32 = 260.0;
/// Status bar compact statistics column width in pixels.
pub const STATUS_BAR_COMPACT_STATS_WIDTH: f32 = 180.0;
/// Status bar ultra-compact statistics column width in pixels.
pub const STATUS_BAR_ULTRA_COMPACT_STATS_WIDTH: f32 = 130.0;

/// Transient status notification message duration in seconds.
pub const STATUS_MSG_DURATION_SECS: f32 = 3.5;

/// Chunk size for virtualized line numbers gutter rendering.
pub const GUTTER_CHUNK_SIZE: usize = 100;

/// Minimum vertical height for multiline text editor canvas in pixels.
pub const MIN_EDITOR_HEIGHT: f32 = 300.0;

/// Computes line number gutter column width based on document line count and font size.
#[inline]
pub fn compute_gutter_width(line_count: usize, font_size: f32) -> f32 {
    let digits = format!("{}", line_count.max(1)).len();
    (digits as f32 * font_size * 0.62 + 14.0).max(24.0)
}

/// Renders the main glass editor container, header, drawers/editor, and status bar.
pub fn render_main_editor(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    card::glass_editor_frame(&app.data.settings).show(ui, |ui| {
        ui.vertical(|ui| {
            // 1. Sleek Header Bar
            ui::header::render_header(app, ctx, ui);
            ui::draw_horizontal_divider(ui);

            // 2. Exact calculation of body vs status bar heights without overflow
            let show_status_bar = !app.show_options && app.data.settings.editor.show_status_bar;
            let status_bar_height = if show_status_bar {
                STATUS_BAR_HEIGHT
            } else {
                0.0
            };
            let divider_height = if show_status_bar { 1.0 } else { 0.0 };
            let body_height =
                (ui.available_height() - status_bar_height - divider_height).max(40.0);

            // 3. Body Area with exact height allocation
            egui::Frame::NONE
                .inner_margin(Margin::symmetric(14, 0))
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
            if show_status_bar {
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
            let new_size = (app.data.settings.editor.font_size + delta).clamp(
                crate::models::settings::MIN_FONT_SIZE,
                crate::models::settings::MAX_FONT_SIZE,
            );
            if (new_size - app.data.settings.editor.font_size).abs() > 0.01 {
                app.data.settings.editor.font_size = new_size;
                app.set_status(format!("Zoom: {:.1}pt", new_size));
                app.is_dirty = true;
                let _ = crate::storage::AppData::save_settings_to_path(
                    &app.data.settings,
                    &crate::storage::AppData::config_path(),
                );
            }
        }
    }

    let font_size = app.data.settings.editor.font_size;
    let is_monospace = app.data.settings.editor.monospace_font;
    let preview_mode = app.preview_mode;
    let should_focus = app.focus_editor;
    app.focus_editor = false;

    let palette = app.active_palette();
    let enable_ghost_text = app.data.settings.editor.enable_ghost_text;
    let show_line_numbers = app.data.settings.editor.show_line_numbers;
    let enable_syntax = app.data.settings.editor.enable_syntax_highlighting;
    let line_count = app.cached_active_stats.2;
    let mut content_changed = false;
    let mut context_menu_action: Option<crate::ui::context_menu::ContextMenuAction> = None;

    let active_path_opt = app.active_note().and_then(|n| n.file_path.clone());
    let active_id_opt = app.data.active_note_id.clone();
    let show_sidebar = app.show_folder_sidebar && app.folder_workspace.is_some();
    let mut folder_action = None;

    let panel_h = if let Some(ref p) = app.active_plugin_panel {
        p.height
            .clamp(60.0, (ui.available_height() - 100.0).max(60.0))
    } else {
        0.0
    };
    let panel_divider_h = if panel_h > 0.0 { 8.0 } else { 0.0 };

    let suggestion_engine = &mut app.suggestion_engine;
    let active_ghost_suffix = &mut app.active_ghost_suffix;
    let last_cursor_range = &mut app.last_cursor_range;
    let folder_workspace = &mut app.folder_workspace;
    let notes = &mut app.data.notes;
    let split_ratio = &mut app.split_ratio;
    let active_plugin_panel = &mut app.active_plugin_panel;
    let mut bottom_panel_action = None;

    let total_width = ui.available_width();
    let full_height = ui.available_height();
    let editor_height = (full_height - panel_h - panel_divider_h).max(60.0);

    let plugin_menu_items: Vec<_> = if app.data.settings.plugins.enabled {
        app.plugin_manager
            .all_menu_items()
            .into_iter()
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    let sidebar_width = if let Some(ws) = folder_workspace.as_ref() {
        ws.sidebar_width
    } else {
        0.0
    };
    let sep_width = if show_sidebar { 8.0 } else { 0.0 };
    let editor_width = (total_width - sidebar_width - sep_width).max(100.0);

    ui.horizontal_top(|ui| {
        if show_sidebar {
            ui.allocate_ui_with_layout(
                egui::vec2(sidebar_width, full_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    if let Some(ws) = folder_workspace {
                        folder_action = crate::ui::folder_tree::render_folder_sidebar(
                            ws,
                            active_path_opt.as_deref(),
                            &palette,
                            ui,
                            full_height,
                        );
                    }
                },
            );

            // Resizable separator between sidebar and editor
            let (div_rect, div_resp) = ui.allocate_exact_size(
                egui::vec2(sep_width, full_height),
                egui::Sense::click_and_drag(),
            );
            if div_resp.hovered() || div_resp.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if div_resp.dragged()
                && let Some(ws) = folder_workspace
            {
                let delta_x = ui.input(|i| i.pointer.delta().x);
                ws.sidebar_width = (ws.sidebar_width + delta_x).clamp(
                    crate::ui::folder_tree::MIN_SIDEBAR_WIDTH,
                    crate::ui::folder_tree::MAX_SIDEBAR_WIDTH,
                );
            }
            let div_color = if div_resp.hovered() || div_resp.dragged() {
                palette.accent
            } else {
                Palette::with_alpha(palette.border, 90)
            };
            let center_x = div_rect.center().x;
            ui.painter().line_segment(
                [
                    egui::pos2(center_x, div_rect.min.y),
                    egui::pos2(center_x, div_rect.max.y),
                ],
                Stroke::new(1.0, div_color),
            );
        }

        ui.allocate_ui_with_layout(
            egui::vec2(editor_width, full_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                if let Some(ref active_id) = active_id_opt
                    && let Some(note) = notes.iter_mut().find(|n| &n.id == active_id)
                {
                    let line_font = if is_monospace {
                        FontId::monospace(font_size)
                    } else {
                        FontId::proportional(font_size)
                    };

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
                    let code_theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
                        ui.ctx(),
                        ui.style(),
                    );

                    match effective_mode {
                        crate::app::MarkdownViewMode::Edit => {
                            egui::ScrollArea::vertical()
                                .id_salt("editor_scroll_area")
                                .auto_shrink([false, false])
                                .max_height(editor_height)
                                .min_scrolled_height(editor_height)
                                .show(ui, |ui| {
                                    render_multiline_editor_pane(
                                        ui,
                                        note,
                                        show_line_numbers,
                                        line_count,
                                        enable_ghost_text,
                                        active_ghost_suffix,
                                        suggestion_engine,
                                        last_cursor_range,
                                        &palette,
                                        &line_font,
                                        &code_theme,
                                        language,
                                        is_monospace,
                                        font_size,
                                        "Start typing your note here... (Markdown supported)",
                                        should_focus,
                                        &mut content_changed,
                                        &plugin_menu_items,
                                        &mut context_menu_action,
                                    );
                                });
                        }

                        crate::app::MarkdownViewMode::Preview => {
                            let preview_resp = egui::ScrollArea::vertical()
                                .id_salt("markdown_preview_scroll_area")
                                .auto_shrink([false, false])
                                .max_height(editor_height)
                                .min_scrolled_height(editor_height)
                                .show(ui, |ui| {
                                    ui.set_min_height(editor_height);
                                    if note.content.trim().is_empty() {
                                        ui.add_space(20.0);
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                RichText::new("Preview is empty. Switch to Edit mode to write content.")
                                                    .font(FontId::proportional(14.0))
                                                    .color(Color32::from_gray(130)),
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
                                    &plugin_menu_items,
                                    &mut context_menu_action,
                                );
                            });
                        }

                        crate::app::MarkdownViewMode::Split => {
                            let total_width = ui.available_width();
                            let sep_width = 12.0;
                            let usable_width = (total_width - sep_width).max(100.0);
                            let left_width = usable_width * *split_ratio;
                            let right_width = usable_width - left_width;

                            ui.horizontal(|ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(left_width, editor_height),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        egui::ScrollArea::vertical()
                                            .id_salt("split_editor_scroll_area")
                                            .auto_shrink([false, false])
                                            .max_height(editor_height)
                                            .min_scrolled_height(editor_height)
                                            .show(ui, |ui| {
                                                render_multiline_editor_pane(
                                                    ui,
                                                    note,
                                                    show_line_numbers,
                                                    line_count,
                                                    enable_ghost_text,
                                                    active_ghost_suffix,
                                                    suggestion_engine,
                                                    last_cursor_range,
                                                    &palette,
                                                    &line_font,
                                                    &code_theme,
                                                    language,
                                                    is_monospace,
                                                    font_size,
                                                    "Start typing in Split View...",
                                                    should_focus,
                                                    &mut content_changed,
                                                    &plugin_menu_items,
                                                    &mut context_menu_action,
                                                );
                                            });
                                    },
                                );

                                // Split view resizer bar
                                let (div_rect, div_resp) = ui.allocate_exact_size(
                                    egui::vec2(sep_width, editor_height),
                                    egui::Sense::click_and_drag(),
                                );
                                if div_resp.hovered() || div_resp.dragged() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                                }
                                if div_resp.dragged() {
                                    let delta_x = ui.input(|i| i.pointer.delta().x);
                                    let delta_ratio = delta_x / usable_width;
                                    *split_ratio = (*split_ratio + delta_ratio).clamp(0.15, 0.85);
                                }
                                let div_color = if div_resp.hovered() || div_resp.dragged() {
                                    palette.accent
                                } else {
                                    Palette::with_alpha(palette.border, 90)
                                };
                                let center_x = div_rect.center().x;
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center_x, div_rect.min.y),
                                        egui::pos2(center_x, div_rect.max.y),
                                    ],
                                    Stroke::new(1.0, div_color),
                                );

                                let split_prev_resp = egui::ScrollArea::vertical()
                                    .id_salt("split_preview_scroll_area")
                                    .auto_shrink([false, false])
                                    .max_height(editor_height)
                                    .min_scrolled_height(editor_height)
                                    .show(ui, |ui| {
                                        ui.set_width(right_width);
                                        ui.set_min_height(editor_height);
                                        if note.content.trim().is_empty() {
                                            ui.add_space(20.0);
                                            ui.vertical_centered(|ui| {
                                                ui.label(
                                                    RichText::new("Split Preview is empty.")
                                                        .font(FontId::proportional(14.0))
                                                        .color(Color32::from_gray(130)),
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
                                        &plugin_menu_items,
                                        &mut context_menu_action,
                                    );
                                });
                            });
                        }
                    }

                    if content_changed {
                        note.update_timestamp();
                        note.is_dirty = true;
                        note.has_disk_conflict = false;
                        app.is_dirty = true;

                        if app.data.settings.general.auto_title_from_first_line && note.file_path.is_none() {
                            let prefix = app.data.settings.general.default_title_prefix.to_lowercase();
                            let is_default_title = note.title.to_lowercase().starts_with(&prefix)
                                || note.title.starts_with("untitled");
                            if is_default_title
                                && let Some(first_line) = note.content.lines().find(|l| !l.trim().is_empty())
                            {
                                let clean_line = first_line.trim().trim_start_matches('#').trim();
                                if !clean_line.is_empty() {
                                    let ext = if note.is_markdown() {
                                        ".md"
                                    } else {
                                        &app.data.settings.editor.default_extension
                                    };
                                    let mut truncated: String = clean_line.chars().take(24).collect();
                                    if !truncated.ends_with(ext) {
                                        truncated.push_str(ext);
                                    }
                                    note.title = crate::models::Note::sanitize_title(&truncated);
                                }
                            }
                        }
                    }

                    // Render bottom plugin output console panel if active
                    if panel_h > 0.0 {
                        let (div_rect, div_resp) = ui.allocate_exact_size(
                            egui::vec2(editor_width, panel_divider_h),
                            egui::Sense::click_and_drag(),
                        );
                        if div_resp.hovered() || div_resp.dragged() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                        }
                        if div_resp.dragged() {
                            let delta_y = ui.input(|i| i.pointer.delta().y);
                            if let Some(p) = active_plugin_panel.as_mut() {
                                p.height = (p.height - delta_y)
                                    .clamp(60.0, (full_height - 100.0).max(60.0));
                            }
                        }
                        let div_color = if div_resp.hovered() || div_resp.dragged() {
                            palette.accent
                        } else {
                            Palette::with_alpha(palette.border, 90)
                        };
                        let center_y = div_rect.center().y;
                        ui.painter().line_segment(
                            [
                                egui::pos2(div_rect.min.x, center_y),
                                egui::pos2(div_rect.max.x, center_y),
                            ],
                            Stroke::new(1.0, div_color),
                        );

                        bottom_panel_action = render_plugin_bottom_panel(
                            active_plugin_panel,
                            ui,
                            editor_width,
                            panel_h,
                            &palette,
                        );
                    }
                }
            },
        );
    });

    if content_changed {
        app.is_dirty = true;
        if app.data.settings.plugins.enabled {
            let note = app.active_note();
            let cursor = app.last_cursor_range;
            let palette = app.active_palette();
            let outcome = app
                .plugin_manager
                .dispatch_on_note_change(note, cursor, &palette);
            app.apply_plugin_outcome(outcome);
        }
    }

    if let Some(action) = bottom_panel_action {
        match action {
            BottomPanelAction::Close => {
                app.active_plugin_panel = None;
            }
            BottomPanelAction::Clear => {
                if let Some(ref mut p) = app.active_plugin_panel {
                    p.content.clear();
                }
            }
            BottomPanelAction::Copy(text) => {
                crate::ui::context_menu::set_clipboard_text(&text);
                app.show_toast(
                    "Output copied to clipboard",
                    crate::ui::toast::ToastKind::Success,
                );
            }
        }
    }

    if let Some(action) = folder_action {
        match action {
            crate::ui::folder_tree::FolderTreeAction::OpenFile(path) => {
                app.open_file_from_path(&path);
            }
            crate::ui::folder_tree::FolderTreeAction::CloseWorkspace => {
                app.close_folder_workspace();
            }
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
            crate::ui::context_menu::ContextMenuAction::OpenFile => {
                crate::ui::drag_drop::open_file_dialog(app);
            }
            crate::ui::context_menu::ContextMenuAction::OpenFolder => {
                crate::ui::drag_drop::open_folder_dialog(app);
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
            crate::ui::context_menu::ContextMenuAction::PluginAction(ref action_id) => {
                app.dispatch_plugin_context_menu(action_id);
            }
        }
        _ctx.request_repaint();
    }
}

/// Renders the bottom status bar with clean, unboxed typography, subtle dots, and generous padding.
fn render_status_bar(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = app.active_palette();
    let mut attachment_changed = false;

    let (active_file_path, is_markdown) = match app.active_note() {
        Some(n) => (n.file_path.clone(), n.is_markdown()),
        None => (None, false),
    };

    let total_w = ui.available_width();
    let is_compact = total_w < STATUS_BAR_COMPACT_BREAKPOINT;
    let is_ultra_compact = total_w < STATUS_BAR_ULTRA_COMPACT_BREAKPOINT;

    let stats_w = if is_ultra_compact {
        STATUS_BAR_ULTRA_COMPACT_STATS_WIDTH
    } else if is_compact {
        STATUS_BAR_COMPACT_STATS_WIDTH
    } else {
        STATUS_BAR_WIDE_STATS_WIDTH
    };
    let left_w = (total_w - stats_w - 12.0).max(60.0);

    egui::Frame::NONE
        .inner_margin(Margin::symmetric(14, 3))
        .show(ui, |ui| {
            ui.set_height(STATUS_BAR_HEIGHT - 6.0);
            ui.horizontal(|ui| {
                // Left side: State dot, Monospace toggle, Truncated file path, Mode toggle, and Transient notifications
                ui.allocate_ui_with_layout(
                    egui::vec2(left_w, ui.available_height()),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;

                        // 1. Interactive Sync / Save status indicator (Click to save)
                        let (dot_color, label_text, label_color) = if app.is_dirty {
                            (
                                Color32::from_rgb(251, 191, 36),
                                "Unsaved",
                                Color32::from_rgb(251, 191, 36),
                            )
                        } else {
                            (ACCENT_EMERALD, "Saved", Color32::from_gray(210))
                        };

                        let save_resp = ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.label(RichText::new("●").font(FontId::proportional(11.0)).color(dot_color));
                            ui.add(
                                egui::Label::new(
                                    RichText::new(label_text)
                                        .font(FontId::proportional(11.5))
                                        .color(label_color),
                                )
                                .sense(egui::Sense::click()),
                            )
                        })
                        .inner;

                        let save_tooltip = if app.is_dirty {
                            "● Unsaved changes in note\n• Click to save immediately to disk (Ctrl+S)"
                        } else {
                            "● All notes saved to disk ✓\n• Click to force save (Ctrl+S)"
                        };
                        if save_resp.on_hover_text(save_tooltip).clicked() {
                            app.save_notes_to_disk();
                        }

                        // 2. Interactive Language / Monospace indicator (Click to toggle font)
                        let font_label = if app.data.settings.editor.monospace_font {
                            "Mono"
                        } else {
                            "Sans"
                        };
                        let font_resp = ui.add(
                            egui::Label::new(
                                RichText::new(font_label)
                                    .font(if app.data.settings.editor.monospace_font {
                                        FontId::monospace(11.0)
                                    } else {
                                        FontId::proportional(11.5)
                                    })
                                    .color(Color32::from_gray(210)),
                            )
                            .sense(egui::Sense::click()),
                        );
                        let font_tooltip = format!(
                            "Font Family: {}\n• Click to toggle between Monospace and Proportional font",
                            if app.data.settings.editor.monospace_font {
                                "Monospace"
                            } else {
                                "Proportional"
                            }
                        );
                        if font_resp.on_hover_text(font_tooltip).clicked() {
                            app.data.settings.editor.monospace_font = !app.data.settings.editor.monospace_font;
                            app.is_dirty = true;
                            app.set_status(format!(
                                "Font: {}",
                                if app.data.settings.editor.monospace_font {
                                    "Monospace"
                                } else {
                                    "Proportional"
                                }
                            ));
                        }

                        ui.label(
                            RichText::new("•")
                                .font(FontId::proportional(10.0))
                                .color(Color32::from_gray(100)),
                        );

                        // 3. Linked File indicator or Interactive Mode toggle
                        if let Some(ref path_str) = active_file_path {
                            let is_status_active =
                                app.status_msg.as_ref().is_some_and(|(_, created)| {
                                    created.elapsed().as_secs_f32() < STATUS_MSG_DURATION_SECS
                                });
                            let max_path_len = if is_status_active {
                                14
                            } else if is_compact {
                                18
                            } else {
                                32
                            };
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
                                app.show_toast(
                                    "File path copied to clipboard",
                                    crate::ui::toast::ToastKind::Success,
                                );
                            }

                            path_resp.context_menu(|ui| {
                                if ui.button("📋 Copy Full Path").clicked() {
                                    ui.ctx().copy_text(path_str.clone());
                                    ui.close();
                                }
                                if ui.button("📂 Reveal Folder in File Manager").clicked() {
                                    if let Some(parent) =
                                        std::path::Path::new(path_str).parent()
                                    {
                                        crate::ui::drag_drop::safe_open_folder(parent);
                                    }
                                    ui.close();
                                }
                            });
                        } else {
                            let mode_label = if is_markdown {
                                "Markdown"
                            } else {
                                "Plain Text"
                            };
                            let mode_resp = ui.add(
                                egui::Label::new(
                                    RichText::new(mode_label)
                                        .font(FontId::proportional(11.5))
                                        .color(Color32::from_gray(190)),
                                )
                                .sense(egui::Sense::click()),
                            );

                            if is_markdown {
                                let mode_tooltip = format!(
                                    "Document Type: Markdown ({})\n• Click to cycle preview mode (Edit / Split / View)",
                                    app.preview_mode.label()
                                );
                                if mode_resp.on_hover_text(mode_tooltip).clicked() {
                                    app.preview_mode = app.preview_mode.next();
                                }
                            } else {
                                mode_resp.on_hover_text("Document Type: Plain Text Note");
                            }
                        }

                        // 4. Attached Images minimal indicator
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

                        // 5. Transient Status Message Notification
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
                                            .color(Color32::from_rgba_unmultiplied(
                                                100, 100, 100, alpha,
                                            )),
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
                    },
                );

                // Right side: Statistics & Encoding (Responsive and Interactive)
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height()),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // 1. UTF-8 Indicator
                        if !is_ultra_compact {
                            let utf8_resp = ui.add(
                                egui::Label::new(
                                    RichText::new("UTF-8")
                                        .font(FontId::monospace(11.0))
                                        .color(Color32::from_gray(150)),
                                )
                                .sense(egui::Sense::hover()),
                            );
                            utf8_resp.on_hover_text("Text Encoding: UTF-8 Unicode");
                        }

                        // 2. Interactive Tab Size (Click or Context menu to cycle/change)
                        if !is_compact {
                            let tab_resp = ui.add(
                                egui::Label::new(
                                    RichText::new(format!("Tab: {} sp", app.data.settings.editor.tab_size))
                                        .font(FontId::proportional(11.5))
                                        .color(Color32::from_gray(170)),
                                )
                                .sense(egui::Sense::click()),
                            );
                            let tab_tooltip = format!(
                                "Tab Indentation: {} spaces\n• Click to cycle tab size (2, 4, 8)\n• Right-click for options",
                                app.data.settings.editor.tab_size
                            );
                            let tab_resp = tab_resp.on_hover_text(tab_tooltip);

                            if tab_resp.clicked() {
                                let new_size = match app.data.settings.editor.tab_size {
                                    2 => 4,
                                    4 => 8,
                                    _ => 2,
                                };
                                app.data.settings.editor.tab_size = new_size;
                                app.is_dirty = true;
                                app.set_status(format!("Tab size: {} spaces", new_size));
                            }

                            tab_resp.context_menu(|ui| {
                                for sz in [2, 4, 8] {
                                    if ui
                                        .selectable_label(
                                            app.data.settings.editor.tab_size == sz,
                                            format!("{} Spaces", sz),
                                        )
                                        .clicked()
                                    {
                                        app.data.settings.editor.tab_size = sz;
                                        app.is_dirty = true;
                                        ui.close();
                                    }
                                }
                            });
                        }

                        // 3. Document Statistics (Responsive formatting + Click to copy)
                        let (words, chars, lines) = app.cached_active_stats;
                        let stats_text = if is_ultra_compact {
                            format!("L: {} • W: {}", lines, words)
                        } else if is_compact {
                            format!("L: {} • W: {} • C: {}", lines, words, chars)
                        } else {
                            format!("Lines: {} • Words: {} • Chars: {}", lines, words, chars)
                        };

                        let stats_resp = ui.add(
                            egui::Label::new(
                                RichText::new(stats_text)
                                    .font(FontId::proportional(11.5))
                                    .color(Color32::from_gray(190)),
                            )
                            .sense(egui::Sense::click()),
                        );

                        let stats_tooltip = format!(
                            "📊 Document Metrics:\n• Lines: {}\n• Words: {}\n• Characters: {}\n• Click to copy statistics to clipboard",
                            lines, words, chars
                        );
                        if stats_resp.on_hover_text(stats_tooltip).clicked() {
                            let summary = format!(
                                "Lines: {}, Words: {}, Characters: {}",
                                lines, words, chars
                            );
                            ui.ctx().copy_text(summary);
                            app.show_toast(
                                "Stats copied to clipboard",
                                crate::ui::toast::ToastKind::Success,
                            );
                        }
                    },
                );
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
                        ui.spacing_mut().item_spacing.x = 8.0;

                        let cancel_btn = crate::components::button::animated_action_button(
                            ui,
                            "Cancel",
                            &palette,
                            egui::vec2(70.0, 28.0),
                        );

                        if cancel_btn.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            ctx.input_mut(|i| {
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                            });
                            app.confirm_close_id = None;
                            ctx.request_repaint();
                        }

                        let discard_btn = crate::components::button::animated_danger_button(
                            ui,
                            "Discard",
                            egui::vec2(80.0, 28.0),
                        );

                        if discard_btn.clicked() {
                            app.close_note(&close_id);
                            app.confirm_close_id = None;
                            ctx.request_repaint();
                        }

                        let save_btn = crate::components::button::animated_primary_button(
                            ui,
                            "Save & Close",
                            &palette,
                            egui::vec2(105.0, 28.0),
                        );

                        if save_btn.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            ctx.input_mut(|i| {
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                            });
                            if let Some(note) = app.data.notes.iter_mut().find(|n| n.id == close_id)
                                && note.file_path.is_some()
                            {
                                let _ =
                                    crate::storage::linked_files::sync_single_linked_note_to_disk(
                                        note,
                                    );
                            }
                            app.save_if_dirty();
                            app.close_note(&close_id);
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
    show_line_numbers: bool,
    line_count: usize,
    enable_ghost_text: bool,
    active_ghost_suffix: &mut Option<String>,
    suggestion_engine: &mut crate::suggest::SuggestionEngine,
    last_cursor_range: &mut Option<(usize, usize)>,
    palette: &theme::Palette,
    line_font: &FontId,
    code_theme: &egui_extras::syntax_highlighting::CodeTheme,
    language: &'static str,
    is_monospace: bool,
    font_size: f32,
    hint_text: &str,
    should_focus: bool,
    content_changed: &mut bool,
    plugin_items: &[crate::plugins::PluginMenuItem],
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
    let gutter_width = if show_line_numbers {
        compute_gutter_width(line_count, font_size)
    } else {
        0.0
    };

    let (gutter_rect, output) = ui
        .horizontal_top(|ui| {
            let g_rect = if show_line_numbers {
                let (r, _) = ui.allocate_exact_size(
                    egui::vec2(gutter_width, min_editor_h),
                    egui::Sense::hover(),
                );
                ui.add_space(8.0);
                Some(r)
            } else {
                None
            };

            let text_edit = egui::TextEdit::multiline(&mut note.content)
                .frame(egui::Frame::NONE)
                .hint_text(hint_text)
                .desired_width(ui.available_width())
                .min_size(egui::vec2(ui.available_width(), min_editor_h))
                .lock_focus(true)
                .layouter(&mut layouter);

            let out = text_edit.show(ui);
            (g_rect, out)
        })
        .inner;

    let resp = &output.response;

    // Paint line numbers directly aligned with TextEdit visual rows (culling-aware, handles word wrapping flawlessly)
    if let Some(gutter_r) = gutter_rect {
        let divider_x = gutter_r.max.x + 4.0;
        let editor_top = output.response.rect.min.y;
        let editor_bottom = output.response.rect.max.y;

        ui.painter().line_segment(
            [
                egui::pos2(divider_x, editor_top),
                egui::pos2(divider_x, editor_bottom),
            ],
            Stroke::new(1.0, Palette::with_alpha(palette.border, 60)),
        );

        let mut logical_line = 1;
        let mut next_row_is_new_line = true;

        if output.galley.rows.is_empty() {
            ui.painter().text(
                egui::pos2(gutter_r.max.x - 2.0, editor_top + font_size * 0.7),
                egui::Align2::RIGHT_CENTER,
                "1",
                line_font.clone(),
                Palette::with_alpha(palette.muted_text, 140),
            );
        } else {
            for row in &output.galley.rows {
                let row_center_y = editor_top + row.rect().center().y;
                if next_row_is_new_line {
                    let line_str = format!("{}", logical_line);
                    ui.painter().text(
                        egui::pos2(gutter_r.max.x - 2.0, row_center_y),
                        egui::Align2::RIGHT_CENTER,
                        line_str,
                        line_font.clone(),
                        Palette::with_alpha(palette.muted_text, 140),
                    );
                    logical_line += 1;
                }
                next_row_is_new_line = row.ends_with_newline;
            }
        }
    }

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

    if !accepted && let Some(range) = output.state.cursor.char_range() {
        let p: usize = range.primary.index.into();
        let s: usize = range.secondary.index.into();
        *last_cursor_range = Some((p.min(s), p.max(s)));
    }

    resp.context_menu(|ui| {
        crate::ui::context_menu::render_editor_context_menu(
            ui,
            note,
            *last_cursor_range,
            palette,
            plugin_items,
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

/// Action emitted from the bottom plugin console drawer.
#[derive(Debug, Clone, PartialEq, Eq)]
enum BottomPanelAction {
    Close,
    Clear,
    Copy(String),
}

/// Renders the collapsible interactive bottom plugin console drawer.
fn render_plugin_bottom_panel(
    panel_opt: &mut Option<crate::app::PluginPanelDisplayState>,
    ui: &mut Ui,
    panel_width: f32,
    panel_height: f32,
    palette: &Palette,
) -> Option<BottomPanelAction> {
    let mut action = None;

    if let Some(panel) = panel_opt.as_mut() {
        // Un-escape escaped newlines and tabs from JSON/script outputs
        let normalized = panel
            .content
            .replace("\\r\\n", "\n")
            .replace("\\n", "\n")
            .replace("\\t", "    ");

        let line_count = if normalized.trim().is_empty() {
            0
        } else {
            normalized.lines().count()
        };

        let frame = egui::Frame::NONE
            .fill(Palette::with_alpha(palette.card, 210))
            .stroke(Stroke::new(1.0, Palette::with_alpha(palette.border, 160)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 8));

        frame.show(ui, |ui| {
            ui.set_width(panel_width);
            ui.set_height(panel_height);

            // Panel Header Bar
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("📟 {}", panel.title))
                        .font(FontId::proportional(12.5))
                        .strong()
                        .color(palette.accent),
                );

                if line_count > 0 {
                    ui.add_space(4.0);
                    let badge_text = if line_count == 1 {
                        "1 line".to_string()
                    } else {
                        format!("{} lines", line_count)
                    };
                    ui.label(
                        RichText::new(badge_text)
                            .font(FontId::proportional(10.0))
                            .color(palette.muted_text),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Close button
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✕")
                                    .font(FontId::proportional(12.0))
                                    .color(palette.muted_text),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Close Console Drawer")
                        .clicked()
                    {
                        action = Some(BottomPanelAction::Close);
                    }

                    // Clear button
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("🗑 Clear")
                                    .font(FontId::proportional(11.0))
                                    .color(palette.muted_text),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Clear Output")
                        .clicked()
                    {
                        action = Some(BottomPanelAction::Clear);
                    }

                    // Copy button
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("📋 Copy")
                                    .font(FontId::proportional(11.0))
                                    .color(palette.muted_text),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Copy Output to Clipboard")
                        .clicked()
                    {
                        action = Some(BottomPanelAction::Copy(normalized.clone()));
                    }
                });
            });

            ui.add_space(4.0);

            // Monospace Inner Output Canvas
            let content_h = (panel_height - 36.0).max(30.0);
            let inner_canvas = egui::Frame::NONE
                .fill(Palette::with_alpha(palette.bg, 190))
                .stroke(Stroke::new(1.0, Palette::with_alpha(palette.border, 90)))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(8, 6));

            inner_canvas.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(content_h);

                egui::ScrollArea::vertical()
                    .id_salt("plugin_panel_scroll_area")
                    .auto_shrink([false, false])
                    .max_height(content_h)
                    .stick_to_bottom(panel.auto_scroll)
                    .show(ui, |ui| {
                        if normalized.trim().is_empty() {
                            ui.label(
                                RichText::new("~ Console is empty... Waiting for output.")
                                    .font(FontId::monospace(11.5))
                                    .color(Palette::with_alpha(palette.muted_text, 140)),
                            );
                        } else {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&normalized)
                                        .font(FontId::monospace(12.0))
                                        .color(palette.text),
                                )
                                .wrap_mode(egui::TextWrapMode::Wrap),
                            );
                        }
                    });
            });
        });
    }

    action
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
