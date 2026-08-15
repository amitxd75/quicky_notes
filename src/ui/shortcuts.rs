//! Global keyboard shortcuts and navigation handler.

use crate::app::QuickyNotesApp;
use crate::ui;
use eframe::egui::{self, ViewportCommand};

/// Evaluates global keyboard shortcuts.
pub fn handle_keyboard_shortcuts(app: &mut QuickyNotesApp, ctx: &egui::Context) {
    let ctrl_shift_tab = ctx.input_mut(|i| {
        i.consume_key(
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            egui::Key::Tab,
        )
    });
    let ctrl_tab = ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Tab));

    if (ctrl_tab || ctrl_shift_tab) && !app.data.notes.is_empty() {
        let current_idx = app
            .data
            .notes
            .iter()
            .position(|n| Some(n.id.as_str()) == app.data.active_note_id.as_deref())
            .unwrap_or(0);

        let next_idx = if ctrl_shift_tab {
            if current_idx == 0 {
                app.data.notes.len() - 1
            } else {
                current_idx - 1
            }
        } else {
            (current_idx + 1) % app.data.notes.len()
        };

        app.data.active_note_id = Some(app.data.notes[next_idx].id.clone());
        app.focus_editor = true;
    }

    ctx.input(|i| {
        // Ctrl + N: New note tab
        if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
            app.create_new_note();
        }

        // Ctrl + W: Close active note tab
        if i.modifiers.ctrl
            && i.key_pressed(egui::Key::W)
            && let Some(id) = app.data.active_note_id.clone()
        {
            app.prompt_close_note(&id);
        }

        // Ctrl + S: Save to disk
        if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
            app.save_notes_to_disk();
        }

        // Ctrl + P: Toggle/cycle Markdown preview mode (Edit -> Split -> Preview for .md files)
        if i.modifiers.ctrl && i.key_pressed(egui::Key::P) {
            if app.active_note().is_some_and(|n| n.is_markdown()) {
                app.preview_mode = app.preview_mode.next();
                app.set_status(format!("Markdown: {:?}", app.preview_mode));
            } else {
                app.set_status("Markdown preview only available for .md files");
            }
        }

        // Ctrl + K: Search notes modal
        if i.modifiers.ctrl && i.key_pressed(egui::Key::K) {
            app.show_search = !app.show_search;
            app.show_options = false;
            if app.show_search {
                app.focus_search = true;
            } else {
                app.focus_editor = true;
            }
        }

        // Ctrl + ,: Options modal
        if i.modifiers.ctrl && i.key_pressed(egui::Key::Comma) {
            app.show_options = !app.show_options;
            app.show_search = false;
            if !app.show_options {
                app.focus_editor = true;
            }
        }

        // Ctrl + Shift + E: Export note
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::E) {
            ui::drag_drop::export_active_note(app);
        }

        // Ctrl + Shift + T: Toggle Always on Top
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::T) {
            app.data.settings.always_on_top = !app.data.settings.always_on_top;
            let level = if app.data.settings.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
            app.is_dirty = true;
        }

        // Ctrl + + / Ctrl + =: Increase font size
        if i.modifiers.ctrl && (i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus))
        {
            app.data.settings.font_size = (app.data.settings.font_size + 1.0).min(36.0);
            app.data.settings.validate_and_clamp();
            app.is_dirty = true;
            let _ = app.data.save();
            app.set_status(format!("Font size: {:.0}pt", app.data.settings.font_size));
        }

        // Ctrl + -: Decrease font size
        if i.modifiers.ctrl && i.key_pressed(egui::Key::Minus) {
            app.data.settings.font_size = (app.data.settings.font_size - 1.0).max(8.0);
            app.data.settings.validate_and_clamp();
            app.is_dirty = true;
            let _ = app.data.save();
            app.set_status(format!("Font size: {:.0}pt", app.data.settings.font_size));
        }

        // Ctrl + 1..=9 / Ctrl + 0: Switch to tab by index
        if i.modifiers.ctrl {
            let num_keys = [
                (egui::Key::Num1, 0),
                (egui::Key::Num2, 1),
                (egui::Key::Num3, 2),
                (egui::Key::Num4, 3),
                (egui::Key::Num5, 4),
                (egui::Key::Num6, 5),
                (egui::Key::Num7, 6),
                (egui::Key::Num8, 7),
                (egui::Key::Num9, 8),
            ];
            for (key, idx) in num_keys {
                if i.key_pressed(key)
                    && let Some(note) = app.data.notes.get(idx)
                {
                    app.data.active_note_id = Some(note.id.clone());
                    app.focus_editor = true;
                }
            }
            if i.key_pressed(egui::Key::Num0)
                && let Some(note) = app.data.notes.last()
            {
                app.data.active_note_id = Some(note.id.clone());
                app.focus_editor = true;
            }
        }

        // ArrowUp, ArrowDown, Enter inside search drawer
        if app.show_search && !app.data.notes.is_empty() {
            let query = app.search_query.trim().to_lowercase();
            let matching_indices: Vec<_> = app
                .data
                .notes
                .iter()
                .enumerate()
                .filter(|(_, n)| {
                    ui::search_drawer::note_matches_query(&n.title, &n.content, &query)
                })
                .map(|(idx, _)| idx)
                .collect();

            let filtered_count = matching_indices.len();

            if filtered_count > 0 {
                if app.search_selected_idx >= filtered_count {
                    app.search_selected_idx = 0;
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    app.search_selected_idx = (app.search_selected_idx + 1) % filtered_count;
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    if app.search_selected_idx == 0 {
                        app.search_selected_idx = filtered_count - 1;
                    } else {
                        app.search_selected_idx -= 1;
                    }
                }
                if i.key_pressed(egui::Key::Enter)
                    && let Some(&note_idx) = matching_indices.get(app.search_selected_idx)
                    && let Some(note) = app.data.notes.get(note_idx)
                {
                    app.data.active_note_id = Some(note.id.clone());
                    app.show_search = false;
                    app.focus_editor = true;
                }
            }
        }

        // Escape: Close modals if open, else close app window
        if i.key_pressed(egui::Key::Escape) {
            if app.confirm_close_id.is_some() {
                app.confirm_close_id = None;
            } else if app.show_options || app.show_search {
                app.show_options = false;
                app.show_search = false;
                app.focus_editor = true;
            } else {
                app.is_closing = true;
                app.save_if_dirty();
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    });
}
