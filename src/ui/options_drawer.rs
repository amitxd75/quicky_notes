//! Settings & Preferences view matching image.png design mockup.

use crate::app::QuickyNotesApp;
use crate::components::{button, card, slider, toggle};
use crate::settings::WindowSizePreset;
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, Ui, ViewportCommand};

/// Navigation tabs in the Settings & Preferences drawer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Appearance,
    Editor,
    Ai,
    Plugins,
    FilesBackup,
    Shortcuts,
    About,
}

impl SettingsTab {
    pub const ALL: [Self; 8] = [
        Self::General,
        Self::Appearance,
        Self::Editor,
        Self::Ai,
        Self::Plugins,
        Self::FilesBackup,
        Self::Shortcuts,
        Self::About,
    ];

    pub fn icon_and_label(self) -> (&'static str, &'static str) {
        match self {
            Self::General => ("⚙", "General"),
            Self::Appearance => ("🎨", "Appearance"),
            Self::Editor => ("📝", "Editor"),
            Self::Ai => ("✨", "AI Copilot"),
            Self::Plugins => ("🔌", "Plugins"),
            Self::FilesBackup => ("📁", "Files & Backup"),
            Self::Shortcuts => ("⌨", "Shortcuts"),
            Self::About => ("ℹ", "About"),
        }
    }
}

/// Renders the Settings & Preferences drawer matching image.png.
pub fn render_options_drawer(app: &mut QuickyNotesApp, ctx: &egui::Context, ui: &mut Ui) {
    let palette = app.active_palette();

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
                                    render_general_system_card(app, ctx, ui, &palette);
                                    render_general_workspace_card(app, ctx, ui, &palette);
                                    render_general_sync_card(app, ctx, ui, &palette);
                                    render_general_cheatsheet_card(ui, &palette);
                                }
                                SettingsTab::Appearance => {
                                    render_theme_palette_card(app, ctx, ui, &palette);
                                    if app.data.settings.appearance.theme_mode
                                        == theme::ThemeMode::Custom
                                    {
                                        render_ai_theme_generator_card(app, ctx, ui, &palette);
                                        render_custom_colors_card(app, ctx, ui, &palette);
                                    }
                                    render_appearance_card(app, ctx, ui, &palette);
                                    render_window_presets_card(app, ctx, ui, &palette);
                                    render_system_font_card(app, ctx, ui, &palette);
                                }
                                SettingsTab::Editor => {
                                    render_system_font_card(app, ctx, ui, &palette);
                                    render_editor_behavior_card(app, ctx, ui, &palette);
                                }
                                SettingsTab::Ai => {
                                    render_ai_settings_card(app, ctx, ui, &palette);
                                }
                                SettingsTab::Plugins => {
                                    render_plugins_manager_card(app, ctx, ui, &palette);
                                    render_installed_plugins_list(app, ctx, ui, &palette);
                                }
                                SettingsTab::FilesBackup => {
                                    render_backup_info_card(app, ctx, ui, &palette);
                                    render_learned_dictionary_card(app, ctx, ui, &palette);
                                    render_quick_actions_card(app, ui, &palette);
                                }
                                SettingsTab::Shortcuts => {
                                    render_shortcuts_card(app, ctx, ui, &palette);
                                }
                                SettingsTab::About => {
                                    render_about_card(app, ctx, ui, &palette);
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

/// Renders System & Startup integration card in the General tab.
fn render_general_system_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "SYSTEM & STARTUP INTEGRATION", palette, |ui| {
        // 1. Launch on System Startup (Autostart)
        if toggle::toggle_row(
            ui,
            "Launch on System Startup (Autostart)",
            &mut app.data.settings.general.autostart,
            palette.accent,
        )
        .changed()
        {
            let _ =
                crate::platform::sync_autostart_desktop_file(app.data.settings.general.autostart);
            app.is_dirty = true;
            if app.data.settings.general.autostart {
                app.show_toast(
                    "Autostart enabled (~/.config/autostart/quicky.desktop)",
                    crate::ui::toast::ToastKind::Success,
                );
            } else {
                app.show_toast("Autostart disabled", crate::ui::toast::ToastKind::Info);
            }
            ctx.request_repaint();
        }

        // 2. Restore Session on Startup
        if toggle::toggle_row(
            ui,
            "Restore Open Session & Workspace on Startup",
            &mut app.data.settings.general.restore_session,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // 3. Always on Top (Floating Widget)
        if toggle::toggle_row(
            ui,
            "Always on Top (Floating Scratchpad Mode)",
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
    });
}

/// Renders Note Defaults & Automation settings card in the General tab.
fn render_general_workspace_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "NOTE CREATION & DEFAULTS", palette, |ui| {
        // 1. Default New Note Title Prefix
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("New Note Title Prefix:")
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(230)),
            );
            let prefix_edit =
                egui::TextEdit::singleline(&mut app.data.settings.general.default_title_prefix)
                    .font(FontId::monospace(12.0))
                    .desired_width(ui.available_width());
            if ui.add(prefix_edit).changed() {
                app.data.settings.general.validate_and_clamp();
                app.is_dirty = true;
            }
        });

        ui.add_space(4.0);

        // 2. Auto-Title from First Line
        if toggle::toggle_row(
            ui,
            "Auto-Title Notes from First Line",
            &mut app.data.settings.general.auto_title_from_first_line,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // 3. Default File Format Extension
        let mut ext = app.data.settings.editor.default_extension.clone();
        if button::selection_row(
            ui,
            "Default Note Format",
            &mut ext,
            [
                (".qn Quicky", ".qn".to_string()),
                (".md Markdown", ".md".to_string()),
                (".txt Text", ".txt".to_string()),
            ],
            palette,
        ) {
            app.data.settings.editor.default_extension = ext;
            app.is_dirty = true;
            let _ = crate::storage::AppData::save_settings_to_path(
                &app.data.settings,
                &crate::storage::AppData::config_path(),
            );
            ctx.request_repaint();
        }

        // 4. Confirm Before Closing Note Tab
        if toggle::toggle_row(
            ui,
            "Confirm Before Closing Unsaved Note",
            &mut app.data.settings.editor.confirm_close_tab,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }
    });
}

/// Renders Workspace Sync & Text Automation card in the General tab.
fn render_general_sync_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "LIVE SYNC & TEXT INTEGRATION", palette, |ui| {
        // 1. Live Disk Sync
        if toggle::toggle_row(
            ui,
            "Live External Disk Sync (External file reload)",
            &mut app.data.settings.general.live_disk_sync,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // 2. Dynamic Wallpaper Theme Sync
        if toggle::toggle_row(
            ui,
            "Dynamic Wallpaper Color Sync (Pywal/Caelestia)",
            &mut app.data.settings.general.auto_sync_wallpaper,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // 3. Auto-close brackets, quotes, backticks
        if toggle::toggle_row(
            ui,
            "Auto-Close Paired Brackets & Markdown Quotes",
            &mut app.data.settings.general.auto_close_brackets,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // 4. Strip ANSI escape sequences on paste
        if toggle::toggle_row(
            ui,
            "Strip Terminal ANSI Colors on Paste",
            &mut app.data.settings.general.strip_ansi_on_paste,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        ui.add_space(4.0);
        let reset_general_btn = button::animated_action_button(
            ui,
            "↺ Reset General Settings to Defaults",
            palette,
            egui::vec2(ui.available_width(), 28.0),
        );
        if reset_general_btn.clicked() {
            app.data.settings.general = crate::models::settings::GeneralSettings::default();
            app.data.settings.validate_and_clamp();
            app.is_dirty = true;
            app.show_toast(
                "General settings reset to defaults",
                crate::ui::toast::ToastKind::Info,
            );
            ctx.request_repaint();
        }
    });
}

/// Renders Window Manager & CLI quick cheatsheet in the General tab.
fn render_general_cheatsheet_card(ui: &mut Ui, palette: &theme::Palette) {
    card::settings_card(ui, "⚡ GLOBAL SCRATCHPAD & CLI QUICKSTART", palette, |ui| {
        ui.label(
            RichText::new(
                "Launch Quicky as a global drop-down scratchpad across any Linux desktop:",
            )
            .font(FontId::proportional(11.5))
            .color(Color32::from_gray(200)),
        );

        ui.add_space(4.0);

        ui.label(
            RichText::new("✦ Hyprland Lua (~/.config/hypr/hyprland.lua):\n   hl.bind({ \"SUPER\" }, \"N\", \"exec\", \"quicky\")\n   hl.rule(\"float, class:^(quicky_notes)$\")\n   hl.rule(\"pin, class:^(quicky_notes)$\")\n\n✦ Sway / i3 (~/.config/sway/config):\n   bindsym $mod+n exec quicky\n   for_window [app_id=\"quicky_notes\"] floating enable, sticky enable\n\n✦ Niri (~/.config/niri/config.kdl):\n   binds { Mod+N { spawn \"quicky\"; } }\n\n✦ KDE Plasma / GNOME / XFCE:\n   System Settings → Custom Shortcuts → Add Shortcut → Command: quicky")
                .font(FontId::monospace(10.5))
                .color(palette.accent),
        );

        ui.add_space(4.0);

        ui.label(
            RichText::new("✦ CLI Launch Commands:\n   quicky                  # Open default scratchpad\n   quicky -f <folder>      # Open folder workspace tree\n   quicky <file.md>        # Open specific note file\n   quicky --new            # Open clean new note tab")
                .font(FontId::monospace(10.5))
                .color(Color32::from_gray(210)),
        );
    });
}

/// Renders a sleek, compact AI Theme Generator card for custom theme generation.
fn render_ai_theme_generator_card(
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
fn render_custom_colors_card(
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
fn render_system_font_card(
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
                            app.is_dirty = true;
                            ctx.request_repaint();
                        }
                    }
                });
            });
        });

        ui.add_space(4.0);

        // System UI Font Size Slider
        let ui_size_str = format!("{:.1}pt", app.data.settings.appearance.ui_font_size);
        if slider::slider_row(
            ui,
            "System UI Font Size",
            &ui_size_str,
            &mut app.data.settings.appearance.ui_font_size,
            crate::models::settings::MIN_UI_FONT_SIZE..=crate::models::settings::MAX_UI_FONT_SIZE,
            palette.accent,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
            app.is_dirty = true;
            ctx.request_repaint();
        }
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
fn render_appearance_card(
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

/// Renders Editor Behavior, Typography, Indentation, and Safety controls.
fn render_editor_behavior_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "EDITOR BEHAVIOR & TYPOGRAPHY", palette, |ui| {
        // Buffer / Coding Font Family Chooser
        let mono_fonts = crate::font::get_installed_monospace_fonts();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Buffer Font Family")
                    .font(FontId::proportional(12.5))
                    .color(Color32::from_gray(230)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let current_editor_font = app.data.settings.editor.editor_font.clone();
                let font_combo = egui::ComboBox::from_id_salt("editor_font_select").selected_text(
                    RichText::new(&current_editor_font)
                        .font(FontId::monospace(12.0))
                        .color(Color32::WHITE),
                );

                font_combo.show_ui(ui, |ui| {
                    for f_name in mono_fonts {
                        if ui
                            .selectable_value(
                                &mut app.data.settings.editor.editor_font,
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
                            app.is_dirty = true;
                            ctx.request_repaint();
                        }
                    }
                });
            });
        });

        ui.add_space(4.0);

        // Font Size Slider
        let font_str = format!("{:.0}pt", app.data.settings.editor.font_size);
        if slider::slider_row(
            ui,
            "Buffer Font Size",
            &font_str,
            &mut app.data.settings.editor.font_size,
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
            &mut app.data.settings.editor.monospace_font,
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
            &mut app.data.settings.editor.show_line_numbers,
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
            &mut app.data.settings.editor.tab_size,
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
            &mut app.data.settings.editor.default_extension,
            [
                (".qn (Quicky)", ".qn".to_string()),
                (".md (Markdown)", ".md".to_string()),
                (".txt (Text)", ".txt".to_string()),
            ],
            palette,
        ) {
            app.is_dirty = true;
            let _ = crate::storage::AppData::save_settings_to_path(
                &app.data.settings,
                &crate::storage::AppData::config_path(),
            );
            ctx.request_repaint();
        }

        // Confirm Before Closing Tab
        if toggle::toggle_row(
            ui,
            "Confirm Before Closing Note",
            &mut app.data.settings.editor.confirm_close_tab,
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
            &mut app.data.settings.editor.trim_trailing_whitespace,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Language Syntax Highlighting
        if toggle::toggle_row(
            ui,
            "Language Syntax Highlighting",
            &mut app.data.settings.editor.enable_syntax_highlighting,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Ghost Autocomplete Suggestions
        if toggle::toggle_row(
            ui,
            "Ghost Autocomplete (Tab to accept)",
            &mut app.data.settings.editor.enable_ghost_text,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            ctx.request_repaint();
        }

        // Auto Save Interval Slider
        let auto_save_str = format!("{}s", app.data.settings.editor.auto_save_seconds);
        if slider::slider_row_u32(
            ui,
            "Auto-Save Interval",
            &auto_save_str,
            &mut app.data.settings.editor.auto_save_seconds,
            1..=60,
            palette.accent,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            app.is_dirty = true;
            ctx.request_repaint();
        }

        ui.add_space(4.0);
        let reset_editor_btn = button::animated_action_button(
            ui,
            "↺ Reset Editor Settings to Defaults",
            palette,
            egui::vec2(ui.available_width(), 28.0),
        );
        if reset_editor_btn.clicked() {
            app.data.settings.editor = crate::models::settings::EditorSettings::default();
            app.data.settings.validate_and_clamp();
            app.is_dirty = true;
            app.show_toast(
                "Editor settings reset to defaults",
                crate::ui::toast::ToastKind::Info,
            );
            ctx.request_repaint();
        }
    });
}

/// Renders the AI Copilot & Provider configuration cards.
fn render_ai_settings_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "AI COPILOT & ASSISTANT", palette, |ui| {
        // Toggle Enable AI
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Enable AI Copilot (Ctrl+Enter)")
                    .font(FontId::proportional(13.0))
                    .color(Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if toggle::toggle_switch(ui, &mut app.data.settings.ai.enabled, palette.accent)
                    .changed()
                {
                    app.is_dirty = true;
                    ctx.request_repaint();
                }
            });
        });

        ui.add_space(8.0);

        // Provider Selector
        ui.label(
            RichText::new("AI Provider:")
                .font(FontId::proportional(12.0))
                .color(Color32::from_gray(210)),
        );

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            for provider in crate::ai::AiProvider::ALL {
                let is_selected = app.data.settings.ai.provider == provider;
                if button::selection_pill(ui, provider.label(), is_selected, palette).clicked() {
                    app.data.settings.ai.provider = provider;
                    app.data.settings.ai.model = provider.default_model().to_string();
                    app.is_dirty = true;
                    ctx.request_repaint();
                }
            }
        });

        ui.add_space(8.0);

        // Suggested Models Quick-Pill Selection
        let suggested = app.data.settings.ai.provider.suggested_models();
        if !suggested.is_empty() {
            ui.label(
                RichText::new("Popular / Modern Models:")
                    .font(FontId::proportional(11.5))
                    .color(Color32::from_gray(200)),
            );
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);
                for &model_name in suggested {
                    let is_current = app.data.settings.ai.model == model_name;
                    if button::selection_pill(ui, model_name, is_current, palette).clicked() {
                        app.data.settings.ai.model = model_name.to_string();
                        app.is_dirty = true;
                        ctx.request_repaint();
                    }
                }
            });
            ui.add_space(4.0);
        }

        // Custom Model Name Input
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Model:")
                    .font(FontId::proportional(12.0))
                    .color(Color32::from_gray(210)),
            );
            let model_edit = egui::TextEdit::singleline(&mut app.data.settings.ai.model)
                .font(FontId::monospace(12.0))
                .desired_width(ui.available_width());
            if ui.add(model_edit).changed() {
                app.is_dirty = true;
            }
        });

        ui.add_space(8.0);

        // API Key Input & Environment Variable detection status
        let current_provider = app.data.settings.ai.provider;
        let env_var_info = if let Some(env_name) = current_provider.env_var_name() {
            if std::env::var(env_name).is_ok() {
                format!("✓ Detected ${} in environment", env_name)
            } else {
                format!("Optional: set ${} in environment or enter below", env_name)
            }
        } else {
            "Not required for local Ollama / custom endpoints".to_string()
        };

        ui.label(
            RichText::new(env_var_info)
                .font(FontId::proportional(11.0))
                .color(
                    if current_provider
                        .env_var_name()
                        .is_some_and(|n| std::env::var(n).is_ok())
                    {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_gray(160)
                    },
                ),
        );

        let reveal_id = egui::Id::new("reveal_ai_api_key");
        let mut show_key = ui.data(|d| d.get_temp::<bool>(reveal_id).unwrap_or(false));

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("API Key:")
                    .font(FontId::proportional(12.0))
                    .color(Color32::from_gray(210)),
            );
            let key_edit = egui::TextEdit::singleline(&mut app.data.settings.ai.api_key)
                .password(!show_key)
                .hint_text("sk-...")
                .font(FontId::monospace(12.0))
                .desired_width(ui.available_width() - 40.0);
            if ui.add(key_edit).changed() {
                app.is_dirty = true;
            }

            let toggle_icon = if show_key { "🙈" } else { "👁" };
            let tip = if show_key {
                "Hide API Key"
            } else {
                "Reveal API Key"
            };
            if ui.button(toggle_icon).on_hover_text(tip).clicked() {
                show_key = !show_key;
                ui.data_mut(|d| d.insert_temp(reveal_id, show_key));
            }
        });

        ui.add_space(8.0);

        // Test API Connection button with live in-flight status
        let is_testing = app.ai_test_rx.is_some();
        let btn_label = if is_testing {
            "⏳ Testing Connection..."
        } else {
            "⚡ Test API Connection"
        };
        let test_btn = button::animated_action_button(
            ui,
            btn_label,
            palette,
            egui::vec2(ui.available_width(), 30.0),
        );
        if test_btn.clicked() && !is_testing {
            let req = crate::ai::AiRequest {
                target_text: "Hello".to_string(),
                context_before: String::new(),
                action: crate::ai::AiAction::Custom(
                    "Reply with 'API Connection Successful!' only.".to_string(),
                ),
                provider: app.data.settings.ai.provider,
                model: app.data.settings.ai.model.clone(),
                api_key: app.data.settings.ai.api_key.clone(),
                temperature: Some(app.data.settings.ai.temperature),
                system_prompt: app.data.settings.ai.system_prompt.clone(),
            };
            let rx = crate::ai::spawn_ai_request(req);
            app.ai_test_rx = Some(rx);
            app.show_toast(
                "Testing AI connection in background...",
                crate::ui::toast::ToastKind::Info,
            );
            ctx.request_repaint();
        }
    });

    card::settings_card(ui, "AI SYSTEM PROMPT & BEHAVIOR", palette, |ui| {
        ui.label(
            RichText::new("Custom instructions for how the AI formats and completes notes:")
                .font(FontId::proportional(11.5))
                .color(Color32::from_gray(190)),
        );

        ui.add_space(4.0);

        let prompt_edit = egui::TextEdit::multiline(&mut app.data.settings.ai.system_prompt)
            .font(FontId::proportional(12.0))
            .desired_rows(4)
            .desired_width(ui.available_width());
        if ui.add(prompt_edit).changed() {
            app.is_dirty = true;
        }

        ui.add_space(6.0);
        let reset_ai_btn = button::animated_action_button(
            ui,
            "↺ Reset AI Settings to Defaults",
            palette,
            egui::vec2(ui.available_width(), 28.0),
        );
        if reset_ai_btn.clicked() {
            app.data.settings.ai = crate::ai::AiSettings::default();
            app.is_dirty = true;
            app.show_toast(
                "AI configuration reset to defaults",
                crate::ui::toast::ToastKind::Info,
            );
            ctx.request_repaint();
        }
    });
}

/// Renders plugin manager master configuration, actions, and folder shortcuts.
fn render_plugins_manager_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "PLUGIN RUNTIME & EXTENSIONS", palette, |ui| {
        // Master Enable Toggle
        if toggle::toggle_row(
            ui,
            "Enable Rhai Plugin System",
            &mut app.data.settings.plugins.enabled,
            palette.accent,
        )
        .changed()
        {
            app.is_dirty = true;
            if app.data.settings.plugins.enabled {
                app.show_toast(
                    "Plugin system enabled ✓",
                    crate::ui::toast::ToastKind::Success,
                );
            } else {
                app.show_toast("Plugin system disabled", crate::ui::toast::ToastKind::Info);
            }
            ctx.request_repaint();
        }

        ui.add_space(6.0);

        let plugins_path = app.plugin_manager.plugins_dir.clone();
        ui.label(
            RichText::new(format!("📁 Directory: {}", plugins_path.to_string_lossy()))
                .font(FontId::monospace(10.5))
                .color(Color32::from_gray(190)),
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let col_w = (ui.available_width() - 16.0) / 3.0;

            let open_btn = button::animated_action_button(
                ui,
                "📁 Open Folder",
                palette,
                egui::vec2(col_w, 30.0),
            );
            if open_btn.clicked() {
                crate::ui::drag_drop::safe_open_folder(&plugins_path);
            }

            let reload_btn =
                button::animated_action_button(ui, "🔄 Reload", palette, egui::vec2(col_w, 30.0));
            if reload_btn.clicked() {
                app.plugin_manager
                    .reload(&app.data.settings.plugins.disabled_plugins);
                app.show_toast("Plugins reloaded ✓", crate::ui::toast::ToastKind::Success);
                ctx.request_repaint();
            }

            let example_btn = button::animated_action_button(
                ui,
                "✨ Starter Plugins",
                palette,
                egui::vec2(ui.available_width(), 30.0),
            );
            if example_btn.clicked() {
                app.plugin_manager.create_starter_templates();
                app.plugin_manager
                    .reload(&app.data.settings.plugins.disabled_plugins);
                app.show_toast(
                    "Starter templates created ✓",
                    crate::ui::toast::ToastKind::Success,
                );
                ctx.request_repaint();
            }
        });

        ui.add_space(6.0);
        ui.label(
            RichText::new("✓ Sandboxed pure-Rust Rhai execution engine with zero C-FFI\n✓ Custom header icons, hotkeys, and context menu extensions\n✓ Safe terminal launchers & note buffer automation")
                .font(FontId::proportional(11.5))
                .color(Color32::from_gray(175)),
        );
    });
}

/// Renders individual cards for each discovered plugin with toggle switch, badges, and metadata.
fn render_installed_plugins_list(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    // 1. Errors & Warnings card (if any)
    if !app.plugin_manager.error_log.is_empty() {
        card::settings_card(ui, "PLUGIN COMPILATION WARNINGS & ERRORS", palette, |ui| {
            for (plugin_name, error_msg) in &app.plugin_manager.error_log {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("⚠")
                            .font(FontId::proportional(14.0))
                            .color(crate::theme::ACCENT_AMBER),
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(plugin_name)
                                .font(FontId::proportional(12.5))
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.label(
                            RichText::new(error_msg)
                                .font(FontId::monospace(10.5))
                                .color(crate::theme::ACCENT_ROSE),
                        );
                    });
                });
                ui.add_space(4.0);
            }
        });
    }

    let plugin_count = app.plugin_manager.plugins.len();
    let card_title = format!("INSTALLED PLUGINS ({})", plugin_count);

    card::settings_card(ui, &card_title, palette, |ui| {
        if app.plugin_manager.plugins.is_empty() {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("No plugins found in plugins directory.\nClick 'Starter Plugins' above to generate sample plugins!")
                        .font(FontId::proportional(13.0))
                        .color(Color32::from_gray(140)),
                );
            });
            ui.add_space(10.0);
            return;
        }

        let mut toggle_plugin_id = None;

        for (idx, plugin) in app.plugin_manager.plugins.iter().enumerate() {
            if idx > 0 {
                crate::ui::draw_horizontal_divider(ui);
                ui.add_space(6.0);
            }

            let is_enabled = plugin.metadata.enabled;

            ui.vertical(|ui| {
                // Top row: Name, Version, Author, and Toggle
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&plugin.metadata.name)
                            .font(FontId::proportional(14.0))
                            .strong()
                            .color(if is_enabled {
                                Color32::WHITE
                            } else {
                                Color32::from_gray(140)
                            }),
                    );

                    // Version badge
                    crate::components::badge::custom_color_badge(
                        ui,
                        &format!("v{}", plugin.metadata.version),
                        palette.accent,
                    );

                    // Author badge
                    if !plugin.metadata.author.is_empty() {
                        crate::components::badge::shortcut_badge(
                            ui,
                            &format!("by {}", plugin.metadata.author),
                            palette,
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut enabled_state = is_enabled;
                        let toggle_resp =
                            toggle::toggle_switch(ui, &mut enabled_state, palette.accent);
                        if toggle_resp.changed() {
                            toggle_plugin_id = Some((plugin.metadata.id.clone(), enabled_state));
                        }
                    });
                });

                // Description
                if !plugin.metadata.description.is_empty() {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(&plugin.metadata.description)
                            .font(FontId::proportional(12.0))
                            .color(Color32::from_gray(175)),
                    );
                }

                // Capabilities badges (Header Buttons, Shortcuts, Context Menu)
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.spacing_mut().item_spacing.y = 4.0;

                    for btn in &plugin.header_buttons {
                        crate::components::badge::shortcut_badge(
                            ui,
                            &format!("Header: {} ({})", btn.icon, btn.id),
                            palette,
                        );
                    }

                    for s in &plugin.shortcuts {
                        crate::components::badge::shortcut_badge(
                            ui,
                            &format!("Shortcut: {}", s.key_combination),
                            palette,
                        );
                    }

                    for m in &plugin.menu_items {
                        crate::components::badge::shortcut_badge(
                            ui,
                            &format!("Context Menu: {}", m.label),
                            palette,
                        );
                    }

                    if let Some(ref fp) = plugin.metadata.file_path
                        && let Some(fname) = std::path::Path::new(fp).file_name()
                    {
                        crate::components::badge::shortcut_badge(
                            ui,
                            &format!("File: {}", fname.to_string_lossy()),
                            palette,
                        );
                    }
                });
            });

            ui.add_space(6.0);
        }

        // Apply toggled state
        if let Some((id, enabled)) = toggle_plugin_id {
            if enabled {
                app.data
                    .settings
                    .plugins
                    .disabled_plugins
                    .retain(|d| d != &id);
            } else if !app.data.settings.plugins.disabled_plugins.contains(&id) {
                app.data.settings.plugins.disabled_plugins.push(id.clone());
            }
            if let Some(p) = app
                .plugin_manager
                .plugins
                .iter_mut()
                .find(|p| p.metadata.id == id)
            {
                p.metadata.enabled = enabled;
            }
            app.is_dirty = true;
            ctx.request_repaint();
        }
    });
}

/// Renders the Quick Actions export button card.
fn render_quick_actions_card(app: &mut QuickyNotesApp, ui: &mut Ui, palette: &theme::Palette) {
    card::settings_card(ui, "QUICK ACTIONS", palette, |ui| {
        let export_btn = button::animated_action_button(
            ui,
            "📤  Export Current Note to File",
            palette,
            egui::vec2(ui.available_width(), 32.0),
        );

        if export_btn.clicked() {
            crate::ui::drag_drop::export_active_note(app);
        }
    });
}

/// Renders file persistence, data directory opening, and backup info card.
fn render_backup_info_card(
    app: &mut QuickyNotesApp,
    _ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "STORAGE & CONFIGURATION PORTABILITY", palette, |ui| {
        let cfg_path = crate::storage::AppData::config_path();
        let db_path = crate::storage::AppData::db_path();

        ui.label(
            RichText::new(format!("◆ SQLite DB:   {}", db_path.to_string_lossy()))
                .font(FontId::monospace(10.5))
                .color(Color32::from_gray(190)),
        );
        ui.label(
            RichText::new(format!("⚙ Config JSON: {}", cfg_path.to_string_lossy()))
                .font(FontId::monospace(10.5))
                .color(Color32::from_gray(190)),
        );

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            let col_w = (ui.available_width() - 8.0) * 0.5;
            let export_cfg_btn = button::animated_action_button(
                ui,
                "📤 Export Settings",
                palette,
                egui::vec2(col_w, 30.0),
            );
            if export_cfg_btn.clicked() {
                crate::ui::drag_drop::export_settings(app);
            }

            let import_cfg_btn = button::animated_action_button(
                ui,
                "📥 Import Settings",
                palette,
                egui::vec2(ui.available_width(), 30.0),
            );
            if import_cfg_btn.clicked() {
                crate::ui::drag_drop::import_settings(app);
            }
        });

        ui.add_space(4.0);

        let open_folder_btn = button::animated_action_button(
            ui,
            "📁 Open Data Folder in File Manager",
            palette,
            egui::vec2(ui.available_width(), 30.0),
        );

        if open_folder_btn.clicked()
            && let Some(parent) = db_path.parent()
        {
            crate::ui::drag_drop::safe_open_folder(parent);
        }

        ui.add_space(4.0);

        ui.label(
            RichText::new("✓ SQLite ACID transactional persistence for all notes\n✓ Encrypted / machine-salted API key storage\n✓ Atomic config.json writes with corruption recovery")
                .font(FontId::proportional(11.5))
                .color(Color32::from_gray(180)),
        );
    });
}

/// Renders the Learned Custom Vocabulary, Markov N-Gram stats, and re-indexing card.
fn render_learned_dictionary_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "✦ LEARNED VOCABULARY & DICTIONARY", palette, |ui| {
        let (bigrams, trigrams) = app.suggestion_engine.transition_counts();
        let word_count = app.suggestion_engine.word_count();

        ui.label(
            RichText::new(
                "Real-time statistical Markov language model auto-trained from your notes:",
            )
            .font(FontId::proportional(11.5))
            .color(Color32::from_gray(190)),
        );

        ui.add_space(4.0);

        // Stats grid
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("✦ Total Vocabulary: {} words", word_count))
                    .font(FontId::monospace(11.0))
                    .color(palette.accent),
            );
        });
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "🔗 Learned Markov Transitions: {} bigrams, {} trigrams",
                    bigrams, trigrams
                ))
                .font(FontId::monospace(11.0))
                .color(Color32::from_gray(210)),
            );
        });

        ui.add_space(6.0);

        // Action Buttons: Re-train & Reset
        ui.horizontal(|ui| {
            let retrain_btn = button::animated_action_button(
                ui,
                "🔄 Re-Index from Notes",
                palette,
                egui::vec2((ui.available_width() - 8.0) * 0.60, 30.0),
            );

            if retrain_btn.clicked() {
                let all_contents: Vec<String> =
                    app.data.notes.iter().map(|n| n.content.clone()).collect();
                let count = all_contents.len();
                app.suggestion_engine
                    .retrain_from_notes(all_contents.iter().map(|s| s.as_str()));
                app.show_toast(
                    format!("✓ Re-indexed & trained vocabulary from {} notes", count),
                    crate::ui::toast::ToastKind::Success,
                );
                ctx.request_repaint();
            }

            let clear_btn = button::animated_action_button(
                ui,
                "🗑 Reset Model",
                palette,
                egui::vec2(ui.available_width(), 30.0),
            );

            if clear_btn.clicked() {
                app.suggestion_engine = crate::suggest::SuggestionEngine::new();
                app.show_toast(
                    "Learned Markov transitions reset to base dictionary",
                    crate::ui::toast::ToastKind::Info,
                );
                ctx.request_repaint();
            }
        });
    });
}

/// Renders interactive keyboard shortcut customizer and registry.
fn render_shortcuts_card(
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

const APP_ICON_BYTES: &[u8] = include_bytes!("../../assets/icon.png");

/// Renders the About This App card and Factory Reset control.
fn render_about_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    card::settings_card(ui, "ABOUT QUICKY NOTES", palette, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;

            let icon_source = egui::ImageSource::Bytes {
                uri: std::borrow::Cow::Borrowed("bytes://quicky_icon.png"),
                bytes: egui::load::Bytes::Static(APP_ICON_BYTES),
            };

            ui.add(
                egui::Image::new(icon_source)
                    .fit_to_exact_size(egui::vec2(54.0, 54.0))
                    .corner_radius(CornerRadius::same(10)),
            );

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Quicky Notes")
                            .font(FontId::proportional(16.0))
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .font(FontId::monospace(11.0))
                            .color(palette.accent),
                    );
                });

                ui.label(
                    RichText::new(
                        "Floating glassmorphism note widget and code scratchpad for Linux.",
                    )
                    .font(FontId::proportional(12.0))
                    .color(Color32::from_gray(210)),
                );

                ui.label(
                    RichText::new("Built with Rust • SQLite ACID • egui • Wayland & X11 Native")
                        .font(FontId::proportional(10.5))
                        .color(Color32::from_gray(160)),
                );
            });
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let gh_btn = button::animated_action_button(
                ui,
                "⭐ GitHub Repository ↗",
                palette,
                egui::vec2(165.0, 28.0),
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

    // Reset All Settings to Default Section
    card::settings_card(ui, "FACTORY RESET", palette, |ui| {
        ui.label(
            RichText::new(
                "Restore all appearance, theme colors, editor preferences, typography, window presets, and keybindings back to factory default values.",
            )
            .font(FontId::proportional(11.5))
            .color(Color32::from_gray(190)),
        );

        ui.add_space(8.0);

        let reset_all_btn = button::animated_danger_button(
            ui,
            "↺  Reset All Settings to Default",
            egui::vec2(ui.available_width(), 32.0),
        );

        if reset_all_btn
            .on_hover_text("Reset all settings and shortcuts to factory defaults")
            .clicked()
        {
            app.data.settings = crate::settings::AppSettings::default();
            app.data.settings.validate_and_clamp();
            theme::setup_glassmorphism_theme(ctx, &app.data.settings);
            let level = if app.data.settings.window.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
            app.is_dirty = true;
            let _ = app.data.save();
            app.show_toast(
                "All settings & shortcuts reset to factory defaults",
                crate::app::ToastKind::Success,
            );
            ctx.request_repaint();
        }
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
