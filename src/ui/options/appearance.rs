//! Appearance tab preferences: Themes, AI theme generator, custom color picker, typography, and window presets.

use crate::app::QuickyNotesApp;
use crate::components::{button, card, slider, stepper, toggle};
use crate::settings::WindowSizePreset;
use crate::theme;
use eframe::egui::{self, Color32, FontId, RichText, Ui, ViewportCommand};

/// Renders all cards in the Appearance settings tab.
pub fn render_appearance_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    render_theme_palette_card(app, ctx, ui, palette);
    if app.data.settings.appearance.theme_mode == theme::ThemeMode::Custom {
        render_ai_theme_generator_card(app, ctx, ui, palette);
        render_custom_colors_card(app, ctx, ui, palette);
    }
    render_appearance_card(app, ctx, ui, palette);
    render_window_presets_card(app, ctx, ui, palette);
    render_system_font_card(app, ctx, ui, palette);
}

/// Renders a sleek, compact AI Theme Generator card for custom theme generation.
pub fn render_ai_theme_generator_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "✨ AI THEME GENERATOR", palette, |ui| {
        let is_generating = app.ai_theme_rx.is_some();

        // 1. Compact Prompt Row + Inline Generate Button
        ui.horizontal(|ui| {
            let prompt_edit = egui::TextEdit::singleline(&mut app.ai_theme_prompt)
                .hint_text(
                    "Theme description (e.g. Cyberpunk neon, Nordic moss, Sunset Horizon)...",
                )
                .font(FontId::proportional(12.0))
                .desired_width((ui.available_width() - 110.0).max(100.0));
            ui.add(prompt_edit);

            let btn_label = if is_generating {
                "⏳ Generating"
            } else {
                "✨ Generate"
            };
            let gen_btn =
                button::animated_primary_button(ui, btn_label, palette, egui::vec2(105.0, 26.0));

            if gen_btn.clicked() && !is_generating {
                let prompt = if app.ai_theme_prompt.trim().is_empty() {
                    "Sleek modern dark glassmorphism palette with vibrant accents".to_string()
                } else {
                    app.ai_theme_prompt.trim().to_string()
                };

                let rx = crate::engine::spawn_generate_theme_request(
                    prompt,
                    app.data.settings.ai.provider,
                    app.data.settings.ai.model.clone(),
                    app.data.settings.ai.api_key.clone(),
                    Some(app.data.settings.ai.temperature),
                );
                app.ai_theme_rx = Some(rx);
                app.show_toast(
                    "AI is designing your custom theme...",
                    crate::ui::toast::ToastKind::Info,
                );
                ctx.request_repaint();
            }
        });

        ui.add_space(4.0);

        // 2. Compact Inspiration Presets
        let inspiration_presets = [
            (
                "⚡ Cyberpunk",
                "Vivid cyberpunk Tokyo high-contrast neon cyan, magenta, and deep purple obsidian",
            ),
            (
                "❄ Nordic",
                "Muted Scandinavian deep pine forest, morning mist, and lichen stone",
            ),
            (
                "☕ Autumn",
                "Warm vintage parchment, roasted espresso, amber glow, and cinnamon brown",
            ),
            (
                "◆ Obsidian",
                "Sleek jet-black OLED dark mode with glowing electric amber and golden accents",
            ),
            (
                "✿ Sakura",
                "Soft Tokyo sakura blossoms, pastel pink, lavender dusk, and slate glass",
            ),
            (
                "⌨ Terminal",
                "Monochrome phosphor emerald green, hacker terminal matrix aesthetics",
            ),
        ];

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
            for (label, prompt_text) in inspiration_presets {
                if button::glass_chip_button(ui, label, palette).clicked() {
                    app.ai_theme_prompt = prompt_text.to_string();
                }
            }
        });
    });
}

/// Renders the Theme & Color Palette selection card with checkmarks.
pub fn render_theme_palette_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "THEME & COLOR PALETTE", palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            for mode in theme::ThemeMode::all_modes() {
                let is_selected = app.data.settings.appearance.theme_mode == *mode;
                if button::selection_pill(ui, mode.display_name(), is_selected, palette).clicked() {
                    app.data.settings.appearance.theme_mode = *mode;
                    theme::setup_glassmorphism_theme(ctx, &app.data.settings);
                    app.is_dirty = true;
                    ctx.request_repaint();
                }
            }
        });
    });
}

/// Helper to render an individual color picker with label in a fixed width column.
fn color_picker_item(ui: &mut Ui, label: &str, color: &mut [u8; 3], col_w: f32) -> bool {
    let mut changed = false;
    ui.allocate_ui(egui::vec2(col_w, 28.0), |ui| {
        ui.horizontal(|ui| {
            if ui.color_edit_button_srgb(color).changed() {
                changed = true;
            }
            ui.label(
                RichText::new(label)
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(230)),
            );
        });
    });
    changed
}

/// Renders the Custom Color palette picker card with colored square chips matching image.png.
pub fn render_custom_colors_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "CUSTOM THEME PALETTE COLORS", palette, |ui| {
        let mut color_changed = false;
        let col_w = (ui.available_width() - 16.0) * 0.5;

        // Row 1: Surfaces
        ui.horizontal(|ui| {
            color_changed |= color_picker_item(
                ui,
                "Background",
                &mut app.data.settings.appearance.custom_colors.bg,
                col_w,
            );
            color_changed |= color_picker_item(
                ui,
                "Card Surface",
                &mut app.data.settings.appearance.custom_colors.card,
                col_w,
            );
        });

        ui.add_space(4.0);

        // Row 2: Outline & Main Accent
        ui.horizontal(|ui| {
            color_changed |= color_picker_item(
                ui,
                "Border Tint",
                &mut app.data.settings.appearance.custom_colors.border,
                col_w,
            );
            color_changed |= color_picker_item(
                ui,
                "Primary Accent",
                &mut app.data.settings.appearance.custom_colors.accent,
                col_w,
            );
        });

        ui.add_space(4.0);

        // Row 3: Secondary Accent & Text
        ui.horizontal(|ui| {
            color_changed |= color_picker_item(
                ui,
                "Secondary Accent",
                &mut app.data.settings.appearance.custom_colors.secondary_accent,
                col_w,
            );
            color_changed |= color_picker_item(
                ui,
                "Primary Text",
                &mut app.data.settings.appearance.custom_colors.text,
                col_w,
            );
        });

        ui.add_space(4.0);

        // Row 4: Muted Text & Danger
        ui.horizontal(|ui| {
            color_changed |= color_picker_item(
                ui,
                "Muted Text",
                &mut app.data.settings.appearance.custom_colors.muted_text,
                col_w,
            );
            color_changed |= color_picker_item(
                ui,
                "Danger Action",
                &mut app.data.settings.appearance.custom_colors.danger,
                col_w,
            );
        });

        if color_changed {
            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
            app.is_dirty = true;
            ctx.request_repaint();
        }
    });
}

/// Renders the System & UI Typography card with font family and size scaling.
pub fn render_system_font_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "SYSTEM & UI TYPOGRAPHY", palette, |ui| {
        let fonts = crate::font::get_installed_system_fonts();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("System UI Font Family")
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(230)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let current_font = app.data.settings.appearance.selected_font.clone();
                let font_combo = egui::ComboBox::from_id_salt("sys_font_select").selected_text(
                    RichText::new(&current_font)
                        .font(FontId::proportional(12.0))
                        .color(Color32::WHITE),
                );

                font_combo.show_ui(ui, |ui| {
                    for f_name in fonts {
                        if ui
                            .selectable_value(
                                &mut app.data.settings.appearance.selected_font,
                                f_name.clone(),
                                &f_name,
                            )
                            .clicked()
                        {
                            crate::font::apply_system_fonts(
                                ctx,
                                &app.data.settings.appearance.selected_font,
                                &app.data.settings.editor.editor_font,
                            );
                            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
                            app.is_dirty = true;
                            let _ = crate::storage::AppData::save_settings_to_path(
                                &app.data.settings,
                                &crate::storage::AppData::config_path(),
                            );
                            ctx.request_repaint();
                        }
                    }
                });
            });
        });

        ui.add_space(4.0);

        // System UI Font Size Stepper
        let ui_size_str = format!("{:.1}pt", app.data.settings.appearance.ui_font_size);
        if stepper::stepper_row_f32(
            ui,
            "System UI Font Size",
            &mut app.data.settings.appearance.ui_font_size,
            crate::models::settings::MIN_UI_FONT_SIZE..=crate::models::settings::MAX_UI_FONT_SIZE,
            0.5,
            &ui_size_str,
            palette,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
            app.is_dirty = true;
            let _ = crate::storage::AppData::save_settings_to_path(
                &app.data.settings,
                &crate::storage::AppData::config_path(),
            );
            ctx.request_repaint();
        }
    });
}

/// Renders the Window Size Presets card with checkmarks.
pub fn render_window_presets_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "WINDOW SIZE PRESETS", palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            for preset in WindowSizePreset::all() {
                let w = preset.width;
                let h = preset.height;
                let is_active = (app.data.settings.window.width - w).abs() < 5.0
                    && (app.data.settings.window.height - h).abs() < 5.0;
                let label = format!(
                    "{} ({}x{} • {:.1}pt)",
                    preset.label, w as u32, h as u32, preset.default_font_size
                );

                if button::selection_pill(ui, &label, is_active, palette).clicked() {
                    app.data.settings.window.width = w;
                    app.data.settings.window.height = h;
                    app.data.settings.editor.font_size = preset.default_font_size;
                    app.data.settings.validate_and_clamp();
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(w, h)));
                    app.is_dirty = true;
                    app.show_toast(
                        format!(
                            "Resized to {} ({:.1}pt font)",
                            preset.label, preset.default_font_size
                        ),
                        crate::ui::toast::ToastKind::Success,
                    );
                    ctx.request_repaint();
                }
            }
        });
    });
}

/// Renders the Window & Glass Styling slider and toggle controls.
pub fn render_appearance_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "WINDOW & GLASS STYLING", palette, |ui| {
        // Opacity Slider
        let opacity_pct = (app.data.settings.appearance.opacity * 100.0) as u32;
        if slider::slider_row(
            ui,
            "Window Opacity",
            &format!("{}%", opacity_pct),
            &mut app.data.settings.appearance.opacity,
            0.30..=1.00,
            palette.accent,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Glass Blur Strength / Hardness Slider
        let blur_pct = (app.data.settings.appearance.blur_strength * 100.0) as u32;
        if slider::slider_row(
            ui,
            "Glass Blur & Hardness",
            &format!("{}%", blur_pct),
            &mut app.data.settings.appearance.blur_strength,
            0.0..=1.0,
            palette.accent,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Window Corner Radius Slider
        let radius_str = format!("{:.0}px", app.data.settings.appearance.corner_radius);
        if slider::slider_row(
            ui,
            "Window Corner Rounding",
            &radius_str,
            &mut app.data.settings.appearance.corner_radius,
            4.0..=24.0,
            palette.accent,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Status Bar Visibility
        if toggle::toggle_row(
            ui,
            "Show Bottom Status Bar",
            &mut app.data.settings.editor.show_status_bar,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Always on Top (Modern Switch)
        if toggle::toggle_row(
            ui,
            "Always on Top (Floating Mode)",
            &mut app.data.settings.window.always_on_top,
            palette.accent,
        )
        .changed()
        {
            let level = if app.data.settings.window.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
            app.is_dirty = true;
            ctx.request_repaint();
        }

        ui.add_space(4.0);
        let reset_appearance_btn = button::animated_action_button(
            ui,
            "↺ Reset Appearance to Defaults",
            palette,
            egui::vec2(ui.available_width(), 28.0),
        );
        if reset_appearance_btn.clicked() {
            app.data.settings.appearance = crate::models::settings::AppearanceSettings::default();
            app.data.settings.validate_and_clamp();
            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
            app.is_dirty = true;
            app.show_toast(
                "Appearance reset to defaults",
                crate::ui::toast::ToastKind::Info,
            );
            ctx.request_repaint();
        }
    });
}
