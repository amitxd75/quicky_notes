//! Plugin management subsystem coordinating discovery, execution, and UI lifecycle hooks.

pub mod api;
pub mod engine;
pub mod templates;
pub mod types;

pub use engine::PluginEngine;
pub use types::{
    HeaderButtonPosition, NoteMutation, PluginActionOutcome, PluginHeaderButton, PluginMenuItem,
    PluginMetadata, PluginShortcut,
};

use crate::note::Note;
use crate::plugins::api::{NoteHandle, SystemHandle, UiHandle};
use directories::ProjectDirs;
use rhai::AST;
use std::fs;
use std::path::{Path, PathBuf};

/// Loaded in-memory instance of an active or disabled plugin.
#[derive(Clone)]
pub struct PluginInstance {
    /// Manifest metadata.
    pub metadata: PluginMetadata,
    /// Pre-compiled script AST.
    pub ast: AST,
    /// Header buttons registered during `init()`.
    pub header_buttons: Vec<PluginHeaderButton>,
    /// Global shortcuts registered during `init()`.
    pub shortcuts: Vec<PluginShortcut>,
    /// Context menu items registered during `init()`.
    pub menu_items: Vec<PluginMenuItem>,
}

/// Central manager orchestrating plugin discovery, compilation, and event dispatch.
pub struct PluginManager {
    /// List of loaded plugin instances.
    pub plugins: Vec<PluginInstance>,
    /// Underlying sandboxed Rhai script engine.
    pub engine: PluginEngine,
    /// Absolute filesystem path to the user plugins directory.
    pub plugins_dir: PathBuf,
    /// Diagnostic errors and compilation warnings `(plugin_name, error_details)`.
    pub error_log: Vec<(String, String)>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Returns the default user plugins directory path (`~/.config/quicky_notes/plugins`).
    pub fn default_plugins_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "quicky", "quicky_notes") {
            let dir = proj_dirs.config_dir().join("plugins");
            let _ = fs::create_dir_all(&dir);
            dir
        } else {
            let dir = PathBuf::from("plugins");
            let _ = fs::create_dir_all(&dir);
            dir
        }
    }

    /// Initializes a new PluginManager and loads installed plugins.
    pub fn new() -> Self {
        let plugins_dir = Self::default_plugins_dir();
        let engine = PluginEngine::new();

        let mut manager = Self {
            plugins: Vec::new(),
            engine,
            plugins_dir,
            error_log: Vec::new(),
        };

        // Create default template scripts if directory is empty
        manager.ensure_default_templates();
        manager.load_plugins(&[]);
        manager
    }

    /// Generates default starter template plugins if the plugins directory contains no `.rhai` files.
    pub fn ensure_default_templates(&self) {
        let _ = fs::create_dir_all(&self.plugins_dir);

        let term_path = self.plugins_dir.join("quick_terminal.rhai");
        if !term_path.exists() {
            let _ = fs::write(&term_path, templates::QUICK_TERMINAL_TEMPLATE);
        }

        let table_path = self.plugins_dir.join("markdown_table_formatter.rhai");
        if !table_path.exists() {
            let _ = fs::write(&table_path, templates::MARKDOWN_TABLE_FORMATTER_TEMPLATE);
        }
    }

    /// Forcefully recreates the built-in starter templates.
    pub fn create_starter_templates(&mut self) {
        let term_path = self.plugins_dir.join("quick_terminal.rhai");
        let _ = fs::write(&term_path, templates::QUICK_TERMINAL_TEMPLATE);

        let table_path = self.plugins_dir.join("markdown_table_formatter.rhai");
        let _ = fs::write(&table_path, templates::MARKDOWN_TABLE_FORMATTER_TEMPLATE);
    }

    /// Scans the plugins directory and compiles all discovered `.rhai` scripts.
    pub fn load_plugins(&mut self, disabled_ids: &[String]) {
        self.plugins.clear();
        self.error_log.clear();

        if !self.plugins_dir.exists() {
            return;
        }

        let entries = match fs::read_dir(&self.plugins_dir) {
            Ok(iter) => iter,
            Err(err) => {
                self.error_log
                    .push(("Directory Read".to_string(), err.to_string()));
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "rhai") {
                self.load_single_plugin_file(&path, disabled_ids);
            } else if path.is_dir() {
                // Check for plugin.rhai or main.rhai inside subdirectories
                let candidate1 = path.join("plugin.rhai");
                let candidate2 = path.join("main.rhai");
                if candidate1.exists() {
                    self.load_single_plugin_file(&candidate1, disabled_ids);
                } else if candidate2.exists() {
                    self.load_single_plugin_file(&candidate2, disabled_ids);
                }
            }
        }
    }

    /// Loads, parses, compiles, and registers a single `.rhai` file.
    fn load_single_plugin_file(&mut self, path: &Path, disabled_ids: &[String]) {
        let file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let script_content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                self.error_log
                    .push((file_stem, format!("Failed to read file: {err}")));
                return;
            }
        };

        let ast = match self.engine.compile_script(&script_content) {
            Ok(ast) => ast,
            Err(err) => {
                self.error_log.push((file_stem, err));
                return;
            }
        };

        let mut builder = match self.engine.run_init(&ast, &file_stem) {
            Ok(builder) => builder,
            Err(err) => {
                self.error_log.push((file_stem, err));
                return;
            }
        };

        let builder_name = builder.get_name();
        let id = if builder_name.is_empty() {
            file_stem.clone()
        } else {
            builder_name.to_lowercase().replace(' ', "_")
        };

        let is_disabled = disabled_ids.contains(&id);

        let metadata = PluginMetadata {
            id,
            name: if builder_name.is_empty() {
                file_stem
            } else {
                builder_name
            },
            author: {
                let a = builder.get_author();
                if a.is_empty() {
                    "Community".to_string()
                } else {
                    a
                }
            },
            version: {
                let v = builder.get_version();
                if v.is_empty() { "1.0.0".to_string() } else { v }
            },
            description: builder.get_description(),
            file_path: Some(path.to_string_lossy().to_string()),
            enabled: !is_disabled,
        };

        self.plugins.push(PluginInstance {
            metadata,
            ast,
            header_buttons: builder.header_buttons(),
            shortcuts: builder.shortcuts(),
            menu_items: builder.menu_items(),
        });
    }

    /// Reloads all plugins while preserving disabled settings.
    pub fn reload(&mut self, disabled_ids: &[String]) {
        self.load_plugins(disabled_ids);
    }

    /// Returns a list of all custom header buttons from all enabled plugins.
    pub fn all_header_buttons(&self) -> Vec<&PluginHeaderButton> {
        self.plugins
            .iter()
            .filter(|p| p.metadata.enabled)
            .flat_map(|p| &p.header_buttons)
            .collect()
    }

    /// Returns a list of all registered shortcuts from all enabled plugins.
    pub fn all_shortcuts(&self) -> Vec<&PluginShortcut> {
        self.plugins
            .iter()
            .filter(|p| p.metadata.enabled)
            .flat_map(|p| &p.shortcuts)
            .collect()
    }

    /// Returns a list of all context menu items from all enabled plugins.
    pub fn all_menu_items(&self) -> Vec<&PluginMenuItem> {
        self.plugins
            .iter()
            .filter(|p| p.metadata.enabled)
            .flat_map(|p| &p.menu_items)
            .collect()
    }

    /// Dispatches a header button click event to active plugins.
    pub fn dispatch_header_click(
        &self,
        button_id: &str,
        note: Option<&Note>,
        cursor_range: Option<(usize, usize)>,
    ) -> PluginActionOutcome {
        self.dispatch_event("on_header_click", Some(button_id), note, cursor_range)
    }

    /// Dispatches a global keyboard shortcut event to active plugins.
    pub fn dispatch_shortcut(
        &self,
        action_id: &str,
        note: Option<&Note>,
        cursor_range: Option<(usize, usize)>,
    ) -> PluginActionOutcome {
        self.dispatch_event("on_shortcut", Some(action_id), note, cursor_range)
    }

    /// Dispatches an editor context menu selection event to active plugins.
    pub fn dispatch_context_menu(
        &self,
        action_id: &str,
        note: Option<&Note>,
        cursor_range: Option<(usize, usize)>,
    ) -> PluginActionOutcome {
        self.dispatch_event("on_context_menu_click", Some(action_id), note, cursor_range)
    }

    /// Dispatches the `on_save` hook event to active plugins.
    pub fn dispatch_on_save(
        &self,
        note: Option<&Note>,
        cursor_range: Option<(usize, usize)>,
    ) -> PluginActionOutcome {
        self.dispatch_event("on_save", None, note, cursor_range)
    }

    /// Dispatches the `on_note_change` hook event to active plugins.
    pub fn dispatch_on_note_change(
        &self,
        note: Option<&Note>,
        cursor_range: Option<(usize, usize)>,
    ) -> PluginActionOutcome {
        self.dispatch_event("on_note_change", None, note, cursor_range)
    }

    /// Generic internal dispatcher executing an event hook across all enabled plugin ASTs.
    fn dispatch_event(
        &self,
        hook_name: &str,
        arg: Option<&str>,
        note: Option<&Note>,
        cursor_range: Option<(usize, usize)>,
    ) -> PluginActionOutcome {
        let mut outcome = PluginActionOutcome::new();
        let mut note_handle = if let Some(n) = note {
            NoteHandle::from_note(n, cursor_range)
        } else {
            NoteHandle::default()
        };

        let mut ui_handle = UiHandle::new();
        let mut system_handle = SystemHandle::new();

        for plugin in self.plugins.iter().filter(|p| p.metadata.enabled) {
            let res = self.engine.call_event_hook(
                &plugin.ast,
                hook_name,
                arg,
                &mut note_handle,
                &mut ui_handle,
                &mut system_handle,
            );

            if let Err(err) = res {
                outcome.toasts.push((
                    format!("{}: {err}", plugin.metadata.name),
                    crate::app::ToastKind::Error,
                ));
            }
        }

        outcome.mutations = note_handle.take_mutations();
        outcome.toasts.extend(ui_handle.take_toasts());
        outcome.status_msg = ui_handle.take_status_msg();
        outcome.copy_to_clipboard = ui_handle.take_copy_clipboard();
        outcome.request_repaint = ui_handle.is_repaint_requested();

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_engine_basic_script_execution() {
        let engine = PluginEngine::new();
        let script = r#"
            fn init(plugin) {
                plugin.name = "Test Plugin";
                plugin.author = "Tester";
                plugin.version = "2.0.0";
                plugin.description = "A test plugin";
                plugin.add_header_button("btn1", "🚀", "Launch rocket", "right");
                plugin.add_shortcut("rocket", "Ctrl+R");
                plugin.add_context_menu_item("rocket", "Launch Rocket");
                return plugin;
            }

            fn on_header_click(button_id, note, ui, system) {
                if button_id == "btn1" {
                    ui.toast_success("Rocket launched!");
                    note.insert_at_cursor("🚀");
                }
                return note;
            }
        "#;

        let ast = engine.compile_script(script).expect("Compile should pass");
        let mut builder = engine.run_init(&ast, "test").expect("Init should pass");

        assert_eq!(builder.get_name(), "Test Plugin");
        assert_eq!(builder.get_author(), "Tester");
        assert_eq!(builder.get_version(), "2.0.0");
        let header_buttons = builder.header_buttons();
        assert_eq!(header_buttons.len(), 1);
        assert_eq!(header_buttons[0].id, "btn1");
        let shortcuts = builder.shortcuts();
        assert_eq!(shortcuts.len(), 1);
        assert_eq!(shortcuts[0].action_id, "rocket");
        let menu_items = builder.menu_items();
        assert_eq!(menu_items.len(), 1);

        // Test event hook call
        let mut note_handle = NoteHandle::default();
        let mut ui_handle = UiHandle::new();
        let mut system_handle = SystemHandle::new();

        engine
            .call_event_hook(
                &ast,
                "on_header_click",
                Some("btn1"),
                &mut note_handle,
                &mut ui_handle,
                &mut system_handle,
            )
            .expect("Event hook should execute");

        let toasts = ui_handle.take_toasts();
        assert_eq!(toasts.len(), 1);
        assert_eq!(toasts[0].0, "Rocket launched!");
        assert_eq!(
            note_handle.take_mutations(),
            vec![NoteMutation::InsertAtCursor("🚀".to_string())]
        );
    }

    #[test]
    fn test_plugin_safety_max_operations_blocks_infinite_loop() {
        let engine = PluginEngine::new();
        let script = r#"
            fn infinite_loop(note, ui, system) {
                let x = 0;
                while true {
                    x += 1;
                }
            }
        "#;

        let ast = engine.compile_script(script).expect("Compile should pass");
        let mut note_handle = NoteHandle::default();
        let mut ui_handle = UiHandle::new();
        let mut system_handle = SystemHandle::new();

        let result = engine.call_event_hook(
            &ast,
            "infinite_loop",
            None,
            &mut note_handle,
            &mut ui_handle,
            &mut system_handle,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Too many operations"));
    }

    #[test]
    fn test_plugin_note_mutations_and_stats() {
        let engine = PluginEngine::new();
        let script = r#"
            fn modify_note(note, ui, system) {
                let lines = note.get_line_count();
                let words = note.get_word_count();
                note.set_title("New Title");
                note.set_text("Line 1\nLine 2");
                ui.set_status("Done with " + words + " words");
                ui.copy_to_clipboard("Copied Text");
            }
        "#;

        let ast = engine.compile_script(script).expect("Compile should pass");
        let mut note = Note::new("test_note".to_string(), "Initial Title".to_string());
        note.content = "One Two Three".to_string();
        let mut note_handle = NoteHandle::from_note(&note, None);
        let mut ui_handle = UiHandle::new();
        let mut system_handle = SystemHandle::new();

        engine
            .call_event_hook(
                &ast,
                "modify_note",
                None,
                &mut note_handle,
                &mut ui_handle,
                &mut system_handle,
            )
            .expect("Should succeed");

        let mutations = note_handle.take_mutations();
        assert_eq!(
            mutations,
            vec![
                NoteMutation::SetTitle("New Title".to_string()),
                NoteMutation::SetText("Line 1\nLine 2".to_string()),
            ]
        );

        assert_eq!(
            ui_handle.take_status_msg(),
            Some("Done with 3 words".to_string())
        );
        assert_eq!(
            ui_handle.take_copy_clipboard(),
            Some("Copied Text".to_string())
        );
    }
}
