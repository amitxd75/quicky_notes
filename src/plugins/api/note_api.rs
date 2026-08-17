//! Note content and metadata mutation API exposed to Rhai scripts.

use crate::models::note::Note;
use crate::plugins::types::NoteMutation;
use std::sync::{Arc, Mutex};

/// Internal snapshot and mutation buffer for a Note.
#[derive(Debug, Clone, Default)]
pub struct NoteHandleInner {
    pub id: String,
    pub title: String,
    pub content: String,
    pub file_path: String,
    pub is_markdown: bool,
    pub selection: String,
    pub cursor_range: Option<(usize, usize)>,
    pub mutations: Vec<NoteMutation>,
}

/// Note manipulation proxy object exposed to Rhai event hooks as `note`.
#[derive(Debug, Clone, Default)]
pub struct NoteHandle {
    inner: Arc<Mutex<NoteHandleInner>>,
}

impl NoteHandle {
    /// Constructs a NoteHandle snapshot from a live Note and cursor range.
    pub fn from_note(note: &Note, cursor_range: Option<(usize, usize)>) -> Self {
        let selection = if let Some((start, end)) = cursor_range {
            let s = start.min(end);
            let e = start.max(end);
            note.char_slice(s, e)
        } else {
            String::new()
        };

        Self {
            inner: Arc::new(Mutex::new(NoteHandleInner {
                id: note.id.clone(),
                title: note.title.clone(),
                content: note.content.clone(),
                file_path: note.file_path.clone().unwrap_or_default(),
                is_markdown: note.is_markdown(),
                selection,
                cursor_range,
                mutations: Vec::new(),
            })),
        }
    }

    /// Returns the full text content of the active note.
    pub fn get_text(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.content.clone())
            .unwrap_or_default()
    }

    /// Overwrites the full text content of the active note.
    pub fn set_text(&mut self, text: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.content = text.clone();
            inner.mutations.push(NoteMutation::SetText(text));
        }
    }

    /// Returns the currently selected text in the editor.
    pub fn get_selection(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.selection.clone())
            .unwrap_or_default()
    }

    /// Replaces the currently selected text with new text.
    pub fn replace_selection(&mut self, text: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.mutations.push(NoteMutation::ReplaceSelection(text));
        }
    }

    /// Inserts text at the current cursor position.
    pub fn insert_at_cursor(&mut self, text: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.mutations.push(NoteMutation::InsertAtCursor(text));
        }
    }

    /// Returns the title of the active note.
    pub fn get_title(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.title.clone())
            .unwrap_or_default()
    }

    /// Sets the title of the active note.
    pub fn set_title(&mut self, title: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.title = title.clone();
            inner.mutations.push(NoteMutation::SetTitle(title));
        }
    }

    /// Returns the linked file path if this note is backed by an external file.
    pub fn get_file_path(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.file_path.clone())
            .unwrap_or_default()
    }

    /// Returns the unique ID of the active note.
    pub fn get_id(&mut self) -> String {
        self.inner.lock().map(|i| i.id.clone()).unwrap_or_default()
    }

    /// Returns the line count of the note content.
    pub fn get_line_count(&mut self) -> i64 {
        self.inner
            .lock()
            .map(|i| i.content.lines().count() as i64)
            .unwrap_or(0)
    }

    /// Returns the word count of the note content.
    pub fn get_word_count(&mut self) -> i64 {
        self.inner
            .lock()
            .map(|i| i.content.split_whitespace().count() as i64)
            .unwrap_or(0)
    }

    /// Returns true if the active note is formatted as Markdown.
    pub fn is_markdown(&mut self) -> bool {
        self.inner.lock().map(|i| i.is_markdown).unwrap_or(true)
    }

    /// Takes all queued mutations.
    pub fn take_mutations(&self) -> Vec<NoteMutation> {
        self.inner
            .lock()
            .map(|mut i| std::mem::take(&mut i.mutations))
            .unwrap_or_default()
    }
}
