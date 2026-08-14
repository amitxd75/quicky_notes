//! Options & Settings drawer component with spacious, breathable card layout.

use crate::app::QuickyNotesApp;
use crate::settings::WindowSizePreset;
use crate::theme::{self, ACCENT_AMBER};
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, Ui, ViewportCommand};

/// Renders the spacious, breathable glass Options & Settings drawer.
pub fn render_options_drawer(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = theme::get_palette(app.data.settings.theme_mode);

    ui.vertical(|ui| {
        ui.add_space(6.0);

        // Drawer Header Bar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⚙ Options & Settings")
                    .font(FontId::proportional(16.0))
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close_btn = ui.add(
                    egui::Button::new(
                        RichText::new("Close  ✕")
                            .font(FontId::proportional(13.0))
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
                    .min_size(egui::vec2(80.0, 32.0)),
                );
                if close_btn.clicked() {
                    app.show_options = false;
                    app.focus_editor = true;
                    ctx.request_repaint();
                }
            });
        });

        ui.add_space(10.0);

        // Options Drawer Body (Scrollable Cards with Spacious Spacing)
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 48.0)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 16.0;

                // 1. Theme & Color Palette Card
                theme::glass_card_frame(app.data.settings.opacity, app.data.settings.theme_mode)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.label(
                                RichText::new("THEME & COLOR PALETTE")
                                    .font(FontId::proportional(11.5))
                                    .strong()
                                    .color(palette.accent),
                            );

                            ui.add_space(2.0);

                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                                for mode in theme::ThemeMode::all_modes() {
                                    let is_selected = app.data.settings.theme_mode == *mode;
                                    let (fill, stroke, text_color) = if is_selected {
                                        (
                                            Color32::from_rgba_unmultiplied(
                                                (palette.card.r() as u16 + 45).min(255) as u8,
                                                (palette.card.g() as u16 + 40).min(255) as u8,
                                                (palette.card.b() as u16 + 50).min(255) as u8,
                                                245,
                                            ),
                                            Stroke::new(1.2_f32, palette.accent),
                                            Color32::WHITE,
                                        )
                                    } else {
                                        (
                                            Color32::from_rgba_unmultiplied(
                                                palette.card.r(),
                                                palette.card.g(),
                                                palette.card.b(),
                                                150,
                                            ),
                                            Stroke::new(
                                                1.0_f32,
                                                Color32::from_rgba_unmultiplied(
                                                    palette.border.r(),
                                                    palette.border.g(),
                                                    palette.border.b(),
                                                    80,
                                                ),
                                            ),
                                            Color32::from_gray(190),
                                        )
                                    };

                                    let label_text = if is_selected {
                                        format!("● {}", mode.display_name())
                                    } else {
                                        format!("  {}", mode.display_name())
                                    };

                                    let btn = ui.add(
                                        egui::Button::new(
                                            RichText::new(&label_text)
                                                .font(FontId::proportional(12.5))
                                                .color(text_color),
                                        )
                                        .fill(fill)
                                        .stroke(stroke)
                                        .corner_radius(CornerRadius::same(8))
                                        .min_size(egui::vec2(0.0, 32.0)),
                                    );

                                    if btn.clicked() {
                                        app.data.settings.theme_mode = *mode;
                                        theme::setup_glassmorphism_theme(
                                            ctx,
                                            app.data.settings.opacity,
                                            *mode,
                                        );
                                        app.is_dirty = true;
                                        ctx.request_repaint();
                                    }
                                }
                            });
                        });
                    });

                // 2. System Font Family Card
                theme::glass_card_frame(app.data.settings.opacity, app.data.settings.theme_mode)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.label(
                                RichText::new("SYSTEM FONT FAMILY")
                                    .font(FontId::proportional(11.5))
                                    .strong()
                                    .color(palette.accent),
                            );

                            ui.add_space(2.0);

                            let fonts = crate::font::get_installed_system_fonts();
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Active Font Family")
                                        .font(FontId::proportional(13.5))
                                        .color(Color32::from_gray(230)),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let current_font = app.data.settings.selected_font.clone();
                                        let font_combo =
                                            egui::ComboBox::from_id_salt("sys_font_select")
                                                .selected_text(
                                                    RichText::new(&current_font)
                                                        .font(FontId::proportional(13.0))
                                                        .color(Color32::WHITE),
                                                );

                                        font_combo.show_ui(ui, |ui| {
                                            for f_name in fonts {
                                                if ui
                                                    .selectable_value(
                                                        &mut app.data.settings.selected_font,
                                                        f_name.clone(),
                                                        &f_name,
                                                    )
                                                    .clicked()
                                                {
                                                    crate::font::apply_system_font(
                                                        ctx,
                                                        &app.data.settings.selected_font,
                                                    );
                                                    app.is_dirty = true;
                                                    ctx.request_repaint();
                                                }
                                            }
                                        });
                                    },
                                );
                            });
                        });
                    });

                // 3. Window Size Presets Card
                theme::glass_card_frame(app.data.settings.opacity, app.data.settings.theme_mode)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.label(
                                RichText::new("WINDOW SIZE PRESETS")
                                    .font(FontId::proportional(11.5))
                                    .strong()
                                    .color(palette.accent),
                            );

                            ui.add_space(2.0);

                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                                for preset in WindowSizePreset::all() {
                                    let w = preset.width;
                                    let h = preset.height;
                                    let is_active = (app.data.settings.window_width - w).abs()
                                        < 5.0
                                        && (app.data.settings.window_height - h).abs() < 5.0;
                                    let (btn_bg, stroke, text_color) = if is_active {
                                        (
                                            Color32::from_rgba_unmultiplied(
                                                (palette.card.r() as u16 + 45).min(255) as u8,
                                                (palette.card.g() as u16 + 40).min(255) as u8,
                                                (palette.card.b() as u16 + 50).min(255) as u8,
                                                245,
                                            ),
                                            Stroke::new(1.2_f32, palette.accent),
                                            Color32::WHITE,
                                        )
                                    } else {
                                        (
                                            Color32::from_rgba_unmultiplied(
                                                palette.card.r(),
                                                palette.card.g(),
                                                palette.card.b(),
                                                150,
                                            ),
                                            Stroke::new(
                                                1.0_f32,
                                                Color32::from_rgba_unmultiplied(
                                                    palette.border.r(),
                                                    palette.border.g(),
                                                    palette.border.b(),
                                                    80,
                                                ),
                                            ),
                                            Color32::from_gray(190),
                                        )
                                    };

                                    let btn = ui.add(
                                        egui::Button::new(
                                            RichText::new(format!(
                                                "{} ({}x{})",
                                                preset.label, w as u32, h as u32
                                            ))
                                            .font(FontId::proportional(12.0))
                                            .color(text_color),
                                        )
                                        .fill(btn_bg)
                                        .stroke(stroke)
                                        .corner_radius(CornerRadius::same(8))
                                        .min_size(egui::vec2(0.0, 30.0)),
                                    );

                                    if btn.clicked() {
                                        app.data.settings.window_width = w;
                                        app.data.settings.window_height = h;
                                        ctx.send_viewport_cmd(ViewportCommand::InnerSize(
                                            egui::vec2(w, h),
                                        ));

                                        let _ = std::process::Command::new("hyprctl")
                                            .arg("dispatch")
                                            .arg(format!(
                                                "hl.dsp.window.resize({{ x = {}, y = {} }})",
                                                w as u32, h as u32
                                            ))
                                            .spawn();

                                        app.is_dirty = true;
                                        ctx.request_repaint();
                                    }
                                }
                            });
                        });
                    });

                // 4. Appearance & Typography Card
                theme::glass_card_frame(app.data.settings.opacity, app.data.settings.theme_mode)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 12.0;
                            ui.label(
                                RichText::new("APPEARANCE & TYPOGRAPHY")
                                    .font(FontId::proportional(11.5))
                                    .strong()
                                    .color(palette.accent),
                            );

                            ui.add_space(2.0);

                            // Opacity Slider
                            let prev_opacity = app.data.settings.opacity;
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Window Opacity")
                                        .font(FontId::proportional(13.5))
                                        .color(Color32::from_gray(230)),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let opacity_pct =
                                            (app.data.settings.opacity * 100.0) as u32;
                                        ui.label(
                                            RichText::new(format!("{}%", opacity_pct))
                                                .font(FontId::monospace(12.5))
                                                .color(Color32::from_gray(190)),
                                        );
                                        if ui
                                            .add_sized(
                                                egui::vec2(150.0, 22.0),
                                                egui::Slider::new(
                                                    &mut app.data.settings.opacity,
                                                    0.30..=1.00,
                                                )
                                                .show_value(false)
                                                .trailing_fill(true),
                                            )
                                            .changed()
                                        {
                                            ctx.request_repaint();
                                        }
                                    },
                                );
                            });

                            if (prev_opacity - app.data.settings.opacity).abs() > 0.01 {
                                theme::setup_glassmorphism_theme(
                                    ctx,
                                    app.data.settings.opacity,
                                    app.data.settings.theme_mode,
                                );
                                app.is_dirty = true;
                                ctx.request_repaint();
                            }

                            // Font Size Slider
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Editor Font Size")
                                        .font(FontId::proportional(13.5))
                                        .color(Color32::from_gray(230)),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "{:.0}pt",
                                                app.data.settings.font_size
                                            ))
                                            .font(FontId::monospace(12.5))
                                            .color(Color32::from_gray(190)),
                                        );
                                        if ui
                                            .add_sized(
                                                egui::vec2(150.0, 22.0),
                                                egui::Slider::new(
                                                    &mut app.data.settings.font_size,
                                                    10.0..=28.0,
                                                )
                                                .show_value(false)
                                                .trailing_fill(true),
                                            )
                                            .changed()
                                        {
                                            app.is_dirty = true;
                                            ctx.request_repaint();
                                        }
                                    },
                                );
                            });

                            // Monospace Font Toggle
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Monospace Font (Code Mode)")
                                        .font(FontId::proportional(13.5))
                                        .color(Color32::from_gray(230)),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let chk =
                                            ui.checkbox(&mut app.data.settings.monospace_font, "");
                                        if chk.changed() {
                                            app.is_dirty = true;
                                            ctx.request_repaint();
                                        }
                                    },
                                );
                            });

                            // Auto Save Interval Slider
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Auto-Save Interval")
                                        .font(FontId::proportional(13.5))
                                        .color(Color32::from_gray(230)),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "{}s",
                                                app.data.settings.auto_save_seconds
                                            ))
                                            .font(FontId::monospace(12.5))
                                            .color(Color32::from_gray(190)),
                                        );
                                        if ui
                                            .add_sized(
                                                egui::vec2(150.0, 22.0),
                                                egui::Slider::new(
                                                    &mut app.data.settings.auto_save_seconds,
                                                    1..=10,
                                                )
                                                .show_value(false)
                                                .trailing_fill(true),
                                            )
                                            .changed()
                                        {
                                            app.is_dirty = true;
                                            ctx.request_repaint();
                                        }
                                    },
                                );
                            });

                            // Always on Top
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Always on Top")
                                        .font(FontId::proportional(13.5))
                                        .color(Color32::from_gray(230)),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let chk =
                                            ui.checkbox(&mut app.data.settings.always_on_top, "");
                                        if chk.changed() {
                                            let level = if app.data.settings.always_on_top {
                                                egui::WindowLevel::AlwaysOnTop
                                            } else {
                                                egui::WindowLevel::Normal
                                            };
                                            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
                                                level,
                                            ));
                                            app.is_dirty = true;
                                            ctx.request_repaint();
                                        }
                                    },
                                );
                            });
                        });
                    });

                // 5. Quick Actions Card
                theme::glass_card_frame(app.data.settings.opacity, app.data.settings.theme_mode)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.label(
                                RichText::new("QUICK ACTIONS")
                                    .font(FontId::proportional(11.5))
                                    .strong()
                                    .color(palette.accent),
                            );

                            ui.add_space(2.0);

                            let export_btn = ui.add(
                                egui::Button::new(
                                    RichText::new("📤 Export Current Note to File")
                                        .font(FontId::proportional(13.0))
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgba_unmultiplied(
                                    palette.card.r(),
                                    palette.card.g(),
                                    palette.card.b(),
                                    210,
                                ))
                                .stroke(Stroke::new(1.0_f32, palette.border))
                                .corner_radius(CornerRadius::same(8))
                                .min_size(egui::vec2(ui.available_width(), 34.0)),
                            );

                            if export_btn.clicked() {
                                app.export_active_note();
                                ctx.request_repaint();
                            }
                        });
                    });

                // 6. Shortcuts Card
                theme::glass_card_frame(app.data.settings.opacity, app.data.settings.theme_mode)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 8.0;
                            ui.label(
                                RichText::new("KEYBOARD SHORTCUTS")
                                    .font(FontId::proportional(11.5))
                                    .strong()
                                    .color(ACCENT_AMBER),
                            );

                            ui.add_space(2.0);

                            let help_items = [
                                ("Ctrl + N", "New note tab"),
                                ("Ctrl + W", "Close active tab"),
                                ("Ctrl + S", "Manual save to disk"),
                                ("Ctrl + 1..9 / 0", "Switch to tab index"),
                                ("Ctrl + K", "Search & browse notes"),
                                ("Ctrl + ,", "Toggle options drawer"),
                                ("Ctrl + Shift + E", "Export current note"),
                                ("Ctrl + Tab", "Cycle next tab"),
                            ];

                            for (k, d) in help_items {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(k)
                                            .font(FontId::monospace(11.5))
                                            .color(palette.accent),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(d)
                                                    .font(FontId::proportional(12.0))
                                                    .color(Color32::from_gray(180)),
                                            );
                                        },
                                    );
                                });
                            }
                        });
                    });

                ui.add_space(8.0);

                // Done / Close Drawer Button
                let done_btn = ui.add(
                    egui::Button::new(
                        RichText::new("✓  Done & Close Settings  (Esc)")
                            .font(FontId::proportional(13.5))
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgba_unmultiplied(
                        (palette.card.r() as u16 + 40).min(255) as u8,
                        (palette.card.g() as u16 + 35).min(255) as u8,
                        (palette.card.b() as u16 + 45).min(255) as u8,
                        245,
                    ))
                    .stroke(Stroke::new(1.2_f32, palette.accent))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(egui::vec2(ui.available_width(), 38.0)),
                );

                if done_btn.clicked() {
                    app.show_options = false;
                    app.focus_editor = true;
                    ctx.request_repaint();
                }
            });
    });
}
