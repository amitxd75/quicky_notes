//! Standardized animated and styled UI buttons.

use crate::theme::Palette;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, Ui};

/// Renders a selectable glass pill button with smooth animated transitions, checkmark, and stable responsive dimensions.
pub fn selection_pill(
    ui: &mut Ui,
    label: &str,
    is_selected: bool,
    palette: &Palette,
) -> egui::Response {
    let pill_id = ui.make_persistent_id(label);
    let sel_anim = ui
        .ctx()
        .animate_bool_responsive(pill_id.with("sel"), is_selected);

    let font_size = 11.5_f32;
    let label_font = FontId::proportional(font_size);
    let label_galley =
        ui.painter()
            .layout_no_wrap(label.to_string(), label_font.clone(), Color32::WHITE);
    let check_galley =
        ui.painter()
            .layout_no_wrap("✓".to_string(), label_font.clone(), Color32::WHITE);

    // Compute stable, responsive width that does NOT jump or resize when selected/enabled
    let label_w = label_galley.size().x;
    let check_w = check_galley.size().x;
    let icon_gap = 5.0_f32;
    let min_size = egui::vec2((label_w + check_w + icon_gap + 20.0).max(46.0), 28.0);
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

    let unselected_bg = Palette::with_alpha(palette.card, 150);
    let hovered_bg = Palette::lighten(palette.card, 25, 210);
    let selected_bg = Palette::lighten(palette.card, 45, 245);

    let bg = if sel_anim > 0.01 {
        Palette::interpolate_color(unselected_bg, selected_bg, sel_anim)
    } else {
        Palette::interpolate_color(unselected_bg, hovered_bg, hov_anim)
    };

    let resting_border = Palette::with_alpha(palette.border, 75);
    let hovered_border = Palette::with_alpha(palette.border, 160);
    let selected_border = palette.accent;

    let border_color = if sel_anim > 0.01 {
        Palette::interpolate_color(resting_border, selected_border, sel_anim)
    } else if hov_anim > 0.01 {
        Palette::interpolate_color(resting_border, hovered_border, hov_anim)
    } else {
        resting_border
    };

    ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0 + 0.2 * sel_anim, border_color),
        egui::StrokeKind::Inside,
    );

    let text_color = if sel_anim > 0.01 {
        Color32::WHITE
    } else if hov_anim > 0.01 {
        Palette::interpolate_color(Color32::from_gray(210), Color32::WHITE, hov_anim)
    } else {
        Color32::from_gray(210)
    };

    let y_lift = if sel_anim < 0.5 { -hov_anim * 0.5 } else { 0.0 };

    // When sel_anim = 0, label is centered at rect.center().x.
    // When sel_anim > 0, label smoothly glides left to make room for checkmark ✓ on right.
    let unselected_center_x = rect.center().x;
    let selected_label_center_x = rect.center().x - (check_w + icon_gap) * 0.5;
    let label_center_x = egui::lerp(unselected_center_x..=selected_label_center_x, sel_anim);
    let check_center_x = label_center_x + label_w * 0.5 + icon_gap + check_w * 0.5;

    ui.painter().text(
        egui::pos2(label_center_x, rect.center().y + y_lift),
        egui::Align2::CENTER_CENTER,
        label,
        label_font.clone(),
        text_color,
    );

    if sel_anim > 0.01 {
        let check_alpha = (255.0 * sel_anim).clamp(0.0, 255.0) as u8;
        let check_color = Color32::from_rgba_unmultiplied(
            palette.accent.r(),
            palette.accent.g(),
            palette.accent.b(),
            check_alpha,
        );
        ui.painter().text(
            egui::pos2(check_center_x, rect.center().y + y_lift),
            egui::Align2::CENTER_CENTER,
            "✓",
            label_font,
            check_color,
        );
    }

    response
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
/// Automatically and responsively wraps to a multi-line layout if the available width is constrained.
pub fn selection_row<'a, T: PartialEq + Clone>(
    ui: &mut Ui,
    label: &str,
    current_value: &mut T,
    options: impl IntoIterator<Item = (&'a str, T)>,
    palette: &Palette,
) -> bool {
    let mut changed = false;
    let opts: Vec<(&'a str, T)> = options.into_iter().collect();
    let font_size = 11.5_f32;
    let label_font = FontId::proportional(font_size);
    let check_w = ui
        .painter()
        .layout_no_wrap("✓".to_string(), label_font.clone(), Color32::WHITE)
        .size()
        .x;
    let icon_gap = 5.0_f32;
    let item_spacing_x = 6.0_f32;

    // Calculate total width needed by all pills combined in their stable size
    let mut total_pills_w = 0.0_f32;
    for (i, (opt_label, _)) in opts.iter().enumerate() {
        let label_w = ui
            .painter()
            .layout_no_wrap(opt_label.to_string(), label_font.clone(), Color32::WHITE)
            .size()
            .x;
        let pill_w = (label_w + check_w + icon_gap + 20.0).max(46.0);
        total_pills_w += pill_w;
        if i > 0 {
            total_pills_w += item_spacing_x;
        }
    }

    let title_w = ui
        .painter()
        .layout_no_wrap(
            label.to_string(),
            FontId::proportional(12.5),
            Color32::WHITE,
        )
        .size()
        .x;

    let avail_w = ui.available_width();
    let right_pad = 2.0_f32;
    let fits_single_row = (title_w + 16.0 + total_pills_w + right_pad) <= avail_w;

    if fits_single_row {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = item_spacing_x;
            ui.label(
                RichText::new(label)
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(230)),
            );

            // Align pills cleanly to the right in natural left-to-right order
            let remaining_w = ui.available_width();
            let left_gap = (remaining_w - total_pills_w - right_pad).max(0.0);
            if left_gap > 0.0 {
                ui.add_space(left_gap);
            }

            for (opt_label, val) in opts {
                let is_active = *current_value == val;
                if selection_pill(ui, opt_label, is_active, palette).clicked() {
                    *current_value = val;
                    changed = true;
                }
            }
        });
    } else {
        // Multi-line responsive stack for narrow widths or cards with many choices
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.label(
                RichText::new(label)
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(230)),
            );
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(item_spacing_x, 6.0);
                for (opt_label, val) in opts {
                    let is_active = *current_value == val;
                    if selection_pill(ui, opt_label, is_active, palette).clicked() {
                        *current_value = val;
                        changed = true;
                    }
                }
            });
        });
    }

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

    let base_bg = Palette::with_alpha(palette.card, 180);
    let hovered_bg = Palette::lighten(palette.card, 30, 240);
    let bg = Palette::interpolate_color(base_bg, hovered_bg, hov_anim);

    let resting_border = Palette::with_alpha(palette.border, 75);
    let hovered_border = Palette::with_alpha(palette.accent, 200);
    let border_color = Palette::interpolate_color(resting_border, hovered_border, hov_anim);

    ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0 + 0.2 * hov_anim, border_color),
        egui::StrokeKind::Inside,
    );

    let text_color = Palette::interpolate_color(Color32::from_gray(215), Color32::WHITE, hov_anim);
    let y_lift = -hov_anim * 0.75;
    let center = egui::pos2(rect.center().x, rect.center().y + y_lift);

    ui.painter().text(
        center,
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

    let resting_border = Palette::with_alpha(palette.accent, 160);
    let hovered_border = Color32::WHITE;
    let border_color = Palette::interpolate_color(resting_border, hovered_border, hov_anim * 0.5);

    ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0 + 0.3 * hov_anim, border_color),
        egui::StrokeKind::Inside,
    );

    let y_lift = -hov_anim * 0.75;
    let center = egui::pos2(rect.center().x, rect.center().y + y_lift);

    ui.painter().text(
        center,
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

    let base_bg = Color32::from_rgba_unmultiplied(160, 25, 25, 210);
    let hover_bg = Color32::from_rgba_unmultiplied(225, 38, 38, 255);
    let bg = Palette::interpolate_color(base_bg, hover_bg, hov_anim);

    let resting_border = Color32::from_rgba_unmultiplied(220, 38, 38, 120);
    let hover_border = Color32::from_rgba_unmultiplied(254, 202, 202, 220);
    let border_color = Palette::interpolate_color(resting_border, hover_border, hov_anim);

    ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0 + 0.3 * hov_anim, border_color),
        egui::StrokeKind::Inside,
    );

    let y_lift = -hov_anim * 0.75;
    let center = egui::pos2(rect.center().x, rect.center().y + y_lift);

    ui.painter().text(
        center,
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
            .rect_filled(rect, CornerRadius::same(8), pulse_bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(8),
            Stroke::new(1.4, Color32::WHITE),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "⌨ Press keys...",
            FontId::proportional(11.5),
            Color32::WHITE,
        );
    } else {
        let base_bg = Palette::with_alpha(palette.card, 180);
        let hover_bg = Palette::lighten(palette.card, 30, 240);
        let bg = Palette::interpolate_color(base_bg, hover_bg, hov_anim);

        let base_border = if is_modified {
            palette.accent
        } else {
            Palette::with_alpha(palette.border, 75)
        };
        let hover_border = palette.accent;
        let border_color = Palette::interpolate_color(base_border, hover_border, hov_anim);

        ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(8),
            Stroke::new(1.0 + 0.3 * hov_anim, border_color),
            egui::StrokeKind::Inside,
        );

        let text_color = if is_modified {
            Palette::interpolate_color(palette.accent, Color32::WHITE, hov_anim)
        } else {
            Palette::interpolate_color(Color32::from_gray(210), Color32::WHITE, hov_anim)
        };

        let y_lift = -hov_anim * 0.5;
        let center = egui::pos2(rect.center().x, rect.center().y + y_lift);

        ui.painter().text(
            center,
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
    let (mut rect, response) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::click());
    let is_hovered = response.hovered();
    let is_pressed = response.is_pointer_button_down_on();

    let hov_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("rev_hov"), is_hovered);
    let press_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("rev_press"), is_pressed);

    if press_anim > 0.01 {
        rect = rect.translate(egui::vec2(0.0, 1.0 * press_anim));
    }

    let resting_bg = Palette::with_alpha(palette.card, 140);
    let hovered_bg = Palette::with_alpha(palette.card, 220);
    let bg = Palette::interpolate_color(resting_bg, hovered_bg, hov_anim);

    let resting_border = Palette::with_alpha(palette.border, 75);
    let hover_border = palette.accent;
    let border_color = Palette::interpolate_color(resting_border, hover_border, hov_anim);

    ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0 + 0.2 * hov_anim, border_color),
        egui::StrokeKind::Inside,
    );

    let icon_color = Palette::interpolate_color(Color32::from_gray(180), Color32::WHITE, hov_anim);
    let scale = 1.0 + (hov_anim * 0.1);
    let font_size = (12.0 * scale).round();
    let y_lift = -hov_anim * 0.5;

    ui.painter().text(
        egui::pos2(rect.center().x, rect.center().y + y_lift),
        egui::Align2::CENTER_CENTER,
        "↺",
        FontId::proportional(font_size),
        icon_color,
    );

    response
}

/// Renders a sleek, unboxed interactive status bar text item with subtle hover brightening.
pub fn status_bar_item(
    ui: &mut Ui,
    icon: Option<(&str, Color32)>,
    text: &str,
    text_color: Color32,
    font_monospace: bool,
) -> egui::Response {
    let font_id = if font_monospace {
        FontId::monospace(11.0)
    } else {
        FontId::proportional(11.5)
    };
    let mut total_w = 4.0;
    let mut ic_w = 0.0;
    if let Some((ic, _)) = icon {
        let ic_galley =
            ui.painter()
                .layout_no_wrap(ic.to_string(), FontId::proportional(11.0), Color32::WHITE);
        ic_w = ic_galley.size().x;
        total_w += ic_w + 4.0;
    }
    let text_galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), font_id.clone(), Color32::WHITE);
    total_w += text_galley.size().x;

    let item_size = egui::vec2(total_w.max(12.0), 18.0);
    let (rect, response) = ui.allocate_exact_size(item_size, Sense::click());
    let is_hovered = response.hovered();

    let hov_anim = ui
        .ctx()
        .animate_bool_responsive(response.id.with("sbi_hov"), is_hovered);

    let mut cur_x = rect.min.x + 2.0;

    if let Some((ic, color)) = icon {
        let final_ic_color = if hov_anim > 0.01 {
            Palette::interpolate_color(color, Color32::WHITE, hov_anim * 0.5)
        } else {
            color
        };
        ui.painter().text(
            egui::pos2(cur_x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            ic,
            FontId::proportional(11.0),
            final_ic_color,
        );
        cur_x += ic_w + 4.0;
    }

    let final_text_color = if hov_anim > 0.01 {
        Palette::interpolate_color(text_color, Color32::WHITE, hov_anim)
    } else {
        text_color
    };

    ui.painter().text(
        egui::pos2(cur_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        font_id,
        final_text_color,
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
    let is_close_type = icon == "✕" || icon == "×" || icon == "x";

    let (bg, stroke, text_color) = if is_close_type {
        let normal_bg = Color32::from_rgba_unmultiplied(
            palette.card.r(),
            palette.card.g(),
            palette.card.b(),
            140,
        );
        let hover_bg = Color32::from_rgba_unmultiplied(239, 68, 68, 45);
        let active_bg = Color32::from_rgba_unmultiplied(220, 38, 38, 110);

        let bg = if response.is_pointer_button_down_on() {
            active_bg
        } else if hov_anim > 0.01 {
            Palette::interpolate_color(normal_bg, hover_bg, hov_anim)
        } else {
            normal_bg
        };

        let resting_border = Palette::with_alpha(palette.border, 75);
        let hover_border = Color32::from_rgba_unmultiplied(248, 113, 113, (160.0 * hov_anim) as u8);
        let border_color = if hov_anim > 0.01 {
            Palette::interpolate_color(resting_border, hover_border, hov_anim)
        } else {
            resting_border
        };
        let stroke = Stroke::new(1.0, border_color);

        let cross_color = if hov_anim > 0.01 {
            Palette::interpolate_color(
                Color32::from_gray(180),
                Color32::from_rgb(254, 226, 226),
                hov_anim,
            )
        } else {
            Color32::from_gray(180)
        };

        (bg, stroke, cross_color)
    } else {
        let normal_bg = Color32::from_rgba_unmultiplied(
            palette.card.r(),
            palette.card.g(),
            palette.card.b(),
            180,
        );
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

        let resting_border = Palette::with_alpha(palette.border, 75);
        let hovered_border = Palette::with_alpha(palette.border, 155);
        let active_border = Palette::with_alpha(palette.accent, 185);

        let border_color = if act_anim > 0.01 {
            Palette::interpolate_color(resting_border, active_border, act_anim)
        } else if hov_anim > 0.01 {
            Palette::interpolate_color(resting_border, hovered_border, hov_anim)
        } else {
            resting_border
        };
        let stroke = Stroke::new(1.0 + 0.2 * act_anim, border_color);

        let text_color = if is_active {
            palette.accent
        } else if hov_anim > 0.01 {
            Palette::interpolate_color(Color32::from_gray(215), Color32::WHITE, hov_anim)
        } else {
            Color32::from_gray(215)
        };

        (bg, stroke, text_color)
    };

    ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
    if stroke != Stroke::NONE {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(8),
            stroke,
            egui::StrokeKind::Inside,
        );
    }

    if is_close_type {
        // Render crisp anti-aliased vector cross lines with smooth micro-rotation animation on hover
        let center = rect.center();
        let angle = hov_anim * 0.08_f32;
        let half = (font_size * 0.38).round().max(3.0);

        let cos = angle.cos();
        let sin = angle.sin();

        let p1 = egui::pos2(
            center.x + (-half * cos - -half * sin),
            center.y + (-half * sin + -half * cos),
        );
        let p2 = egui::pos2(
            center.x + (half * cos - half * sin),
            center.y + (half * sin + half * cos),
        );
        let p3 = egui::pos2(
            center.x + (half * cos - -half * sin),
            center.y + (half * sin + -half * cos),
        );
        let p4 = egui::pos2(
            center.x + (-half * cos - half * sin),
            center.y + (-half * sin + half * cos),
        );

        let cross_stroke = Stroke::new(1.65_f32, text_color);
        ui.painter().line_segment([p1, p2], cross_stroke);
        ui.painter().line_segment([p3, p4], cross_stroke);
    } else {
        // Dynamic micro-scale and subtle lift on hover for all icon buttons
        let scale = 1.0 + (hov_anim * 0.08);
        let dynamic_font_size = (font_size * scale).round();
        let y_lift = -hov_anim * 0.75;
        let center = egui::pos2(rect.center().x, rect.center().y + y_lift);

        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(dynamic_font_size),
            text_color,
        );
    }

    response
}

/// Renders a modern sleek drawer close icon button (×).
pub fn close_button(ui: &mut Ui, palette: &Palette) -> egui::Response {
    icon_button(ui, "×", false, palette, 13.0, egui::vec2(28.0, 28.0))
}
