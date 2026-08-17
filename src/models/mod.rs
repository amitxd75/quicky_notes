//! Core data models and invariant enforcers.

pub mod note;
pub mod settings;

pub use note::{DEFAULT_NOTE_TITLE, MAX_NOTE_TITLE_LEN, Note};
pub use settings::{AppSettings, WindowSizePreset};
