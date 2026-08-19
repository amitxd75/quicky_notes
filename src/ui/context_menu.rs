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
    /// Custom action registered by an active plugin.
    PluginAction(String),
}

/// Minimum width of the context menu in pixels.
pub const CONTEXT_MENU_MIN_WIDTH: f32 = 280.0;
/// Maximum width of the context menu in pixels.
pub const CONTEXT_MENU_MAX_WIDTH: f32 = 320.0;
/// Maximum scrollable height of the context menu in pixels.
pub const CONTEXT_MENU_MAX_HEIGHT: f32 = 600.0;
/// Height of a single context menu row item in pixels.
pub const CONTEXT_MENU_ITEM_HEIGHT: f32 = 30.0;

/// Definition of an interactive context menu item.
struct ContextMenuItemDef {
    label: String,
    shortcut: &'static str,
    action: ContextMenuAction,
    enabled: bool,
    separator_after: bool,
}

/// Renders the floating right-click context menu and captures user action.
pub fn render_editor_context_menu(
    ui: &mut Ui,
    note: &Note,
    cursor_range: Option<(usize, usize)>,
    palette: &Palette,
    plugin_items: &[crate::plugins::PluginMenuItem],
    action_out: &mut Option<ContextMenuAction>,
) {
    const {
        assert!(
            CONTEXT_MENU_MIN_WIDTH <= CONTEXT_MENU_MAX_WIDTH,
            "Context menu min width must not exceed max width"
        );
    }

    let has_selection = cursor_range.is_some_and(|(start, end)| start < end);

    ui.set_min_width(CONTEXT_MENU_MIN_WIDTH);
    ui.set_max_width(CONTEXT_MENU_MAX_WIDTH);
    ui.spacing_mut().item_spacing.y = 3.0;

    // Build structured context menu entries
    let mut items = Vec::with_capacity(16);

    // 1. AI Copilot
    let ai_label = if has_selection {
        "✨ AI Copilot (Selection)...".to_string()
    } else {
        "✨ AI Copilot & Fixer...".to_string()
    };
    items.push(ContextMenuItemDef {
        label: ai_label,
        shortcut: "Ctrl+Enter",
        action: ContextMenuAction::LaunchAi,
        enabled: true,
        separator_after: true,
    });

    // 2. Clipboard Operations
    items.push(ContextMenuItemDef {
        label: "✂ Cut".to_string(),
        shortcut: "Ctrl+X",
        action: ContextMenuAction::Cut,
        enabled: has_selection,
        separator_after: false,
    });
    let copy_label = if has_selection {
        "📋 Copy"
    } else {
        "📋 Copy All"
    };
    items.push(ContextMenuItemDef {
        label: copy_label.to_string(),
        shortcut: "Ctrl+C",
        action: ContextMenuAction::Copy,
        enabled: true,
        separator_after: false,
    });
    items.push(ContextMenuItemDef {
        label: "📥 Paste".to_string(),
        shortcut: "Ctrl+V",
        action: ContextMenuAction::Paste,
        enabled: true,
        separator_after: false,
    });
    items.push(ContextMenuItemDef {
        label: "🖼 Add Image...".to_string(),
        shortcut: "Ctrl+Shift+I",
        action: ContextMenuAction::AttachImage,
        enabled: true,
        separator_after: false,
    });
    items.push(ContextMenuItemDef {
        label: "🗑 Delete".to_string(),
        shortcut: "Del",
        action: ContextMenuAction::Delete,
        enabled: has_selection,
        separator_after: true,
    });

    // 3. File & Folder Operations
    items.push(ContextMenuItemDef {
        label: "📥 Open File...".to_string(),
        shortcut: "Ctrl+O",
        action: ContextMenuAction::OpenFile,
        enabled: true,
        separator_after: false,
    });
    items.push(ContextMenuItemDef {
        label: "📁 Open Folder...".to_string(),
        shortcut: "Ctrl+Shift+O",
        action: ContextMenuAction::OpenFolder,
        enabled: true,
        separator_after: true,
    });

    // 4. Selection & Search Operations
    items.push(ContextMenuItemDef {
        label: "🔲 Select All".to_string(),
        shortcut: "Ctrl+A",
        action: ContextMenuAction::SelectAll,
        enabled: !note.content.is_empty(),
        separator_after: false,
    });
    items.push(ContextMenuItemDef {
        label: "🔍 Search Notes".to_string(),
        shortcut: "Ctrl+K",
        action: ContextMenuAction::SearchNotes,
        enabled: true,
        separator_after: false,
    });
    let has_plugins = !plugin_items.is_empty();
    items.push(ContextMenuItemDef {
        label: "💾 Save to Disk".to_string(),
        shortcut: "Ctrl+S",
        action: ContextMenuAction::SaveNotes,
        enabled: true,
        separator_after: has_plugins,
    });

    // 5. Plugin Actions
    for item in plugin_items {
        let label = if let Some(ref icon) = item.icon {
            format!("{} {}", icon, item.label)
        } else {
            format!("🔌 {}", item.label)
        };
        items.push(ContextMenuItemDef {
            label,
            shortcut: "",
            action: ContextMenuAction::PluginAction(item.action_id.clone()),
            enabled: true,
            separator_after: false,
        });
    }

    // Keyboard Arrow Navigation
    let menu_sel_id = ui.id().with("ctx_menu_keyboard_sel");
    let mut current_sel: Option<usize> = ui.ctx().data(|d| d.get_temp(menu_sel_id));
    let enabled_indices: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.enabled)
        .map(|(i, _)| i)
        .collect();

    if !enabled_indices.is_empty() {
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            let next_pos =
                match current_sel.and_then(|cs| enabled_indices.iter().position(|&ei| ei == cs)) {
                    Some(pos) => (pos + 1) % enabled_indices.len(),
                    None => 0,
                };
            let next_idx = enabled_indices[next_pos];
            current_sel = Some(next_idx);
            ui.ctx()
                .data_mut(|d| d.insert_temp(menu_sel_id, current_sel));
            ui.ctx().request_repaint();
        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            let prev_pos =
                match current_sel.and_then(|cs| enabled_indices.iter().position(|&ei| ei == cs)) {
                    Some(pos) => {
                        if pos == 0 {
                            enabled_indices.len() - 1
                        } else {
                            pos - 1
                        }
                    }
                    None => enabled_indices.len() - 1,
                };
            let prev_idx = enabled_indices[prev_pos];
            current_sel = Some(prev_idx);
            ui.ctx()
                .data_mut(|d| d.insert_temp(menu_sel_id, current_sel));
            ui.ctx().request_repaint();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Enter))
            && let Some(sel) = current_sel
            && let Some(item) = items.get(sel)
            && item.enabled
        {
            *action_out = Some(item.action.clone());
            ui.close();
        }
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        ui.close();
    }

    egui::ScrollArea::vertical()
        .id_salt("editor_context_menu_scroll")
        .max_height(CONTEXT_MENU_MAX_HEIGHT)
        .auto_shrink([true, true])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                for (idx, item) in items.into_iter().enumerate() {
                    let is_sel = current_sel == Some(idx);
                    let resp = menu_item(
                        ui,
                        &item.label,
                        item.shortcut,
                        palette,
                        item.enabled,
                        is_sel,
                    );
                    if resp.clicked() {
                        *action_out = Some(item.action);
                        ui.close();
                    }
                    if item.separator_after {
                        menu_separator(ui, palette);
                    }
                }
            });
        });
}

/// Helper function to render a styled, interactive context menu item with shortcut badge.
fn menu_item(
    ui: &mut Ui,
    label: &str,
    shortcut: &str,
    palette: &Palette,
    enabled: bool,
    is_keyboard_selected: bool,
) -> Response {
    let desired_width = ui.available_width().max(CONTEXT_MENU_MIN_WIDTH);
    let height = CONTEXT_MENU_ITEM_HEIGHT;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(desired_width, height), Sense::click());

    if enabled {
        let hover = response.hovered();
        let is_highlighted = hover || is_keyboard_selected;
        let bg_color = if is_highlighted {
            Palette::with_alpha(palette.accent, if is_keyboard_selected { 90 } else { 75 })
        } else {
            Color32::TRANSPARENT
        };

        if is_highlighted {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(6), bg_color);
            if is_keyboard_selected {
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(6),
                    Stroke::new(1.0, Palette::with_alpha(palette.accent, 180)),
                    egui::StrokeKind::Inside,
                );
            }
        }

        let text_color = if is_highlighted {
            Color32::WHITE
        } else {
            Color32::from_gray(245)
        };
        let shortcut_color = if is_highlighted {
            Color32::from_gray(225)
        } else {
            Color32::from_gray(140)
        };

        // Left: Label with proper padding
        let left_pos = egui::pos2(rect.min.x + 12.0, rect.center().y);
        ui.painter().text(
            left_pos,
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.0),
            text_color,
        );

        // Right: Shortcut key hint with proper right margin
        if !shortcut.is_empty() {
            let right_pos = egui::pos2(rect.max.x - 12.0, rect.center().y);
            ui.painter().text(
                right_pos,
                egui::Align2::RIGHT_CENTER,
                shortcut,
                FontId::proportional(11.5),
                shortcut_color,
            );
        }

        response
    } else {
        // Disabled item
        let left_pos = egui::pos2(rect.min.x + 12.0, rect.center().y);
        ui.painter().text(
            left_pos,
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.0),
            Color32::from_gray(110),
        );

        if !shortcut.is_empty() {
            let right_pos = egui::pos2(rect.max.x - 12.0, rect.center().y);
            ui.painter().text(
                right_pos,
                egui::Align2::RIGHT_CENTER,
                shortcut,
                FontId::proportional(11.5),
                Color32::from_gray(80),
            );
        }

        response
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
