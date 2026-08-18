//! Shortcuts tab preferences: Interactive keybinding rebinding, conflict detection, and reset controls.

use crate::app::QuickyNotesApp;
use crate::components::{button, card};
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, Ui};

/// Renders all cards in the Shortcuts settings tab.
pub fn render_shortcuts_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    render_shortcuts_card(app, ctx, ui, palette);
}

/// Renders interactive keyboard shortcut customizer and registry.
pub fn render_shortcuts_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    use crate::ui::shortcuts::ShortcutAction;

    let mut action_to_reset = None;
    let mut action_to_record = None;
    let mut reset_all_triggered = false;

    // Header banner with instructions and global reset button
    card::settings_card(ui, "KEYBOARD SHORTCUTS MANAGER", palette, |ui| {
        let avail_w = ui.available_width();
        let is_compact = avail_w < 480.0;

        if is_compact {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Click any key combination badge below to reassign it.")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(210)),
                );
                ui.label(
                    RichText::new("Press Esc while recording to cancel rebinding.")
                        .font(FontId::proportional(11.0))
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(6.0);
                let reset_btn = button::animated_action_button(
                    ui,
                    "↺ Reset All to Defaults",
                    palette,
                    egui::vec2(ui.available_width(), 28.0),
                );
                if reset_btn
                    .on_hover_text("Restore all default keyboard shortcuts")
                    .clicked()
                {
                    reset_all_triggered = true;
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Click any key combination badge below to reassign it.")
                            .font(FontId::proportional(12.0))
                            .color(Color32::from_gray(210)),
                    );
                    ui.label(
                        RichText::new("Press Esc while recording to cancel rebinding.")
                            .font(FontId::proportional(11.0))
                            .color(Color32::from_gray(160)),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let reset_btn = button::animated_action_button(
                        ui,
                        "↺ Reset All to Defaults",
                        palette,
                        egui::vec2(175.0, 26.0),
                    );
                    if reset_btn
                        .on_hover_text("Restore all default keyboard shortcuts")
                        .clicked()
                    {
                        reset_all_triggered = true;
                    }
                });
            });
        }

        if let Some(recording_action) = app.recording_shortcut {
            ui.add_space(8.0);
            egui::Frame::NONE
                .fill(theme::Palette::with_alpha(palette.accent, 40))
                .stroke(Stroke::new(1.2, palette.accent))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "⌨ Listening for shortcut: {}...",
                                recording_action.title()
                            ))
                            .font(FontId::proportional(12.0))
                            .strong()
                            .color(Color32::WHITE),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(
                                    RichText::new("Cancel (Esc)")
                                        .font(FontId::proportional(11.0))
                                        .color(Color32::from_gray(200)),
                                )
                                .clicked()
                            {
                                app.recording_shortcut = None;
                                ctx.request_repaint();
                            }
                        });
                    });
                });
        }
    });

    let categories = [
        "Tabs & Navigation",
        "Note Operations",
        "Editor & View",
        "Window & Modals",
    ];

    for cat in categories {
        let cat_actions: Vec<ShortcutAction> = ShortcutAction::ALL
            .iter()
            .copied()
            .filter(|a| a.category() == cat)
            .collect();

        if cat_actions.is_empty() {
            continue;
        }

        let header = cat.to_uppercase();
        card::settings_card(ui, &header, palette, |ui| {
            for action in cat_actions {
                let current_binding = app.data.settings.keybindings.get(action);
                let default_binding = action.default_binding();
                let is_modified = current_binding != default_binding;
                let is_recording = app.recording_shortcut == Some(action);
                let conflict = app.data.settings.keybindings.find_conflict(action);
                let display_text = current_binding.to_display_string();

                let font = FontId::monospace(11.5);
                let measured_text_width = ui.fonts_mut(|f| {
                    f.layout_no_wrap(display_text.clone(), font, Color32::WHITE)
                        .size()
                        .x
                });
                let badge_width = (measured_text_width + 24.0).max(105.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    let right_width = if is_modified && !is_recording {
                        badge_width + 26.0 + 8.0
                    } else {
                        badge_width + 8.0
                    };
                    let left_width = (ui.available_width() - right_width).max(60.0);

                    // Left side: Action title & optional conflict badge
                    ui.allocate_ui(egui::vec2(left_width, 24.0), |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(action.title())
                                    .font(FontId::proportional(12.5))
                                    .color(Color32::WHITE),
                            );

                            if let Some(other) = conflict {
                                ui.label(
                                    RichText::new(format!("⚠️ Conflict: {}", other.title()))
                                        .font(FontId::proportional(10.5))
                                        .color(Color32::from_rgb(251, 146, 60)),
                                );
                            }
                        });
                    });

                    // Right side: Reset button + Shortcut Badge
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Keybinding click-to-rebind badge button
                        let bind_btn = button::animated_shortcut_badge(
                            ui,
                            &display_text,
                            is_recording,
                            is_modified,
                            palette,
                            egui::vec2(badge_width, 24.0),
                        );

                        if bind_btn
                            .on_hover_text(if is_recording {
                                "Press desired key combination or Esc to cancel"
                            } else {
                                "Click to reassign this shortcut"
                            })
                            .clicked()
                        {
                            if is_recording {
                                action_to_record = Some(None);
                            } else {
                                action_to_record = Some(Some(action));
                            }
                        }

                        // Revert single modified shortcut
                        if is_modified && !is_recording {
                            let revert_btn = button::animated_revert_button(ui, palette);
                            let tooltip = format!(
                                "Reset to default ({})",
                                default_binding.to_display_string()
                            );
                            if revert_btn.on_hover_text(tooltip).clicked() {
                                action_to_reset = Some(action);
                            }
                        }
                    });
                });

                ui.add_space(2.0);
            }
        });
    }

    if reset_all_triggered {
        app.data.settings.keybindings.reset_all();
        app.is_dirty = true;
        let _ = app.data.save();
        app.show_toast(
            "All shortcuts reset to factory defaults",
            crate::app::ToastKind::Success,
        );
        ctx.request_repaint();
    }

    if let Some(action) = action_to_reset {
        app.data.settings.keybindings.reset_action(action);
        app.is_dirty = true;
        let _ = app.data.save();
        app.show_toast(
            format!("Reset {} to default", action.title()),
            crate::app::ToastKind::Info,
        );
        ctx.request_repaint();
    }

    if let Some(record_opt) = action_to_record {
        app.recording_shortcut = record_opt;
        ctx.request_repaint();
    }
}
