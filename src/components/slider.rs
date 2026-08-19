//! Modern glassmorphic slider controls with glowing knobs and accent filled tracks.

use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, Ui};

/// Renders a modern glassmorphic slider with an accent filled track and white knob.
pub fn glass_slider(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    accent: Color32,
    desired_size: egui::Vec2,
) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let min = *range.start();
    let max = *range.end();

    if (response.clicked() || response.dragged())
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let norm = ((mouse_pos.x - (rect.left() + 7.0)) / (rect.width() - 14.0)).clamp(0.0, 1.0);
        *value = min + norm * (max - min);
        response.mark_changed();
    }

    let norm = ((*value - min) / (max - min)).clamp(0.0, 1.0);

    // Track background
    let track_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left(), rect.center().y - 3.5),
        egui::pos2(rect.right(), rect.center().y + 3.5),
    );
    ui.painter().rect_filled(
        track_rect,
        CornerRadius::same(4),
        Color32::from_rgba_unmultiplied(45, 30, 65, 220),
    );
    ui.painter().rect_stroke(
        track_rect,
        CornerRadius::same(4),
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(100, 70, 140, 100)),
        egui::StrokeKind::Inside,
    );

    // Filled track
    let fill_right = rect.left() + 7.0 + norm * (rect.width() - 14.0);
    let filled_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left(), rect.center().y - 3.5),
        egui::pos2(fill_right, rect.center().y + 3.5),
    );
    ui.painter()
        .rect_filled(filled_rect, CornerRadius::same(4), accent);

    // Knob
    let knob_pos = egui::pos2(fill_right, rect.center().y);
    let is_active = response.hovered() || response.dragged();
    let knob_radius = if is_active { 8.0 } else { 7.0 };

    if is_active {
        ui.painter().circle_filled(
            knob_pos,
            knob_radius + 3.0,
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 60),
        );
    }

    ui.painter()
        .circle_filled(knob_pos, knob_radius, Color32::WHITE);
    ui.painter()
        .circle_stroke(knob_pos, knob_radius, Stroke::new(1.5_f32, accent));

    response
}

/// Renders integer slider using glass_slider.
pub fn glass_slider_u32(
    ui: &mut Ui,
    value: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    accent: Color32,
    desired_size: egui::Vec2,
) -> egui::Response {
    let mut float_val = *value as f32;
    let float_range = (*range.start() as f32)..=(*range.end() as f32);
    let resp = glass_slider(ui, &mut float_val, float_range, accent, desired_size);
    if resp.changed() {
        *value = float_val.round() as u32;
    }
    resp
}

/// Renders a labeled slider row with left title, right value text, and glass slider.
pub fn slider_row(
    ui: &mut Ui,
    label: &str,
    value_display: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    accent: Color32,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .font(FontId::proportional(12.5))
                .color(Color32::from_gray(230)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(2.0);
            ui.label(
                RichText::new(value_display)
                    .font(FontId::monospace(12.0))
                    .color(Color32::from_gray(190)),
            );
            ui.add_space(4.0);
            glass_slider(ui, value, range, accent, egui::vec2(150.0, 20.0))
        })
        .inner
    })
    .inner
}

/// Renders a labeled slider row for floating-point percentages (alias for `slider_row`).
pub fn slider_row_percent(
    ui: &mut Ui,
    label: &str,
    value_display: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    accent: Color32,
) -> egui::Response {
    slider_row(ui, label, value_display, value, range, accent)
}

/// Renders a labeled slider row for integer values.
pub fn slider_row_u32(
    ui: &mut Ui,
    label: &str,
    value_display: &str,
    value: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    accent: Color32,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .font(FontId::proportional(12.5))
                .color(Color32::from_gray(230)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(value_display)
                    .font(FontId::monospace(12.0))
                    .color(Color32::from_gray(190)),
            );
            ui.add_space(4.0);
            glass_slider_u32(ui, value, range, accent, egui::vec2(150.0, 20.0))
        })
        .inner
    })
    .inner
}

/// Renders a labeled slider row for usize values.
pub fn slider_row_usize(
    ui: &mut Ui,
    label: &str,
    value_display: &str,
    value: &mut usize,
    range: std::ops::RangeInclusive<usize>,
    accent: Color32,
) -> egui::Response {
    let mut u32_val = *value as u32;
    let u32_range = (*range.start() as u32)..=(*range.end() as u32);
    let resp = slider_row_u32(ui, label, value_display, &mut u32_val, u32_range, accent);
    if resp.changed() {
        *value = u32_val as usize;
    }
    resp
}

/// Renders a labeled slider row for u64 values.
pub fn slider_row_u64(
    ui: &mut Ui,
    label: &str,
    value_display: &str,
    value: &mut u64,
    range: std::ops::RangeInclusive<u64>,
    accent: Color32,
) -> egui::Response {
    let mut float_val = *value as f32;
    let float_range = (*range.start() as f32)..=(*range.end() as f32);
    let resp = slider_row(
        ui,
        label,
        value_display,
        &mut float_val,
        float_range,
        accent,
    );
    if resp.changed() {
        *value = float_val.round() as u64;
    }
    resp
}
