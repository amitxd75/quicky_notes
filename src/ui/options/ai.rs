//! AI tab preferences: AI Copilot settings, provider configuration, model selection, and system prompt.

use crate::app::QuickyNotesApp;
use crate::components::{button, card, toggle};
use crate::theme;
use eframe::egui::{self, Color32, FontId, RichText, Ui};

/// Renders all cards in the AI Copilot settings tab.
pub fn render_ai_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    render_ai_settings_card(app, ctx, ui, palette);
}

/// Renders the AI Copilot & Provider configuration cards.
pub fn render_ai_settings_card(
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
