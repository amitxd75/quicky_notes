//! Individual note model, text statistic utilities, and invariant validation.

use serde::{Deserialize, Serialize};

/// Maximum title length to prevent UI overflow or excessive allocation.
pub const MAX_NOTE_TITLE_LEN: usize = 128;

/// Default fallback title for unnamed notes.
pub const DEFAULT_NOTE_TITLE: &str = "untitled.txt";

/// Represents an individual note tab in Quicky Notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier for the note (must not be empty).
    pub id: String,

    /// Display title of the note tab (e.g. `ideas.txt`).
    pub title: String,

    /// Text contents of the note editor.
    pub content: String,

    /// Formatted timestamp string of note creation (`YYYY-MM-DD HH:MM:SS`).
    pub created_at: String,

    /// Formatted timestamp string of last modification (`YYYY-MM-DD HH:MM:SS`).
    pub updated_at: String,

    /// Whether this note tab is pinned to the front of the tab bar.
    pub pinned: bool,

    /// Optional color tag hex string for categorization.
    pub color_tag: Option<String>,

    /// Optional filesystem path if this note is linked directly to a file on disk.
    #[serde(default)]
    pub file_path: Option<String>,
}

impl Note {
    /// Creates a new note with a validated ID and title.
    ///
    /// # Invariants
    /// - `id` is verified non-empty (empty IDs are replaced with timestamp-based fallbacks instead of panicking).
    /// - `title` is sanitized (trimmed, clamped in length, falling back to `DEFAULT_NOTE_TITLE` if blank).
    pub fn new(id: String, title: String) -> Self {
        let id = if id.trim().is_empty() {
            format!("note-{}", chrono::Local::now().timestamp_millis())
        } else {
            id
        };

        let sanitized_title = Self::sanitize_title(&title);
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Self {
            id,
            title: sanitized_title,
            content: String::new(),
            created_at: now.clone(),
            updated_at: now,
            pinned: false,
            color_tag: None,
            file_path: None,
        }
    }

    /// Sanitizes and clamps note title.
    pub fn sanitize_title(title: &str) -> String {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            DEFAULT_NOTE_TITLE.to_string()
        } else if trimmed.len() > MAX_NOTE_TITLE_LEN {
            let mut end = MAX_NOTE_TITLE_LEN;
            while !trimmed.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            trimmed[..end].to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Validates and repairs internal invariants after deserialization.
    pub fn validate_and_repair(&mut self, fallback_id: &str) {
        if self.id.trim().is_empty() {
            self.id = fallback_id.to_string();
        }
        self.title = Self::sanitize_title(&self.title);

        if self.created_at.is_empty() {
            self.created_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        }
        if self.updated_at.is_empty() {
            self.updated_at = self.created_at.clone();
        }
    }

    /// Computes (word_count, char_count, line_count) in a single fast pass.
    pub fn compute_stats(&self) -> (usize, usize, usize) {
        let mut words = 0;
        let mut chars = 0;
        let mut lines = 1;
        let mut in_word = false;

        for c in self.content.chars() {
            chars += 1;
            if c == '\n' {
                lines += 1;
                in_word = false;
            } else if c.is_whitespace() {
                in_word = false;
            } else if !in_word {
                in_word = true;
                words += 1;
            }
        }

        (words, chars, lines)
    }

    /// Returns estimated word count of the content.
    #[inline]
    #[allow(dead_code)]
    pub fn word_count(&self) -> usize {
        self.compute_stats().0
    }

    /// Returns total character count of the content.
    #[inline]
    #[allow(dead_code)]
    pub fn char_count(&self) -> usize {
        self.compute_stats().1
    }

    /// Returns total number of lines (minimum 1).
    #[inline]
    #[allow(dead_code)]
    pub fn line_count(&self) -> usize {
        self.compute_stats().2
    }

    /// Updates the modification timestamp to current local time.
    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    /// Formats updated timestamp safely for display in status and search lists.
    ///
    /// Extracts `HH:MM` from `YYYY-MM-DD HH:MM:SS` without risking UTF-8 panics.
    /// Safety: byte slicing is safe here because `chrono::Local::now().format("%Y-%m-%d %H:%M:%S")`
    /// produces exclusively ASCII characters. The `is_char_boundary` checks provide defense-in-depth.
    pub fn display_time(&self) -> String {
        let text = self.updated_at.trim();
        if text.len() >= 16 && text.is_char_boundary(11) && text.is_char_boundary(16) {
            text[11..16].to_string()
        } else {
            text.to_string()
        }
    }

    /// Returns whether this note is a Markdown document (.md or .markdown).
    #[inline]
    pub fn is_markdown(&self) -> bool {
        let lower = self.title.to_lowercase();
        lower.ends_with(".md") || lower.ends_with(".markdown")
    }

    /// Returns whether this note is linked to an external file on the local filesystem.
    #[inline]
    pub fn is_linked_file(&self) -> bool {
        self.file_path.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation_and_defaults() {
        let note = Note::new("note-100".to_string(), "  my_notes.txt  ".to_string());
        assert_eq!(note.id, "note-100");
        assert_eq!(note.title, "my_notes.txt");
        assert_eq!(note.word_count(), 0);
    }

    #[test]
    fn test_safe_display_time() {
        let mut note = Note::new("n1".to_string(), "t.txt".to_string());
        note.updated_at = "2026-08-15 14:32:05".to_string();
        assert_eq!(note.display_time(), "14:32");
    }
}
