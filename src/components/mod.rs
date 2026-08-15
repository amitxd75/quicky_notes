//! Reusable glassmorphism UI components for Quicky Notes.
//!
//! Provides optimized, boilerplate-free UI elements including buttons, cards,
//! input controls, modal dialogs, status badges, and divider lines.

use crate::settings::AppSettings;
use crate::theme::{self, Palette};
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Sense, Stroke, Ui, UiBuilder,
};

/// Card and glass panel container frames.
pub mod card {
    use super::*;

    /// Creates a main glass editor outer frame with translucent background and accent border.
    pub fn glass_editor_frame(settings: &AppSettings) -> Frame {
        let palette = theme::get_palette(settings);
        let alpha = (settings.opacity * 255.0).clamp(40.0, 255.0) as u8;
        let radius = settings.corner_radius.round().clamp(0.0, 32.0) as u8;
        Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(
                palette.bg.r(),
                palette.bg.g(),
                palette.bg.b(),
                alpha,
            ))
            .stroke(Stroke::new(1.2_f32, palette.border))
            .corner_radius(CornerRadius::same(radius))
            .inner_margin(Margin::ZERO)
    }

    /// Creates a glass card container frame for settings cards and dialogs.
    pub fn glass_card_frame(settings: &AppSettings) -> Frame {
        let palette = theme::get_palette(settings);
        let alpha = (settings.opacity * 255.0).clamp(160.0, 240.0) as u8;
        Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(
                palette.card.r(),
                palette.card.g(),
                palette.card.b(),
                alpha,
            ))
            .stroke(Stroke::new(1.0_f32, palette.border))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::same(12))
    }

    /// Renders a titled settings section card with standard padding, title typography, and layout.
    /// Standardized settings card frame with title header.
    pub fn settings_card<R>(
        ui: &mut Ui,
        title: &str,
        palette: &Palette,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) {
        Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(
                palette.card.r(),
                palette.card.g(),
                palette.card.b(),
                230,
            ))
            .stroke(Stroke::new(1.0_f32, palette.border))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 8.0;
                    ui.label(
                        RichText::new(title)
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(palette.accent),
                    );
                    add_contents(ui);
                });
            });
    }

    /// Creates a solid outer header container bar frame.
    pub fn glass_header_frame(settings: &AppSettings) -> Frame {
        let palette = theme::get_palette(settings);
        Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(
                palette.bg.r(),
                palette.bg.g(),
                palette.bg.b(),
                245,
            ))
            .stroke(Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(
                    palette.border.r(),
                    palette.border.g(),
                    palette.border.b(),
                    100,
                ),
            ))
            .inner_margin(Margin::symmetric(16, 10))
    }

    /// Creates a search/input container frame.
    pub fn glass_input_frame(settings: &AppSettings) -> Frame {
        let palette = theme::get_palette(settings);
        Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(
                palette.bg.r(),
                palette.bg.g(),
                palette.bg.b(),
                220,
            ))
            .stroke(Stroke::new(1.2_f32, palette.border))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(10, 6))
    }
}

/// Standardized UI buttons.
pub mod button {
    use super::*;

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

    /// Renders a secondary translucent button (e.g. "Cancel", "Export Note").
    #[allow(dead_code)]
    pub fn secondary_button(
        ui: &mut Ui,
        text: &str,
        card: Color32,
        border: Color32,
        min_size: egui::Vec2,
    ) -> egui::Response {
        ui.add(
            egui::Button::new(
                RichText::new(text)
                    .font(FontId::proportional(12.5))
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgba_unmultiplied(
                card.r(),
                card.g(),
                card.b(),
                210,
            ))
            .stroke(Stroke::new(1.0_f32, border))
            .corner_radius(CornerRadius::same(8))
            .min_size(min_size),
        )
    }

    /// Renders a red-accented destructive action button (e.g. "Close Tab").
    #[allow(dead_code)]
    pub fn danger_button(ui: &mut Ui, text: &str, min_size: egui::Vec2) -> egui::Response {
        ui.add(
            egui::Button::new(
                RichText::new(text)
                    .font(FontId::proportional(12.5))
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgba_unmultiplied(180, 45, 60, 240))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(239, 68, 68)))
            .corner_radius(CornerRadius::same(8))
            .min_size(min_size),
        )
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

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(font_size),
            Color32::WHITE,
        );

        response
    }

    /// Renders a drawer close button ("Close ✕").
    pub fn close_button(ui: &mut Ui, palette: &Palette) -> egui::Response {
        ui.add(
            egui::Button::new(
                RichText::new("Close  ✕")
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
}

/// Animated toggle switches.
pub mod toggle {
    use super::*;

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
        let mut resp = None;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(label)
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(230)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                resp = Some(toggle_switch(ui, on, accent));
            });
        });
        resp.unwrap()
    }
}

/// Modern glassmorphic slider controls with glowing knobs and accent filled tracks.
pub mod slider {
    use super::*;

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
            let norm =
                ((mouse_pos.x - (rect.left() + 7.0)) / (rect.width() - 14.0)).clamp(0.0, 1.0);
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
            egui::StrokeKind::Outside,
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
        let mut resp = None;
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
                resp = Some(glass_slider(
                    ui,
                    value,
                    range,
                    accent,
                    egui::vec2(150.0, 20.0),
                ));
            });
        });
        resp.unwrap()
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
        let mut resp = None;
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
                resp = Some(glass_slider_u32(
                    ui,
                    value,
                    range,
                    accent,
                    egui::vec2(150.0, 20.0),
                ));
            });
        });
        resp.unwrap()
    }
}

/// Badges and status dot indicators.
pub mod badge {
    use super::*;

    /// Renders a small 10pt colored status indicator dot.
    #[allow(dead_code)]
    pub fn status_dot(ui: &mut Ui, color: Color32) -> egui::Response {
        ui.label(RichText::new("●").size(10.0).color(color))
    }

    /// Renders an interactive pin badge indicator icon for tab pinning.
    pub fn pin_badge(ui: &mut Ui, _color: Color32) -> egui::Response {
        ui.add(
            egui::Label::new(RichText::new("📌").size(12.5).color(Color32::WHITE))
                .sense(Sense::click()),
        )
    }
}

/// Horizontal and vertical dividers.
pub mod divider {
    use super::*;

    /// Renders a thin horizontal divider line with subtle accent transparency.
    pub fn horizontal_divider(ui: &mut Ui) {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), Sense::hover());
        ui.painter().line_segment(
            [rect.min, egui::pos2(rect.max.x, rect.min.y)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(130, 80, 190, 80)),
        );
    }
}

/// Text input components.
pub mod input {
    use super::*;

    /// Renders a framed search text input field with optional auto-focus.
    pub fn search_input(
        ui: &mut Ui,
        query: &mut String,
        hint: &str,
        focus: bool,
    ) -> egui::Response {
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
}

/// Modal dialog overlay component.
pub mod modal {
    use super::*;

    /// Renders a dimmed fullscreen backdrop with a centered glass modal dialog wrapper.
    pub fn modal_overlay<R>(
        ctx: &egui::Context,
        id: &str,
        settings: &AppSettings,
        modal_size: egui::Vec2,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) {
        egui::Area::new(egui::Id::new(id))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                let screen_rect = ui.clip_rect();

                // Dimmed backdrop mask
                let _ = ui.allocate_rect(screen_rect, Sense::click());
                ui.painter().rect_filled(
                    screen_rect,
                    CornerRadius::ZERO,
                    Color32::from_black_alpha(180),
                );

                let modal_rect = egui::Rect::from_center_size(screen_rect.center(), modal_size);
                let mut child_ui = ui.new_child(UiBuilder::new().max_rect(modal_rect));

                let palette = theme::get_palette(settings);
                let frame = card::glass_card_frame(settings)
                    .stroke(Stroke::new(1.2, palette.border))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::same(16));

                frame.show(&mut child_ui, add_contents);
            });
    }
}
