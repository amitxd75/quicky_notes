//! Glassmorphism card, panel, and container frames.

use crate::models::AppSettings;
use crate::theme::{self, Palette};
use eframe::egui::{Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, Ui};

/// Creates a main glass editor outer frame with translucent background and accent border.
pub fn glass_editor_frame(settings: &AppSettings) -> Frame {
    let palette = theme::get_palette(settings);
    let blur_factor = settings.appearance.blur_strength.clamp(0.0, 1.0);
    let alpha = (settings.appearance.opacity * 255.0 * (0.85 + 0.15 * blur_factor))
        .clamp(40.0, 255.0) as u8;
    let radius = settings.appearance.corner_radius.round().clamp(0.0, 32.0) as u8;
    let border_stroke = 1.0 + 0.4 * blur_factor;

    Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(
            palette.bg.r(),
            palette.bg.g(),
            palette.bg.b(),
            alpha,
        ))
        .stroke(Stroke::new(border_stroke, palette.border))
        .corner_radius(CornerRadius::same(radius))
        .inner_margin(Margin::ZERO)
}

/// Creates a glass card container frame for settings cards and dialogs.
#[allow(dead_code)]
pub fn glass_card_frame(settings: &AppSettings) -> Frame {
    let palette = theme::get_palette(settings);
    let blur_factor = settings.appearance.blur_strength.clamp(0.0, 1.0);
    let alpha = (settings.appearance.opacity * 255.0 * (0.85 + 0.15 * blur_factor))
        .clamp(140.0, 245.0) as u8;
    let border_stroke = 1.0 + 0.3 * blur_factor;

    Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(
            palette.card.r(),
            palette.card.g(),
            palette.card.b(),
            alpha,
        ))
        .stroke(Stroke::new(border_stroke, palette.border))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(12))
}

/// Standardized titled settings section card with standard padding and title typography.
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
    let blur_factor = settings.appearance.blur_strength.clamp(0.0, 1.0);
    let border_alpha = (100.0_f32 + 80.0_f32 * blur_factor).clamp(60.0, 255.0) as u8;

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
                border_alpha,
            ),
        ))
        .inner_margin(Margin::symmetric(16, 10))
}

/// Creates a search/input container frame.
pub fn glass_input_frame(settings: &AppSettings) -> Frame {
    let palette = theme::get_palette(settings);
    let blur_factor = settings.appearance.blur_strength.clamp(0.0, 1.0);

    Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(
            palette.bg.r(),
            palette.bg.g(),
            palette.bg.b(),
            220,
        ))
        .stroke(Stroke::new(1.0 + 0.3 * blur_factor, palette.border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 6))
}
