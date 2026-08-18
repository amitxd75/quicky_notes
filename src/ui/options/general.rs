//! General tab preferences: System & startup integration, note creation defaults, and workspace sync.

use crate::app::QuickyNotesApp;
use crate::components::{button, card, toggle};
use crate::theme;
use eframe::egui::{self, Color32, FontId, RichText, Ui, ViewportCommand};

/// Renders all cards in the General settings tab.
pub fn render_general_tab(
    app: &mut QuickyNotesApp,
    ctx: &egui::Context,
    ui: &mut Ui,
    palette: &theme::Palette,
) {
    render_general_system_card(app, ctx, ui, palette);
    render_general_workspace_card(app, ctx, ui, palette);
    render_general_sync_card(app, ctx, ui, palette);
    render_general_cheatsheet_card(ui, palette);
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
