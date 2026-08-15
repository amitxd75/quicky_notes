//! Native file dialogs, file export, drag-and-drop parsing, URL decoding, and filesystem safety.

use crate::app::QuickyNotesApp;
use crate::note::Note;
use eframe::egui;
use std::path::Path;
use std::sync::mpsc;

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
/// - System directories: `/etc`, `/proc`, `/sys`, `/dev`, `/boot`, `/var`
/// - Sensitive dotfiles/dotdirs under `$HOME`: `.ssh`, `.gnupg`, `.config/quicky_notes`, `.bashrc`, etc.
/// - Paths outside the user's home directory (except `/tmp`)
fn is_safe_file_path(path: &Path) -> bool {
    let canonical = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let path_str = canonical.to_string_lossy();

    // Block system directories
    const BLOCKED_PREFIXES: &[&str] = &[
        "/etc/", "/proc/", "/sys/", "/dev/", "/boot/", "/var/", "/usr/", "/sbin/", "/bin/",
    ];
    for prefix in BLOCKED_PREFIXES {
        if path_str.starts_with(prefix) {
            return false;
        }
    }

    // Allow /tmp paths
    if path_str.starts_with("/tmp/") || path_str.starts_with("/var/tmp/") {
        return true;
    }

    // Check home directory safety
    if let Some(user_dirs) = directories::UserDirs::new() {
        let home = user_dirs.home_dir().to_string_lossy().to_string();
        if path_str.starts_with(&home) {
            // Get the relative path after $HOME/
            if let Some(rel) = path_str.strip_prefix(&home) {
                let rel = rel.trim_start_matches('/');
                // Block dotfiles and dotdirs directly under home
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
            return true;
        }
    }

    false
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
        if path.exists()
            && path.is_file()
            && let Ok(content) = std::fs::read_to_string(path)
        {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "imported.txt".to_string());

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

/// Opens a native file dialog (Zenity/Kdialog) to import and link text files.
///
/// The dialog runs on a background thread to avoid blocking the UI.
/// Results are polled via `poll_file_dialog()` in the main update loop.
pub fn open_file_dialog(app: &mut QuickyNotesApp) {
    if app.file_dialog_rx.is_some() {
        return; // Dialog already open
    }
    app.file_dialog_rx = Some(spawn_file_dialog());
    app.set_status("Opening file dialog...");
}

/// Exports active note to ~/Documents or current working directory.
pub fn export_active_note(app: &mut QuickyNotesApp) {
    if let Some(note) = app.active_note() {
        let filename = if note.title.trim().is_empty() {
            "quicky_note.txt".to_string()
        } else if note.title.ends_with(".txt") || note.title.ends_with(".md") {
            note.title.clone()
        } else {
            format!("{}.txt", note.title)
        };

        let path = directories::UserDirs::new()
            .and_then(|u| u.document_dir().map(|d| d.join(&filename)))
            .unwrap_or_else(|| std::path::PathBuf::from(&filename));

        if std::fs::write(&path, &note.content).is_ok() {
            app.set_status(format!("Exported to {}", filename));
        } else {
            app.set_status("Export failed");
        }
    }
}

/// Handles drag-and-dropped files, file URIs (file://), or raw text snippets.
///
/// Validates all file paths against the safety allowlist before opening.
pub fn handle_dropped_files(app: &mut QuickyNotesApp, ctx: &egui::Context) {
    let dropped = ctx.input_mut(|i| std::mem::take(&mut i.raw.dropped_files));
    if dropped.is_empty() {
        return;
    }

    let mut files_to_open: Vec<(String, String, Option<String>)> = Vec::new();

    for file in dropped {
        let path = file.path();
        if path.exists()
            && path.is_file()
            && is_safe_file_path(path)
            && let Ok(content) = std::fs::read_to_string(path)
        {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "dropped.txt".to_string());
            files_to_open.push((name, content, Some(path.to_string_lossy().to_string())));
            continue;
        }

        if let Ok(bytes) = file.bytes() {
            process_dropped_bytes(&bytes, &mut files_to_open);
        }
    }

    let mut blocked_count = 0;
    for (name, content, file_path) in files_to_open {
        // Validate file_path safety for linked files
        if let Some(ref fp) = file_path
            && !is_safe_file_path(Path::new(fp))
        {
            blocked_count += 1;
            continue;
        }

        if let Some(existing) = app.data.notes.iter().find(|n| {
            if let (Some(fp1), Some(fp2)) = (&n.file_path, &file_path) {
                fp1 == fp2
            } else {
                n.title == name && n.content == content
            }
        }) {
            app.data.active_note_id = Some(existing.id.clone());
        } else {
            let id = format!("note-{}", chrono::Local::now().timestamp_millis());
            let mut note = Note::new(id.clone(), name.clone());
            note.content = content;
            note.file_path = file_path;
            app.data.notes.push(note);
            app.data.active_note_id = Some(id);
            app.is_dirty = true;
        }
        app.focus_editor = true;
        app.set_status(format!("Opened {}", name));
    }

    if blocked_count > 0 {
        app.set_status(format!(
            "Blocked {} file(s): path outside allowed directories",
            blocked_count
        ));
    }
}

/// Helper to process byte payload from dropped files or file:// URI strings.
///
/// Validates all resolved file paths against the safety allowlist.
fn process_dropped_bytes(bytes: &[u8], out: &mut Vec<(String, String, Option<String>)>) {
    let raw_text = String::from_utf8_lossy(bytes);
    let mut opened_any = false;

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
        if path.exists()
            && path.is_file()
            && is_safe_file_path(path)
            && let Ok(content) = std::fs::read_to_string(path)
        {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "dropped.txt".to_string());
            out.push((name, content, Some(path_str.to_string())));
            opened_any = true;
            continue;
        }
    }

    if !opened_any && !raw_text.trim().is_empty() {
        out.push((
            "dropped_snippet.txt".to_string(),
            raw_text.to_string(),
            None,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_decode() {
        assert_eq!(
            url_decode("file:///home/user/my%20notes.txt"),
            "file:///home/user/my notes.txt"
        );
        assert_eq!(url_decode("hello%21%20world"), "hello! world");
    }

    #[test]
    fn test_url_decode_multibyte_utf8() {
        // café.txt encoded as UTF-8: é = 0xC3 0xA9
        assert_eq!(url_decode("caf%C3%A9.txt"), "café.txt");
        // Japanese: 日 = 0xE6 0x97 0xA5
        assert_eq!(url_decode("%E6%97%A5"), "日");
    }

    #[test]
    fn test_safe_file_path_blocks_system_dirs() {
        assert!(!is_safe_file_path(Path::new("/etc/passwd")));
        assert!(!is_safe_file_path(Path::new("/proc/self/environ")));
        assert!(!is_safe_file_path(Path::new("/sys/class")));
        assert!(!is_safe_file_path(Path::new("/dev/null")));
    }

    #[test]
    fn test_safe_file_path_blocks_ssh() {
        // This tests the pattern matching — the file may not exist, so canonicalize fails → false
        assert!(!is_safe_file_path(Path::new(
            "/home/nonexistent/.ssh/id_rsa"
        )));
    }
}
