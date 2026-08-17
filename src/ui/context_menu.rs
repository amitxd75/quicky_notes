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
    /// Opens file dialog to attach an image.
    AttachImage,
    /// Opens file picker to open a file.
    OpenFile,
    /// Opens folder picker to open a folder workspace.
    OpenFolder,
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
    let can_paste = !clipboard_content.is_empty() || has_clipboard_image();

    ui.set_min_width(220.0);
    ui.spacing_mut().item_spacing.y = 3.0;

    ui.vertical(|ui| {
        // ─── 1. AI Copilot ───
        let ai_label = if has_selection {
            "✨ AI Copilot (Selection)..."
        } else {
            "✨ AI Copilot & Fixer..."
        };
        if menu_item(ui, ai_label, "Ctrl+Enter", palette, true).clicked() {
            *action_out = Some(ContextMenuAction::LaunchAi);
            ui.close();
        }

        menu_separator(ui, palette);

        // ─── 2. Clipboard Operations ───
        if menu_item(ui, "✂ Cut", "Ctrl+X", palette, has_selection).clicked() {
            *action_out = Some(ContextMenuAction::Cut);
            ui.close();
        }

        let copy_label = if has_selection {
            "📋 Copy"
        } else {
            "📋 Copy All"
        };
        if menu_item(ui, copy_label, "Ctrl+C", palette, true).clicked() {
            *action_out = Some(ContextMenuAction::Copy);
            ui.close();
        }

        if menu_item(ui, "📥 Paste", "Ctrl+V", palette, can_paste).clicked() {
            *action_out = Some(ContextMenuAction::Paste);
            ui.close();
        }

        if menu_item(ui, "🖼 Add Image...", "Ctrl+Shift+I", palette, true).clicked() {
            *action_out = Some(ContextMenuAction::AttachImage);
            ui.close();
        }

        if menu_item(ui, "🗑 Delete", "Del", palette, has_selection).clicked() {
            *action_out = Some(ContextMenuAction::Delete);
            ui.close();
        }

        menu_separator(ui, palette);

        // ─── 3. File & Folder Operations ───
        if menu_item(ui, "📥 Open File...", "Ctrl+O", palette, true).clicked() {
            *action_out = Some(ContextMenuAction::OpenFile);
            ui.close();
        }

        if menu_item(ui, "📁 Open Folder...", "Ctrl+Shift+O", palette, true).clicked() {
            *action_out = Some(ContextMenuAction::OpenFolder);
            ui.close();
        }

        menu_separator(ui, palette);

        // ─── 4. Selection & Search Operations ───
        if menu_item(
            ui,
            "🔲 Select All",
            "Ctrl+A",
            palette,
            !note.content.is_empty(),
        )
        .clicked()
        {
            *action_out = Some(ContextMenuAction::SelectAll);
            ui.close();
        }

        if menu_item(ui, "🔍 Search Notes", "Ctrl+K", palette, true).clicked() {
            *action_out = Some(ContextMenuAction::SearchNotes);
            ui.close();
        }

        if menu_item(ui, "💾 Save to Disk", "Ctrl+S", palette, true).clicked() {
            *action_out = Some(ContextMenuAction::SaveNotes);
            ui.close();
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
    let desired_width = ui.available_width().max(210.0);
    let height = 28.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(desired_width, height), Sense::click());

    if enabled {
        let hover = response.hovered();
        let bg_color = if hover {
            Palette::with_alpha(palette.accent, 60)
        } else {
            Color32::TRANSPARENT
        };

        if hover {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(6), bg_color);
        }

        let text_color = if hover {
            Color32::WHITE
        } else {
            Color32::from_gray(235)
        };
        let shortcut_color = if hover {
            Color32::from_gray(215)
        } else {
            Color32::from_gray(145)
        };

        // Left: Label
        let left_pos = egui::pos2(rect.min.x + 10.0, rect.center().y);
        ui.painter().text(
            left_pos,
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.5),
            text_color,
        );

        // Right: Shortcut key hint
        if !shortcut.is_empty() {
            let right_pos = egui::pos2(rect.max.x - 10.0, rect.center().y);
            ui.painter().text(
                right_pos,
                egui::Align2::RIGHT_CENTER,
                shortcut,
                FontId::proportional(11.0),
                shortcut_color,
            );
        }

        response
    } else {
        // Disabled item
        let left_pos = egui::pos2(rect.min.x + 10.0, rect.center().y);
        ui.painter().text(
            left_pos,
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.5),
            Color32::from_gray(100),
        );

        if !shortcut.is_empty() {
            let right_pos = egui::pos2(rect.max.x - 10.0, rect.center().y);
            ui.painter().text(
                right_pos,
                egui::Align2::RIGHT_CENTER,
                shortcut,
                FontId::proportional(11.0),
                Color32::from_gray(75),
            );
        }

        ui.allocate_rect(rect, Sense::hover())
    }
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

/// Sets system clipboard text via arboard and wl-copy fallback on Wayland.
pub fn set_clipboard_text(text: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
    if let Ok(mut child) = std::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write as _;
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Reads system clipboard text via arboard and wl-paste fallback on Wayland.
pub fn get_clipboard_text() -> String {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let t = cb.get_text().unwrap_or_default();
        if !t.is_empty() {
            return t;
        }
    }

    if let Ok(output) = std::process::Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        && output.status.success()
        && !output.stdout.is_empty()
    {
        return String::from_utf8_lossy(&output.stdout).to_string();
    }

    String::new()
}

/// Extracts a clean filesystem path from a clipboard text line or file URI.
pub fn parse_clipboard_file_path(line: &str) -> Option<std::path::PathBuf> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path_str = if trimmed.starts_with("file://") {
        crate::ui::drag_drop::url_decode(trimmed.trim_start_matches("file://"))
    } else if trimmed.starts_with("file:") {
        crate::ui::drag_drop::url_decode(trimmed.trim_start_matches("file:"))
    } else {
        trimmed.to_string()
    };
    let path = std::path::PathBuf::from(path_str);
    if path.exists() && path.is_file() {
        Some(path)
    } else {
        None
    }
}

/// Checks whether the system clipboard contains an image (raw pixel buffer, image file path, or wl-paste).
pub fn has_clipboard_image() -> bool {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        if cb.get_image().is_ok() {
            return true;
        }

        if let Ok(text) = cb.get_text() {
            for line in text.lines() {
                if let Some(path) = parse_clipboard_file_path(line)
                    && crate::ui::drag_drop::is_image_path(&path)
                {
                    return true;
                }
            }
        }
    }

    // Wayland check via wl-paste list-types
    if let Ok(output) = std::process::Command::new("wl-paste")
        .arg("--list-types")
        .output()
        && output.status.success()
    {
        let list = String::from_utf8_lossy(&output.stdout);
        if list.contains("image/png") || list.contains("image/jpeg") || list.contains("image/webp")
        {
            return true;
        }
    }

    false
}

/// Reads system clipboard image via arboard (supports both raw pixel screenshots and copied image files).
pub fn get_clipboard_image() -> Option<(String, &'static str, Vec<u8>)> {
    // 1. Try reading raw image pixels from clipboard via arboard
    if let Ok(mut cb) = arboard::Clipboard::new() {
        if let Ok(img_data) = cb.get_image() {
            let width = img_data.width as u32;
            let height = img_data.height as u32;
            if width > 0
                && height > 0
                && let Some(rgba_img) =
                    image::RgbaImage::from_raw(width, height, img_data.bytes.into_owned())
            {
                let mut png_bytes = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut png_bytes);
                if rgba_img
                    .write_to(&mut cursor, image::ImageFormat::Png)
                    .is_ok()
                {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let name = format!("screenshot_{}.png", timestamp);
                    return Some((name, "image/png", png_bytes));
                }
            }
        }

        // 2. Try reading file path / URI list from clipboard text (e.g. copied image file in Nautilus/Dolphin)
        if let Ok(text) = cb.get_text() {
            for line in text.lines() {
                if let Some(path) = parse_clipboard_file_path(line)
                    && crate::ui::drag_drop::is_image_path(&path)
                    && let Ok(bytes) = std::fs::read(&path)
                {
                    let name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "pasted_image.png".to_string());
                    let mime = crate::models::NoteAttachment::detect_mime(&name);
                    return Some((name, mime, bytes));
                }
            }
        }
    }

    // 3. Wayland compositor fallback via wl-paste
    for mime_candidate in &["image/png", "image/jpeg", "image/webp"] {
        if let Ok(output) = std::process::Command::new("wl-paste")
            .arg("--type")
            .arg(mime_candidate)
            .output()
            && output.status.success()
            && !output.stdout.is_empty()
        {
            let (ext, mime) = match *mime_candidate {
                "image/jpeg" => ("jpg", "image/jpeg"),
                "image/webp" => ("webp", "image/webp"),
                _ => ("png", "image/png"),
            };
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let name = format!("screenshot_{}.{}", timestamp, ext);
            return Some((name, mime, output.stdout));
        }
    }

    None
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
