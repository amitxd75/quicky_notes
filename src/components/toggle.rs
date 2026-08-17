//! Animated toggle switch components.

use crate::theme::Palette;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, Ui};

/// Renders a modern smooth animated pill toggle switch matching modern glassmorphism UI.
pub fn toggle_switch(ui: &mut Ui, on: &mut bool, accent: Color32) -> egui::Response {
    let desired_size = egui::vec2(38.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
    let off_bg = Color32::from_rgba_unmultiplied(50, 35, 70, 200);
    let bg_color = Palette::interpolate_color(off_bg, accent, how_on);

    ui.painter()
        .rect_filled(rect, CornerRadius::same(10), bg_color);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(100, 60, 140, 120)),
        egui::StrokeKind::Outside,
    );

    let circle_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), how_on);
    let center = egui::pos2(circle_x, rect.center().y);
    ui.painter().circle_filled(center, 7.0, Color32::WHITE);

    response
}

/// Renders a labeled toggle setting row with left title and right-aligned animated switch.
pub fn toggle_row(ui: &mut Ui, label: &str, on: &mut bool, accent: Color32) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .font(FontId::proportional(12.5))
                .color(Color32::from_gray(230)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            toggle_switch(ui, on, accent)
        })
        .inner
    })
    .inner
}
