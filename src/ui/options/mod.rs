//! Settings & Preferences modular navigation drawer and control shell.

pub mod about;
pub mod advanced;
pub mod ai;
pub mod appearance;
pub mod editor_tab;
pub mod files_backup;
pub mod general;
pub mod plugins;
pub mod shortcuts;

use crate::app::QuickyNotesApp;
use crate::components::button;
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, Ui};

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
    Advanced,
    About,
}

impl SettingsTab {
    pub const ALL: [Self; 9] = [
        Self::General,
        Self::Appearance,
        Self::Editor,
        Self::Ai,
        Self::Plugins,
        Self::FilesBackup,
        Self::Shortcuts,
        Self::Advanced,
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
            Self::Advanced => ("⚡", "Advanced"),
            Self::About => ("ℹ", "About"),
        }
    }
}

/// Renders the Settings & Preferences drawer shell with left sidebar and active tab content.
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
                                    general::render_general_tab(app, ctx, ui, &palette);
                                }
                                SettingsTab::Appearance => {
                                    appearance::render_appearance_tab(app, ctx, ui, &palette);
                                }
                                SettingsTab::Editor => {
                                    editor_tab::render_editor_tab(app, ctx, ui, &palette);
                                }
                                SettingsTab::Ai => {
                                    ai::render_ai_tab(app, ctx, ui, &palette);
                                }
                                SettingsTab::Plugins => {
                                    plugins::render_plugins_tab(app, ctx, ui, &palette);
                                }
                                SettingsTab::FilesBackup => {
                                    files_backup::render_files_backup_tab(app, ctx, ui, &palette);
                                }
                                SettingsTab::Shortcuts => {
                                    shortcuts::render_shortcuts_tab(app, ctx, ui, &palette);
                                }
                                SettingsTab::Advanced => {
                                    advanced::render_advanced_tab(app, ctx, ui, &palette);
                                }
                                SettingsTab::About => {
                                    about::render_about_tab(app, ctx, ui, &palette);
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
            let is_pressed = response.is_pointer_button_down_on();

            let sel_anim = ui
                .ctx()
                .animate_bool_responsive(response.id.with("sel"), is_selected);
            let hov_anim = ui
                .ctx()
                .animate_bool_responsive(response.id.with("hov"), is_hovered);
            let press_anim = ui
                .ctx()
                .animate_bool_responsive(response.id.with("press"), is_pressed);

            let mut draw_rect = rect;
            if press_anim > 0.01 {
                draw_rect = draw_rect.translate(egui::vec2(0.0, 1.0 * press_anim));
            }

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

            let resting_border = theme::Palette::with_alpha(palette.border, 65);
            let hovered_border = theme::Palette::with_alpha(palette.border, 150);
            let selected_border = theme::Palette::with_alpha(palette.accent, 220);

            let border_color = if sel_anim > 0.01 {
                theme::Palette::interpolate_color(resting_border, selected_border, sel_anim)
            } else if hov_anim > 0.01 {
                theme::Palette::interpolate_color(resting_border, hovered_border, hov_anim)
            } else {
                resting_border
            };
            let stroke = Stroke::new(1.0 + 0.2 * sel_anim, border_color);

            ui.painter()
                .rect_filled(draw_rect, CornerRadius::same(8), bg);
            ui.painter().rect_stroke(
                draw_rect,
                CornerRadius::same(8),
                stroke,
                egui::StrokeKind::Inside,
            );

            // Centered icon in dedicated 28px left area with smooth hover micro-scale and lift
            let icon_scale = 1.0 + (hov_anim * 0.08);
            let icon_y_lift = -hov_anim * 0.75;
            let icon_font_size = (14.0 * icon_scale).round();
            ui.painter().text(
                egui::pos2(draw_rect.left() + 18.0, draw_rect.center().y + icon_y_lift),
                egui::Align2::CENTER_CENTER,
                icon,
                FontId::proportional(icon_font_size),
                Color32::WHITE,
            );

            // Left-aligned Label with consistent offset and smooth color transition on hover and selection
            let base_text_color = Color32::from_gray(185);
            let hovered_text_color = Color32::WHITE;
            let active_text_color = palette.accent;

            let text_color = if sel_anim > 0.01 {
                theme::Palette::interpolate_color(base_text_color, active_text_color, sel_anim)
            } else if hov_anim > 0.01 {
                theme::Palette::interpolate_color(base_text_color, hovered_text_color, hov_anim)
            } else {
                base_text_color
            };
            ui.painter().text(
                egui::pos2(draw_rect.left() + 38.0, draw_rect.center().y),
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

/// Renders the bottom action bar matching image.png with persistent Done button and status.
fn render_bottom_bar(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;

        let btn_size = egui::vec2(72.0, 26.0);
        let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
        let is_hovered = response.hovered();
        let is_pressed = response.is_pointer_button_down_on();

        let hov_anim = ui
            .ctx()
            .animate_bool_responsive(response.id.with("done_hov"), is_hovered);
        let press_anim = ui
            .ctx()
            .animate_bool_responsive(response.id.with("done_press"), is_pressed);

        let mut draw_rect = rect;
        if press_anim > 0.01 {
            draw_rect = draw_rect.translate(egui::vec2(0.0, 1.0 * press_anim));
        }

        let normal_bg = Color32::from_rgba_unmultiplied(
            palette.card.r(),
            palette.card.g(),
            palette.card.b(),
            200,
        );
        let hovered_bg = theme::Palette::lighten(palette.card, 35, 240);
        let bg_color = theme::Palette::interpolate_color(normal_bg, hovered_bg, hov_anim);

        let resting_border = theme::Palette::with_alpha(palette.border, 100);
        let hovered_border = palette.accent;
        let stroke_color =
            theme::Palette::interpolate_color(resting_border, hovered_border, hov_anim);

        ui.painter()
            .rect_filled(draw_rect, CornerRadius::same(6), bg_color);
        ui.painter().rect_stroke(
            draw_rect,
            CornerRadius::same(6),
            Stroke::new(1.0 + 0.2 * hov_anim, stroke_color),
            egui::StrokeKind::Inside,
        );

        let y_lift = -hov_anim * 0.5;
        ui.painter().text(
            egui::pos2(draw_rect.center().x, draw_rect.center().y + y_lift),
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
            egui::StrokeKind::Inside,
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
