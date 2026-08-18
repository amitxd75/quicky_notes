//! Modern glassmorphic numeric stepper controls and precision value inputs.

use crate::theme::Palette;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, Ui};

/// Action triggered by interacting with the stepper buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepperAction {
    None,
    Minus,
    Plus,
}

/// Internal helper rendering a unified glass capsule stepper widget.
fn render_stepper_core(
    ui: &mut Ui,
    label: &str,
    display_str: &str,
    palette: &Palette,
) -> (egui::Response, StepperAction) {
    let mut action = StepperAction::None;
    let resp = ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .font(FontId::proportional(12.5))
                .color(Color32::from_gray(230)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let container_size = egui::vec2(116.0, 24.0);
            let (container_rect, _) = ui.allocate_exact_size(container_size, Sense::hover());

            // 1. Unified Glass Background & Subtle Stroke
            ui.painter().rect_filled(
                container_rect,
                CornerRadius::same(6),
                Palette::with_alpha(palette.card, 150),
            );
            ui.painter().rect_stroke(
                container_rect,
                CornerRadius::same(6),
                Stroke::new(1.0, Palette::with_alpha(palette.border, 45)),
                egui::StrokeKind::Inside,
            );

            // 2. Segment Geometry: Minus (Left 26px), Value (Middle 64px), Plus (Right 26px)
            let btn_width = 26.0;
            let minus_rect = egui::Rect::from_min_size(
                container_rect.min,
                egui::vec2(btn_width, container_size.y),
            );
            let plus_rect = egui::Rect::from_min_size(
                egui::pos2(container_rect.max.x - btn_width, container_rect.min.y),
                egui::vec2(btn_width, container_size.y),
            );
            let val_rect = egui::Rect::from_min_max(
                egui::pos2(minus_rect.max.x, container_rect.min.y),
                egui::pos2(plus_rect.min.x, container_rect.max.y),
            );

            // 3. Interactive Plus Button
            let plus_resp =
                ui.interact(plus_rect, ui.id().with(label).with("plus"), Sense::click());
            if plus_resp.hovered() || plus_resp.is_pointer_button_down_on() {
                let fill = if plus_resp.is_pointer_button_down_on() {
                    Palette::with_alpha(palette.accent, 140)
                } else {
                    Palette::lighten(palette.card, 30, 220)
                };
                ui.painter().rect_filled(
                    plus_rect,
                    CornerRadius {
                        nw: 0,
                        ne: 6,
                        se: 6,
                        sw: 0,
                    },
                    fill,
                );
            }
            ui.painter().text(
                plus_rect.center(),
                egui::Align2::CENTER_CENTER,
                "+",
                FontId::proportional(14.0),
                if plus_resp.hovered() {
                    Color32::WHITE
                } else {
                    Color32::from_gray(210)
                },
            );
            if plus_resp.clicked() {
                action = StepperAction::Plus;
            }

            // 4. Value Text in Center
            ui.painter().text(
                val_rect.center(),
                egui::Align2::CENTER_CENTER,
                display_str,
                FontId::monospace(11.0),
                Color32::WHITE,
            );

            // 5. Interactive Minus Button
            let minus_resp = ui.interact(
                minus_rect,
                ui.id().with(label).with("minus"),
                Sense::click(),
            );
            if minus_resp.hovered() || minus_resp.is_pointer_button_down_on() {
                let fill = if minus_resp.is_pointer_button_down_on() {
                    Palette::with_alpha(palette.accent, 140)
                } else {
                    Palette::lighten(palette.card, 30, 220)
                };
                ui.painter().rect_filled(
                    minus_rect,
                    CornerRadius {
                        nw: 6,
                        ne: 0,
                        se: 0,
                        sw: 6,
                    },
                    fill,
                );
            }
            ui.painter().text(
                minus_rect.center(),
                egui::Align2::CENTER_CENTER,
                "−",
                FontId::proportional(14.0),
                if minus_resp.hovered() {
                    Color32::WHITE
                } else {
                    Color32::from_gray(210)
                },
            );
            if minus_resp.clicked() {
                action = StepperAction::Minus;
            }

            // Subtle vertical dividers between buttons and value
            let divider_stroke = Stroke::new(1.0, Palette::with_alpha(palette.border, 35));
            ui.painter().line_segment(
                [minus_rect.right_top(), minus_rect.right_bottom()],
                divider_stroke,
            );
            ui.painter().line_segment(
                [plus_rect.left_top(), plus_rect.left_bottom()],
                divider_stroke,
            );
        });
    });

    (resp.response, action)
}

/// Renders a labeled stepper row for `usize` values with `[-]` and `[+]` controls.
pub fn stepper_row_usize(
    ui: &mut Ui,
    label: &str,
    value: &mut usize,
    range: std::ops::RangeInclusive<usize>,
    step: usize,
    unit_suffix: &str,
    palette: &Palette,
) -> egui::Response {
    let min = *range.start();
    let max = *range.end();
    let display_text = if unit_suffix.is_empty() {
        format!("{}", value)
    } else {
        format!("{} {}", value, unit_suffix)
    };

    let (mut response, action) = render_stepper_core(ui, label, &display_text, palette);
    match action {
        StepperAction::Minus => {
            *value = value.saturating_sub(step).max(min);
            response.mark_changed();
        }
        StepperAction::Plus => {
            *value = (*value + step).min(max);
            response.mark_changed();
        }
        StepperAction::None => {}
    }
    response
}

/// Renders a labeled stepper row for `u32` values with `[-]` and `[+]` controls.
pub fn stepper_row_u32(
    ui: &mut Ui,
    label: &str,
    value: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    step: u32,
    unit_suffix: &str,
    palette: &Palette,
) -> egui::Response {
    let mut usize_val = *value as usize;
    let usize_range = (*range.start() as usize)..=(*range.end() as usize);
    let resp = stepper_row_usize(
        ui,
        label,
        &mut usize_val,
        usize_range,
        step as usize,
        unit_suffix,
        palette,
    );
    if resp.changed() {
        *value = usize_val as u32;
    }
    resp
}

/// Renders a labeled stepper row for `u64` values with custom format strings.
pub fn stepper_row_u64(
    ui: &mut Ui,
    label: &str,
    value: &mut u64,
    range: std::ops::RangeInclusive<u64>,
    step: u64,
    display_str: &str,
    palette: &Palette,
) -> egui::Response {
    let min = *range.start();
    let max = *range.end();

    let (mut response, action) = render_stepper_core(ui, label, display_str, palette);
    match action {
        StepperAction::Minus => {
            *value = value.saturating_sub(step).max(min);
            response.mark_changed();
        }
        StepperAction::Plus => {
            *value = (*value + step).min(max);
            response.mark_changed();
        }
        StepperAction::None => {}
    }
    response
}

/// Renders a labeled stepper row for floating point seconds/durations.
pub fn stepper_row_f32(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    step: f32,
    display_str: &str,
    palette: &Palette,
) -> egui::Response {
    let min = *range.start();
    let max = *range.end();

    let (mut response, action) = render_stepper_core(ui, label, display_str, palette);
    match action {
        StepperAction::Minus => {
            *value = (*value - step).max(min);
            response.mark_changed();
        }
        StepperAction::Plus => {
            *value = (*value + step).min(max);
            response.mark_changed();
        }
        StepperAction::None => {}
    }
    response
}
