//! Transient floating toast notification system with glassmorphism styling and spring animations.

use crate::app::QuickyNotesApp;
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, RichText, Stroke};
use std::time::Instant;

/// Severity and style categories for transient floating toast notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Info,
    Warning,
    Error,
}

impl ToastKind {
    /// Distinct emoji / icon indicator for toast.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Success => "✓",
            Self::Info => "⚡",
            Self::Warning => "⚠️",
            Self::Error => "✕",
        }
    }

    /// Color tone for toast border and icon.
    pub fn color(self, palette: &theme::Palette) -> Color32 {
        match self {
            Self::Success => Color32::from_rgb(34, 197, 94),
            Self::Info => palette.accent,
            Self::Warning => Color32::from_rgb(249, 115, 22),
            Self::Error => Color32::from_rgb(239, 68, 68),
        }
    }
}

/// A transient notification message with timestamp and styling category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub created_at: Instant,
}

impl Toast {
    /// Creates a new toast notification.
    pub fn new(message: impl Into<String>, kind: ToastKind) -> Self {
        Self {
            message: message.into(),
            kind,
            created_at: Instant::now(),
        }
    }
}

/// Renders an animated floating glass toast notification overlay with smooth entry/exit.
pub fn render_floating_toast(app: &mut QuickyNotesApp, ctx: &egui::Context) {
    let Some(toast) = &app.toast else {
        return;
    };

    let elapsed = toast.created_at.elapsed().as_secs_f32();
    if elapsed >= 3.2 {
        return;
    }

    // Request continuous repaint for smooth animation
    ctx.request_repaint();

    // Fade in 0.0..0.2s, stay 0.2..2.6s, fade out 2.6..3.2s
    let alpha_factor = if elapsed < 0.2 {
        (elapsed / 0.2).clamp(0.0, 1.0)
    } else if elapsed > 2.6 {
        ((3.2 - elapsed) / 0.6).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let slide_offset = (1.0 - alpha_factor) * 8.0;
    let palette = theme::get_palette(&app.data.settings);
    let kind_color = toast.kind.color(&palette);

    let alpha_u8 = (255.0 * alpha_factor) as u8;
    let bg_color = Color32::from_rgba_unmultiplied(
        palette.card.r(),
        palette.card.g(),
        palette.card.b(),
        (245.0 * alpha_factor) as u8,
    );
    let border_color = Color32::from_rgba_unmultiplied(
        kind_color.r(),
        kind_color.g(),
        kind_color.b(),
        (230.0 * alpha_factor) as u8,
    );

    let base_bottom_offset = if app.data.settings.editor.show_status_bar && !app.show_options {
        -68.0
    } else {
        -36.0
    };

    egui::Area::new(egui::Id::new("floating_toast_layer"))
        .order(egui::Order::Tooltip)
        .anchor(
            egui::Align2::CENTER_BOTTOM,
            egui::vec2(0.0, base_bottom_offset + slide_offset),
        )
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(bg_color)
                .stroke(Stroke::new(1.2, border_color))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.label(
                            RichText::new(toast.kind.icon())
                                .font(FontId::proportional(13.5))
                                .strong()
                                .color(Color32::from_rgba_unmultiplied(
                                    kind_color.r(),
                                    kind_color.g(),
                                    kind_color.b(),
                                    alpha_u8,
                                )),
                        );
                        ui.label(
                            RichText::new(&toast.message)
                                .font(FontId::proportional(12.5))
                                .strong()
                                .color(Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8)),
                        );
                    });
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_creation_and_icon() {
        let toast = Toast::new("Saved", ToastKind::Success);
        assert_eq!(toast.kind.icon(), "✓");
        assert_eq!(toast.message, "Saved");
    }
}
