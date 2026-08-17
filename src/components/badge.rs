//! Badges and status dot indicators.

use eframe::egui::{self, Color32, RichText, Sense, Ui};

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
