//! Horizontal and vertical divider components.

use eframe::egui::{self, Color32, Sense, Stroke, Ui};

/// Renders a thin, subtle horizontal divider line that does not clash with the window border.
pub fn horizontal_divider(ui: &mut Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), Sense::hover());
    let stroke_color = Color32::from_rgba_unmultiplied(255, 255, 255, 20);
    ui.painter().line_segment(
        [rect.min, egui::pos2(rect.max.x, rect.min.y)],
        Stroke::new(1.0_f32, stroke_color),
    );
}
