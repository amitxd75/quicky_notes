//! Modal dialog overlay components.

use crate::models::AppSettings;
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke};

/// Renders a dimmed fullscreen backdrop with a centered glass modal dialog wrapper.
pub fn modal_overlay<R>(
    ctx: &egui::Context,
    id: &str,
    settings: &AppSettings,
    modal_size: egui::Vec2,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) {
    let palette = theme::get_palette(settings);
    let screen_rect = ctx.content_rect();

    // 1. Paint subtle translucent backdrop directly to layer
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new((id, "backdrop")),
    ))
    .rect_filled(
        screen_rect,
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(0, 0, 0, 50),
    );

    // 2. Centered interactive modal Area
    let modal_rect = egui::Rect::from_center_size(screen_rect.center(), modal_size);
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Foreground)
        .fixed_pos(modal_rect.min)
        .show(ctx, |ui| {
            ui.set_width(modal_size.x);
            ui.set_height(modal_size.y);

            let frame = egui::Frame::NONE
                .fill(Color32::from_rgba_unmultiplied(
                    palette.card.r(),
                    palette.card.g(),
                    palette.card.b(),
                    245,
                ))
                .stroke(Stroke::new(1.0, palette.border))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::same(18));

            frame.show(ui, add_contents);
        });
}
