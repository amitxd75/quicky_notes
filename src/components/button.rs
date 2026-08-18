//! Standardized animated and styled UI buttons.

use crate::theme::Palette;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, Ui};

/// Renders a selectable glass pill button with smooth animated transitions and checkmark if selected.
pub fn selection_pill(
    ui: &mut Ui,
    label: &str,
    is_selected: bool,
    palette: &Palette,
) -> egui::Response {
    let pill_id = ui.make_persistent_id(label);
    let anim = ui.ctx().animate_bool_responsive(pill_id, is_selected);

    let unselected_bg = Palette::with_alpha(palette.card, 150);
    let selected_bg = Palette::lighten(palette.card, 45, 245);
    let fill = Palette::interpolate_color(unselected_bg, selected_bg, anim);

    let stroke_alpha = ((80.0 * (1.0 - anim) + 255.0 * anim) as u8).clamp(80, 255);
    let stroke_color = Color32::from_rgba_unmultiplied(
        palette.accent.r(),
        palette.accent.g(),
        palette.accent.b(),
        stroke_alpha,
    );

    let display_text = if is_selected {
        format!("{}  ✓", label)
    } else {
        label.to_string()
    };

    ui.add(
        egui::Button::new(
            RichText::new(&display_text)
                .font(FontId::proportional(11.5))
                .color(Color32::WHITE),
        )
        .fill(fill)
        .stroke(Stroke::new(1.0 + 0.2 * anim, stroke_color))
        .corner_radius(CornerRadius::same(6))
        .min_size(egui::vec2(0.0, 28.0)),
    )
}

/// Renders a compact, clickable glass inspiration or tag chip.
pub fn glass_chip_button(ui: &mut Ui, label: &str, palette: &Palette) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .font(FontId::proportional(11.0))
                .color(Color32::WHITE),
        )
        .fill(Palette::with_alpha(palette.card, 180))
        .stroke(Stroke::new(1.0, Palette::with_alpha(palette.border, 120)))
        .corner_radius(CornerRadius::same(6)),
    )
}

/// Renders a labeled row with a title on the left and a right-aligned group of selection pills.
pub fn selection_row<'a, T: PartialEq + Clone>(
    ui: &mut Ui,
    label: &str,
    current_value: &mut T,
    options: impl IntoIterator<Item = (&'a str, T)>,
    palette: &Palette,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .font(FontId::proportional(12.5))
                .color(Color32::from_gray(230)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for (opt_label, val) in options {
                let is_active = *current_value == val;
                if selection_pill(ui, opt_label, is_active, palette).clicked() {
                    *current_value = val;
                    changed = true;
                }
            }
        });
    });
    changed
}

/// Renders a dynamic, animated interactive button with smooth hover glow and tactile click depression.
pub fn animated_action_button(
    ui: &mut Ui,
    label: &str,
    palette: &Palette,
    min_size: egui::Vec2,
) -> egui::Response {
    let (mut rect, response) = ui.allocate_exact_size(min_size, Sense::click());
    let is_hovered = response.hovered();
    let is_pressed = response.is_pointer_button_down_on();

    let hov_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("hov"), is_hovered);
    let press_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("press"), is_pressed);

    if press_anim > 0.01 {
        rect = rect.translate(egui::vec2(0.0, 1.0 * press_anim));
    }

    let base_bg = Palette::with_alpha(palette.card, 210);
    let hovered_bg = Palette::lighten(palette.card, 35, 245);
    let bg = Palette::interpolate_color(base_bg, hovered_bg, hov_anim);

    let base_border = Color32::from_rgba_unmultiplied(
        palette.border.r(),
        palette.border.g(),
        palette.border.b(),
        120,
    );
    let active_border = palette.accent;
    let border_color = Palette::interpolate_color(base_border, active_border, hov_anim);

    ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0 + 0.3 * hov_anim, border_color),
        egui::StrokeKind::Outside,
    );

    let text_color = Palette::interpolate_color(Color32::from_gray(215), Color32::WHITE, hov_anim);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.0),
        text_color,
    );

    response
}

/// Renders a dynamic primary action button filled with the active theme accent color.
pub fn animated_primary_button(
    ui: &mut Ui,
    label: &str,
    palette: &Palette,
    min_size: egui::Vec2,
) -> egui::Response {
    let (mut rect, response) = ui.allocate_exact_size(min_size, Sense::click());
    let is_hovered = response.hovered();
    let is_pressed = response.is_pointer_button_down_on();

    let hov_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("hov"), is_hovered);
    let press_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("press"), is_pressed);

    if press_anim > 0.01 {
        rect = rect.translate(egui::vec2(0.0, 1.0 * press_anim));
    }

    let base_bg = Palette::with_alpha(palette.accent, 220);
    let hovered_bg = Palette::lighten(palette.accent, 30, 255);
    let bg = Palette::interpolate_color(base_bg, hovered_bg, hov_anim);

    let border_color = Palette::interpolate_color(
        Palette::with_alpha(palette.accent, 160),
        Color32::WHITE,
        hov_anim * 0.4,
    );

    ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0 + 0.3 * hov_anim, border_color),
        egui::StrokeKind::Outside,
    );

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.0),
        Color32::WHITE,
    );

    response
}

/// Renders a dynamic danger button with vivid pulsing red hover and tactile press feedback.
pub fn animated_danger_button(ui: &mut Ui, label: &str, min_size: egui::Vec2) -> egui::Response {
    let (mut rect, response) = ui.allocate_exact_size(min_size, Sense::click());
    let is_hovered = response.hovered();
    let is_pressed = response.is_pointer_button_down_on();

    let hov_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("dang_hov"), is_hovered);
    let press_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("dang_press"), is_pressed);

    if press_anim > 0.01 {
        rect = rect.translate(egui::vec2(0.0, 1.0 * press_anim));
    }

    let base_bg = Color32::from_rgba_unmultiplied(170, 25, 25, 230);
    let hover_bg = Color32::from_rgba_unmultiplied(225, 38, 38, 255);
    let bg = Palette::interpolate_color(base_bg, hover_bg, hov_anim);

    let base_border = Color32::from_rgba_unmultiplied(220, 38, 38, 160);
    let hover_border = Color32::from_rgba_unmultiplied(252, 165, 165, 255);
    let border_color = Palette::interpolate_color(base_border, hover_border, hov_anim);

    ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0 + 0.4 * hov_anim, border_color),
        egui::StrokeKind::Outside,
    );

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.0),
        Color32::WHITE,
    );

    response
}

/// Renders a dynamic keybinding badge button with interactive states and recording pulse.
pub fn animated_shortcut_badge(
    ui: &mut Ui,
    text: &str,
    is_recording: bool,
    is_modified: bool,
    palette: &Palette,
    min_size: egui::Vec2,
) -> egui::Response {
    let (mut rect, response) = ui.allocate_exact_size(min_size, Sense::click());
    let is_hovered = response.hovered();
    let is_pressed = response.is_pointer_button_down_on();

    let hov_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("sc_hov"), is_hovered);
    let press_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("sc_press"), is_pressed);

    if press_anim > 0.01 && !is_recording {
        rect = rect.translate(egui::vec2(0.0, 1.0 * press_anim));
    }

    if is_recording {
        ui.ctx().request_repaint();
        let t = ui.input(|i| i.time);
        let wave = (t * 4.0).sin().abs() as f32;

        let pulse_bg = Palette::interpolate_color(
            palette.accent,
            Palette::lighten(palette.accent, 30, 255),
            wave,
        );
        ui.painter()
            .rect_filled(rect, CornerRadius::same(6), pulse_bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(6),
            Stroke::new(1.4, Color32::WHITE),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "⌨ Press keys...",
            FontId::proportional(11.5),
            Color32::WHITE,
        );
    } else {
        let base_bg = Palette::with_alpha(palette.card, 220);
        let hover_bg = Palette::lighten(palette.card, 40, 245);
        let bg = Palette::interpolate_color(base_bg, hover_bg, hov_anim);

        let base_border = if is_modified {
            palette.accent
        } else {
            Palette::with_alpha(palette.border, 140)
        };
        let hover_border = palette.accent;
        let border_color = Palette::interpolate_color(base_border, hover_border, hov_anim);

        ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(6),
            Stroke::new(1.0 + 0.3 * hov_anim, border_color),
            egui::StrokeKind::Outside,
        );

        let text_color = if is_modified {
            Palette::interpolate_color(palette.accent, Color32::WHITE, hov_anim)
        } else {
            Palette::interpolate_color(Color32::from_gray(210), Color32::WHITE, hov_anim)
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            FontId::monospace(11.5),
            text_color,
        );
    }

    response
}

/// Renders a small animated revert/reset button.
pub fn animated_revert_button(ui: &mut Ui, palette: &Palette) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), Sense::click());
    let is_hovered = response.hovered();
    let hov_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("rev_hov"), is_hovered);

    let bg = Palette::interpolate_color(
        Color32::TRANSPARENT,
        Palette::with_alpha(palette.card, 200),
        hov_anim,
    );
    let border = Palette::interpolate_color(Color32::TRANSPARENT, palette.border, hov_anim);

    ui.painter().rect_filled(rect, CornerRadius::same(4), bg);
    if hov_anim > 0.05 {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, border),
            egui::StrokeKind::Outside,
        );
    }

    let icon_color = Palette::interpolate_color(Color32::from_gray(160), Color32::WHITE, hov_anim);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "↺",
        FontId::proportional(12.0),
        icon_color,
    );

    response
}

/// Renders a square icon button for header bars with smooth hover & active animations.
pub fn icon_button(
    ui: &mut Ui,
    icon: &str,
    is_active: bool,
    palette: &Palette,
    font_size: f32,
    size: egui::Vec2,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let is_hovered = response.hovered();

    let hov_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("hov"), is_hovered);
    let act_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("act"), is_active);

    let normal_bg =
        Color32::from_rgba_unmultiplied(palette.card.r(), palette.card.g(), palette.card.b(), 180);
    let active_bg = Color32::from_rgba_unmultiplied(
        (palette.card.r() as u16 + 40).min(255) as u8,
        (palette.card.g() as u16 + 30).min(255) as u8,
        (palette.card.b() as u16 + 50).min(255) as u8,
        240,
    );
    let hovered_bg = Color32::from_rgba_unmultiplied(
        (palette.card.r() as u16 + 25).min(255) as u8,
        (palette.card.g() as u16 + 25).min(255) as u8,
        (palette.card.b() as u16 + 35).min(255) as u8,
        210,
    );

    let bg = if act_anim > 0.01 {
        Palette::interpolate_color(normal_bg, active_bg, act_anim)
    } else {
        Palette::interpolate_color(normal_bg, hovered_bg, hov_anim)
    };

    let stroke_alpha = (255.0 * act_anim.max(hov_anim * 0.5)) as u8;
    let stroke = if stroke_alpha > 5 {
        Stroke::new(
            1.0 + 0.2 * act_anim,
            Color32::from_rgba_unmultiplied(
                palette.accent.r(),
                palette.accent.g(),
                palette.accent.b(),
                stroke_alpha,
            ),
        )
    } else {
        Stroke::NONE
    };

    ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
    if stroke != Stroke::NONE {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(8),
            stroke,
            egui::StrokeKind::Outside,
        );
    }

    let text_color = Color32::WHITE;

    if icon == "✕" || icon == "×" || icon == "x" {
        // Render crisp anti-aliased vector cross lines instead of font glyphs
        // preventing missing font glyph box '[]' rendering across all platforms and font configs
        let center = rect.center();
        let half_cross = (font_size * 0.42).round().max(3.5);
        let cross_stroke = Stroke::new(1.65_f32, text_color);
        ui.painter().line_segment(
            [
                egui::pos2(center.x - half_cross, center.y - half_cross),
                egui::pos2(center.x + half_cross, center.y + half_cross),
            ],
            cross_stroke,
        );
        ui.painter().line_segment(
            [
                egui::pos2(center.x + half_cross, center.y - half_cross),
                egui::pos2(center.x - half_cross, center.y + half_cross),
            ],
            cross_stroke,
        );
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(font_size),
            text_color,
        );
    }

    response
}

/// Renders a drawer close button ("Close ×").
pub fn close_button(ui: &mut Ui, palette: &Palette) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new("Close  ×")
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
    )
}
