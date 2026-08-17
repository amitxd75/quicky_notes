//! Text and search input components.

use eframe::egui::{self, Color32, RichText, Ui};

/// Renders a framed search text input field with optional auto-focus.
pub fn search_input(ui: &mut Ui, query: &mut String, hint: &str, focus: bool) -> egui::Response {
    let search_edit = egui::TextEdit::singleline(query)
        .hint_text(hint)
        .text_color(Color32::WHITE)
        .frame(egui::Frame::NONE)
        .desired_width(ui.available_width());

    let resp = ui.add(search_edit);
    if focus {
        resp.request_focus();
    }
    resp
}

/// Renders an icon-prefixed search bar wrapper icon.
pub fn search_bar_icon(ui: &mut Ui) {
    ui.label(
        RichText::new("🔍")
            .size(12.0)
            .color(Color32::from_gray(160)),
    );
}
