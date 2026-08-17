//! Core Quicky Notes application controller and event loop coordination.

use crate::note::Note;
use crate::storage::AppData;
use crate::theme;
use crate::ui;
pub use crate::ui::markdown::MarkdownViewMode;
pub use crate::ui::options_drawer::SettingsTab;
pub use crate::ui::toast::{Toast, ToastKind};
use eframe::egui::{self, Ui, ViewportCommand};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Main application state for Quicky Notes.
pub struct QuickyNotesApp {
    /// Saved data container (notes, active note ID, settings).
    pub data: AppData,

    /// Live query string entered in search drawer.
    pub search_query: String,

    /// Highlighted index in search result list.
    pub search_selected_idx: usize,

    /// ID and text buffer of note title currently undergoing inline rename.
    /// Stores `(note_id, editable_buffer)` so keystrokes persist across frames.
    pub editing_title: Option<(String, String)>,

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
    pub last_wallpaper_colors: Option<crate::theme::PaletteColors>,

    /// ID of note tab waiting for close confirmation in modal dialog.
    pub confirm_close_id: Option<String>,

    /// Shutdown flag to prevent drawing during Wayland surface teardown.
    pub is_closing: bool,

    /// Whether unsaved modifications exist.
    pub is_dirty: bool,

    /// Receiver for async file dialog results.
    pub file_dialog_rx: Option<mpsc::Receiver<Option<String>>>,

    /// Receiver for async file export dialog results.
    pub export_dialog_rx: Option<mpsc::Receiver<Result<String, String>>>,

    /// Receiver for async font loading results.
    pub font_loading_rx: Option<mpsc::Receiver<egui::FontDefinitions>>,

    /// Cached active note statistics (words, chars, lines), computed once per frame.
    pub cached_active_stats: (usize, usize, usize),

    /// Active shortcut action currently awaiting user keypress to rebind.
    pub recording_shortcut: Option<crate::ui::shortcuts::ShortcutAction>,

    /// Active floating toast notification.
    pub toast: Option<Toast>,

    /// Interactive AI Copilot & Fixer modal state.
    pub ai_modal: crate::ui::ai_modal::AiModalState,

    /// Dedicated receiver for AI provider connection testing.
    pub ai_test_rx: Option<mpsc::Receiver<crate::ai::AiResult>>,

    /// Receiver for async AI custom theme generation.
    pub ai_theme_rx: Option<mpsc::Receiver<Result<crate::engine::GeneratedTheme, String>>>,

    /// User prompt input buffer for AI theme generation.
    pub ai_theme_prompt: String,

    /// Suggestion engine for inline ghost writing autocomplete.
    pub suggestion_engine: crate::suggest::SuggestionEngine,

    /// Receiver for async background loading of the suggestion engine dictionary.
    pub suggestion_rx: Option<mpsc::Receiver<crate::suggest::SuggestionEngine>>,

    /// Current active ghost completion suffix displayed at the cursor.
    pub active_ghost_suffix: Option<String>,

    /// Last recorded cursor selection character range `(start_idx, end_idx)`.
    pub last_cursor_range: Option<(usize, usize)>,

    /// Split preview ratio (0.15 to 0.85) defining the width of the editor vs preview pane.
    pub split_ratio: f32,

    /// Last timestamp for window size persistence check.
    pub last_window_size_check: Instant,

    /// Active collapsible folder explorer workspace.
    pub folder_workspace: Option<crate::ui::folder_tree::FolderWorkspace>,

    /// Whether the folder tree sidebar is currently visible.
    pub show_folder_sidebar: bool,

    /// Receiver for async directory selection dialogs.
    pub folder_dialog_rx: Option<mpsc::Receiver<Option<String>>>,

    /// Last timestamp for real-time external disk change polling.
    pub last_disk_sync: Instant,
}

impl QuickyNotesApp {
    /// Primary constructor using pre-loaded AppData.
    ///
    /// Font loading is performed asynchronously to avoid blocking the UI thread.
    pub fn new_with_data(cc: &eframe::CreationContext<'_>, data: AppData) -> Self {
        theme::setup_glassmorphism_theme(&cc.egui_ctx, &data.settings);
        let initial_colors = theme::get_wallpaper_colors();

        if data.settings.always_on_top {
            cc.egui_ctx
                .send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
        }

        // Load fonts asynchronously on a background thread to avoid startup delay
        let font_rx = crate::font::setup_fonts_async(&cc.egui_ctx, &data.settings.selected_font);

        // Pre-index existing notes and load dictionary asynchronously
        let note_texts = data.notes.iter().map(|n| n.content.clone()).collect();
        let suggest_rx = crate::suggest::SuggestionEngine::start_async_load(note_texts);

        Self {
            data,
            search_query: String::new(),
            search_selected_idx: 0,
            editing_title: None,
            show_options: false,
            show_search: false,
            focus_search: false,
            focus_editor: true,
            preview_mode: MarkdownViewMode::Edit,
            settings_tab: SettingsTab::General,
            status_msg: None,
            last_auto_save: Instant::now(),
            last_wallpaper_check: Instant::now(),
            last_wallpaper_colors: initial_colors,
            confirm_close_id: None,
            is_closing: false,
            is_dirty: false,
            file_dialog_rx: None,
            export_dialog_rx: None,
            font_loading_rx: Some(font_rx),

            cached_active_stats: (0, 0, 0),
            recording_shortcut: None,
            toast: None,
            ai_modal: crate::ui::ai_modal::AiModalState::default(),
            ai_test_rx: None,
            ai_theme_rx: None,
            ai_theme_prompt: String::new(),
            suggestion_engine: crate::suggest::SuggestionEngine::default(),
            suggestion_rx: Some(suggest_rx),
            active_ghost_suffix: None,
            last_cursor_range: None,
            split_ratio: 0.5,
            last_window_size_check: Instant::now(),
            folder_workspace: None,
            show_folder_sidebar: false,
            folder_dialog_rx: None,
            last_disk_sync: Instant::now(),
        }
    }

    /// Initializes QuickyNotesApp with AppData and executes parsed CLI options.
    pub fn new_with_data_and_cli(
        cc: &eframe::CreationContext<'_>,
        data: AppData,
        cli: crate::platform::CliArgs,
    ) -> Self {
        let mut app = Self::new_with_data(cc, data);

        if let Some(folder_path) = cli.folder {
            app.open_folder_workspace(&folder_path);
        }

        for file_path in cli.files {
            app.open_file_from_path(&file_path);
        }

        if cli.new_tab {
            app.create_new_note();
        }

        app
    }

    /// Opens a folder workspace for browsing files and projects in a collapsible sidebar.
    pub fn open_folder_workspace(&mut self, path: &std::path::Path) {
        if let Some(workspace) = crate::ui::folder_tree::FolderWorkspace::open(path) {
            let name = workspace.root_name.clone();
            self.folder_workspace = Some(workspace);
            self.show_folder_sidebar = true;
            self.show_options = false;
            self.show_search = false;
            self.show_toast(format!("Opened folder: {}", name), ToastKind::Success);
            self.set_status(format!("Folder: {}", name));
        } else {
            self.show_toast("Failed to open folder workspace", ToastKind::Error);
        }
    }

    /// Closes the active folder workspace sidebar.
    pub fn close_folder_workspace(&mut self) {
        self.folder_workspace = None;
        self.show_folder_sidebar = false;
        self.set_status("Folder workspace closed");
    }

    /// Opens or selects a note tab for the specified file path.
    pub fn open_file_from_path(&mut self, path: &std::path::Path) {
        crate::ui::drag_drop::open_path_into_app(self, path);
    }

    /// Launches the AI Copilot modal using the currently selected text or surrounding cursor context.
    pub fn trigger_ai_assist(&mut self) {
        crate::ui::ai_modal::launch_copilot(self);
    }

    // -------------------------------------------------------------------------
    // Notifications & Status
    // -------------------------------------------------------------------------

    /// Returns human-readable keybinding label for a shortcut action.
    pub fn shortcut_label(&self, action: crate::ui::shortcuts::ShortcutAction) -> String {
        self.data
            .settings
            .keybindings
            .get(action)
            .to_display_string()
    }

    /// Sets status bar notification text.
    pub fn set_status(&mut self, text: impl Into<String>) {
        self.status_msg = Some((text.into(), Instant::now()));
    }

    /// Triggers an animated floating toast notification.
    pub fn show_toast(&mut self, text: impl Into<String>, kind: ToastKind) {
        let msg = text.into();
        self.set_status(msg.clone());
        self.toast = Some(Toast::new(msg, kind));
    }

    // -------------------------------------------------------------------------
    // Note Tab Management
    // -------------------------------------------------------------------------

    /// Returns immutable reference to active note.
    pub fn active_note(&self) -> Option<&Note> {
        let active_id = self.data.active_note_id.as_deref()?;
        self.data.notes.iter().find(|n| n.id == active_id)
    }

    /// Returns mutable reference to active note.
    pub fn active_note_mut(&mut self) -> Option<&mut Note> {
        let active_id = self.data.active_note_id.clone()?;
        self.data.notes.iter_mut().find(|n| n.id == active_id)
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

    /// Prompts close confirmation modal if note has unsaved changes, or closes immediately for linked files / empty notes.
    pub fn prompt_close_note(&mut self, id: &str) {
        if let Some(note) = self.data.notes.iter().find(|n| n.id == id) {
            // Linked disk files or notes without unsaved custom content close immediately without modal darkening
            if note.file_path.is_some()
                || !self.data.settings.confirm_close_tab
                || note.content.trim().is_empty()
            {
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
                note.file_path = None;
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

    // -------------------------------------------------------------------------
    // Persistence & Synchronization
    // -------------------------------------------------------------------------

    /// Explicitly forces an immediate save of all notes and settings to disk, syncing linked external files.
    ///
    /// Unlike auto-save, this always triggers a save regardless of the dirty flag.
    pub fn save_notes_to_disk(&mut self) {
        self.save_if_dirty_internal(true);
        self.show_toast("Saved to disk ✓", ToastKind::Success);
    }

    /// Saves notes and settings if dirty flag is set.
    /// If notes are linked to external files on disk, they are saved directly to their respective file paths.
    pub fn save_if_dirty(&mut self) {
        self.save_if_dirty_internal(false);
    }

    /// Internal core persistence logic called by both debounced autosave and manual save.
    fn save_if_dirty_internal(&mut self, force: bool) {
        if !self.is_dirty && !force {
            return;
        }

        // Defensive normalization of tab state before persisting
        if self.data.notes.is_empty() {
            let note = Note::new(
                "default".to_string(),
                format!("note_1{}", self.data.settings.default_extension),
            );
            self.data.active_note_id = Some("default".to_string());
            self.data.notes.push(note);
        }

        // Automatic trailing whitespace trimming on manual Ctrl+S
        if force && self.data.settings.trim_trailing_whitespace {
            for note in &mut self.data.notes {
                let trimmed: String = note
                    .content
                    .lines()
                    .map(|l| l.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n");
                if trimmed != note.content {
                    note.content = trimmed;
                    note.update_timestamp();
                }
            }
        }

        // Sync any externally linked notes directly to their target files on disk
        let sync_res = crate::storage::sync_linked_notes_to_disk(&mut self.data.notes);
        let (linked_count, sync_errors) = match sync_res {
            Ok(count) => (count, Vec::new()),
            Err(errs) => (0, errs),
        };

        // Check save result and re-set dirty on failure
        match crate::storage::save_app_data(&self.data) {
            Ok(()) => {
                self.is_dirty = !sync_errors.is_empty();
            }
            Err(e) => {
                self.is_dirty = true;
                self.show_toast(format!("Save failed: {}", e), ToastKind::Error);
                return;
            }
        }

        if !sync_errors.is_empty() {
            self.show_toast(
                format!("Failed to save linked file(s): {}", sync_errors.join(", ")),
                ToastKind::Error,
            );
        } else if linked_count > 0 {
            self.set_status("Synced & saved");
        } else if !force {
            self.set_status("Auto-saved");
        }
    }

    /// Checks all open notes that are linked to files on disk, and reloads buffer if modified externally.
    pub fn reconcile_linked_notes_from_disk(&mut self) -> bool {
        crate::storage::reconcile_linked_notes_from_disk(&mut self.data.notes)
    }

    // -------------------------------------------------------------------------
    // Frame Lifecycle Loop
    // -------------------------------------------------------------------------

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

        // Poll for async font loading completion
        if let Some(rx) = &self.font_loading_rx
            && let Ok(font_defs) = rx.try_recv()
        {
            ctx.set_fonts(font_defs);
            self.font_loading_rx = None;
        }

        // Poll for async suggestion engine dictionary loading
        if let Some(rx) = &self.suggestion_rx
            && let Ok(engine) = rx.try_recv()
        {
            self.suggestion_engine = engine;
            self.suggestion_rx = None;
        }

        // Poll for async AI test connection results
        if let Some(rx) = &self.ai_test_rx
            && let Ok(result) = rx.try_recv()
        {
            self.ai_test_rx = None;
            match result {
                crate::ai::AiResult::Success { result_text, .. } => {
                    self.show_toast(
                        format!("✓ AI Online: {}", result_text),
                        crate::ui::toast::ToastKind::Success,
                    );
                }
                crate::ai::AiResult::Error(err) => {
                    self.show_toast(
                        format!("✗ AI Test Failed: {}", err),
                        crate::ui::toast::ToastKind::Error,
                    );
                }
            }
            ctx.request_repaint();
        }

        // Poll for async AI theme generation results
        if let Some(rx) = &self.ai_theme_rx
            && let Ok(result) = rx.try_recv()
        {
            self.ai_theme_rx = None;
            match result {
                Ok(theme) => {
                    self.data.settings.apply_generated_theme(&theme);
                    theme::setup_glassmorphism_theme(ctx, &self.data.settings);
                    self.show_toast(
                        format!("🎨 Applied AI Theme: {}", theme.name),
                        crate::ui::toast::ToastKind::Success,
                    );
                    self.is_dirty = true;
                }
                Err(err) => {
                    self.show_toast(
                        format!("Theme Generation Failed: {}", err),
                        crate::ui::toast::ToastKind::Error,
                    );
                }
            }
            ctx.request_repaint();
        }

        // Keep frame loop reactive while background AI requests are in flight
        if self.ai_test_rx.is_some() || self.ai_modal.is_loading || self.ai_theme_rx.is_some() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        // Poll for async file, folder, and export dialog completion
        ui::drag_drop::poll_file_dialog(self);
        ui::drag_drop::poll_folder_dialog(self);
        ui::drag_drop::poll_export_dialog(self);

        ui::drag_drop::handle_dropped_files(self, ctx);
        ui::shortcuts::handle_keyboard_shortcuts(self, ctx);

        // Compute active note stats once per frame
        self.cached_active_stats = self.active_note().map_or((0, 0, 0), |n| n.compute_stats());

        // Defense-in-depth guard against zero auto_save_seconds
        if self.last_auto_save.elapsed()
            > Duration::from_secs(self.data.settings.auto_save_seconds.max(1) as u64)
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

        // Persist window size on resize (throttled to once per second)
        if self.last_window_size_check.elapsed() > Duration::from_secs(1) {
            self.last_window_size_check = Instant::now();
            if let Some(rect) = ctx.input(|i| i.viewport().inner_rect) {
                let new_w = rect.width();
                let new_h = rect.height();
                if (new_w - self.data.settings.window_width).abs() > 1.0
                    || (new_h - self.data.settings.window_height).abs() > 1.0
                {
                    self.data.settings.window_width = new_w;
                    self.data.settings.window_height = new_h;
                    self.is_dirty = true;
                }
            }
        }

        // Real-time live disk sync for externally modified files (every 500ms or on window focus)
        let is_window_focused = ctx.input(|i| i.raw.focused);
        if self.last_disk_sync.elapsed() > Duration::from_millis(500) || is_window_focused {
            self.last_disk_sync = Instant::now();
            if self.reconcile_linked_notes_from_disk() {
                self.set_status("Live reloaded from disk");
                ctx.request_repaint();
            }
        }

        // Render main editor and active drawers
        ui::editor::render_main_editor(self, ctx, ui);

        // Render modal overlays
        ui::editor::render_drop_hover_overlay(ctx);
        ui::ai_modal::render_ai_copilot_modal(self, ctx);
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
            editing_title: None,
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
            file_dialog_rx: None,
            export_dialog_rx: None,
            font_loading_rx: None,

            cached_active_stats: (0, 0, 0),
            recording_shortcut: None,
            toast: None,
            ai_modal: crate::ui::ai_modal::AiModalState::default(),
            ai_test_rx: None,
            ai_theme_rx: None,
            ai_theme_prompt: String::new(),
            suggestion_engine: crate::suggest::SuggestionEngine::default(),
            suggestion_rx: None,
            active_ghost_suffix: None,
            last_cursor_range: None,
            split_ratio: 0.5,
            last_window_size_check: Instant::now(),
            folder_workspace: None,
            show_folder_sidebar: false,
            folder_dialog_rx: None,
            last_disk_sync: Instant::now(),
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
        let temp_dir = std::env::temp_dir().join(format!(
            "quicky_notes_test_sync_{}",
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let external_file = temp_dir.join("test_file.txt");
        std::fs::write(&external_file, "Initial file contents").unwrap();

        let mut note = Note::new("test-linked-1".to_string(), "test_file.txt".to_string());
        note.content = "Modified content inside Quicky Notes".to_string();
        note.file_path = Some(external_file.to_string_lossy().to_string());

        // Verify direct atomic write to linked disk file without touching user config
        if let Some(ref path_str) = note.file_path {
            let path = std::path::Path::new(path_str);
            crate::storage::atomic_write_file(path, note.content.as_bytes()).unwrap();
        }

        let disk_content = std::fs::read_to_string(&external_file).unwrap();
        assert_eq!(disk_content, "Modified content inside Quicky Notes");

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_reconcile_linked_notes_live_reload() {
        let temp_dir = std::env::temp_dir().join(format!(
            "quicky_notes_test_reload_{}",
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let external_file = temp_dir.join("external_doc.md");
        std::fs::write(&external_file, "Version 1 on disk").unwrap();

        let mut app = create_test_app();
        app.open_file_from_path(&external_file);

        let active_note = app.active_note().expect("should have opened note");
        assert_eq!(active_note.content, "Version 1 on disk");

        // External editor modifies the file on disk
        std::fs::write(&external_file, "Version 2 modified by external editor!").unwrap();

        // Run live reload reconciliation
        let reloaded = app.reconcile_linked_notes_from_disk();
        assert!(reloaded);

        let reloaded_note = app.active_note().expect("should have active note");
        assert_eq!(
            reloaded_note.content,
            "Version 2 modified by external editor!"
        );

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
