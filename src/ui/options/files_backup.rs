//! Files & Backup tab preferences: SQLite storage, settings import/export, data folder access, and learned vocabulary model.

use crate::app::QuickyNotesApp;
use crate::components::{button, card};
use crate::theme;
use eframe::egui::{self, Color32, FontId, RichText, Ui};

/// Renders all cards in the Files & Backup settings tab.
pub fn render_files_backup_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    render_backup_info_card(app, ctx, ui, palette);
    render_learned_dictionary_card(app, ctx, ui, palette);
    render_quick_actions_card(app, ui, palette);
}

/// Renders file persistence, data directory opening, and backup info card.
pub fn render_backup_info_card(
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
pub fn render_learned_dictionary_card(
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

        let vocab_size = &mut app.data.settings.advanced.suggest_vocab_size;
        if button::selection_row(
            ui,
            "Base Dictionary Capacity",
            vocab_size,
            [
                ("10k", 10_000),
                ("25k", 25_000),
                ("50k (Default)", 50_000),
                ("100k", 100_000),
                ("Full 333k", 333_304),
            ],
            palette,
        ) {
            let note_texts: Vec<String> =
                app.data.notes.iter().map(|n| n.content.clone()).collect();
            app.suggestion_engine.reload_with_limit(
                app.data.settings.advanced.suggest_vocab_size,
                note_texts.iter().map(|s| s.as_str()),
            );
            app.is_dirty = true;
            let _ = crate::storage::AppData::save_settings_to_path(
                &app.data.settings,
                &crate::storage::AppData::config_path(),
            );
            app.show_toast(
                format!(
                    "Vocabulary capacity set to {} words",
                    app.suggestion_engine.word_count()
                ),
                crate::ui::toast::ToastKind::Success,
            );
            ctx.request_repaint();
        }

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

/// Renders the Quick Actions export button card.
pub fn render_quick_actions_card(app: &mut QuickyNotesApp, ui: &mut Ui, palette: &theme::Palette) {
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
