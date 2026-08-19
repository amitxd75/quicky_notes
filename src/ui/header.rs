//! Top header bar rendering note tabs, title editing, tab drag/pin/close, and action buttons.

use crate::app::QuickyNotesApp;
use crate::components::{badge, button, card};
use crate::theme;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, RichText, Sense, Stroke, Ui, ViewportCommand,
};

/// Tab animation transition time in seconds.
pub const TAB_ANIM_TIME_SECS: f32 = 0.15;

/// Header tab corner radius in pixels.
pub const TAB_CORNER_RADIUS: u8 = 8;

/// Default desired width for in-place title editing textbox.
pub const TITLE_EDIT_DESIRED_WIDTH: f32 = 100.0;

/// Header action button dimensions in pixels.
pub const HEADER_BTN_SIZE: egui::Vec2 = egui::vec2(28.0, 28.0);
/// Window control button dimensions (minimize, close) in pixels.
pub const WINDOW_CTRL_BTN_SIZE: egui::Vec2 = egui::vec2(28.0, 28.0);
/// Options / search button dimensions in pixels.
pub const OPTIONS_BTN_SIZE: egui::Vec2 = egui::vec2(28.0, 28.0);

/// Base width reserved for right-side action controls (Close, Settings, Search, File/Folder buttons, dividers).
pub const RIGHT_CONTROLS_BASE_WIDTH: f32 = 236.0;
/// Additional width reserved when a folder workspace is open (sidebar toggle button).
pub const RIGHT_CONTROLS_FOLDER_WIDTH: f32 = 36.0;
/// Additional width reserved when active note is Markdown (preview mode button).
pub const RIGHT_CONTROLS_MARKDOWN_WIDTH: f32 = 36.0;
/// Minimum safety width for the tab scroll area to prevent underflow.
pub const MIN_TABS_SCROLL_WIDTH: f32 = 40.0;

/// Renders the top header bar containing open note tabs on the left and grouped action buttons on the right.
pub fn render_header(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = app.active_palette();

    // Solid outer header container bar
    card::glass_header_frame(&app.data.settings).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // 1. Calculate exact width needed for all right-hand action button groups
            let has_folder = app.folder_workspace.is_some();
            let has_md = app.active_note().is_some_and(|n| n.is_markdown());
            let plugin_btn_count = if app.data.settings.plugins.enabled {
                app.plugin_manager
                    .all_header_buttons()
                    .iter()
                    .filter(|b| b.position == crate::plugins::HeaderButtonPosition::Right)
                    .count()
            } else {
                0
            };
            let plugin_btn_width = plugin_btn_count as f32 * (HEADER_BTN_SIZE.x + 6.5);

            let right_buttons_width = RIGHT_CONTROLS_BASE_WIDTH
                + (if has_folder {
                    RIGHT_CONTROLS_FOLDER_WIDTH
                } else {
                    0.0
                })
                + (if has_md {
                    RIGHT_CONTROLS_MARKDOWN_WIDTH
                } else {
                    0.0
                })
                + plugin_btn_width;

            let available_tab_width =
                (ui.available_width() - right_buttons_width - 10.0).max(MIN_TABS_SCROLL_WIDTH);

            // 2. Note tabs on the LEFT with hidden scrollbar for pixel-perfect vertical alignment
            egui::ScrollArea::horizontal()
                .id_salt("header_tabs_scroll_container")
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .max_width(available_tab_width)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        render_tabs_list(app, ctx, ui, &palette);
                    });
                });

            // 3. Right side: All action buttons, modal toggles, and window controls placed right-to-left
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 6.5;
                render_right_controls(app, ctx, ui, &palette);
            });
        });
    });
}

/// Renders the scrollable list of note tabs with selection, pinning, renaming, and closing.
fn render_tabs_list(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    let active_id = app.data.active_note_id.as_deref();
    let mut tab_to_select = None;
    let mut tab_to_close = None;
    let mut tab_to_pin = None;
    let mut tab_to_reload = None;
    let mut tab_to_save_disk = false;
    let mut rename_finish = None;

    for note in &app.data.notes {
        let is_active = active_id == Some(note.id.as_str());
        let is_editing =
            app.editing_title.as_ref().map(|(id, _)| id.as_str()) == Some(note.id.as_str());

        let active_anim = ctx.animate_bool_with_time(
            egui::Id::new((&note.id, "tab_anim")),
            is_active,
            TAB_ANIM_TIME_SECS,
        );

        let active_tint = theme::Palette::interpolate_color(palette.card, palette.accent, 0.35);
        let base_tab_bg = theme::Palette::with_alpha(palette.card, 160);
        let target_tab_bg = theme::Palette::with_alpha(active_tint, 240);
        let tab_bg = theme::Palette::interpolate_color(base_tab_bg, target_tab_bg, active_anim);

        let base_stroke = theme::Palette::with_alpha(palette.border, 90);
        let target_stroke = theme::Palette::with_alpha(palette.accent, 180);
        let stroke_color =
            theme::Palette::interpolate_color(base_stroke, target_stroke, active_anim);
        let tab_stroke = Stroke::new(1.0 + active_anim * 0.2, stroke_color);

        let tab_frame = egui::Frame::NONE
            .fill(tab_bg)
            .stroke(tab_stroke)
            .corner_radius(CornerRadius::same(TAB_CORNER_RADIUS))
            .inner_margin(Margin::symmetric(12, 6));

        tab_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                // Pin icon indicator for pinned notes
                if note.pinned {
                    let pin_btn = badge::pin_badge(ui);
                    if pin_btn.on_hover_text("Unpin note").clicked() {
                        tab_to_pin = Some(note.id.clone());
                    }
                } else if is_active {
                    ui.label(RichText::new("●").size(11.0).color(palette.accent));
                }

                // Editable Title or Clickable Label
                if is_editing && let Some((_, ref mut buf)) = app.editing_title {
                    let title_edit = egui::TextEdit::singleline(buf)
                        .font(FontId::proportional(13.0))
                        .text_color(Color32::WHITE)
                        .frame(egui::Frame::NONE)
                        .desired_width(TITLE_EDIT_DESIRED_WIDTH);
                    let resp = ui.add(title_edit);
                    resp.request_focus();

                    if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        rename_finish = Some((note.id.clone(), buf.clone()));
                    }
                } else {
                    let base_title = if note.title.trim().is_empty() {
                        crate::models::DEFAULT_NOTE_TITLE
                    } else {
                        &note.title
                    };

                    let title_color = if is_active {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(190)
                    };

                    let tab_btn = if note.is_linked_file() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            let icon = if note.has_disk_conflict {
                                "⚠️"
                            } else {
                                "🔗"
                            };
                            ui.label(
                                RichText::new(icon)
                                    .font(FontId::proportional(12.0))
                                    .color(if note.has_disk_conflict {
                                        palette.danger
                                    } else {
                                        Color32::WHITE
                                    }),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(base_title)
                                        .font(FontId::proportional(13.0))
                                        .color(title_color),
                                )
                                .sense(Sense::click()),
                            )
                        })
                        .inner
                    } else {
                        ui.add(
                            egui::Label::new(
                                RichText::new(base_title)
                                    .font(FontId::proportional(13.0))
                                    .color(title_color),
                            )
                            .sense(Sense::click()),
                        )
                    };

                    let tooltip = if let Some(ref fp) = note.file_path {
                        if note.has_disk_conflict {
                            format!(
                                "⚠️ DISK CONFLICT: File modified externally while you have unsaved edits!\n\n📁 Linked File Path:\n{}\n\n• Right-click tab to Reload or Force Save\n• Click to switch",
                                fp
                            )
                        } else {
                            format!(
                                "📁 Linked File Path:\n{}\n\n• Click to switch\n• Right-click to rename or view options",
                                fp
                            )
                        }
                    } else {
                        format!(
                            "📝 Note: {}\nQuicky Notes internal storage\n\n• Click to switch\n• Right-click to rename or view options",
                            base_title
                        )
                    };
                    let tab_btn = tab_btn.on_hover_text(tooltip);

                    // Right-click context menu on tab
                    tab_btn.context_menu(|ui| {
                        if note.has_disk_conflict {
                            ui.label(
                                RichText::new("⚠️ External Conflict Detected")
                                    .color(palette.danger)
                                    .strong(),
                            );
                            ui.separator();
                        }
                        let pin_label = if note.pinned {
                            "Unpin Tab"
                        } else {
                            "Pin Tab 📌"
                        };
                        if ui.button(pin_label).clicked() {
                            tab_to_pin = Some(note.id.clone());
                            ui.close();
                        }
                        if ui.button("Rename").clicked() {
                            app.editing_title = Some((note.id.clone(), note.title.clone()));
                            ui.close();
                        }
                        if let Some(ref fp) = note.file_path {
                            ui.separator();
                            if ui.button("🔄 Reload from Disk").clicked() {
                                tab_to_reload = Some(note.id.clone());
                                ui.close();
                            }
                            if ui.button("💾 Save to Disk Now").clicked() {
                                tab_to_save_disk = true;
                                ui.close();
                            }
                            if ui.button("📋 Copy File Path").clicked() {
                                ui.ctx().copy_text(fp.clone());
                                ui.close();
                            }
                            if ui.button("📂 Open in File Manager").clicked() {
                                if let Some(parent) = std::path::Path::new(fp).parent() {
                                    crate::ui::drag_drop::safe_open_folder(parent);
                                }
                                ui.close();
                            }
                        }
                        ui.separator();
                        if ui.button("Close").clicked() {
                            tab_to_close = Some(note.id.clone());
                            ui.close();
                        }
                    });

                    if tab_btn.clicked() {
                        tab_to_select = Some(note.id.clone());
                    }
                }

                // Close "x" button inside tab
                if app.data.notes.len() > 1 {
                    let close_x = ui.add(
                        egui::Label::new(
                            RichText::new("×")
                                .font(FontId::proportional(13.0))
                                .color(Color32::from_gray(140)),
                        )
                        .sense(Sense::click()),
                    );
                    if close_x.on_hover_text("Close tab").clicked() {
                        tab_to_close = Some(note.id.clone());
                    }
                }
            });
        });
    }

    // Apply tab actions after iteration
    if let Some((id, new_title)) = rename_finish {
        if let Some(n) = app.data.notes.iter_mut().find(|n| n.id == id) {
            let sanitized = crate::note::Note::sanitize_title(&new_title);
            if n.title != sanitized {
                n.title = sanitized;
                n.update_timestamp();
                app.is_dirty = true;
            }
        }
        app.editing_title = None;
    }

    if let Some(id) = tab_to_select {
        app.data.active_note_id = Some(id);
        app.show_options = false;
        app.show_search = false;
        app.focus_editor = true;
    }

    if let Some(id) = tab_to_pin {
        if let Some(n) = app.data.notes.iter_mut().find(|n| n.id == id) {
            n.pinned = !n.pinned;
            app.is_dirty = true;
        }
        app.data.notes.sort_by_key(|n| !n.pinned);
    }

    if let Some(id) = tab_to_close {
        app.prompt_close_note(&id);
    }

    if let Some(id) = tab_to_reload
        && let Some(n) = app.data.notes.iter_mut().find(|n| n.id == id)
        && let Some(ref fp) = n.file_path
        && let Ok(disk_content) = std::fs::read_to_string(std::path::Path::new(fp))
    {
        n.content = disk_content;
        n.update_timestamp();
        app.show_toast("Reloaded from disk ✓", crate::app::ToastKind::Success);
    }

    if tab_to_save_disk {
        app.save_notes_to_disk();
    }
}

/// Renders the right-hand action controls (Close, Settings, Search, Markdown, Open Folder, Open File, New Tab).
fn render_right_controls(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    // 1. Close Window (×) Button
    let close_app_btn = button::icon_button(ui, "×", false, palette, 13.0, WINDOW_CTRL_BTN_SIZE);
    if close_app_btn.on_hover_text("Close window").clicked() {
        app.is_closing = true;
        app.save_if_dirty();
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    ui.separator();

    // 2. Options ⚙ Button
    let opt_btn = button::icon_button(ui, "⚙", app.show_options, palette, 15.0, OPTIONS_BTN_SIZE);
    let opt_tooltip = format!(
        "Settings & Preferences ({})",
        app.shortcut_label(crate::ui::shortcuts::ShortcutAction::OpenSettings)
    );
    if opt_btn.on_hover_text(opt_tooltip).clicked() {
        app.show_options = !app.show_options;
        app.show_search = false;
        if !app.show_options {
            app.focus_editor = true;
        }
        ctx.request_repaint();
    }

    // 3. Search 🔍 Button
    let search_btn =
        button::icon_button(ui, "🔍", app.show_search, palette, 14.0, OPTIONS_BTN_SIZE);
    let search_tooltip = format!(
        "Search notes ({})",
        app.shortcut_label(crate::ui::shortcuts::ShortcutAction::SearchNotes)
    );
    if search_btn.on_hover_text(search_tooltip).clicked() {
        app.show_search = !app.show_search;
        app.show_options = false;
        if app.show_search {
            app.focus_search = true;
            app.search_selected_idx = 0;
        } else {
            app.focus_editor = true;
        }
        ctx.request_repaint();
    }

    // 4. Markdown Preview Mode Button (📝 / ◫ / 👁)
    if app.active_note().is_some_and(|n| n.is_markdown()) {
        let md_active = app.preview_mode != crate::app::MarkdownViewMode::Edit;
        let md_btn = button::icon_button(
            ui,
            app.preview_mode.icon(),
            md_active,
            palette,
            14.0,
            HEADER_BTN_SIZE,
        );
        let md_shortcut = app.shortcut_label(crate::ui::shortcuts::ShortcutAction::ToggleMarkdown);
        if md_btn
            .on_hover_text(app.preview_mode.tooltip_with_shortcut(&md_shortcut))
            .clicked()
        {
            app.preview_mode = app.preview_mode.next();
            ctx.request_repaint();
        }
    }

    ui.separator();

    // 7. '📁' Open Folder Workspace button
    let open_folder_btn = button::icon_button(ui, "📁", false, palette, 14.0, HEADER_BTN_SIZE);
    let open_folder_tooltip = format!(
        "Open folder workspace ({})",
        app.shortcut_label(crate::ui::shortcuts::ShortcutAction::OpenFolder)
    );
    if open_folder_btn.on_hover_text(open_folder_tooltip).clicked() {
        app.show_options = false;
        app.show_search = false;
        crate::ui::drag_drop::open_folder_dialog(app);
    }

    // 8. '📥' Open File button
    let open_file_btn = button::icon_button(ui, "📥", false, palette, 14.0, HEADER_BTN_SIZE);
    let open_file_tooltip = format!(
        "Open file from disk ({})",
        app.shortcut_label(crate::ui::shortcuts::ShortcutAction::OpenFile)
    );
    if open_file_btn.on_hover_text(open_file_tooltip).clicked() {
        app.show_options = false;
        app.show_search = false;
        crate::ui::drag_drop::open_file_dialog(app);
    }

    // 9. '📂' Toggle Folder Sidebar button (visible if folder loaded)
    if app.folder_workspace.is_some() {
        let sidebar_btn = button::icon_button(
            ui,
            "📂",
            app.show_folder_sidebar,
            palette,
            14.0,
            HEADER_BTN_SIZE,
        );
        let sidebar_tooltip = format!(
            "Toggle folder sidebar ({})",
            app.shortcut_label(crate::ui::shortcuts::ShortcutAction::ToggleFolderSidebar)
        );
        if sidebar_btn.on_hover_text(sidebar_tooltip).clicked() {
            app.show_folder_sidebar = !app.show_folder_sidebar;
            ctx.request_repaint();
        }
    }

    // 10. '+' New Tab button
    let plus_btn = button::icon_button(ui, "+", false, palette, 16.0, HEADER_BTN_SIZE);
    let new_tab_tooltip = format!(
        "New tab ({})",
        app.shortcut_label(crate::ui::shortcuts::ShortcutAction::NewNote)
    );
    if plus_btn.on_hover_text(new_tab_tooltip).clicked() {
        app.show_options = false;
        app.show_search = false;
        app.create_new_note();
    }

    // 11. Custom Plugin Header Action Buttons
    if app.data.settings.plugins.enabled {
        let plugin_btns: Vec<_> = app
            .plugin_manager
            .all_header_buttons()
            .into_iter()
            .filter(|b| b.position == crate::plugins::HeaderButtonPosition::Right)
            .cloned()
            .collect();

        let mut clicked_id = None;
        for btn in plugin_btns {
            let p_btn = button::icon_button(ui, &btn.icon, false, palette, 14.0, HEADER_BTN_SIZE);
            if p_btn.on_hover_text(&btn.tooltip).clicked() {
                clicked_id = Some(btn.id.clone());
            }
        }

        if let Some(id) = clicked_id {
            app.dispatch_plugin_header_button(&id);
            ctx.request_repaint();
        }
    }
}
