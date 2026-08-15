//! Persistent JSON data storage manager with atomic writes, corruption recovery, and invariant enforcement.

use crate::note::Note;
use crate::settings::AppSettings;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Structured storage error type for persistence operations.
#[derive(Debug)]
pub enum StorageError {
    /// Standard filesystem I/O error.
    Io(io::Error),
    /// JSON serialization or deserialization error.
    Serialization(serde_json::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "Storage I/O error: {}", err),
            Self::Serialization(err) => write!(f, "Storage serialization error: {}", err),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Serialization(err) => Some(err),
        }
    }
}

impl From<io::Error> for StorageError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

/// Root container for application data, including note tabs and user settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppData {
    /// List of open note tabs (guaranteed non-empty after validation).
    pub notes: Vec<Note>,

    /// ID of the currently active note tab (guaranteed to match an existing note in `notes`).
    pub active_note_id: Option<String>,

    /// Application settings.
    pub settings: AppSettings,
}

impl AppData {
    /// Returns the target filesystem path for `notes_data.json`.
    pub fn config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "quicky", "quicky_notes") {
            let dir = proj_dirs.config_dir();
            let _ = fs::create_dir_all(dir);
            dir.join("notes_data.json")
        } else {
            PathBuf::from("notes_data.json")
        }
    }

    /// Returns the target directory for crash and diagnostic logs.
    pub fn logs_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "quicky", "quicky_notes") {
            let dir = proj_dirs.config_dir().join("logs");
            let _ = fs::create_dir_all(&dir);
            dir
        } else {
            let dir = PathBuf::from("logs");
            let _ = fs::create_dir_all(&dir);
            dir
        }
    }

    /// Loads application data from disk with crash/corruption recovery.
    ///
    /// - If the file does not exist, returns initial default sample notes.
    /// - If the file exists and is valid, repairs any invariant discrepancies and returns it.
    /// - If the file is corrupted, renames it to `.corrupt.<timestamp>` to prevent data loss,
    ///   and falls back to defaults.
    pub fn load() -> Self {
        let path = Self::config_path();
        Self::load_from_path(&path)
    }

    /// Loads application data from a specific file path.
    pub fn load_from_path(path: &Path) -> Self {
        if !path.exists() {
            let mut data = Self::default_initial();
            data.sanitize_and_validate();
            return data;
        }

        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<AppData>(&content) {
                Ok(mut data) => {
                    data.sanitize_and_validate();
                    data
                }
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to parse config at {:?}: {}. Preserving corrupt file.",
                        path, err
                    );
                    Self::backup_corrupt_file(path);
                    let mut data = Self::default_initial();
                    data.sanitize_and_validate();
                    data
                }
            },
            Err(err) => {
                eprintln!("Warning: Failed to read config file at {:?}: {}", path, err);
                let mut data = Self::default_initial();
                data.sanitize_and_validate();
                data
            }
        }
    }

    /// Backs up a corrupted data file to preserve user data before replacing it.
    fn backup_corrupt_file(path: &Path) {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = path.with_extension(format!("corrupt.{}.json", timestamp));
        if let Err(e) = fs::rename(path, &backup_path) {
            eprintln!("Failed to rename corrupt file {:?}: {}", path, e);
        } else {
            eprintln!("Corrupt configuration backed up to {:?}", backup_path);
        }
    }

    /// Saves current notes and settings atomically to `notes_data.json`.
    ///
    /// Writes to a temporary file in the same directory, flushes to disk, and
    /// performs an atomic rename to prevent partial/truncated writes.
    pub fn save(&self) -> Result<(), StorageError> {
        let path = Self::config_path();
        self.save_to_path(&path)
    }

    /// Saves application data atomically to a specific file path.
    pub fn save_to_path(&self, path: &Path) -> Result<(), StorageError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_filename = format!(
            ".{}.tmp.{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("notes_data"),
            std::process::id()
        );
        let temp_path = path
            .parent()
            .map(|p| p.join(&temp_filename))
            .unwrap_or_else(|| PathBuf::from(&temp_filename));

        let json_bytes = serde_json::to_vec_pretty(self)?;

        // Write and sync to temporary file
        let mut file = File::create(&temp_path)?;
        file.write_all(&json_bytes)?;
        file.sync_all()?;
        drop(file);

        // Atomic rename over target file
        if let Err(err) = fs::rename(&temp_path, path) {
            let _ = fs::remove_file(&temp_path);
            return Err(StorageError::Io(err));
        }

        Ok(())
    }

    /// Enforces storage data invariants:
    /// 1. `notes` is never empty.
    /// 2. Note IDs are unique and non-empty.
    /// 3. `active_note_id` references a valid note in `notes`.
    /// 4. Settings are validated and clamped.
    pub fn sanitize_and_validate(&mut self) {
        if self.notes.is_empty() {
            self.notes
                .push(Note::new("note-1".to_string(), "untitled.txt".to_string()));
        }

        let mut seen_ids = HashSet::new();
        for (i, note) in self.notes.iter_mut().enumerate() {
            let fallback_id = format!("note-{}", i + 1);
            note.validate_and_repair(&fallback_id);

            if !seen_ids.insert(note.id.clone()) {
                note.id = format!("{}-{}", note.id, i + 1);
                seen_ids.insert(note.id.clone());
            }
        }

        let active_valid = self
            .active_note_id
            .as_ref()
            .is_some_and(|id| self.notes.iter().any(|n| &n.id == id));

        if !active_valid {
            self.active_note_id = self.notes.first().map(|n| n.id.clone());
        }

        self.settings.validate_and_clamp();
    }

    /// Generates default initial notes for first launch.
    pub fn default_initial() -> Self {
        let mut welcome = Note::new("note-1".to_string(), "welcome.txt".to_string());
        welcome.content = r#"✨ Welcome to Quicky Notes!
A blazing fast, minimal, glassmorphic scratchpad for developers.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚀 Quick Start & Features
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
• Auto-Save: Everything you type is automatically saved in real-time.
• External Files: Drag & drop any text or markdown file to edit directly.
• Direct Sync: Saving (Ctrl+S) updates linked files at their disk location.
• Markdown Preview: Press Ctrl+P on .md notes to toggle live split preview.
• Dynamic Theming: Adapts to system Pywal / Caelestia wallpaper palettes.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⌨️ Essential Shortcuts
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Ctrl + N         Create new note tab
  Ctrl + W         Close active note
  Ctrl + S         Save all notes to disk
  Ctrl + K         Search & browse notes
  Ctrl + ,         Open Settings & Preferences
  Ctrl + P         Toggle Markdown preview (.md)
  Ctrl + Shift + E Export note to file
  Ctrl + Tab       Switch between note tabs
  Double-Click Tab Rename active note

Built with ❤️ for speed, flow, and focus."#
            .to_string();

        Self {
            notes: vec![welcome],
            active_note_id: Some("note-1".to_string()),
            settings: AppSettings::default(),
        }
    }
}

/// Atomically saves AppData to disk.
pub fn save_app_data(data: &AppData) -> Result<(), StorageError> {
    data.save()
}

/// Gracefully formats a filesystem path for display, substituting `$HOME` with `~` and truncating cleanly.
pub fn format_display_path(path_str: &str, max_chars: usize) -> String {
    let home_str = directories::UserDirs::new()
        .and_then(|u| u.home_dir().to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let pretty_path = if !home_str.is_empty() {
        if let Some(suffix) = path_str.strip_prefix(&home_str) {
            format!("~{}", suffix)
        } else {
            path_str.to_string()
        }
    } else {
        path_str.to_string()
    };

    let count = pretty_path.chars().count();
    if count > max_chars {
        let chars: Vec<char> = pretty_path.chars().collect();
        let suffix_len = max_chars.saturating_sub(3);
        let start = chars.len().saturating_sub(suffix_len);
        let suffix: String = chars[start..].iter().collect();
        format!("...{}", suffix)
    } else {
        pretty_path
    }
}

/// Atomically writes arbitrary byte contents to a file path on disk.
///
/// Uses a write-to-temp + rename strategy for atomicity. If the atomic rename fails
/// (e.g., cross-filesystem, NFS, FUSE), falls back to direct `fs::write()` with a
/// warning logged to stderr. The fallback is NOT crash-safe.
pub fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let temp_filename = format!(
        ".{}.tmp.{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
        std::process::id()
    );
    let temp_path = path
        .parent()
        .map(|p| p.join(&temp_filename))
        .unwrap_or_else(|| PathBuf::from(&temp_filename));

    let atomic_result = (|| -> io::Result<()> {
        let mut file = File::create(&temp_path)?;
        file.write_all(content)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp_path, path)?;
        Ok(())
    })();

    if let Err(ref rename_err) = atomic_result {
        let _ = fs::remove_file(&temp_path);
        // Fallback to direct in-place write (works on symlinks, special mounts, /tmp, NFS/remotes).
        // WARNING: This write is NOT atomic — partial/truncated content is possible on crash.
        eprintln!(
            "Warning: Atomic rename failed for {:?} ({}), falling back to non-atomic write",
            path, rename_err
        );
        fs::write(path, content)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let thread_id = format!("{:?}", std::thread::current().id());
        let dir = std::env::temp_dir().join(format!(
            "quicky_notes_test_{}_{}",
            nanos,
            thread_id.replace(|c: char| !c.is_alphanumeric(), "")
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_atomic_save_and_load_roundtrip() {
        let dir = create_test_dir();
        let path = dir.join("test_notes.json");

        let mut initial = AppData::default_initial();
        initial.notes[0].content = "Hello TigerStyle world!".to_string();
        initial.settings.font_size = 20.0;

        initial.save_to_path(&path).expect("Save must succeed");
        assert!(path.exists(), "File must exist after save");

        let loaded = AppData::load_from_path(&path);
        assert_eq!(loaded.notes.len(), initial.notes.len());
        assert_eq!(loaded.notes[0].content, initial.notes[0].content);
        assert_eq!(loaded.settings.font_size, 20.0);
        assert_eq!(loaded.active_note_id, Some("note-1".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_corrupt_file_handling_and_backup() {
        let dir = create_test_dir();
        let path = dir.join("corrupt_notes.json");

        fs::write(&path, "{ invalid json corrupt data ]").unwrap();

        let loaded = AppData::load_from_path(&path);
        assert_eq!(loaded.notes.len(), 1, "Must fall back to default notes");
        assert_eq!(loaded.notes[0].title, "welcome.txt");

        // Verify a corrupt backup file was created
        let entries: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert!(
            entries.iter().any(|name| name.contains("corrupt")),
            "Backup file must be created on corrupt JSON load"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_atomic_write_file_and_fallback() {
        let dir = create_test_dir();
        let target_file = dir.join("test_linked_file.txt");
        let content = b"Hello from linked external file test!";

        atomic_write_file(&target_file, content).expect("atomic write must succeed");
        assert!(target_file.exists());
        let read_back = fs::read(&target_file).expect("must read back");
        assert_eq!(read_back, content);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_format_display_path() {
        let short_path = "/tmp/test.txt";
        let formatted = format_display_path(short_path, 20);
        assert_eq!(formatted, "/tmp/test.txt");

        let long_path = "/a/very/long/nested/path/to/some/deep/directory/structure/file.txt";
        let formatted_long = format_display_path(long_path, 20);
        assert!(formatted_long.starts_with("..."));
        assert!(formatted_long.chars().count() <= 20);
    }
}
