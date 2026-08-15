//! Core Quicky Notes application controller and event loop coordination.

use crate::note::Note;
use crate::storage::AppData;
use crate::theme;
use crate::ui;
use eframe::egui::{self, Color32, Ui, ViewportCommand};
use std::time::{Duration, Instant};

/// Markdown preview mode for note editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkdownViewMode {
    #[default]
    Edit,
    Split,
    Preview,
}

impl MarkdownViewMode {
    /// Returns the next view mode in the cycle: Edit -> Split -> Preview -> Edit.
    pub fn next(self) -> Self {
        match self {
            Self::Edit => Self::Split,
            Self::Split => Self::Preview,
            Self::Preview => Self::Edit,
        }
    }

    /// Display icon for mode switcher.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Edit => "📝",
            Self::Split => "◫",
            Self::Preview => "👁",
        }
    }

    /// Tooltip label for mode switcher.
    pub fn tooltip(self) -> &'static str {
        match self {
            Self::Edit => "Edit Mode (Ctrl+P)",
            Self::Split => "Split Mode (Ctrl+P)",
            Self::Preview => "Preview Mode (Ctrl+P)",
        }
    }
}

/// Navigation tabs in the Settings & Preferences drawer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Appearance,
    Editor,
    FilesBackup,
    Shortcuts,
    About,
}

impl SettingsTab {
    pub const ALL: [Self; 6] = [
        Self::General,
        Self::Appearance,
        Self::Editor,
        Self::FilesBackup,
        Self::Shortcuts,
        Self::About,
    ];

    pub fn icon_and_label(self) -> (&'static str, &'static str) {
        match self {
            Self::General => ("⚙", "General"),
            Self::Appearance => ("🎨", "Appearance"),
            Self::Editor => ("📝", "Editor"),
            Self::FilesBackup => ("📁", "Files & Backup"),
            Self::Shortcuts => ("⌨", "Shortcuts"),
            Self::About => ("ℹ", "About"),
        }
    }
}

/// Main application state for Quicky Notes.
pub struct QuickyNotesApp {
    /// Saved data container (notes, active note ID, settings).
    pub data: AppData,

    /// Live query string entered in search drawer.
    pub search_query: String,

    /// Highlighted index in search result list.
    pub search_selected_idx: usize,

    /// ID of note title currently undergoing inline rename.
    pub editing_title_id: Option<String>,

    /// Whether the Options & Settings drawer is open.
    pub show_options: bool,

    /// Whether the Search & Browse drawer is open.
    pub show_search: bool,

    /// Trigger flag to request focus on the search input field.
    pub focus_search: bool,

    /// Trigger flag to request focus on the main text editor.
    pub focus_editor: bool,

    /// Markdown preview mode (Edit, Split, or Preview).
    pub preview_mode: MarkdownViewMode,

    /// Active tab in Settings & Preferences drawer.
    pub settings_tab: SettingsTab,

    /// Status bar notification text and creation timestamp.
    pub status_msg: Option<(String, Instant)>,

    /// Last auto-save timestamp.
    pub last_auto_save: Instant,

    /// Last wallpaper color polling timestamp.
    pub last_wallpaper_check: Instant,

    /// Cached wallpaper colors for change detection.
    pub last_wallpaper_colors: Option<(Color32, Color32, Color32)>,

    /// ID of note tab waiting for close confirmation in modal dialog.
    pub confirm_close_id: Option<String>,

    /// Shutdown flag to prevent drawing during Wayland surface teardown.
    pub is_closing: bool,

    /// Whether unsaved modifications exist.
    pub is_dirty: bool,
}

impl QuickyNotesApp {
    /// Primary constructor using pre-loaded AppData.
    pub fn new_with_data(cc: &eframe::CreationContext<'_>, data: AppData) -> Self {
        crate::font::setup_default_fonts(&cc.egui_ctx);
        theme::setup_glassmorphism_theme(&cc.egui_ctx, &data.settings);
        crate::font::apply_system_font(&cc.egui_ctx, &data.settings.selected_font);
        let initial_colors = theme::get_wallpaper_colors();

        if data.settings.always_on_top {
            cc.egui_ctx
                .send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
        }

        Self {
            data,
            search_query: String::new(),
            search_selected_idx: 0,
            editing_title_id: None,
            show_options: false,
            show_search: false,
            focus_search: false,
            focus_editor: true,
            preview_mode: MarkdownViewMode::Edit,
            settings_tab: SettingsTab::General,
            status_msg: Some(("Quicky Notes ready".to_string(), Instant::now())),
            last_auto_save: Instant::now(),
            last_wallpaper_check: Instant::now(),
            last_wallpaper_colors: initial_colors,
            confirm_close_id: None,
            is_closing: false,
            is_dirty: false,
        }
    }

    /// Sets status bar notification text.
    pub fn set_status(&mut self, text: impl Into<String>) {
        self.status_msg = Some((text.into(), Instant::now()));
    }

    /// Returns immutable reference to active note.
    pub fn active_note(&self) -> Option<&Note> {
        let active_id = self.data.active_note_id.as_deref()?;
        self.data.notes.iter().find(|n| n.id == active_id)
    }

    /// Creates a new note tab and selects it.
    pub fn create_new_note(&mut self) {
        let count = self.data.notes.len() + 1;
        let id = format!("note-{}", chrono::Local::now().timestamp_millis());
        let ext = if self.data.settings.default_extension.starts_with('.') {
            &self.data.settings.default_extension
        } else {
            ".txt"
        };
        let title = format!("note_{}{}", count, ext);
        let note = Note::new(id.clone(), title);
        self.data.notes.push(note);
        self.data.active_note_id = Some(id);
        self.focus_editor = true;
        self.is_dirty = true;
        self.set_status("New tab created");
    }

    /// Prompts close confirmation modal if note has content, or closes immediately if empty or disabled.
    pub fn prompt_close_note(&mut self, id: &str) {
        if let Some(note) = self.data.notes.iter().find(|n| n.id == id) {
            if !self.data.settings.confirm_close_tab || note.content.trim().is_empty() {
                self.close_note(id);
            } else {
                self.confirm_close_id = Some(id.to_string());
            }
        }
    }

    /// Closes note tab by ID, resetting to untitled if it was the last open tab.
    pub fn close_note(&mut self, id: &str) {
        if self.data.notes.len() <= 1 {
            if let Some(note) = self.data.notes.first_mut() {
                let ext = &self.data.settings.default_extension;
                note.title = format!("untitled{}", ext);
                note.content.clear();
                note.update_timestamp();
            }
            self.set_status("Tab cleared");
            self.is_dirty = true;
            return;
        }

        if let Some(pos) = self.data.notes.iter().position(|n| n.id == id) {
            self.data.notes.remove(pos);
            if self.data.active_note_id.as_deref() == Some(id) {
                let next_idx = pos.saturating_sub(1);
                self.data.active_note_id = self.data.notes.get(next_idx).map(|n| n.id.clone());
            }
            self.is_dirty = true;
            self.set_status("Tab closed");
        }
    }

    /// Explicitly forces an immediate save of all notes and settings to disk, syncing linked external files.
    pub fn save_notes_to_disk(&mut self) {
        self.is_dirty = true;
        self.save_if_dirty();
        self.set_status("Saved to disk ✓");
    }

    /// Saves notes and settings if dirty flag is set.
    /// If notes are linked to external files on disk, they are saved directly to their respective file paths.
    pub fn save_if_dirty(&mut self) {
        if self.is_dirty {
            if self.data.settings.trim_trailing_whitespace {
                for note in &mut self.data.notes {
                    if note
                        .content
                        .lines()
                        .any(|l| l.ends_with(' ') || l.ends_with('\t'))
                    {
                        let trimmed: Vec<&str> =
                            note.content.lines().map(|l| l.trim_end()).collect();
                        let mut new_content = trimmed.join("\n");
                        if note.content.ends_with('\n') {
                            new_content.push('\n');
                        }
                        note.content = new_content;
                    }
                }
            }

            // Sync any externally linked notes directly to their target files on disk
            let mut sync_errors = Vec::new();
            let mut linked_count = 0;
            for note in &self.data.notes {
                if let Some(ref path_str) = note.file_path {
                    linked_count += 1;
                    let path = std::path::Path::new(path_str);
                    if let Err(e) = crate::storage::atomic_write_file(path, note.content.as_bytes())
                    {
                        sync_errors.push(format!("{}: {}", note.title, e));
                    }
                }
            }

            let _ = crate::storage::save_app_data(&self.data);
            self.is_dirty = false;

            if !sync_errors.is_empty() {
                self.set_status(format!("Disk save warning: {}", sync_errors.join(", ")));
            } else if linked_count > 0 {
                self.set_status("Synced & saved");
            } else {
                self.set_status("Auto-saved");
            }
        }
    }

    /// Main frame update cycle.
    fn update_app(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        if self.is_closing {
            return;
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            self.is_closing = true;
            self.save_if_dirty();
            return;
        }

        ui::drag_drop::handle_dropped_files(self, ctx);
        ui::shortcuts::handle_keyboard_shortcuts(self, ctx);

        // Auto-save interval check
        if self.last_auto_save.elapsed()
            > Duration::from_secs(self.data.settings.auto_save_seconds as u64)
        {
            self.save_if_dirty();
            self.last_auto_save = Instant::now();
        }

        // Real-time wallpaper theme sync (polls Caelestia/Pywal every 1s with mtime caching)
        if self.data.settings.theme_mode == theme::ThemeMode::WallpaperSync
            && self.last_wallpaper_check.elapsed() > Duration::from_secs(1)
        {
            self.last_wallpaper_check = Instant::now();
            if theme::check_wallpaper_color_change(&mut self.last_wallpaper_colors) {
                theme::setup_glassmorphism_theme(ctx, &self.data.settings);
                ctx.request_repaint();
            }
        }

        // Render main editor and active drawers
        ui::editor::render_main_editor(self, ctx, ui);

        // Render modal overlays
        ui::editor::render_close_confirmation_modal(self, ctx);
        ui::editor::render_drop_hover_overlay(ctx);
    }
}

impl eframe::App for QuickyNotesApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.update_app(&ctx, ui);
    }

    fn on_exit(&mut self) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.save_if_dirty();
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> QuickyNotesApp {
        let mut data = AppData::default_initial();
        data.sanitize_and_validate();
        QuickyNotesApp {
            data,
            search_query: String::new(),
            search_selected_idx: 0,
            editing_title_id: None,
            show_options: false,
            show_search: false,
            focus_search: false,
            focus_editor: true,
            preview_mode: MarkdownViewMode::Edit,
            settings_tab: SettingsTab::General,
            status_msg: None,
            last_auto_save: Instant::now(),
            last_wallpaper_check: Instant::now(),
            last_wallpaper_colors: None,
            confirm_close_id: None,
            is_closing: false,
            is_dirty: false,
        }
    }

    #[test]
    fn test_app_create_and_close_notes() {
        let mut app = create_test_app();
        let initial_len = app.data.notes.len();
        app.create_new_note();
        assert_eq!(app.data.notes.len(), initial_len + 1);

        let new_id = app.data.active_note_id.clone().unwrap();
        app.close_note(&new_id);
        assert_eq!(app.data.notes.len(), initial_len);
    }

    #[test]
    fn test_linked_note_direct_disk_sync() {
        let mut app = create_test_app();
        let temp_dir = std::env::temp_dir().join(format!(
            "quicky_notes_test_sync_{}",
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let external_file = temp_dir.join("linked_doc.txt");
        std::fs::write(&external_file, "Initial file contents").unwrap();

        let mut note = Note::new("test-linked-1".to_string(), "linked_doc.txt".to_string());
        note.content = "Modified content inside Quicky Notes".to_string();
        note.file_path = Some(external_file.to_string_lossy().to_string());
        app.data.notes.push(note);
        app.data.active_note_id = Some("test-linked-1".to_string());

        app.save_notes_to_disk();

        let disk_content = std::fs::read_to_string(&external_file).unwrap();
        assert_eq!(disk_content, "Modified content inside Quicky Notes");

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
