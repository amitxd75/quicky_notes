//! Editor tab preferences: Typography, indentation, buffer font, line numbers, syntax highlighting, and auto-save.

use crate::app::QuickyNotesApp;
use crate::components::{button, card, slider, toggle};
use crate::theme;
use eframe::egui::{self, Color32, FontId, RichText, Ui};

/// Renders all cards in the Editor settings tab.
pub fn render_editor_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    super::appearance::render_system_font_card(app, ctx, ui, palette);
    render_editor_behavior_card(app, ctx, ui, palette);
}

/// Renders Editor Behavior, Typography, Indentation, and Safety controls.
pub fn render_editor_behavior_card(
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

        // Buffer Font Size Slider
        let font_str = format!("{:.0}pt", app.data.settings.editor.font_size);
        if slider::slider_row(
            ui,
            "Buffer Font Size",
            &font_str,
            &mut app.data.settings.editor.font_size,
            crate::models::settings::MIN_FONT_SIZE..=crate::models::settings::MAX_FONT_SIZE,
            palette.accent,
        )
        .changed()
        {
            app.data.settings.validate_and_clamp();
            app.is_dirty = true;
            let _ = crate::storage::AppData::save_settings_to_path(
                &app.data.settings,
                &crate::storage::AppData::config_path(),
            );
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
