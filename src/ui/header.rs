//! Top header bar rendering note tabs, title editing, and action buttons.

use crate::app::QuickyNotesApp;
use crate::theme;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, RichText, Sense, Stroke, Ui, ViewportCommand,
};

/// Renders the top header bar containing open note tabs and header action buttons.
pub fn render_header(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = theme::get_palette(app.data.settings.theme_mode);

    // Solid outer header container bar
    egui::Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(
            palette.bg.r(),
            palette.bg.g(),
            palette.bg.b(),
            245,
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
        .inner_margin(Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;

                // Left side: Note Tabs
                let active_id = app.data.active_note_id.clone();
                let mut tab_to_select = None;
                let mut tab_to_close = None;
                let mut tab_to_pin = None;
                let mut rename_finish = None;

                for note in &app.data.notes {
                    let is_active = active_id.as_deref() == Some(&note.id);
                    let is_editing = app.editing_title_id.as_deref() == Some(&note.id);

                    // 150ms smooth tab transition animation
                    let active_anim = ctx.animate_bool_with_time(
                        egui::Id::new(format!("tab_anim_{}", note.id)),
                        is_active,
                        0.15,
                    );

                    // Smooth active tab background and border tinting using palette.accent (no white shift)
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
                        + palette.accent.r() as f32 * active_anim)
                        as u8;
                    let stroke_g = (palette.border.g() as f32 * (1.0 - active_anim)
                        + palette.accent.g() as f32 * active_anim)
                        as u8;
                    let stroke_b = (palette.border.b() as f32 * (1.0 - active_anim)
                        + palette.accent.b() as f32 * active_anim)
                        as u8;

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
                                let pin_btn = ui.add(
                                    egui::Label::new(
                                        RichText::new("📌").size(12.5).color(theme::ACCENT_AMBER),
                                    )
                                    .sense(Sense::click()),
                                );
                                if pin_btn.on_hover_text("Unpin note").clicked() {
                                    tab_to_pin = Some(note.id.clone());
                                }
                            } else if is_active {
                                // Accent dot indicator for active tab
                                ui.label(RichText::new("●").size(11.0).color(palette.accent));
                            }

                            if is_editing {
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut app.temp_title_input)
                                        .desired_width(100.0)
                                        .font(FontId::proportional(13.5)),
                                );
                                if response.lost_focus()
                                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    rename_finish =
                                        Some((note.id.clone(), app.temp_title_input.clone()));
                                }
                            } else {
                                let title_text = if note.title.trim().is_empty() {
                                    "untitled.txt".to_string()
                                } else {
                                    note.title.clone()
                                };

                                let tab_lbl = ui.add(
                                    egui::Label::new(RichText::new(&title_text).size(14.0).color(
                                        if is_active {
                                            Color32::WHITE
                                        } else {
                                            Color32::from_gray(180)
                                        },
                                    ))
                                    .sense(Sense::click()),
                                );

                                let tab_lbl = tab_lbl.on_hover_text(if note.pinned {
                                    "Right-click to unpin note"
                                } else {
                                    "Right-click to pin note | Double-click to rename"
                                });

                                if tab_lbl.clicked() {
                                    tab_to_select = Some(note.id.clone());
                                }

                                if tab_lbl.secondary_clicked() {
                                    tab_to_pin = Some(note.id.clone());
                                }

                                if tab_lbl.double_clicked() {
                                    app.editing_title_id = Some(note.id.clone());
                                    app.temp_title_input = note.title.clone();
                                }
                            }

                            // Close tab x button
                            let close_lbl = ui.add(
                                egui::Label::new(RichText::new("×").size(15.0).color(
                                    if is_active {
                                        Color32::from_gray(220)
                                    } else {
                                        Color32::from_gray(140)
                                    },
                                ))
                                .sense(Sense::click()),
                            );

                            if close_lbl.clicked() {
                                tab_to_close = Some(note.id.clone());
                            }
                        });
                    });
                }

                if let Some((id, new_title)) = rename_finish {
                    if let Some(n) = app.data.notes.iter_mut().find(|n| n.id == id)
                        && !new_title.trim().is_empty()
                    {
                        n.title = new_title;
                        n.update_timestamp();
                        app.is_dirty = true;
                    }
                    app.editing_title_id = None;
                }

                if let Some(id) = tab_to_pin {
                    if let Some(n) = app.data.notes.iter_mut().find(|n| n.id == id) {
                        n.pinned = !n.pinned;
                        app.is_dirty = true;
                    }
                    app.data.notes.sort_by_key(|n| !n.pinned);
                }

                if let Some(id) = tab_to_select {
                    app.data.active_note_id = Some(id);
                    app.focus_editor = true;
                }

                if let Some(id) = tab_to_close {
                    app.prompt_close_note(&id);
                }

                // '+' New Tab button
                let plus_btn = ui.add(
                    egui::Button::new(RichText::new("+").size(16.0).color(theme::ACCENT_EMERALD))
                        .fill(Color32::from_rgba_unmultiplied(28, 45, 36, 200))
                        .corner_radius(CornerRadius::same(8))
                        .min_size(egui::vec2(34.0, 32.0)),
                );

                if plus_btn.on_hover_text("New tab (Ctrl+N)").clicked() {
                    app.create_new_note();
                }

                // '📥' Open File button (Zenity/Kdialog file chooser)
                let open_file_btn = ui.add(
                    egui::Button::new(RichText::new("📥").size(14.0).color(palette.accent))
                        .fill(Color32::from_rgba_unmultiplied(
                            palette.card.r(),
                            palette.card.g(),
                            palette.card.b(),
                            200,
                        ))
                        .corner_radius(CornerRadius::same(8))
                        .min_size(egui::vec2(34.0, 32.0)),
                );

                if open_file_btn
                    .on_hover_text("Open file from disk (Dolphin/Picker)")
                    .clicked()
                {
                    app.open_file_dialog();
                }

                // Window drag area in header
                let space_rect = ui.available_size();
                let drag_resp = ui.allocate_response(space_rect, Sense::click_and_drag());
                if drag_resp.dragged() {
                    ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                }

                // Right side: Action Buttons (⚙ Options, 🔍 Search)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

                    // Options ⚙ Button
                    let opt_bg = if app.show_options {
                        Color32::from_rgba_unmultiplied(
                            (palette.card.r() as u16 + 40).min(255) as u8,
                            (palette.card.g() as u16 + 35).min(255) as u8,
                            (palette.card.b() as u16 + 45).min(255) as u8,
                            255,
                        )
                    } else {
                        Color32::from_rgba_unmultiplied(
                            palette.card.r(),
                            palette.card.g(),
                            palette.card.b(),
                            180,
                        )
                    };
                    let opt_btn = ui.add(
                        egui::Button::new(RichText::new("⚙").size(15.5).color(
                            if app.show_options {
                                palette.accent
                            } else {
                                Color32::from_gray(200)
                            },
                        ))
                        .fill(opt_bg)
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(8))
                        .min_size(egui::vec2(34.0, 32.0)),
                    );

                    if opt_btn
                        .on_hover_text("Options & Settings (Ctrl+,)")
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
                    let search_bg = if app.show_search {
                        Color32::from_rgba_unmultiplied(
                            (palette.card.r() as u16 + 40).min(255) as u8,
                            (palette.card.g() as u16 + 35).min(255) as u8,
                            (palette.card.b() as u16 + 45).min(255) as u8,
                            255,
                        )
                    } else {
                        Color32::from_rgba_unmultiplied(
                            palette.card.r(),
                            palette.card.g(),
                            palette.card.b(),
                            180,
                        )
                    };
                    let search_btn = ui.add(
                        egui::Button::new(RichText::new("🔍").size(14.5).color(
                            if app.show_search {
                                palette.accent
                            } else {
                                Color32::from_gray(200)
                            },
                        ))
                        .fill(search_bg)
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(8))
                        .min_size(egui::vec2(34.0, 32.0)),
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
                });
            });
        });
}
