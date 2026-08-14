//! UI components, headers, status bars, and integrated glass drawers.

pub mod header;
pub mod options_drawer;
pub mod search_drawer;

use eframe::egui::{self, Color32, Sense, Stroke, Ui};

/// Renders a thin horizontal divider line with subtle accent transparency.
pub fn draw_horizontal_divider(ui: &mut Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), Sense::hover());
    ui.painter().line_segment(
        [rect.min, egui::pos2(rect.max.x, rect.min.y)],
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(130, 80, 190, 80)),
    );
}
