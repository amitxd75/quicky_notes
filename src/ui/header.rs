//! Top header bar rendering note tabs, title editing, tab drag/pin/close, and action buttons.

use crate::app::QuickyNotesApp;
use crate::components::{badge, button, card};
use crate::theme;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, RichText, Sense, Stroke, Ui, ViewportCommand,
};

/// Renders the top header bar containing open note tabs and header action buttons.
pub fn render_header(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = theme::get_palette(&app.data.settings);

    // Solid outer header container bar
    card::glass_header_frame(&app.data.settings).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            // Left side: Note Tabs
            let active_id = app.data.active_note_id.as_deref();
            let mut tab_to_select = None;
            let mut tab_to_close = None;
            let mut tab_to_pin = None;
            let mut rename_finish = None;

            for note in &app.data.notes {
                let is_active = active_id == Some(note.id.as_str());
                let is_editing = app.editing_title_id.as_deref() == Some(note.id.as_str());

                // Smooth tab transition animation (using note.id hash directly to avoid string alloc)
                let active_anim = ctx.animate_bool_with_time(
                    egui::Id::new((&note.id, "tab_anim")),
                    is_active,
                    0.15,
                );

                // Smooth active tab background and border tinting using palette.accent
                let active_r = (palette.card.r() as f32 * (1.0 - active_anim)
                    + (palette.accent.r() as f32 * 0.35 + palette.card.r() as f32 * 0.65)
                        * active_anim) as u8;
                let active_g = (palette.card.g() as f32 * (1.0 - active_anim)
                    + (palette.accent.g() as f32 * 0.35 + palette.card.g() as f32 * 0.65)
                        * active_anim) as u8;
                let active_b = (palette.card.b() as f32 * (1.0 - active_anim)
                    + (palette.accent.b() as f32 * 0.35 + palette.card.b() as f32 * 0.65)
                        * active_anim) as u8;

                let tab_bg = Color32::from_rgba_unmultiplied(
                    active_r,
                    active_g,
                    active_b,
                    (160.0 + active_anim * 80.0) as u8,
                );

                let stroke_r = (palette.border.r() as f32 * (1.0 - active_anim)
                    + palette.accent.r() as f32 * active_anim) as u8;
                let stroke_g = (palette.border.g() as f32 * (1.0 - active_anim)
                    + palette.accent.g() as f32 * active_anim) as u8;
                let stroke_b = (palette.border.b() as f32 * (1.0 - active_anim)
                    + palette.accent.b() as f32 * active_anim) as u8;

                let tab_stroke = Stroke::new(
                    1.0 + active_anim * 0.2,
                    Color32::from_rgba_unmultiplied(
                        stroke_r,
                        stroke_g,
                        stroke_b,
                        (90.0 + active_anim * 90.0) as u8,
                    ),
                );

                let tab_frame = egui::Frame::NONE
                    .fill(tab_bg)
                    .stroke(tab_stroke)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(14, 8));

                tab_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // Pin icon indicator for pinned notes
                        if note.pinned {
                            let pin_btn = badge::pin_badge(ui, theme::ACCENT_AMBER);
                            if pin_btn.on_hover_text("Unpin note").clicked() {
                                tab_to_pin = Some(note.id.clone());
                            }
                        } else if is_active {
                            // Accent dot indicator for active tab
                            ui.label(RichText::new("●").size(11.0).color(palette.accent));
                        }

                        // Editable Title or Clickable Label
                        if is_editing {
                            let mut title_buf = note.title.clone();
                            let title_edit = egui::TextEdit::singleline(&mut title_buf)
                                .font(FontId::proportional(13.0))
                                .text_color(Color32::WHITE)
                                .frame(egui::Frame::NONE)
                                .desired_width(100.0);
                            let resp = ui.add(title_edit);
                            resp.request_focus();

                            if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                rename_finish = Some((note.id.clone(), title_buf));
                            }
                        } else {
                            let base_title = if note.title.trim().is_empty() {
                                "untitled.txt"
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
                                    ui.label(
                                        RichText::new("🔗")
                                            .font(FontId::proportional(12.0))
                                            .color(Color32::WHITE),
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
                                format!(
                                    "📁 Linked File Path:\n{}\n\n• Click to switch\n• Right-click for options / copy path\n• Double-click to rename",
                                    fp
                                )
                            } else {
                                format!(
                                    "📝 Note: {}\nQuicky Notes internal storage\n\n• Click to switch\n• Right-click for options\n• Double-click to rename",
                                    base_title
                                )
                            };
                            let tab_btn = tab_btn.on_hover_text(tooltip);

                            // Right-click context menu on tab
                            tab_btn.context_menu(|ui| {
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
                                    app.editing_title_id = Some(note.id.clone());
                                    ui.close();
                                }
                                if let Some(ref fp) = note.file_path {
                                    ui.separator();
                                    if ui.button("📋 Copy File Path").clicked() {
                                        ui.ctx().copy_text(fp.clone());
                                        ui.close();
                                    }
                                    if ui.button("📂 Open Folder in File Manager").clicked() {
                                        if let Some(parent) = std::path::Path::new(fp).parent() {
                                            let _ = std::process::Command::new("xdg-open")
                                                .arg(parent)
                                                .spawn();
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

                            if tab_btn.double_clicked() {
                                app.editing_title_id = Some(note.id.clone());
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
                app.editing_title_id = None;
            }

            if let Some(id) = tab_to_select {
                app.data.active_note_id = Some(id);
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

            // '+' New Tab button
            let plus_btn =
                button::icon_button(ui, "+", false, &palette, 16.0, egui::vec2(34.0, 32.0));

            if plus_btn.on_hover_text("New tab (Ctrl+N)").clicked() {
                app.create_new_note();
            }

            // '📥' Open File button
            let open_file_btn =
                button::icon_button(ui, "📥", false, &palette, 14.0, egui::vec2(34.0, 32.0));

            if open_file_btn
                .on_hover_text("Open file from disk (Dolphin/Picker)")
                .clicked()
            {
                crate::ui::drag_drop::open_file_dialog(app);
            }

            // Window drag area in header
            let space_rect = ui.available_size();
            let drag_resp = ui.allocate_response(space_rect, Sense::click_and_drag());
            if drag_resp.dragged() {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }

            // Right side: Action Buttons (Close, Minimize, ⚙ Settings, 🔍 Search, 👁 Markdown)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // Close Window (✕) Button
                let close_app_btn =
                    button::icon_button(ui, "✕", false, &palette, 13.0, egui::vec2(28.0, 28.0));
                if close_app_btn.on_hover_text("Close window").clicked() {
                    app.is_closing = true;
                    app.save_if_dirty();
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }

                // Minimize Window (-) Button
                let min_app_btn =
                    button::icon_button(ui, "−", false, &palette, 14.0, egui::vec2(28.0, 28.0));
                if min_app_btn.on_hover_text("Minimize").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                }

                // Options ⚙ Button
                let opt_btn = button::icon_button(
                    ui,
                    "⚙",
                    app.show_options,
                    &palette,
                    15.0,
                    egui::vec2(32.0, 30.0),
                );

                if opt_btn
                    .on_hover_text("Settings & Preferences (Ctrl+,)")
                    .clicked()
                {
                    app.show_options = !app.show_options;
                    app.show_search = false;
                    if !app.show_options {
                        app.focus_editor = true;
                    }
                    ctx.request_repaint();
                }

                // Search 🔍 Button
                let search_btn = button::icon_button(
                    ui,
                    "🔍",
                    app.show_search,
                    &palette,
                    14.0,
                    egui::vec2(32.0, 30.0),
                );

                if search_btn.on_hover_text("Search notes (Ctrl+K)").clicked() {
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

                // Markdown Preview Mode Button (📝 / ◫ / 👁) - only visible for .md files
                if app.active_note().is_some_and(|n| n.is_markdown()) {
                    let md_active = app.preview_mode != crate::app::MarkdownViewMode::Edit;
                    let md_btn = button::icon_button(
                        ui,
                        app.preview_mode.icon(),
                        md_active,
                        &palette,
                        14.0,
                        egui::vec2(32.0, 30.0),
                    );

                    if md_btn.on_hover_text(app.preview_mode.tooltip()).clicked() {
                        app.preview_mode = app.preview_mode.next();
                        ctx.request_repaint();
                    }
                }
            });
        });
    });
}
