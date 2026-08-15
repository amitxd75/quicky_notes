//! Settings & Preferences view matching image.png design mockup.

use crate::app::{QuickyNotesApp, SettingsTab};
use crate::components::{button, card, slider, toggle};
use crate::settings::WindowSizePreset;
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, Ui, ViewportCommand};

/// Renders the Settings & Preferences drawer matching image.png.
pub fn render_options_drawer(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = theme::get_palette(&app.data.settings);

    ui.vertical(|ui| {
        ui.add_space(2.0);

        // 1. Settings Header Bar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⚙  Settings & Preferences")
                    .font(FontId::proportional(16.5))
                    .strong()
                    .color(Color32::WHITE),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close_btn = button::close_button(ui, &palette);
                if close_btn.clicked() {
                    app.show_options = false;
                    app.focus_editor = true;
                    ctx.request_repaint();
                }
            });
        });

        ui.add_space(6.0);
        crate::ui::draw_horizontal_divider(ui);
        ui.add_space(6.0);

        // 2. Main Content (2-Panel: Left Navigation Sidebar + Full-Width Settings Workspace)
        let bottom_bar_height = 42.0;
        let total_height = (ui.available_height() - bottom_bar_height).max(120.0);
        let sidebar_width = 160.0;

        ui.horizontal(|ui| {
            // --- Column 1: Left Navigation Sidebar ---
            ui.vertical(|ui| {
                ui.set_width(sidebar_width);
                ui.set_height(total_height);
                render_navigation_sidebar(app, ui, &palette);
            });

            // Vertical divider between sidebar and content
            let (_, rect) = ui.allocate_space(egui::vec2(1.0, total_height));
            ui.painter().line_segment(
                [rect.min, egui::pos2(rect.min.x, rect.max.y)],
                Stroke::new(
                    1.0_f32,
                    Color32::from_rgba_unmultiplied(
                        palette.border.r(),
                        palette.border.g(),
                        palette.border.b(),
                        90,
                    ),
                ),
            );

            ui.add_space(10.0);

            // --- Column 2: Main Settings Workspace (Full remaining width) ---
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                ui.set_height(total_height);

                egui::ScrollArea::vertical()
                    .id_salt("settings_main_scroll")
                    .auto_shrink([false, false])
                    .max_height(total_height)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = 12.0;

                            match app.settings_tab {
                                SettingsTab::General => {
                                    render_theme_palette_card(app, ctx, ui, &palette);
                                    if app.data.settings.theme_mode == theme::ThemeMode::Custom {
                                        render_custom_colors_card(app, ctx, ui, &palette);
                                    }
                                    render_appearance_card(app, ctx, ui, &palette);
                                    render_editor_behavior_card(app, ctx, ui, &palette);
                                    render_quick_actions_card(app, ui, &palette);
                                }
                                SettingsTab::Appearance => {
                                    render_theme_palette_card(app, ctx, ui, &palette);
                                    if app.data.settings.theme_mode == theme::ThemeMode::Custom {
                                        render_custom_colors_card(app, ctx, ui, &palette);
                                    }
                                    render_system_font_card(app, ctx, ui, &palette);
                                    render_window_presets_card(app, ctx, ui, &palette);
                                    render_appearance_card(app, ctx, ui, &palette);
                                }
                                SettingsTab::Editor => {
                                    render_system_font_card(app, ctx, ui, &palette);
                                    render_editor_behavior_card(app, ctx, ui, &palette);
                                }
                                SettingsTab::FilesBackup => {
                                    render_backup_info_card(ui, &palette);
                                    render_quick_actions_card(app, ui, &palette);
                                }
                                SettingsTab::Shortcuts => {
                                    render_shortcuts_card(ui, &palette);
                                }
                                SettingsTab::About => {
                                    render_about_card(ui, &palette);
                                }
                            }
                        });
                    });
            });
        });

        ui.add_space(4.0);
        crate::ui::draw_horizontal_divider(ui);
        ui.add_space(4.0);

        // 3. Bottom Action Bar (Done & Close Settings on left, status on right)
        render_bottom_bar(app, ctx, ui, &palette);
    });
}

/// Renders the vertical navigation sidebar with left-aligned icons and labels.
fn render_navigation_sidebar(app: &mut QuickyNotesApp, ui: &mut Ui, palette: &theme::Palette) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;

        for tab in SettingsTab::ALL {
            let is_selected = app.settings_tab == tab;
            let (icon, label) = tab.icon_and_label();

            let desired_size = egui::vec2(ui.available_width(), 36.0);
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

            let is_hovered = response.hovered();
            let sel_anim = ui
                .ctx()
                .animate_bool_responsive(response.id.with("sel"), is_selected);
            let hov_anim = ui
                .ctx()
                .animate_bool_responsive(response.id.with("hov"), is_hovered);

            let normal_bg = Color32::from_rgba_unmultiplied(
                palette.card.r(),
                palette.card.g(),
                palette.card.b(),
                80,
            );
            let selected_bg = Color32::from_rgba_unmultiplied(
                (palette.card.r() as u16 + 45).min(255) as u8,
                (palette.card.g() as u16 + 35).min(255) as u8,
                (palette.card.b() as u16 + 55).min(255) as u8,
                240,
            );
            let hovered_bg = Color32::from_rgba_unmultiplied(
                (palette.card.r() as u16 + 25).min(255) as u8,
                (palette.card.g() as u16 + 25).min(255) as u8,
                (palette.card.b() as u16 + 35).min(255) as u8,
                180,
            );

            let bg = if sel_anim > 0.01 {
                theme::Palette::interpolate_color(normal_bg, selected_bg, sel_anim)
            } else {
                theme::Palette::interpolate_color(normal_bg, hovered_bg, hov_anim)
            };

            let stroke_alpha = (255.0 * sel_anim) as u8;
            let stroke = if stroke_alpha > 5 {
                Stroke::new(
                    1.2_f32,
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

            // Centered icon in dedicated 28px left area (Never tint emojis)
            ui.painter().text(
                egui::pos2(rect.left() + 18.0, rect.center().y),
                egui::Align2::CENTER_CENTER,
                icon,
                FontId::proportional(14.0),
                Color32::WHITE,
            );

            // Left-aligned Label with consistent offset and smooth color transition
            let text_color = theme::Palette::interpolate_color(
                Color32::from_gray(185),
                Color32::WHITE,
                sel_anim,
            );
            ui.painter().text(
                egui::pos2(rect.left() + 38.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                FontId::proportional(13.0),
                text_color,
            );

            if response.clicked() {
                app.settings_tab = tab;
            }
        }
    });
}
/// Renders the Theme & Color Palette selection card with checkmarks.
fn render_theme_palette_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "THEME & COLOR PALETTE", palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            for mode in theme::ThemeMode::all_modes() {
                let is_selected = app.data.settings.theme_mode == *mode;
                if button::selection_pill(ui, mode.display_name(), is_selected, palette).clicked() {
                    app.data.settings.theme_mode = *mode;
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
fn render_custom_colors_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "CUSTOM THEME PALETTE COLORS", palette, |ui| {
        let mut color_changed = false;
        let col_w = (ui.available_width() - 16.0) * 0.5;

        ui.horizontal(|ui| {
            color_changed |= color_picker_item(
                ui,
                "Background",
                &mut app.data.settings.custom_bg_color,
                col_w,
            );
            color_changed |= color_picker_item(
                ui,
                "Card Container",
                &mut app.data.settings.custom_card_color,
                col_w,
            );
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            color_changed |= color_picker_item(
                ui,
                "Border Tint",
                &mut app.data.settings.custom_border_color,
                col_w,
            );
            color_changed |= color_picker_item(
                ui,
                "Accent Color",
                &mut app.data.settings.custom_accent_color,
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

/// Renders the System Font family dropdown picker card.
fn render_system_font_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "SYSTEM FONT", palette, |ui| {
        let fonts = crate::font::get_installed_system_fonts();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Active Font Family")
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(230)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let current_font = app.data.settings.selected_font.clone();
                let font_combo = egui::ComboBox::from_id_salt("sys_font_select").selected_text(
                    RichText::new(&current_font)
                        .font(FontId::proportional(12.0))
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
                            crate::font::apply_system_font(ctx, &app.data.settings.selected_font);
                            app.is_dirty = true;
                            ctx.request_repaint();
                        }
                    }
                });
            });
        });
    });
}

/// Renders the Window Size Presets card with checkmarks.
fn render_window_presets_card(
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
                let is_active = (app.data.settings.window_width - w).abs() < 5.0
                    && (app.data.settings.window_height - h).abs() < 5.0;
                let label = format!("{} ({}x{})", preset.label, w as u32, h as u32);

                if button::selection_pill(ui, &label, is_active, palette).clicked() {
                    app.data.settings.window_width = w;
                    app.data.settings.window_height = h;
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(w, h)));
                    app.is_dirty = true;
                    ctx.request_repaint();
                }
            }
        });
    });
}

/// Renders the Window & Glass Styling slider and toggle controls.
fn render_appearance_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "WINDOW & GLASS STYLING", palette, |ui| {
        // Opacity Slider
        let opacity_pct = (app.data.settings.opacity * 100.0) as u32;
        if slider::slider_row(
            ui,
            "Window Opacity",
            &format!("{}%", opacity_pct),
            &mut app.data.settings.opacity,
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

        // Window Corner Radius Slider
        let radius_str = format!("{:.0}px", app.data.settings.corner_radius);
        if slider::slider_row(
            ui,
            "Window Corner Rounding",
            &radius_str,
            &mut app.data.settings.corner_radius,
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
            &mut app.data.settings.show_status_bar,
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
            &mut app.data.settings.always_on_top,
            palette.accent,
        )
        .changed()
        {
            let level = if app.data.settings.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
            app.is_dirty = true;
            ctx.request_repaint();
        }
    });
}

/// Renders Editor Behavior, Typography, Indentation, and Safety controls.
fn render_editor_behavior_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "EDITOR BEHAVIOR & TYPOGRAPHY", palette, |ui| {
        // Font Size Slider
        let font_str = format!("{:.0}pt", app.data.settings.font_size);
        if slider::slider_row(
            ui,
            "Editor Font Size",
            &font_str,
            &mut app.data.settings.font_size,
            8.0..=36.0,
            palette.accent,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Monospace Font Toggle
        if toggle::toggle_row(
            ui,
            "Monospace Font (Code Mode)",
            &mut app.data.settings.monospace_font,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Show Line Numbers Gutter
        if toggle::toggle_row(
            ui,
            "Show Line Numbers in Gutter",
            &mut app.data.settings.show_line_numbers,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Tab Indentation Size
        if button::selection_row(
            ui,
            "Tab Indentation Width",
            &mut app.data.settings.tab_size,
            [("2 sp", 2), ("4 sp", 4), ("8 sp", 8)],
            palette,
        ) {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Default New Note File Extension
        if button::selection_row(
            ui,
            "Default New Note Format",
            &mut app.data.settings.default_extension,
            [
                ("Text (.txt)", ".txt".to_string()),
                ("Markdown (.md)", ".md".to_string()),
            ],
            palette,
        ) {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Confirm Before Closing Tab
        if toggle::toggle_row(
            ui,
            "Confirm Before Closing Note",
            &mut app.data.settings.confirm_close_tab,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Auto-trim Trailing Whitespace
        if toggle::toggle_row(
            ui,
            "Trim Trailing Whitespace on Save",
            &mut app.data.settings.trim_trailing_whitespace,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Auto Save Interval Slider
        let auto_save_str = format!("{}s", app.data.settings.auto_save_seconds);
        if slider::slider_row_u32(
            ui,
            "Auto-Save Interval",
            &auto_save_str,
            &mut app.data.settings.auto_save_seconds,
            1..=60,
            palette.accent,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            app.is_dirty = true;
            ctx.request_repaint();
        }
    });
}

/// Renders the Quick Actions export button card.
fn render_quick_actions_card(app: &mut QuickyNotesApp, ui: &mut Ui, palette: &theme::Palette) {
    card::settings_card(ui, "QUICK ACTIONS", palette, |ui| {
        let export_btn = ui.add(
            egui::Button::new(
                RichText::new("📤  Export Current Note to File")
                    .font(FontId::proportional(12.5))
                    .color(Color32::WHITE),
            )
            .fill(theme::Palette::with_alpha(palette.card, 210))
            .stroke(Stroke::new(1.0_f32, palette.border))
            .corner_radius(CornerRadius::same(6))
            .min_size(egui::vec2(ui.available_width(), 32.0)),
        );

        if export_btn.clicked() {
            crate::ui::drag_drop::export_active_note(app);
        }
    });
}

/// Renders file persistence, data directory opening, and backup info card.
fn render_backup_info_card(ui: &mut Ui, palette: &theme::Palette) {
    card::settings_card(ui, "STORAGE & DURABILITY", palette, |ui| {
        let path = crate::storage::AppData::config_path();
        ui.label(
            RichText::new(format!("Config Path: {}", path.to_string_lossy()))
                .font(FontId::monospace(10.5))
                .color(Color32::from_gray(190)),
        );

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let open_folder_btn = ui.add(
                egui::Button::new(
                    RichText::new("📂  Open Data Folder in File Manager")
                        .font(FontId::proportional(11.5))
                        .color(Color32::WHITE),
                )
                .fill(theme::Palette::with_alpha(palette.card, 210))
                .stroke(Stroke::new(1.0_f32, palette.border))
                .corner_radius(CornerRadius::same(6))
                .min_size(egui::vec2(0.0, 28.0)),
            );

            if open_folder_btn.clicked()
                && let Some(parent) = path.parent()
            {
                crate::ui::drag_drop::safe_open_folder(parent);
            }
        });

        ui.add_space(4.0);

        ui.label(
            RichText::new("✓ Atomic save with PID temp file swap\n✓ Corrupt configuration auto-backup recovery\n✓ Zero-data-loss safe disk serialization")
                .font(FontId::proportional(11.5))
                .color(Color32::from_gray(180)),
        );
    });
}

/// Renders the Keyboard Shortcuts reference card matching image.png.
fn render_shortcuts_card(ui: &mut Ui, palette: &theme::Palette) {
    card::settings_card(ui, "KEYBOARD SHORTCUTS", palette, |ui| {
        let help_items = [
            ("Ctrl + N", "New note tab"),
            ("Ctrl + W", "Close active tab"),
            ("Ctrl + S", "Manual save to disk"),
            ("Ctrl + 1..9 / 0", "Switch to tab index"),
            ("Ctrl + K", "Search & browse notes"),
            ("Ctrl + ,", "Open settings"),
            ("Ctrl + P", "Toggle Markdown preview (.md files)"),
            ("Ctrl + Shift + E", "Export current note"),
            ("Ctrl + Tab", "Next tab"),
            ("Ctrl + Shift + Tab", "Previous tab"),
        ];

        for (k, d) in help_items {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(k)
                        .font(FontId::monospace(11.0))
                        .color(palette.accent),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(d)
                            .font(FontId::proportional(11.5))
                            .color(Color32::from_gray(180)),
                    );
                });
            });
        }
    });
}

/// Renders the About This App card with developer text, version, and links.
fn render_about_card(ui: &mut Ui, palette: &theme::Palette) {
    card::settings_card(ui, "ABOUT THIS APP", palette, |ui| {
        ui.label(
            RichText::new(
                "A fast and minimal note taking app for developers.\nBuilt with ❤️ for your flow.",
            )
            .font(FontId::proportional(12.0))
            .color(Color32::from_gray(210)),
        );

        ui.label(
            RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                .font(FontId::monospace(11.5))
                .color(palette.accent),
        );

        ui.horizontal(|ui| {
            let gh_btn = ui.add(
                egui::Button::new(
                    RichText::new("GitHub ↗")
                        .font(FontId::proportional(12.0))
                        .color(Color32::WHITE),
                )
                .fill(theme::Palette::with_alpha(palette.card, 200))
                .stroke(Stroke::new(1.0_f32, palette.border))
                .corner_radius(CornerRadius::same(6))
                .min_size(egui::vec2(120.0, 30.0)),
            );
            if gh_btn
                .on_hover_text("https://github.com/amitxd75/quicky_notes")
                .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl::same_tab(
                    "https://github.com/amitxd75/quicky_notes",
                ));
            }
        });
    });
}

/// Renders the bottom action bar matching image.png with persistent Done button and status.
fn render_bottom_bar(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;

        let btn_size = egui::vec2(72.0, 24.0);
        let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
        let is_hovered = response.hovered();
        let hov_anim = ui
            .ctx()
            .animate_bool_responsive(response.id.with("done_hov"), is_hovered);

        let normal_bg = Color32::from_rgba_unmultiplied(
            palette.card.r(),
            palette.card.g(),
            palette.card.b(),
            200,
        );
        let hovered_bg = theme::Palette::lighten(palette.card, 35, 240);
        let bg_color = theme::Palette::interpolate_color(normal_bg, hovered_bg, hov_anim);

        let stroke_color = theme::Palette::interpolate_color(
            theme::Palette::with_alpha(palette.border, 120),
            palette.accent,
            hov_anim,
        );

        ui.painter()
            .rect_filled(rect, CornerRadius::same(6), bg_color);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(6),
            Stroke::new(1.0_f32, stroke_color),
            egui::StrokeKind::Outside,
        );

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Done ✓",
            FontId::proportional(12.0),
            Color32::WHITE,
        );

        if response.clicked() {
            app.show_options = false;
            app.focus_editor = true;
            ctx.request_repaint();
        }

        // Sleek Esc Badge
        let esc_size = egui::vec2(28.0, 18.0);
        let (esc_rect, _) = ui.allocate_exact_size(esc_size, egui::Sense::hover());
        ui.painter().rect_filled(
            esc_rect,
            CornerRadius::same(4),
            Color32::from_rgba_unmultiplied(
                palette.card.r(),
                palette.card.g(),
                palette.card.b(),
                160,
            ),
        );
        ui.painter().rect_stroke(
            esc_rect,
            CornerRadius::same(4),
            Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(
                    palette.border.r(),
                    palette.border.g(),
                    palette.border.b(),
                    90,
                ),
            ),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            esc_rect.center(),
            egui::Align2::CENTER_CENTER,
            "esc",
            FontId::monospace(9.5),
            Color32::from_gray(160),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            ui.label(
                RichText::new("UTF-8")
                    .font(FontId::monospace(11.0))
                    .color(Color32::from_gray(160)),
            );

            // Use cached per-frame stats computation
            {
                let (words, chars, lines) = app.cached_active_stats;

                ui.label(
                    RichText::new(format!(
                        "Words: {}  •  Chars: {}  •  Lines: {}",
                        words, chars, lines
                    ))
                    .font(FontId::proportional(11.5))
                    .color(Color32::from_gray(180)),
                );
            }

            ui.label(
                RichText::new("● Auto-saved")
                    .font(FontId::proportional(11.5))
                    .color(crate::theme::ACCENT_EMERALD),
            );
        });
    });
}
