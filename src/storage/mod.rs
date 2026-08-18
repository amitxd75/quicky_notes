//! Persistent data storage manager: SQLite database for notes & JSON for configuration.
//!
//! Features:
//! - **SQLite Backend (`notes.db`)**: High-performance, transactional ACID storage for all notes and tab positions.
//! - **JSON Configuration (`config.json`)**: User preferences and theme settings with atomic disk writes.
//! - **Masked Credentials**: API keys are saved obfuscated with local machine salting.

pub mod crypto;
pub mod db;
pub mod linked_files;

pub use crypto::{deobfuscate_key, obfuscate_key};
pub use db::Database;
pub use linked_files::{reconcile_linked_notes_from_disk, sync_linked_notes_to_disk};

use crate::models::AppSettings;
use crate::models::Note;
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
    /// SQLite database error.
    Db(rusqlite::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "Storage I/O error: {}", err),
            Self::Serialization(err) => write!(f, "Storage serialization error: {}", err),
            Self::Db(err) => write!(f, "Database storage error: {}", err),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Serialization(err) => Some(err),
            Self::Db(err) => Some(err),
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

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Db(err)
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
    /// Returns the target filesystem path for `config.json`.
    pub fn config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "quicky", "quicky_notes") {
            let dir = proj_dirs.config_dir();
            Self::ensure_private_dir(dir);
            dir.join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    /// Helper to create and set 0700 permissions on private directories on Unix.
    pub fn ensure_private_dir(dir: &Path) {
        let _ = fs::create_dir_all(dir);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
        }
    }

    /// Returns the target filesystem path for `notes.db`.
    pub fn db_path() -> PathBuf {
        Database::default_path()
    }

    /// Returns the target directory for crash and diagnostic logs.
    pub fn logs_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "quicky", "quicky_notes") {
            let dir = proj_dirs.config_dir().join("logs");
            Self::ensure_private_dir(&dir);
            dir
        } else {
            let dir = PathBuf::from("logs");
            Self::ensure_private_dir(&dir);
            dir
        }
    }

    /// Loads application data from disk (SQLite database for notes + JSON for configuration).
    pub fn load() -> Self {
        let config_path = Self::config_path();
        let db_path = Self::db_path();

        Self::load_with_paths(&config_path, &db_path)
    }

    /// Loads application data from specified configuration and database paths.
    pub fn load_with_paths(config_path: &Path, db_path: &Path) -> Self {
        // 1. Load settings from config.json
        let settings = Self::load_settings_from_path(config_path);

        // 2. Load notes from SQLite notes.db
        let (mut notes, active_note_id) = if !settings.general.restore_session {
            let def = Self::default_initial();
            (def.notes, def.active_note_id)
        } else {
            match Database::open(db_path) {
                Ok(db) => match db.load_all_notes() {
                    Ok((loaded_notes, active_id)) => {
                        if loaded_notes.is_empty() {
                            let def = Self::default_initial();
                            (def.notes, def.active_note_id)
                        } else {
                            (loaded_notes, active_id)
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load notes from SQLite: {}", e);
                        let def = Self::default_initial();
                        (def.notes, def.active_note_id)
                    }
                },
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to open SQLite database at {:?}: {}",
                        db_path, e
                    );
                    let def = Self::default_initial();
                    (def.notes, def.active_note_id)
                }
            }
        };

        // 3. Reconcile linked notes directly from disk if files were modified externally
        for note in &mut notes {
            if let Some(ref path_str) = note.file_path {
                let path = Path::new(path_str);
                if path.exists() && path.is_file() {
                    let mtime = fs::metadata(path).ok().and_then(|m| m.modified().ok());
                    let had_unsaved_state = note.is_dirty || note.has_disk_conflict;

                    if note.is_qn() {
                        if let Ok(bytes) = fs::read(path)
                            && bytes.starts_with(crate::models::QN_MAGIC)
                            && let Ok(decoded) = Note::decode_qn_binary(&bytes)
                        {
                            let differs = note.content != decoded.content
                                || note.attachments != decoded.attachments;
                            if had_unsaved_state && differs {
                                // Genuine unsaved local edits exist: preserve in-memory/SQLite buffer and flag conflict
                                note.has_disk_conflict = true;
                                note.is_dirty = true;
                            } else {
                                // Clean note or matching content: reload from disk
                                note.content = decoded.content;
                                note.attachments = decoded.attachments;
                                note.updated_at = decoded.updated_at;
                                note.has_disk_conflict = false;
                                note.is_dirty = false;
                                note.last_disk_mtime = mtime;
                            }
                        }
                    } else if let Ok(disk_content) = fs::read_to_string(path) {
                        let differs = note.content != disk_content;
                        if had_unsaved_state && differs {
                            // Genuine unsaved local edits exist: preserve in-memory/SQLite buffer and flag conflict
                            note.has_disk_conflict = true;
                            note.is_dirty = true;
                        } else {
                            // Clean note or matching content: reload from disk
                            note.content = disk_content;
                            note.has_disk_conflict = false;
                            note.is_dirty = false;
                            note.last_disk_mtime = mtime;
                        }
                    }
                }
            }
        }

        let mut data = Self {
            notes,
            active_note_id,
            settings,
        };
        data.sanitize_and_validate();
        data
    }

    /// Loads settings from JSON configuration path with corruption backup.
    fn load_settings_from_path(path: &Path) -> AppSettings {
        if !path.exists() {
            return AppSettings::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
                Ok(mut settings) => {
                    settings.validate_and_clamp();
                    settings
                }
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to parse settings at {:?}: {}. Preserving corrupt file.",
                        path, err
                    );
                    Self::backup_corrupt_file(path);
                    AppSettings::default()
                }
            },
            Err(err) => {
                eprintln!(
                    "Warning: Failed to read settings file at {:?}: {}",
                    path, err
                );
                AppSettings::default()
            }
        }
    }

    /// Backs up a corrupted configuration file.
    fn backup_corrupt_file(path: &Path) {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = path.with_extension(format!("corrupt.{}.json", timestamp));
        let _ = fs::rename(path, backup_path);
    }

    /// Saves notes to SQLite (`notes.db`) and settings to JSON (`config.json`).
    pub fn save(&self) -> Result<(), StorageError> {
        let config_path = Self::config_path();
        let db_path = Self::db_path();
        self.save_to_paths(&config_path, &db_path)
    }

    /// Saves application data to specific configuration and database paths.
    pub fn save_to_paths(&self, config_path: &Path, db_path: &Path) -> Result<(), StorageError> {
        // Defensive check: ensure all note IDs are strictly unique before writing, keeping active_id aligned
        let mut sanitized_notes = self.notes.clone();
        let mut seen_ids = HashSet::new();
        let mut active_id = self.active_note_id.clone();

        for note in &mut sanitized_notes {
            if note.id.trim().is_empty() || !seen_ids.insert(note.id.clone()) {
                let old_id = note.id.clone();
                let new_id = crate::models::note::generate_unique_note_id();
                note.id = new_id.clone();
                seen_ids.insert(new_id.clone());
                if active_id.as_deref() == Some(&old_id) {
                    active_id = Some(new_id);
                }
            }
        }

        // 1. Save notes to SQLite in an ACID transaction
        let mut db = Database::open(db_path)?;
        db.save_all_notes(&sanitized_notes, active_id.as_deref())?;

        // 2. Save settings to config.json with atomic write
        Self::save_settings_to_path(&self.settings, config_path)?;

        Ok(())
    }

    /// Atomically writes AppSettings to a JSON file with restricted file permissions.
    pub fn save_settings_to_path(settings: &AppSettings, path: &Path) -> Result<(), StorageError> {
        if let Some(parent) = path.parent() {
            Self::ensure_private_dir(parent);
        }

        let temp_filename = format!(
            ".{}.tmp.{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("config"),
            std::process::id()
        );
        let temp_path = path
            .parent()
            .map(|p| p.join(&temp_filename))
            .unwrap_or_else(|| PathBuf::from(&temp_filename));

        let json_bytes = serde_json::to_vec_pretty(settings)?;

        #[cfg(unix)]
        let mut file = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut options = fs::OpenOptions::new();
            options.write(true).create(true).truncate(true).mode(0o600);
            options.open(&temp_path)?
        };
        #[cfg(not(unix))]
        let mut file = File::create(&temp_path)?;

        file.write_all(&json_bytes)?;
        file.sync_all()?;
        drop(file);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600));
        }

        if let Err(err) = fs::rename(&temp_path, path) {
            let _ = fs::remove_file(&temp_path);
            return Err(StorageError::Io(err));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    /// Enforces storage data invariants.
    pub fn sanitize_and_validate(&mut self) {
        if self.notes.is_empty() {
            self.notes.push(Note::new(
                crate::models::note::generate_unique_note_id(),
                crate::models::DEFAULT_NOTE_TITLE.to_string(),
            ));
        }

        let mut seen_ids = HashSet::new();
        for (i, note) in self.notes.iter_mut().enumerate() {
            let fallback_id = format!("note-{}", i + 1);
            note.validate_and_repair(&fallback_id);

            if !seen_ids.insert(note.id.clone()) {
                let old_id = note.id.clone();
                let new_id = crate::models::note::generate_unique_note_id();
                note.id = new_id.clone();
                seen_ids.insert(new_id.clone());
                if self.active_note_id.as_deref() == Some(&old_id) {
                    self.active_note_id = Some(new_id);
                }
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
        let mut welcome_qn = Note::new("note-1".to_string(), "welcome.qn".to_string());
        welcome_qn.content = r#"# ✨ Quicky Notes — Markdown & Images

Welcome to your fast, lightweight floating scratchpad!

## 🚀 Features
- **Embedded Images**: Drag & drop screenshots or images directly into your notes.
- **True-Color Rendering**: Images render in authentic, unaltered colors without theme tinting.
- **Markdown & Code**: Live syntax highlighting, task lists, and split preview mode.
- **AI Copilot**: Select text and press `Ctrl+Enter` to fix, summarize, or translate.

```rust
fn main() {
    println!("Hello from Quicky Notes .qn bundle! 🚀");
}
```
"#
        .to_string();

        Self {
            notes: vec![welcome_qn],
            active_note_id: Some("note-1".to_string()),
            settings: AppSettings::default(),
        }
    }
}

/// Atomically writes arbitrary content to a target file via a temporary file + rename,
/// falling back to direct write + sync if atomic rename fails (e.g. cross-filesystem boundary or permissions).
pub fn atomic_write_file(path: &Path, data: &[u8]) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_filename = format!(
        ".{}.tmp.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("quicky_file"),
        std::process::id()
    );
    let temp_path = path
        .parent()
        .map(|p| p.join(&temp_filename))
        .unwrap_or_else(|| PathBuf::from(&temp_filename));

    // 1. Try atomic write to temp file then rename
    let atomic_result = (|| -> Result<(), io::Error> {
        let mut file = File::create(&temp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp_path, path)?;
        Ok(())
    })();

    match atomic_result {
        Ok(()) => Ok(()),
        Err(_) => {
            // Clean up temporary file if left behind
            let _ = fs::remove_file(&temp_path);

            // 2. Resilient direct write fallback
            let mut file = File::create(path)?;
            file.write_all(data)?;
            file.sync_all()?;
            Ok(())
        }
    }
}

/// Atomically saves AppData to disk.
pub fn save_app_data(data: &AppData) -> Result<(), StorageError> {
    data.save()
}

/// Gracefully formats a filesystem path for display, substituting `$HOME` with `~` and truncating cleanly.
pub fn format_display_path(path_str: &str, max_chars: usize) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let display = if !home.is_empty() && path_str.starts_with(&home) {
        format!("~{}", &path_str[home.len()..])
    } else {
        path_str.to_string()
    };

    if display.chars().count() <= max_chars {
        display
    } else {
        let mut chars = display.chars();
        let first_len = (max_chars.saturating_sub(3)) / 2;
        let last_len = max_chars.saturating_sub(3) - first_len;

        let prefix: String = (&mut chars).take(first_len).collect();
        let total_count = display.chars().count();
        let suffix: String = display.chars().skip(total_count - last_len).collect();

        format!("{}...{}", prefix, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_and_json_save_load_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!(
            "quicky_notes_test_storage_{}",
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = fs::create_dir_all(&temp_dir);

        let config_path = temp_dir.join("config.json");
        let db_path = temp_dir.join("notes.db");

        let mut data = AppData::default_initial();
        data.notes[0].title = "Custom Title".to_string();
        data.settings.editor.font_size = 20.0;
        data.settings.ai.api_key = "sk-test-secret-key-999".to_string();

        data.save_to_paths(&config_path, &db_path).unwrap();

        // Verify config.json contains obfuscated key
        let config_raw = fs::read_to_string(&config_path).unwrap();
        assert!(config_raw.contains("enc:v1:"));
        assert!(!config_raw.contains("sk-test-secret-key-999"));

        // Load back and verify data integrity
        let loaded = AppData::load_with_paths(&config_path, &db_path);
        assert_eq!(loaded.notes.len(), 1);
        assert_eq!(loaded.notes[0].title, "Custom Title");
        assert_eq!(loaded.settings.editor.font_size, 20.0);
        assert_eq!(loaded.settings.ai.api_key, "sk-test-secret-key-999");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_format_display_path() {
        let long_path = "/var/log/very/long/nested/directory/structure/that/exceeds/limit/file.txt";
        let formatted = format_display_path(long_path, 30);
        assert!(formatted.len() <= 35);
        assert!(formatted.contains("..."));
    }
}
