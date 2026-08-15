//! Native file dialogs, file export, drag-and-drop parsing, and URL decoding.

use crate::app::QuickyNotesApp;
use crate::note::Note;
use eframe::egui;

/// Decodes percent-encoded URL strings (e.g. `%20` -> ` `).
pub fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(sub) = std::str::from_utf8(&bytes[i + 1..i + 3])
            && let Ok(val) = u8::from_str_radix(sub, 16)
        {
            result.push(val as char);
            i += 3;
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Opens a native file dialog (Zenity/Kdialog) to import and link text files.
pub fn open_file_dialog(app: &mut QuickyNotesApp) {
    let output = std::process::Command::new("zenity")
        .arg("--file-selection")
        .output()
        .or_else(|_| {
            std::process::Command::new("kdialog")
                .arg("--getopenfilename")
                .output()
        });

    if let Ok(out) = output
        && out.status.success()
    {
        let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let path = std::path::Path::new(&path_str);
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

    for (name, content, file_path) in files_to_open {
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
}

/// Helper to process byte payload from dropped files or file:// URI strings.
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
}
