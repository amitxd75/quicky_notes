//! Search & Browse notes drawer component.

use crate::app::QuickyNotesApp;
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, Ui};

/// Renders the inline glass Search & Browse notes drawer.
pub fn render_search_drawer(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = theme::get_palette(app.data.settings.theme_mode);

    ui.vertical(|ui| {
        ui.add_space(4.0);

        // Header bar in Search drawer
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("🔍 Search Notes")
                    .font(FontId::proportional(15.0))
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close_btn = ui.add(
                    egui::Button::new(
                        RichText::new("Close  ✕")
                            .font(FontId::proportional(12.5))
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgba_unmultiplied(
                        palette.card.r(),
                        palette.card.g(),
                        palette.card.b(),
                        220,
                    ))
                    .stroke(Stroke::new(1.0_f32, palette.border))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(egui::vec2(72.0, 28.0)),
                );
                if close_btn.clicked() {
                    app.show_search = false;
                    app.focus_editor = true;
                    ctx.request_repaint();
                }
            });
        });

        ui.add_space(6.0);

        // Search input field
        let search_frame = egui::Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(
                palette.bg.r(),
                palette.bg.g(),
                palette.bg.b(),
                220,
            ))
            .stroke(Stroke::new(1.2_f32, palette.border))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(10, 6));

        search_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🔍")
                        .size(12.0)
                        .color(Color32::from_gray(160)),
                );
                let search_edit = egui::TextEdit::singleline(&mut app.search_query)
                    .hint_text("Type to search notes...")
                    .text_color(Color32::WHITE)
                    .frame(egui::Frame::NONE)
                    .desired_width(ui.available_width());

                let resp = ui.add(search_edit);
                if app.focus_search {
                    resp.request_focus();
                    app.focus_search = false;
                }
            });
        });

        ui.add_space(8.0);

        // Search Results List
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 40.0)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;

                let query = app.search_query.trim().to_lowercase();
                let active_id = app.data.active_note_id.clone();
                let mut note_to_select = None;

                let filtered_notes: Vec<_> = app
                    .data
                    .notes
                    .iter()
                    .filter(|n| {
                        query.is_empty()
                            || n.title.to_lowercase().contains(&query)
                            || n.content.to_lowercase().contains(&query)
                    })
                    .collect();

                let filtered_count = filtered_notes.len();
                if filtered_count > 0 && app.search_selected_idx >= filtered_count {
                    app.search_selected_idx = 0;
                }

                if filtered_notes.is_empty() {
                    ui.label(
                        RichText::new("No matching notes found")
                            .size(12.0)
                            .color(Color32::from_gray(160)),
                    );
                } else {
                    for (idx, note) in filtered_notes.iter().enumerate() {
                        let is_highlighted = idx == app.search_selected_idx;
                        let is_active = active_id.as_deref() == Some(&note.id);

                        let (btn_bg, stroke, text_color) = if is_highlighted {
                            (
                                Color32::from_rgba_unmultiplied(
                                    (palette.card.r() as u16 + 40).min(255) as u8,
                                    (palette.card.g() as u16 + 35).min(255) as u8,
                                    (palette.card.b() as u16 + 45).min(255) as u8,
                                    240,
                                ),
                                Stroke::new(1.2_f32, palette.accent),
                                Color32::WHITE,
                            )
                        } else if is_active {
                            (
                                Color32::from_rgba_unmultiplied(
                                    palette.card.r(),
                                    palette.card.g(),
                                    palette.card.b(),
                                    210,
                                ),
                                Stroke::NONE,
                                Color32::WHITE,
                            )
                        } else {
                            (
                                Color32::from_rgba_unmultiplied(
                                    palette.bg.r(),
                                    palette.bg.g(),
                                    palette.bg.b(),
                                    140,
                                ),
                                Stroke::NONE,
                                Color32::from_gray(200),
                            )
                        };

                        let prefix = if note.pinned {
                            if is_highlighted {
                                "➔ 📌"
                            } else {
                                "    📌"
                            }
                        } else if is_highlighted {
                            "➔ 📄"
                        } else {
                            "    📄"
                        };

                        let note_btn = ui.add(
                            egui::Button::new(
                                RichText::new(format!(
                                    "{}  {}   ({})",
                                    prefix,
                                    note.title,
                                    note.display_time()
                                ))
                                .font(FontId::proportional(12.5))
                                .color(text_color),
                            )
                            .fill(btn_bg)
                            .stroke(stroke)
                            .corner_radius(CornerRadius::same(6))
                            .min_size(egui::vec2(ui.available_width(), 30.0)),
                        );

                        if note_btn.clicked() {
                            app.search_selected_idx = idx;
                            note_to_select = Some(note.id.clone());
                        }
                    }
                }

                if let Some(id) = note_to_select {
                    app.data.active_note_id = Some(id);
                    app.show_search = false;
                    app.focus_editor = true;
                    ctx.request_repaint();
                }
            });
    });
}
