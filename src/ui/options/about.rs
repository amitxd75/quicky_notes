//! About tab preferences: Application metadata, hardware acceleration diagnostics, memory metrics, and factory reset.

use crate::app::QuickyNotesApp;
use crate::components::{button, card};
use crate::platform::diagnostics;
use crate::theme;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Ui, ViewportCommand};

const APP_ICON_BYTES: &[u8] = include_bytes!("../../../assets/icon.png");

/// Renders all cards in the About settings tab.
pub fn render_about_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    render_about_card(app, ctx, ui, palette);
    render_diagnostics_card(app, ui, palette);
    render_factory_reset_card(app, ctx, ui, palette);
}

/// Renders the About This App card.
pub fn render_about_card(
    _app: &mut QuickyNotesApp,
    _ctx: &egui::Context,
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
}

/// Renders Hardware Acceleration, Display Protocol, and Live Memory Diagnostics card.
pub fn render_diagnostics_card(app: &mut QuickyNotesApp, ui: &mut Ui, palette: &theme::Palette) {
    card::settings_card(
        ui,
        "⚡ HARDWARE ACCELERATION & SYSTEM DIAGNOSTICS",
        palette,
        |ui| {
            let display_server = diagnostics::detect_display_server();
            let proc_mem = diagnostics::get_process_memory();
            let cpu_usage = diagnostics::get_process_cpu_usage();

            // 1. Hardware & Compositor Architecture
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("✦ Display Compositor:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(200)),
                );
                ui.label(
                    RichText::new(display_server)
                        .font(FontId::monospace(11.5))
                        .color(palette.accent),
                );
            });

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("✦ Graphics Pipeline:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(200)),
                );
                let pipeline_label = if app.data.settings.advanced.hardware_acceleration {
                    "GPU Hardware-Accelerated (OpenGL / Vulkan / wgpu)"
                } else {
                    "Software Fallback (CPU Rasterizer)"
                };
                ui.label(
                    RichText::new(pipeline_label)
                        .font(FontId::monospace(11.5))
                        .color(if app.data.settings.advanced.hardware_acceleration {
                            Color32::WHITE
                        } else {
                            crate::theme::ACCENT_AMBER
                        }),
                );
            });

            ui.add_space(6.0);
            crate::ui::draw_horizontal_divider(ui);
            ui.add_space(6.0);

            // 2. Application Process Footprint
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("✦ App Private Heap:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(200)),
                );

                let heap_str = if let Some(mem) = proc_mem {
                    format!(
                        "{} Private RAM",
                        diagnostics::format_bytes(mem.private_heap_bytes)
                    )
                } else {
                    "Linux /proc unavailable".to_string()
                };

                ui.label(
                    RichText::new(heap_str)
                        .font(FontId::monospace(11.5))
                        .color(palette.accent),
                );
            });

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("✦ Total Mapped RSS:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(200)),
                );

                let mem_str = if let Some(mem) = proc_mem {
                    format!(
                        "{} (includes shared Mesa/GPU driver libraries)",
                        diagnostics::format_bytes(mem.resident_bytes)
                    )
                } else {
                    "Linux /proc unavailable".to_string()
                };

                ui.label(
                    RichText::new(mem_str)
                        .font(FontId::monospace(11.5))
                        .color(Color32::WHITE),
                );
            });

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("✦ Process CPU Load:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(200)),
                );
                ui.label(
                    RichText::new(format!("{:.1}% CPU", cpu_usage))
                        .font(FontId::monospace(11.5))
                        .color(if cpu_usage > 5.0 {
                            crate::theme::ACCENT_AMBER
                        } else {
                            crate::theme::ACCENT_EMERALD
                        }),
                );
            });

            ui.add_space(6.0);
            crate::ui::draw_horizontal_divider(ui);
            ui.add_space(6.0);

            // 3. Tab & Notes Buffer Footprint Calculation
            let total_tabs = app.data.notes.len();
            let mut total_content_bytes: u64 = 0;
            let mut total_att_bytes: u64 = 0;
            let mut total_att_count: usize = 0;
            let mut active_tab_bytes: u64 = 0;

            for note in &app.data.notes {
                let content_b = note.content.len() as u64;
                let att_b: u64 = note.attachments.iter().map(|a| a.data.len() as u64).sum();
                total_content_bytes += content_b;
                total_att_bytes += att_b;
                total_att_count += note.attachments.len();

                if app.data.active_note_id.as_deref() == Some(&note.id) {
                    active_tab_bytes = content_b + att_b;
                }
            }

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("✦ Active Tab Buffer:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(200)),
                );
                ui.label(
                    RichText::new(diagnostics::format_bytes(active_tab_bytes))
                        .font(FontId::monospace(11.5))
                        .color(palette.accent),
                );
            });

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("✦ Total Tabs & Notes:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(200)),
                );
                ui.label(
                    RichText::new(format!(
                        "{} across {} open notes",
                        diagnostics::format_bytes(total_content_bytes),
                        total_tabs
                    ))
                    .font(FontId::monospace(11.5))
                    .color(Color32::WHITE),
                );
            });

            if total_att_count > 0 {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("✦ Embedded Attachments:")
                            .font(FontId::proportional(12.0))
                            .color(Color32::from_gray(200)),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} ({} image files)",
                            diagnostics::format_bytes(total_att_bytes),
                            total_att_count
                        ))
                        .font(FontId::monospace(11.5))
                        .color(Color32::WHITE),
                    );
                });
            }

            // Suggestion Engine Vocabulary Memory
            let vocab_count = app.suggestion_engine.word_count();
            let (bigrams, trigrams) = app.suggestion_engine.transition_counts();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("✦ Markov Engine Vocabulary:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(200)),
                );
                ui.label(
                    RichText::new(format!(
                        "{} words • {} transitions",
                        vocab_count,
                        bigrams + trigrams
                    ))
                    .font(FontId::monospace(11.5))
                    .color(Color32::from_gray(190)),
                );
            });

            // Live continuous refresh while diagnostics card is visible
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        },
    );
}

/// Renders the Factory Reset section card.
pub fn render_factory_reset_card(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
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
