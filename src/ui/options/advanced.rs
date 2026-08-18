//! Advanced tab preferences: Storage quotas, attachment limits, image dimensions, texture LRU, Rhai script limits, timeouts, and engine tunables.

use crate::app::QuickyNotesApp;
use crate::components::{button, card, slider, stepper, toggle};
use crate::theme;
use eframe::egui::{self, Ui};

/// Renders all cards in the Advanced settings tab.
pub fn render_advanced_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    let mut settings_modified = false;

    // 1. Storage & Resource Limits Card
    card::settings_card(ui, "📦 STORAGE & RESOURCE LIMITS", palette, |ui| {
        let max_qn = &mut app.data.settings.advanced.max_qn_file_size_mb;
        if stepper::stepper_row_usize(
            ui,
            "Max .qn Container File Size",
            max_qn,
            1..=200,
            5,
            "MB",
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_att = &mut app.data.settings.advanced.max_attachment_size_mb;
        if stepper::stepper_row_usize(
            ui,
            "Max Single Attachment Size",
            max_att,
            1..=100,
            1,
            "MB",
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_tot = &mut app.data.settings.advanced.max_total_attachments_size_mb;
        if stepper::stepper_row_usize(
            ui,
            "Max Cumulative Note Attachments",
            max_tot,
            5..=500,
            10,
            "MB",
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_cnt = &mut app.data.settings.advanced.max_attachments_per_note;
        if stepper::stepper_row_usize(
            ui,
            "Max Attachments Per Note",
            max_cnt,
            5..=500,
            5,
            "",
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_title = &mut app.data.settings.advanced.max_note_title_len;
        if stepper::stepper_row_usize(
            ui,
            "Max Note Title Length",
            max_title,
            16..=512,
            16,
            "chars",
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }
    });

    ui.add_space(4.0);

    // 2. Images & GPU Texture Pipeline Card
    card::settings_card(ui, "🖼 IMAGES & GPU TEXTURE PIPELINE", palette, |ui| {
        let max_tex = &mut app.data.settings.advanced.max_texture_cache_entries;
        if button::selection_row(
            ui,
            "GPU Texture Cache Capacity",
            max_tex,
            [("32", 32), ("64", 64), ("128", 128), ("256", 256)],
            palette,
        ) {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_dim = &mut app.data.settings.advanced.max_image_dimension;
        if button::selection_row(
            ui,
            "Max Image Dimension",
            max_dim,
            [
                ("2K (2048px)", 2048),
                ("4K (4096px)", 4096),
                ("8K (8192px)", 8192),
                ("16K", 16384),
            ],
            palette,
        ) {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_pix = &mut app.data.settings.advanced.max_image_pixels;
        if button::selection_row(
            ui,
            "Max Image Total Pixels",
            max_pix,
            [
                ("8 MP", 8_388_608),
                ("16 MP", 16_777_216),
                ("33 MP", 33_554_432),
                ("67 MP", 67_108_864),
            ],
            palette,
        ) {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let popup_w = &mut app.data.settings.advanced.image_popup_max_width;
        let disp = format!("{:.0} px", popup_w);
        if slider::slider_row(
            ui,
            "Image Popover Max Width",
            &disp,
            popup_w,
            240.0..=800.0,
            palette.accent,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        if toggle::toggle_row(
            ui,
            "GPU Hardware Acceleration (Requires App Restart)",
            &mut app.data.settings.advanced.hardware_acceleration,
            palette.accent,
        )
        .changed()
        {
            settings_modified = true;
            if app.data.settings.advanced.hardware_acceleration {
                app.show_toast(
                    "GPU acceleration enabled (takes effect on restart)",
                    crate::ui::toast::ToastKind::Success,
                );
            } else {
                app.show_toast(
                    "GPU acceleration disabled (software fallback on restart)",
                    crate::ui::toast::ToastKind::Info,
                );
            }
        }
    });

    ui.add_space(4.0);

    // 3. Plugin Runtime & Execution Quotas Card
    card::settings_card(ui, "🔌 PLUGIN RUNTIME & EXECUTION QUOTAS", palette, |ui| {
        let max_ops = &mut app.data.settings.advanced.max_script_operations;
        let disp_ops = format!("{:.0}k ops", *max_ops as f64 / 1000.0);
        if stepper::stepper_row_u64(
            ui,
            "Script Execution Budget",
            max_ops,
            10_000..=5_000_000,
            50_000,
            &disp_ops,
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_call = &mut app.data.settings.advanced.max_script_call_levels;
        if stepper::stepper_row_usize(
            ui,
            "Max Script Call Depth",
            max_call,
            10..=200,
            5,
            "levels",
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let exec_timeout = &mut app.data.settings.advanced.exec_timeout_ms;
        let disp_timeout = format!("{:.1} s", *exec_timeout as f64 / 1000.0);
        if stepper::stepper_row_u64(
            ui,
            "Command Process Timeout",
            exec_timeout,
            500..=15_000,
            500,
            &disp_timeout,
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let http_timeout = &mut app.data.settings.advanced.http_timeout_secs;
        let disp_http = format!("{} s", http_timeout);
        if stepper::stepper_row_u64(
            ui,
            "HTTP Request Timeout",
            http_timeout,
            1..=30,
            1,
            &disp_http,
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_exec_out = &mut app.data.settings.advanced.max_exec_output_bytes;
        let disp_exec_mb = format!("{:.1} MB", *max_exec_out as f64 / 1_048_576.0);
        if stepper::stepper_row_u64(
            ui,
            "Max Captured Stdout Buffer",
            &mut (*max_exec_out as u64),
            64_000..=10_485_760,
            262_144,
            &disp_exec_mb,
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let max_http_resp = &mut app.data.settings.advanced.max_http_response_bytes;
        let disp_http_mb = format!("{:.1} MB", *max_http_resp as f64 / 1_048_576.0);
        if stepper::stepper_row_u64(
            ui,
            "Max HTTP Response Body Buffer",
            &mut (*max_http_resp as u64),
            524_288..=50_000_000,
            1_048_576,
            &disp_http_mb,
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }
    });

    ui.add_space(4.0);

    // 4. Workspace & Engine Tunables Card
    card::settings_card(ui, "⚙ WORKSPACE & ENGINE TUNABLES", palette, |ui| {
        let status_dur = &mut app.data.settings.advanced.status_msg_duration_secs;
        let disp_dur = format!("{:.1} s", status_dur);
        if stepper::stepper_row_f32(
            ui,
            "Status Notification Duration",
            status_dur,
            1.0..=10.0,
            0.5,
            &disp_dur,
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let scan_depth = &mut app.data.settings.advanced.folder_max_scan_depth;
        if stepper::stepper_row_usize(
            ui,
            "Folder Tree Recursive Scan Depth",
            scan_depth,
            1..=20,
            1,
            "levels",
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let sug_depth = &mut app.data.settings.advanced.suggest_max_search_depth;
        if stepper::stepper_row_usize(
            ui,
            "Autocomplete Trie Search Depth",
            sug_depth,
            3..=30,
            1,
            "chars",
            palette,
        )
        .changed()
        {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let weight_mult = &mut app.data.settings.advanced.suggest_user_weight_multiplier;
        if button::selection_row(
            ui,
            "Learned Word Frequency Weight",
            weight_mult,
            [
                ("10k×", 10_000),
                ("50k×", 50_000),
                ("100k×", 100_000),
                ("500k×", 500_000),
            ],
            palette,
        ) {
            settings_modified = true;
        }

        ui.add_space(4.0);

        let vocab_size = &mut app.data.settings.advanced.suggest_vocab_size;
        if button::selection_row(
            ui,
            "Base Dictionary Vocabulary Limit",
            vocab_size,
            [
                ("10k (Ultra-Low)", 10_000),
                ("25k (Compact)", 25_000),
                ("50k (Default)", 50_000),
                ("100k", 100_000),
                ("Full 333k", 333_304),
            ],
            palette,
        ) {
            settings_modified = true;
            let note_texts: Vec<String> =
                app.data.notes.iter().map(|n| n.content.clone()).collect();
            app.suggestion_engine.reload_with_limit(
                app.data.settings.advanced.suggest_vocab_size,
                note_texts.iter().map(|s| s.as_str()),
            );
            app.show_toast(
                format!(
                    "Vocabulary base updated to {} words",
                    app.suggestion_engine.word_count()
                ),
                crate::ui::toast::ToastKind::Success,
            );
        }
    });

    ui.add_space(6.0);

    // 5. Reset All Advanced Settings Button
    ui.horizontal(|ui| {
        let reset_btn = button::animated_action_button(
            ui,
            "↺ Reset All Advanced Settings to Defaults",
            palette,
            egui::vec2(ui.available_width(), 32.0),
        );
        if reset_btn.clicked() {
            app.data.settings.advanced = crate::models::settings::AdvancedSettings::default();
            app.is_dirty = true;
            let _ = crate::storage::AppData::save_settings_to_path(
                &app.data.settings,
                &crate::storage::AppData::config_path(),
            );
            app.show_toast(
                "Advanced settings reset to defaults",
                crate::ui::toast::ToastKind::Success,
            );
            ctx.request_repaint();
        }
    });

    if settings_modified {
        app.data.settings.advanced.validate_and_clamp();
        app.is_dirty = true;
        let _ = crate::storage::AppData::save_settings_to_path(
            &app.data.settings,
            &crate::storage::AppData::config_path(),
        );
        ctx.request_repaint();
    }
}
