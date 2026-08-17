//! SQLite database backend for notes persistence and tab ordering.
//!
//! Provides transactional ACID storage for notes, metadata, and tab positions.

use crate::models::Note;
use directories::ProjectDirs;
use rusqlite::{Connection, OptionalExtension, params};
use std::fs;
use std::path::{Path, PathBuf};

/// SQLite database manager for Quicky Notes.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Returns default filesystem path for `notes.db`.
    pub fn default_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "quicky", "quicky_notes") {
            let dir = proj_dirs.config_dir();
            let _ = fs::create_dir_all(dir);
            dir.join("notes.db")
        } else {
            PathBuf::from("notes.db")
        }
    }

    /// Opens or creates SQLite database at specified path and runs migrations.
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;

        // WAL mode for fast concurrency and crash resilience
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        let _ = conn.pragma_update(None, "synchronous", "NORMAL");

        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Opens in-memory SQLite database (primarily for automated tests).
    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initializes SQL schema tables and indexes.
    fn init_schema(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                file_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                color_tag TEXT,
                position INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS app_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_notes_position ON notes(position);
            ",
        )?;
        Ok(())
    }

    /// Loads all notes ordered by tab position, plus the active note ID.
    pub fn load_all_notes(&self) -> Result<(Vec<Note>, Option<String>), rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, file_path, created_at, updated_at, pinned, color_tag FROM notes ORDER BY position ASC",
        )?;

        let note_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let content: String = row.get(2)?;
            let file_path: Option<String> = row.get(3)?;
            let created_at: String = row.get(4)?;
            let updated_at: String = row.get(5)?;
            let pinned_int: i64 = row.get(6)?;
            let color_tag: Option<String> = row.get(7)?;

            Ok(Note {
                id,
                title,
                content,
                file_path,
                created_at,
                updated_at,
                pinned: pinned_int != 0,
                color_tag,
            })
        })?;

        let mut notes = Vec::new();
        for note_res in note_iter {
            notes.push(note_res?);
        }

        let active_id: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM app_meta WHERE key = 'active_note_id'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok((notes, active_id))
    }

    /// Atomically replaces all notes and updates the active note ID in a single transaction.
    pub fn save_all_notes(
        &mut self,
        notes: &[Note],
        active_id: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let tx = self.conn.transaction()?;

        // Clear existing records and rewrite with new positions
        tx.execute("DELETE FROM notes", [])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO notes (id, title, content, file_path, created_at, updated_at, pinned, color_tag, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )?;

            for (pos, note) in notes.iter().enumerate() {
                stmt.execute(params![
                    note.id,
                    note.title,
                    note.content,
                    note.file_path,
                    note.created_at,
                    note.updated_at,
                    if note.pinned { 1i64 } else { 0i64 },
                    note.color_tag,
                    pos as i64
                ])?;
            }
        }

        if let Some(act) = active_id {
            tx.execute(
                "INSERT INTO app_meta (key, value) VALUES ('active_note_id', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![act],
            )?;
        } else {
            tx.execute("DELETE FROM app_meta WHERE key = 'active_note_id'", [])?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Deletes a specific note by ID.
    #[allow(dead_code)]
    pub fn delete_note(&mut self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_in_memory_crud() {
        let mut db = Database::in_memory().expect("Failed to create in-memory database");

        let mut note1 = Note::new("id-1".into(), "First Note".into());
        note1.content = "Hello SQLite".into();

        let mut note2 = Note::new("id-2".into(), "Second Note".into());
        note2.content = "Second content".into();

        let notes = vec![note1.clone(), note2.clone()];
        db.save_all_notes(&notes, Some("id-2"))
            .expect("Failed to save notes");

        let (loaded_notes, active_id) = db.load_all_notes().expect("Failed to load notes");
        assert_eq!(loaded_notes.len(), 2);
        assert_eq!(loaded_notes[0].title, "First Note");
        assert_eq!(loaded_notes[1].content, "Second content");
        assert_eq!(active_id, Some("id-2".into()));
    }
}
