//! Native file dialogs, file export, drag-and-drop parsing, image attachments, and filesystem safety.

use crate::app::QuickyNotesApp;
use crate::models::{Note, NoteAttachment, QN_MAGIC};
use crate::ui::toast::ToastKind;
use eframe::egui;
use std::path::Path;
use std::sync::mpsc;

/// Supported image file extensions for direct note insertion and attachments.
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] =
    &["png", "jpg", "jpeg", "webp", "gif", "bmp", "svg"];

/// Supported binary bundle file extensions.
pub const SUPPORTED_QN_EXTENSIONS: &[&str] = &["qn", "qnote"];

/// Returns whether the path points to a supported image file.
pub fn is_image_path(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.as_str())
}

/// Returns whether the path is a .qn or .qnote binary file.
pub fn is_qn_path(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    SUPPORTED_QN_EXTENSIONS.contains(&ext.as_str())
}

/// Decodes percent-encoded URL strings (e.g. `%20` -> ` `).
///
/// Correctly handles multi-byte UTF-8 sequences by collecting decoded bytes into
/// a `Vec<u8>` buffer and converting the entire result via `String::from_utf8_lossy`.
pub fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut buf: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(sub) = std::str::from_utf8(&bytes[i + 1..i + 3])
            && let Ok(val) = u8::from_str_radix(sub, 16)
        {
            buf.push(val);
            i += 3;
            continue;
        }
        buf.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&buf).into_owned()
}

/// Validates that a filesystem path is safe to open, read, or link.
///
/// Resolves symlinks via `canonicalize()` and rejects:
/// - System directories: `/etc`, `/proc`, `/sys`, `/dev`, `/boot`, `/var/log`
/// - Sensitive dotfiles/dotdirs under `$HOME`: `.ssh`, `.gnupg`, `.config/quicky_notes`, `.bashrc`, etc.
/// - Allows user home directories, removable media mounts, and `/tmp`.
pub fn is_safe_file_path(path: &Path) -> bool {
    let canonical = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let path_str = canonical.to_string_lossy();

    // Block sensitive system directories
    const BLOCKED_PREFIXES: &[&str] = &[
        "/etc/",
        "/proc/",
        "/sys/",
        "/dev/",
        "/boot/",
        "/var/log/",
        "/sbin/",
        "/bin/",
    ];
    for prefix in BLOCKED_PREFIXES {
        if path_str.starts_with(prefix) {
            return false;
        }
    }

    // Allow user temp, runtime, and removable media mounts
    if path_str.starts_with("/tmp/")
        || path_str.starts_with("/var/tmp/")
        || path_str.starts_with("/media/")
        || path_str.starts_with("/mnt/")
        || path_str.starts_with("/run/media/")
        || path_str.starts_with("/run/user/")
    {
        return true;
    }

    // Check home directory sensitive entries
    if let Some(user_dirs) = directories::UserDirs::new() {
        let home = user_dirs.home_dir().to_string_lossy().to_string();
        if let Some(rel) = path_str.strip_prefix(&home) {
            let rel = rel.trim_start_matches('/');
            const BLOCKED_DOT_ENTRIES: &[&str] = &[
                ".ssh",
                ".gnupg",
                ".gpg",
                ".config/quicky_notes",
                ".bashrc",
                ".bash_history",
                ".zshrc",
                ".zsh_history",
                ".profile",
                ".netrc",
                ".aws",
                ".kube",
                ".docker",
            ];
            for blocked in BLOCKED_DOT_ENTRIES {
                if rel.starts_with(blocked) {
                    return false;
                }
            }
        }
    }

    true
}

/// Safely opens a directory in the system file manager via `xdg-open`.
///
/// Canonicalizes the path and validates it exists and is a directory before
/// passing to `xdg-open`. Returns `false` if the path is invalid.
pub fn safe_open_folder(path: &Path) -> bool {
    match std::fs::canonicalize(path) {
        Ok(canonical) if canonical.is_dir() => {
            let _ = std::process::Command::new("xdg-open")
                .arg(&canonical)
                .spawn();
            true
        }
        _ => false,
    }
}

/// Spawns a native file dialog (Zenity/Kdialog) on a background thread.
///
/// Returns a `Receiver` that will yield the selected file path (or `None` on cancel).
/// The UI thread remains responsive while the dialog is open.
pub fn spawn_file_dialog() -> mpsc::Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let output = std::process::Command::new("zenity")
            .arg("--file-selection")
            .output()
            .or_else(|_| {
                std::process::Command::new("kdialog")
                    .arg("--getopenfilename")
                    .output()
            });

        let result = if let Ok(out) = output
            && out.status.success()
        {
            let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path_str.is_empty() {
                Some(path_str)
            } else {
                None
            }
        } else {
            None
        };
        let _ = tx.send(result);
    });
    rx
}

/// Spawns a file dialog specifically filtered for images.
pub fn spawn_image_dialog() -> mpsc::Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let output = std::process::Command::new("zenity")
            .arg("--file-selection")
            .arg("--file-filter=Images | *.png *.jpg *.jpeg *.webp *.gif *.bmp *.svg")
            .output()
            .or_else(|_| {
                std::process::Command::new("kdialog")
                    .arg("--getopenfilename")
                    .arg("*.png *.jpg *.jpeg *.webp *.gif *.bmp *.svg")
                    .output()
            });

        let result = if let Ok(out) = output
            && out.status.success()
        {
            let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path_str.is_empty() {
                Some(path_str)
            } else {
                None
            }
        } else {
            None
        };
        let _ = tx.send(result);
    });
    rx
}

/// Polls a pending file dialog result and processes the selected file.
///
/// Called from the main update loop to check if a background file dialog has completed.
pub fn poll_file_dialog(app: &mut QuickyNotesApp) {
    let result = if let Some(rx) = &app.file_dialog_rx {
        match rx.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Some(None),
        }
    } else {
        return;
    };

    app.file_dialog_rx = None;

    if let Some(Some(path_str)) = result {
        let path = std::path::Path::new(&path_str);
        if !is_safe_file_path(path) {
            app.set_status("Blocked: file path is outside allowed directories");
            return;
        }
        if !path.exists() || !path.is_file() {
            return;
        }

        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "imported.qn".to_string());

        // 1. Check if it's an image attachment
        if is_image_path(path) {
            if let Ok(bytes) = std::fs::read(path) {
                let mime = NoteAttachment::detect_mime(&name);
                attach_image_to_active_note(app, &name, mime, bytes);
            }
            return;
        }

        // 2. Check if it's a .qn binary container
        if is_qn_path(path)
            && let Ok(bytes) = std::fs::read(path)
            && bytes.starts_with(QN_MAGIC)
            && let Ok(mut note) = Note::decode_qn_binary(&bytes)
        {
            note.file_path = Some(path_str);
            let id = note.id.clone();
            app.data.notes.push(note);
            app.data.active_note_id = Some(id);
            app.is_dirty = true;
            app.focus_editor = true;
            app.set_status(format!("Opened {}", name));
            return;
        }

        // 3. Fallback: Standard text / Markdown file
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(existing) = app
                .data
                .notes
                .iter()
                .find(|n| n.file_path.as_deref() == Some(&path_str))
            {
                app.data.active_note_id = Some(existing.id.clone());
            } else {
                let id = format!("note-{}", chrono::Local::now().timestamp_millis());
                let mut note = Note::new(id.clone(), name.clone());
                note.content = content;
                note.file_path = Some(path_str);
                app.data.notes.push(note);
                app.data.active_note_id = Some(id);
                app.is_dirty = true;
            }
            app.focus_editor = true;
            app.set_status(format!("Linked {}", name));
        }
    }
}

/// Helper to attach image bytes to the active note (or a new tab) and insert a clean markdown tag.
pub fn attach_image_to_active_note(
    app: &mut QuickyNotesApp,
    name: &str,
    mime: &str,
    bytes: Vec<u8>,
) {
    attach_image_to_active_note_at_cursor(app, name, mime, bytes, None);
}

/// Helper to attach image bytes to the active note (or a new tab) and insert tag at the cursor.
pub fn attach_image_to_active_note_at_cursor(
    app: &mut QuickyNotesApp,
    name: &str,
    mime: &str,
    bytes: Vec<u8>,
    cursor_range: Option<(usize, usize)>,
) {
    if let Some(note) = app.active_note_mut() {
        let id = note.add_attachment(name, mime, bytes);
        let tag = format!("![{}](attachment:{})", name, id);
        let s = cursor_range.map_or(note.char_len(), |(st, _)| st);
        crate::ui::context_menu::insert_or_replace_text(note, &tag, cursor_range);
        let new_pos = s + tag.chars().count();
        app.last_cursor_range = Some((new_pos, new_pos));
        app.is_dirty = true;
        app.show_toast(format!("Pasted image: {}", name), ToastKind::Success);
    } else {
        let mut note = Note::new(
            format!("note-{}", chrono::Local::now().timestamp_millis()),
            "untitled.qn".to_string(),
        );
        let id = note.add_attachment(name, mime, bytes);
        note.content = format!("![{}](attachment:{})\n", name, id);
        app.data.notes.push(note);
        app.is_dirty = true;
        app.show_toast(format!("Pasted image: {}", name), ToastKind::Success);
    }
}

/// Opens a native file dialog (Zenity/Kdialog) to import and link files.
pub fn open_file_dialog(app: &mut QuickyNotesApp) {
    if app.file_dialog_rx.is_some() {
        return;
    }
    app.file_dialog_rx = Some(spawn_file_dialog());
    app.set_status("Opening file dialog...");
}

/// Opens a native file dialog to attach an image to the current note.
pub fn open_image_dialog(app: &mut QuickyNotesApp) {
    if app.file_dialog_rx.is_some() {
        return;
    }
    app.file_dialog_rx = Some(spawn_image_dialog());
    app.set_status("Select image to attach...");
}

/// Spawns a native file save dialog (Zenity/Kdialog) with fallback to default ~/Documents path.
pub fn spawn_export_dialog(
    default_filename: String,
    is_qn: bool,
    binary: Vec<u8>,
    text: String,
) -> mpsc::Receiver<Result<String, String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let initial_path = directories::UserDirs::new()
            .and_then(|u| u.document_dir().map(|d| d.join(&default_filename)))
            .unwrap_or_else(|| std::path::PathBuf::from(&default_filename));

        let initial_str = initial_path.to_string_lossy().to_string();

        // 1. Try native file save dialog via zenity or kdialog
        let output = std::process::Command::new("zenity")
            .arg("--file-selection")
            .arg("--save")
            .arg("--confirm-overwrite")
            .arg(format!("--filename={}", initial_str))
            .output()
            .or_else(|_| {
                std::process::Command::new("kdialog")
                    .arg("--getsavefilename")
                    .arg(&initial_str)
                    .output()
            });

        let target_path = match output {
            Ok(out) if out.status.success() => {
                let chosen = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !chosen.is_empty() {
                    std::path::PathBuf::from(chosen)
                } else {
                    initial_path
                }
            }
            _ => initial_path,
        };

        if let Some(parent) = target_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let write_res = if is_qn {
            std::fs::write(&target_path, &binary)
        } else {
            std::fs::write(&target_path, &text)
        };

        match write_res {
            Ok(()) => {
                let _ = tx.send(Ok(target_path.to_string_lossy().to_string()));
            }
            Err(e) => {
                let _ = tx.send(Err(format!("Export failed: {}", e)));
            }
        }
    });
    rx
}

/// Polls for completed export dialog background tasks.
pub fn poll_export_dialog(app: &mut QuickyNotesApp) {
    let result = if let Some(rx) = &app.export_dialog_rx {
        match rx.try_recv() {
            Ok(res) => Some(res),
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("Export dialog cancelled".to_string()))
            }
        }
    } else {
        return;
    };

    app.export_dialog_rx = None;

    match result {
        Some(Ok(path_str)) => {
            app.set_status(format!("Exported to {}", path_str));
            app.show_toast(format!("Exported to {}", path_str), ToastKind::Success);
        }
        Some(Err(err)) => {
            app.set_status(err.clone());
            app.show_toast(err, ToastKind::Error);
        }
        None => {}
    }
}

/// Initiates asynchronous file export for the active note with a native save picker.
pub fn export_active_note(app: &mut QuickyNotesApp) {
    if app.export_dialog_rx.is_some() {
        return;
    }

    if let Some(note) = app.active_note() {
        let is_qn = note.is_qn();
        let default_ext = if is_qn {
            ".qn"
        } else if note.is_markdown() {
            ".md"
        } else {
            ".txt"
        };

        let filename = if note.title.trim().is_empty() {
            format!("quicky_note{}", default_ext)
        } else if note.title.ends_with(".qn")
            || note.title.ends_with(".qnote")
            || note.title.ends_with(".md")
            || note.title.ends_with(".txt")
        {
            note.title.clone()
        } else {
            format!("{}{}", note.title, default_ext)
        };

        let binary = if is_qn {
            note.encode_qn_binary()
        } else {
            Vec::new()
        };
        let text = note.content.clone();

        app.export_dialog_rx = Some(spawn_export_dialog(filename, is_qn, binary, text));
        app.set_status("Select destination to export note...");
    }
}

/// Handles drag-and-dropped files, file URIs (file://), or raw text snippets.
pub fn handle_dropped_files(app: &mut QuickyNotesApp, ctx: &egui::Context) {
    let dropped = ctx.input_mut(|i| std::mem::take(&mut i.raw.dropped_files));
    if dropped.is_empty() {
        return;
    }

    let mut blocked_count = 0;

    for file in dropped {
        // 1. Direct path available
        let path = file.path();
        if !path.as_os_str().is_empty() && path.exists() && path.is_file() {
            if !is_safe_file_path(path) {
                blocked_count += 1;
                continue;
            }

            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "dropped.qn".to_string());

            if is_image_path(path) {
                if let Ok(bytes) = std::fs::read(path) {
                    let mime = NoteAttachment::detect_mime(&name);
                    attach_image_to_active_note(app, &name, mime, bytes);
                }
                continue;
            }

            if is_qn_path(path)
                && let Ok(bytes) = std::fs::read(path)
                && bytes.starts_with(QN_MAGIC)
                && let Ok(mut note) = Note::decode_qn_binary(&bytes)
            {
                note.file_path = Some(path.to_string_lossy().to_string());
                let id = note.id.clone();
                app.data.notes.push(note);
                app.data.active_note_id = Some(id);
                app.is_dirty = true;
                app.focus_editor = true;
                app.set_status(format!("Opened {}", name));
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                let path_str = path.to_string_lossy().to_string();
                if let Some(existing) = app
                    .data
                    .notes
                    .iter()
                    .find(|n| n.file_path.as_deref() == Some(&path_str))
                {
                    app.data.active_note_id = Some(existing.id.clone());
                } else {
                    let id = format!("note-{}", chrono::Local::now().timestamp_millis());
                    let mut note = Note::new(id.clone(), name.clone());
                    note.content = content;
                    note.file_path = Some(path_str);
                    app.data.notes.push(note);
                    app.data.active_note_id = Some(id);
                    app.is_dirty = true;
                }
                app.focus_editor = true;
                app.set_status(format!("Opened {}", name));
            }
            continue;
        }

        // 2. Raw bytes dropped (e.g. from browser or URI list)
        if let Ok(bytes) = file.bytes() {
            let b: &[u8] = bytes.as_ref();

            // Check if raw image binary
            if b.starts_with(b"\x89PNG")
                || b.starts_with(b"\xFF\xD8\xFF")
                || b.starts_with(b"GIF8")
                || b.starts_with(b"BM")
                || (b.len() > 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP")
            {
                let name = format!("image_{}.png", chrono::Local::now().timestamp_millis());
                let mime = NoteAttachment::detect_mime(&name);
                attach_image_to_active_note(app, &name, mime, b.to_vec());
                continue;
            }

            // Check if .qn binary file
            if b.starts_with(QN_MAGIC)
                && let Ok(note) = Note::decode_qn_binary(b)
            {
                let name = note.title.clone();
                let id = note.id.clone();
                app.data.notes.push(note);
                app.data.active_note_id = Some(id);
                app.is_dirty = true;
                app.focus_editor = true;
                app.set_status(format!("Opened {}", name));
                continue;
            }

            // Process as text or file:// URI list
            process_dropped_bytes(b, app);
        }
    }

    if blocked_count > 0 {
        app.set_status(format!(
            "Blocked {} file(s): path outside allowed directories",
            blocked_count
        ));
    }
}

/// Helper to process byte payload from dropped clipboard or file:// URI strings.
fn process_dropped_bytes(bytes: &[u8], app: &mut QuickyNotesApp) {
    let raw_text = String::from_utf8_lossy(bytes);

    for line in raw_text.lines() {
        let line =
            line.trim_matches(|c: char| c == '\0' || c == '\r' || c == '\n' || c.is_whitespace());
        if line.is_empty() {
            continue;
        }

        let unquoted = line.trim_matches('"').trim_matches('\'');
        let decoded = url_decode(unquoted);

        let path_str = if decoded.starts_with("file://") {
            decoded.trim_start_matches("file://")
        } else if unquoted.starts_with("file://") {
            unquoted.trim_start_matches("file://")
        } else if decoded.starts_with("file:") {
            decoded.trim_start_matches("file:")
        } else {
            decoded.as_str()
        };

        let path = std::path::Path::new(path_str);
        if path.exists() && path.is_file() && is_safe_file_path(path) {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "dropped.qn".to_string());

            if is_image_path(path) {
                if let Ok(img_bytes) = std::fs::read(path) {
                    let mime = NoteAttachment::detect_mime(&name);
                    attach_image_to_active_note(app, &name, mime, img_bytes);
                }
            } else if is_qn_path(path)
                && let Ok(qn_bytes) = std::fs::read(path)
                && let Ok(mut note) = Note::decode_qn_binary(&qn_bytes)
            {
                note.file_path = Some(path_str.to_string());
                let id = note.id.clone();
                app.data.notes.push(note);
                app.data.active_note_id = Some(id);
                app.is_dirty = true;
                app.set_status(format!("Opened {}", name));
            } else if let Ok(content) = std::fs::read_to_string(path) {
                let id = format!("note-{}", chrono::Local::now().timestamp_millis());
                let mut note = Note::new(id.clone(), name.clone());
                note.content = content;
                note.file_path = Some(path_str.to_string());
                app.data.notes.push(note);
                app.data.active_note_id = Some(id);
                app.is_dirty = true;
                app.set_status(format!("Opened {}", name));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_and_qn_path() {
        assert!(is_image_path(Path::new("screenshot.png")));
        assert!(is_image_path(Path::new("photo.JPG")));
        assert!(is_image_path(Path::new("banner.webp")));
        assert!(!is_image_path(Path::new("note.txt")));

        assert!(is_qn_path(Path::new("doc.qn")));
        assert!(is_qn_path(Path::new("bundle.qnote")));
        assert!(!is_qn_path(Path::new("doc.md")));
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(
            url_decode("file:///home/user/my%20notes.qn"),
            "file:///home/user/my notes.qn"
        );
        assert_eq!(url_decode("hello%21%20world"), "hello! world");
    }

    #[test]
    fn test_url_decode_multibyte_utf8() {
        assert_eq!(url_decode("caf%C3%A9.qn"), "café.qn");
        assert_eq!(url_decode("%E6%97%A5"), "日");
    }

    #[test]
    fn test_safe_file_path_blocks_system_dirs() {
        assert!(!is_safe_file_path(Path::new("/etc/passwd")));
        assert!(!is_safe_file_path(Path::new("/proc/self/environ")));
        assert!(!is_safe_file_path(Path::new("/sys/class")));
        assert!(!is_safe_file_path(Path::new("/dev/null")));
    }
}
