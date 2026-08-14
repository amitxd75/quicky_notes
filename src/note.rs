//! Individual note model and text statistic utilities.

use serde::{Deserialize, Serialize};

/// Represents an individual note tab in Quicky Notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier for the note.
    pub id: String,

    /// Display title of the note tab (e.g. `ideas.txt`).
    pub title: String,

    /// Text contents of the note editor.
    pub content: String,

    /// ISO timestamp string of note creation.
    pub created_at: String,

    /// ISO timestamp string of last modification.
    pub updated_at: String,

    /// Whether this note tab is pinned to the front of the tab bar.
    pub pinned: bool,

    /// Optional color tag hex string for categorization.
    pub color_tag: Option<String>,
}

impl Note {
    /// Creates a new note with a title and timestamp.
    pub fn new(id: String, title: String) -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id,
            title,
            content: String::new(),
            created_at: now.clone(),
            updated_at: now,
            pinned: false,
            color_tag: None,
        }
    }

    /// Computes the word count of the note content.
    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    /// Computes total character count of the note content.
    pub fn char_count(&self) -> usize {
        self.content.chars().count()
    }

    /// Updates the modification timestamp to current local time.
    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    /// Formats updated timestamp for display in status lists.
    pub fn display_time(&self) -> String {
        if self.updated_at.len() >= 16 {
            self.updated_at[11..16].to_string()
        } else {
            self.updated_at.clone()
        }
    }
}
