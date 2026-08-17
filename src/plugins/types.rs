//! Core plugin types, metadata structures, and action outcome models.

use serde::{Deserialize, Serialize};

/// Layout position for custom plugin header buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HeaderButtonPosition {
    /// Placed on the left near the tab bar.
    Left,
    /// Placed on the right near standard action controls.
    #[default]
    Right,
}

impl HeaderButtonPosition {
    /// Parses position from string identifier.
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "left" | "tab" | "start" => Self::Left,
            _ => Self::Right,
        }
    }
}

/// A custom button registered by a plugin for the header bar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHeaderButton {
    /// Unique action identifier triggered on click.
    pub id: String,
    /// Icon text or glyph (e.g. ">_", "", "⏱", "🐙").
    pub icon: String,
    /// Hover tooltip text.
    pub tooltip: String,
    /// Bar position placement.
    pub position: HeaderButtonPosition,
}

/// A custom keyboard shortcut registered by a plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginShortcut {
    /// Unique action identifier triggered on shortcut press.
    pub action_id: String,
    /// Key combination description (e.g. "Ctrl+`", "Alt+T", "F12").
    pub key_combination: String,
    /// Human-readable label for shortcut listings.
    pub label: String,
}

/// A custom menu item registered by a plugin for the editor context menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMenuItem {
    /// Unique action identifier triggered when clicked.
    pub action_id: String,
    /// Display label text (e.g. "Format Tables", "Base64 Encode").
    pub label: String,
    /// Optional leading icon glyph.
    pub icon: Option<String>,
}

/// Metadata and manifest attributes for an installed plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique slug ID (e.g. "quick_terminal", "markdown_formatter").
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Author / developer name.
    pub author: String,
    /// Semantic version string (e.g. "1.0.0").
    pub version: String,
    /// Brief description of plugin capabilities.
    pub description: String,
    /// Path to the script file on disk.
    pub file_path: Option<String>,
    /// Whether the plugin is currently active.
    pub enabled: bool,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "Unnamed Plugin".to_string(),
            author: "Anonymous".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            file_path: None,
            enabled: true,
        }
    }
}

/// Individual text or title mutation applied to a note buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteMutation {
    /// Replaces entire buffer text.
    SetText(String),
    /// Replaces currently selected text range (or inserts at cursor if no selection).
    ReplaceSelection(String),
    /// Inserts text at the current cursor offset.
    InsertAtCursor(String),
    /// Renames the note title.
    SetTitle(String),
}

/// Aggregate results and side effects returned from a plugin execution.
#[derive(Debug, Clone, Default)]
pub struct PluginActionOutcome {
    /// Note buffer mutations to apply.
    pub mutations: Vec<NoteMutation>,
    /// Toasts to display to the user.
    pub toasts: Vec<(String, crate::app::ToastKind)>,
    /// Status bar message updates.
    pub status_msg: Option<String>,
    /// Text to copy to system clipboard.
    pub copy_to_clipboard: Option<String>,
    /// Whether an egui frame repaint was requested.
    pub request_repaint: bool,
}

impl PluginActionOutcome {
    /// Creates an empty outcome.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a toast notification to the outcome.
    pub fn with_toast(mut self, message: impl Into<String>, kind: crate::app::ToastKind) -> Self {
        self.toasts.push((message.into(), kind));
        self
    }
}
