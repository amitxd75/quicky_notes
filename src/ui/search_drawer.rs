//! Search & Browse notes drawer component with real-time filtering.

use crate::app::QuickyNotesApp;
use crate::components::{button, card, input};
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, Ui};

/// Returns true if note matches search query (case-insensitive title/content substring).
pub fn note_matches_query(title: &str, content: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    title.to_lowercase().contains(query) || content.to_lowercase().contains(query)
}

/// Renders the inline glass Search & Browse notes drawer.
pub fn render_search_drawer(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = app.active_palette();

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
                let close_btn =
                    button::close_button(ui, &palette).on_hover_text("Close search (Esc)");
                if close_btn.clicked() {
                    app.show_search = false;
                    app.focus_editor = true;
                    ctx.request_repaint();
                }
            });
        });

        ui.add_space(6.0);

        // Search input field
        card::glass_input_frame(&app.data.settings).show(ui, |ui| {
            ui.horizontal(|ui| {
                input::search_bar_icon(ui);
                let focus = app.focus_search;
                input::search_input(ui, &mut app.search_query, "Type to search notes...", focus);
                if focus {
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
                let active_id = app.data.active_note_id.as_deref();
                let mut note_to_select = None;

                let matching_notes: Vec<_> = app
                    .data
                    .notes
                    .iter()
                    .filter(|n| note_matches_query(&n.title, &n.content, &query))
                    .collect();

                let filtered_count = matching_notes.len();
                if filtered_count > 0 && app.search_selected_idx >= filtered_count {
                    app.search_selected_idx = 0;
                }

                if matching_notes.is_empty() {
                    ui.label(
                        RichText::new("No matching notes found")
                            .size(12.0)
                            .color(Color32::from_gray(160)),
                    );
                } else {
                    for (idx, note) in matching_notes.iter().enumerate() {
                        let is_highlighted = idx == app.search_selected_idx;
                        let is_active = active_id == Some(note.id.as_str());

                        let (btn_bg, stroke, text_color) = if is_highlighted {
                            (
                                theme::Palette::lighten(palette.card, 40, 240),
                                Stroke::new(1.2_f32, palette.accent),
                                Color32::WHITE,
                            )
                        } else if is_active {
                            (
                                theme::Palette::with_alpha(palette.card, 210),
                                Stroke::NONE,
                                Color32::WHITE,
                            )
                        } else {
                            (
                                theme::Palette::with_alpha(palette.bg, 140),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_matches_query() {
        assert!(note_matches_query("shopping.txt", "Milk, Eggs", ""));
        assert!(note_matches_query("shopping.txt", "Milk, Eggs", "shop"));
        assert!(note_matches_query("shopping.txt", "Milk, Eggs", "eggs"));
        assert!(!note_matches_query("shopping.txt", "Milk, Eggs", "bread"));
    }
}
