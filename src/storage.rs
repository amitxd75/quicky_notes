//! Persistent JSON data storage manager for notes and application configuration.

use crate::note::Note;
use crate::settings::AppSettings;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Root container for application data, including note tabs and user settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    /// List of open note tabs.
    pub notes: Vec<Note>,

    /// ID of the currently active note tab.
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

    /// Loads application data from disk, creating default initial notes if none exist.
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(data) = serde_json::from_str::<AppData>(&content)
        {
            return data;
        }
        Self::default_initial()
    }

    /// Saves current notes and settings to `notes_data.json`.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(path, content).map_err(|e| format!("IO error: {}", e))
    }

    /// Generates default initial notes for first launch.
    fn default_initial() -> Self {
        let n1 = Note::new("note-1".to_string(), "untitled.txt".to_string());
        let mut n2 = Note::new("note-2".to_string(), "shopping.txt".to_string());
        n2.content = "1. Milk\n2. Eggs\n3. Coffee beans".to_string();

        let mut n3 = Note::new("note-3".to_string(), "ideas.txt".to_string());
        n3.content = "• Glassmorphism dark UI revamp\n• Custom egui styling\n• Keyboard navigation"
            .to_string();

        let mut n4 = Note::new("note-4".to_string(), "tasks.txt".to_string());
        n4.content = "- Finish UI redesign\n- Test auto save\n- Add shortcut hints".to_string();

        Self {
            notes: vec![n1, n2, n3, n4],
            active_note_id: Some("note-1".to_string()),
            settings: AppSettings::default(),
        }
    }
}
