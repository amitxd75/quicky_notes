//! Plugins tab preferences: Plugin runtime configuration, plugins directory, and installed plugins list.

use crate::app::QuickyNotesApp;
use crate::components::{button, card, toggle};
use crate::theme;
use eframe::egui::{self, Color32, FontId, RichText, Ui};

/// Renders all cards in the Plugins settings tab.
pub fn render_plugins_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    render_plugins_manager_card(app, ctx, ui, palette);
    render_installed_plugins_list(app, ctx, ui, palette);
}

/// Renders plugin manager master configuration, actions, and folder shortcuts.
pub fn render_plugins_manager_card(
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
    });
}

/// Renders individual cards for each discovered plugin with toggle switch, badges, and metadata.
pub fn render_installed_plugins_list(
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
                ui.add_space(4.0);
                crate::ui::draw_horizontal_divider(ui);
                ui.add_space(8.0);
            }

            let is_enabled = plugin.metadata.enabled;

            ui.vertical(|ui| {
                // Top row: Name, Version, Author, and Toggle (Fluid responsive layout)
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    let toggle_width = 44.0;
                    let left_w = (ui.available_width() - toggle_width - 8.0).max(80.0);

                    ui.allocate_ui_with_layout(
                        egui::vec2(left_w, 24.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;

                            ui.label(
                                RichText::new(&plugin.metadata.name)
                                    .font(FontId::proportional(13.5))
                                    .strong()
                                    .color(if is_enabled {
                                        Color32::WHITE
                                    } else {
                                        Color32::from_gray(140)
                                    }),
                            );

                            // Compact Version badge
                            crate::components::badge::custom_color_badge(
                                ui,
                                &format!("v{}", plugin.metadata.version),
                                palette.accent,
                            );

                            // Author tag
                            if !plugin.metadata.author.is_empty() {
                                ui.label(
                                    RichText::new(format!("by {}", plugin.metadata.author))
                                        .font(FontId::proportional(11.0))
                                        .color(Color32::from_gray(140)),
                                );
                            }
                        },
                    );

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
                            .font(FontId::proportional(11.5))
                            .color(Color32::from_gray(165)),
                    );
                }

                // Capabilities badges (Clean, icon-prefixed, compact)
                let has_caps = !plugin.header_buttons.is_empty()
                    || !plugin.shortcuts.is_empty()
                    || !plugin.menu_items.is_empty()
                    || plugin.metadata.file_path.is_some();

                if has_caps {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        for btn in &plugin.header_buttons {
                            crate::components::badge::shortcut_badge(
                                ui,
                                &format!("📌 {}", btn.icon),
                                palette,
                            );
                        }

                        for s in &plugin.shortcuts {
                            crate::components::badge::shortcut_badge(
                                ui,
                                &format!("⌨ {}", s.key_combination),
                                palette,
                            );
                        }

                        for m in &plugin.menu_items {
                            crate::components::badge::shortcut_badge(
                                ui,
                                &format!("📋 {}", m.label),
                                palette,
                            );
                        }

                        if let Some(ref fp) = plugin.metadata.file_path
                            && let Some(fname) = std::path::Path::new(fp).file_name()
                        {
                            crate::components::badge::shortcut_badge(
                                ui,
                                &format!("📄 {}", fname.to_string_lossy()),
                                palette,
                            );
                        }
                    });
                }
            });

            ui.add_space(4.0);
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
