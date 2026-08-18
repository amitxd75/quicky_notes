//! Real-time disk synchronization and live reloading for externally linked files.
//!
//! TigerStyle principles applied:
//! - Direct, atomic file I/O operations without intermediate cache drift.
//! - Strict validation of UTF-8 and QN binary header magic.
//! - Clear error reporting for corrupted or inaccessible disk paths.

use crate::models::Note;
use crate::storage::atomic_write_file;
use std::fs;
use std::path::Path;

/// Atomically writes a specific linked note back to disk storage.
pub fn sync_single_linked_note_to_disk(note: &mut Note) -> Result<(), String> {
    if let Some(ref path_str) = note.file_path {
        let path = Path::new(path_str);
        let write_data = if note.is_qn() {
            note.encode_qn_binary()
        } else {
            note.content.as_bytes().to_vec()
        };

        match atomic_write_file(path, &write_data) {
            Ok(()) => {
                note.is_dirty = false;
                note.has_disk_conflict = false;
                note.last_disk_mtime = fs::metadata(path).ok().and_then(|m| m.modified().ok());
                Ok(())
            }
            Err(e) => Err(format!("{}: {}", note.title, e)),
        }
    } else {
        Ok(())
    }
}

/// Atomically writes all open notes that have an associated disk file back to storage.
///
/// Returns `Ok(synced_count)` on total success, or `Err(errors)` containing all per-file failures.
pub fn sync_linked_notes_to_disk(notes: &mut [Note]) -> Result<usize, Vec<String>> {
    let mut sync_errors = Vec::new();
    let mut linked_count = 0;

    for note in notes {
        if note.file_path.is_some() && note.is_dirty && !note.has_disk_conflict {
            linked_count += 1;
            if let Err(err_msg) = sync_single_linked_note_to_disk(note) {
                sync_errors.push(err_msg);
            }
        }
    }

    if sync_errors.is_empty() {
        Ok(linked_count)
    } else {
        Err(sync_errors)
    }
}

/// Checks all open notes linked to disk files and reloads their in-memory buffer if changed externally.
///
/// Returns `true` if any note buffer was reloaded or state updated, indicating a UI repaint is required.
pub fn reconcile_linked_notes_from_disk(notes: &mut [Note]) -> bool {
    let mut any_reloaded = false;

    for note in notes {
        if let Some(ref path_str) = note.file_path {
            let path = Path::new(path_str);
            if !path.exists() || !path.is_file() {
                continue;
            }

            let current_mtime = fs::metadata(path).ok().and_then(|m| m.modified().ok());

            if note.last_disk_mtime.is_some() && current_mtime == note.last_disk_mtime {
                continue;
            }

            // If the note has unsaved in-memory edits, do not overwrite editor buffer
            if note.is_dirty {
                if note.is_qn() {
                    if let Ok(bytes) = fs::read(path)
                        && bytes.starts_with(crate::models::QN_MAGIC)
                        && let Ok(decoded) = Note::decode_qn_binary(&bytes)
                        && (decoded.content != note.content
                            || decoded.attachments != note.attachments)
                    {
                        note.has_disk_conflict = true;
                        note.last_disk_mtime = current_mtime;
                        any_reloaded = true;
                    } else {
                        note.last_disk_mtime = current_mtime;
                    }
                } else if let Ok(disk_content) = fs::read_to_string(path) {
                    if disk_content != note.content {
                        note.has_disk_conflict = true;
                        note.last_disk_mtime = current_mtime;
                        any_reloaded = true;
                    } else {
                        note.last_disk_mtime = current_mtime;
                    }
                }
                continue;
            }

            // Note is clean: safely reload disk contents into memory
            if note.is_qn() {
                if let Ok(bytes) = fs::read(path)
                    && bytes.starts_with(crate::models::QN_MAGIC)
                    && let Ok(decoded) = Note::decode_qn_binary(&bytes)
                {
                    if decoded.content != note.content || decoded.attachments != note.attachments {
                        note.content = decoded.content;
                        note.attachments = decoded.attachments;
                        note.updated_at = decoded.updated_at;
                        any_reloaded = true;
                    }
                    note.has_disk_conflict = false;
                    note.last_disk_mtime = current_mtime;
                }
            } else if let Ok(disk_content) = fs::read_to_string(path) {
                if disk_content != note.content {
                    note.content = disk_content;
                    note.update_timestamp();
                    any_reloaded = true;
                }
                note.has_disk_conflict = false;
                note.last_disk_mtime = current_mtime;
            }
        }
    }

    any_reloaded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_and_reconcile_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!(
            "quicky_test_linked_{}",
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = fs::create_dir_all(&temp_dir);

        let file_path = temp_dir.join("linked_demo.rs");
        fs::write(&file_path, "fn main() { println!(\"Hello\"); }").unwrap();

        let mut note = Note::new("test-id".to_string(), "linked_demo.rs".to_string());
        note.file_path = Some(file_path.to_string_lossy().to_string());
        note.content = "fn main() { println!(\"Modified\"); }".to_string();
        note.is_dirty = true;

        let mut notes = vec![note];

        // 1. Sync in-memory modifications to disk
        let synced = sync_linked_notes_to_disk(&mut notes).expect("sync should succeed");
        assert_eq!(synced, 1);

        let disk_text = fs::read_to_string(&file_path).unwrap();
        assert_eq!(disk_text, "fn main() { println!(\"Modified\"); }");

        // 2. External program modifies file on disk
        fs::write(&file_path, "fn main() { println!(\"External Update\"); }").unwrap();

        // 3. Reconcile from disk
        let reloaded = reconcile_linked_notes_from_disk(&mut notes);
        assert!(reloaded);
        assert_eq!(
            notes[0].content,
            "fn main() { println!(\"External Update\"); }"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_dirty_conflict_preserves_in_memory_content() {
        let temp_dir = std::env::temp_dir().join(format!(
            "quicky_test_conflict_{}",
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = fs::create_dir_all(&temp_dir);

        let file_path = temp_dir.join("conflict_demo.txt");
        fs::write(&file_path, "Initial disk content").unwrap();

        let mut note = Note::new("conflict-id".to_string(), "conflict_demo.txt".to_string());
        note.file_path = Some(file_path.to_string_lossy().to_string());
        note.content = "Unsaved in-memory edit".to_string();
        note.is_dirty = true;
        note.last_disk_mtime = fs::metadata(&file_path)
            .ok()
            .and_then(|m| m.modified().ok());

        let mut notes = vec![note];

        // External process changes the file on disk
        fs::write(&file_path, "External disk change").unwrap();

        // Reconcile must not overwrite in-memory edits when note is dirty
        let changed = reconcile_linked_notes_from_disk(&mut notes);
        assert!(changed);
        assert!(notes[0].has_disk_conflict);
        assert_eq!(notes[0].content, "Unsaved in-memory edit");

        let _ = fs::remove_dir_all(temp_dir);
    }
}
