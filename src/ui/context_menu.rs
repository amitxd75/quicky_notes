//! IDE-style right-click context menu for editor and note operations.
//!
//! Provides quick actions for AI Copilot, clipboard operations (Cut/Copy/Paste),
//! text manipulation, and note management with shortcut hints and glassmorphic styling.

use crate::note::Note;
use crate::theme::Palette;
use eframe::egui::{self, Color32, CornerRadius, FontId, Response, Sense, Stroke, Ui};

/// Actions dispatched from the editor's right-click context menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuAction {
    /// Launches the AI Copilot modal with current selection or cursor context.
    LaunchAi,
    /// Cuts selected text to clipboard.
    Cut,
    /// Copies selected text (or all text) to clipboard.
    Copy,
    /// Pastes clipboard text at cursor/selection.
    Paste,
    /// Deletes selected text.
    Delete,
    /// Selects all text in the note.
    SelectAll,
    /// Opens the note search palette.
    SearchNotes,
    /// Saves current notes to disk.
    SaveNotes,
}

/// Renders the floating right-click context menu and captures user action.
pub fn render_editor_context_menu(
    ui: &mut Ui,
    note: &Note,
    cursor_range: Option<(usize, usize)>,
    palette: &Palette,
    action_out: &mut Option<ContextMenuAction>,
) {
    let has_selection = cursor_range.is_some_and(|(start, end)| start < end);
    let clipboard_content = get_clipboard_text();
    let can_paste = !clipboard_content.is_empty();

    ui.set_min_width(200.0);
    ui.spacing_mut().item_spacing.y = 2.0;

    ui.vertical(|ui| {
        // ─── 1. AI Copilot ───
        let ai_label = if has_selection {
            "AI Copilot (Selection)..."
        } else {
            "AI Copilot & Fixer..."
        };
        if menu_item(ui, ai_label, "Ctrl+Enter", palette, true).clicked() {
            ui.close();
            *action_out = Some(ContextMenuAction::LaunchAi);
        }

        menu_separator(ui, palette);

        // ─── 2. Clipboard Operations ───
        if menu_item(ui, "Cut", "Ctrl+X", palette, has_selection).clicked() {
            ui.close();
            *action_out = Some(ContextMenuAction::Cut);
        }

        let copy_label = if has_selection { "Copy" } else { "Copy All" };
        if menu_item(ui, copy_label, "Ctrl+C", palette, true).clicked() {
            ui.close();
            *action_out = Some(ContextMenuAction::Copy);
        }

        if menu_item(ui, "Paste", "Ctrl+V", palette, can_paste).clicked() {
            ui.close();
            *action_out = Some(ContextMenuAction::Paste);
        }

        if menu_item(ui, "Delete", "Del", palette, has_selection).clicked() {
            ui.close();
            *action_out = Some(ContextMenuAction::Delete);
        }

        menu_separator(ui, palette);

        // ─── 3. Selection & Search Operations ───
        if menu_item(
            ui,
            "Select All",
            "Ctrl+A",
            palette,
            !note.content.is_empty(),
        )
        .clicked()
        {
            ui.close();
            *action_out = Some(ContextMenuAction::SelectAll);
        }

        if menu_item(ui, "Search Notes", "Ctrl+K", palette, true).clicked() {
            ui.close();
            *action_out = Some(ContextMenuAction::SearchNotes);
        }

        if menu_item(ui, "Save to Disk", "Ctrl+S", palette, true).clicked() {
            ui.close();
            *action_out = Some(ContextMenuAction::SaveNotes);
        }
    });
}

/// Helper function to render a styled, interactive context menu item with shortcut badge.
fn menu_item(
    ui: &mut Ui,
    label: &str,
    shortcut: &str,
    palette: &Palette,
    enabled: bool,
) -> Response {
    let desired_size = egui::vec2(ui.available_width(), 26.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if enabled && response.hovered() {
        let hover_bg = Palette::with_alpha(palette.accent, 45);
        ui.painter()
            .rect_filled(rect, CornerRadius::same(6), hover_bg);
    }

    let text_color = if enabled {
        if response.hovered() {
            Color32::WHITE
        } else {
            Color32::from_gray(230)
        }
    } else {
        Color32::from_gray(100)
    };

    // Left: Label
    let left_pos = egui::pos2(rect.min.x + 10.0, rect.center().y);
    ui.painter().text(
        left_pos,
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        text_color,
    );

    // Right: Shortcut key hint
    if !shortcut.is_empty() {
        let right_pos = egui::pos2(rect.max.x - 8.0, rect.center().y);
        ui.painter().text(
            right_pos,
            egui::Align2::RIGHT_CENTER,
            shortcut,
            FontId::proportional(10.5),
            if enabled {
                Color32::from_gray(140)
            } else {
                Color32::from_gray(80)
            },
        );
    }

    if enabled { response } else { response.clone() }
}

/// Helper to render a subtle menu divider line.
fn menu_separator(ui: &mut Ui, palette: &Palette) {
    ui.add_space(3.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().line_segment(
        [rect.min, egui::pos2(rect.max.x, rect.min.y)],
        Stroke::new(1.0, Palette::with_alpha(palette.border, 120)),
    );
    ui.add_space(3.0);
}

/// Sets system clipboard text via arboard.
pub fn set_clipboard_text(text: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}

/// Reads system clipboard text via arboard.
pub fn get_clipboard_text() -> String {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        cb.get_text().unwrap_or_default()
    } else {
        String::new()
    }
}

/// Deletes the currently selected character range from the note content.
pub fn delete_selection(note: &mut Note, start: usize, end: usize) {
    if start < end {
        note.delete_char_range(start, end);
    }
}

/// Inserts or replaces text at the active cursor or selected range.
pub fn insert_or_replace_text(note: &mut Note, text: &str, cursor_range: Option<(usize, usize)>) {
    let total_chars = note.char_len();
    let (s, e) = if let Some((start, end)) = cursor_range {
        let s = start.min(total_chars);
        let e = end.min(total_chars).max(s);
        (s, e)
    } else {
        (total_chars, total_chars)
    };

    note.replace_char_range(s, e, text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_selection() {
        let mut note = Note::new("test-1".to_string(), "test.txt".to_string());
        note.content = "Hello Beautiful World".to_string();
        delete_selection(&mut note, 6, 16);
        assert_eq!(note.content, "Hello World");

        // Unicode & Emoji support without panicking
        let mut unicode_note = Note::new("test-2".to_string(), "test.txt".to_string());
        unicode_note.content = "✨ Rust 🦀 is fast".to_string();
        delete_selection(&mut unicode_note, 0, 7); // Delete "✨ Rust "
        assert_eq!(unicode_note.content, "🦀 is fast");
    }

    #[test]
    fn test_insert_or_replace_text() {
        let mut note = Note::new("test-1".to_string(), "test.txt".to_string());
        note.content = "Hello World".to_string();
        insert_or_replace_text(&mut note, "Rust ", Some((6, 6)));
        assert_eq!(note.content, "Hello Rust World");

        insert_or_replace_text(&mut note, "Universe", Some((11, 16)));
        assert_eq!(note.content, "Hello Rust Universe");

        // Unicode & Emoji insertion
        let mut unicode_note = Note::new("test-2".to_string(), "test.txt".to_string());
        unicode_note.content = "✨ 🚀".to_string();
        insert_or_replace_text(&mut unicode_note, "Code ", Some((2, 2)));
        assert_eq!(unicode_note.content, "✨ Code 🚀");
    }
}
