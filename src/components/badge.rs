//! Badges and status dot indicators.

use crate::theme::Palette;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, Ui};

/// Renders a small 10pt colored status indicator dot.
#[allow(dead_code)]
pub fn status_dot(ui: &mut Ui, color: Color32) -> egui::Response {
    ui.label(RichText::new("●").size(10.0).color(color))
}

/// Renders an interactive pin badge indicator icon for tab pinning.
pub fn pin_badge(ui: &mut Ui) -> egui::Response {
    ui.add(
        egui::Label::new(RichText::new("📌").size(12.5).color(Color32::WHITE))
            .sense(Sense::click()),
    )
}

/// Renders a small glass shortcut or capability badge.
pub fn shortcut_badge(ui: &mut Ui, label: &str, palette: &Palette) -> egui::Response {
    let font_id = FontId::monospace(10.5);
    let row = ui
        .painter()
        .layout_no_wrap(label.to_string(), font_id.clone(), Color32::WHITE);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(row.size().x + 10.0, 18.0), Sense::hover());

    ui.painter().rect_filled(
        rect,
        CornerRadius::same(4),
        Palette::with_alpha(palette.card, 210),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0, Palette::with_alpha(palette.border, 90)),
        egui::StrokeKind::Outside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font_id,
        Color32::from_gray(210),
    );

    response
}

/// Renders a custom colored badge pill.
pub fn custom_color_badge(ui: &mut Ui, label: &str, accent: Color32) -> egui::Response {
    let font_id = FontId::monospace(10.5);
    let row = ui
        .painter()
        .layout_no_wrap(label.to_string(), font_id.clone(), Color32::WHITE);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(row.size().x + 10.0, 18.0), Sense::hover());

    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), Palette::with_alpha(accent, 70));
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0, Palette::with_alpha(accent, 150)),
        egui::StrokeKind::Outside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font_id,
        Color32::WHITE,
    );

    response
}
