//! Plugin registration builder exposed to Rhai `init(plugin)` hook.

use crate::plugins::types::{
    HeaderButtonPosition, PluginHeaderButton, PluginMenuItem, PluginMetadata, PluginShortcut,
};
use std::sync::{Arc, Mutex};

/// Internal builder state shared safely across Rhai execution boundaries.
#[derive(Debug, Clone, Default)]
pub struct PluginBuilderInner {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub header_buttons: Vec<PluginHeaderButton>,
    pub shortcuts: Vec<PluginShortcut>,
    pub menu_items: Vec<PluginMenuItem>,
    pub timers: Vec<crate::plugins::types::PluginTimer>,
}

/// Builder object passed to the `init(plugin)` entrypoint in Rhai scripts.
#[derive(Debug, Clone, Default)]
pub struct PluginBuilder {
    inner: Arc<Mutex<PluginBuilderInner>>,
}

impl PluginBuilder {
    /// Creates a new empty builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets plugin name from Rhai.
    pub fn set_name(&mut self, name: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.name = name.trim().to_string();
        }
    }

    /// Gets plugin name.
    pub fn get_name(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.name.clone())
            .unwrap_or_default()
    }

    /// Sets plugin author from Rhai.
    pub fn set_author(&mut self, author: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.author = author.trim().to_string();
        }
    }

    /// Gets plugin author.
    pub fn get_author(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.author.clone())
            .unwrap_or_default()
    }

    /// Sets plugin version from Rhai.
    pub fn set_version(&mut self, version: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.version = version.trim().to_string();
        }
    }

    /// Gets plugin version.
    pub fn get_version(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.version.clone())
            .unwrap_or_default()
    }

    /// Sets plugin description from Rhai.
    pub fn set_description(&mut self, description: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.description = description.trim().to_string();
        }
    }

    /// Gets plugin description.
    pub fn get_description(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.description.clone())
            .unwrap_or_default()
    }

    /// Registers a custom header icon button.
    pub fn add_header_button(
        &mut self,
        id: String,
        icon: String,
        tooltip: String,
        position: String,
    ) {
        let id = id.trim().to_string();
        if id.is_empty() {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            inner.header_buttons.retain(|b| b.id != id);
            inner.header_buttons.push(PluginHeaderButton {
                id,
                icon,
                tooltip,
                position: HeaderButtonPosition::from_str_loose(&position),
            });
        }
    }

    /// Registers a global shortcut keybinding.
    pub fn add_shortcut(&mut self, action_id: String, key_combination: String) {
        let action_id = action_id.trim().to_string();
        if action_id.is_empty() {
            return;
        }
        let key_comb = key_combination.trim().to_string();
        if let Ok(mut inner) = self.inner.lock() {
            inner.shortcuts.retain(|s| s.action_id != action_id);
            inner.shortcuts.push(PluginShortcut {
                action_id: action_id.clone(),
                key_combination: key_comb,
                label: action_id,
            });
        }
    }

    /// Registers a global shortcut keybinding with a descriptive display label.
    pub fn add_shortcut_with_label(
        &mut self,
        action_id: String,
        key_combination: String,
        label: String,
    ) {
        let action_id = action_id.trim().to_string();
        if action_id.is_empty() {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            inner.shortcuts.retain(|s| s.action_id != action_id);
            inner.shortcuts.push(PluginShortcut {
                action_id,
                key_combination: key_combination.trim().to_string(),
                label: label.trim().to_string(),
            });
        }
    }

    /// Registers an editor right-click context menu item.
    pub fn add_context_menu_item(&mut self, action_id: String, label: String) {
        let action_id = action_id.trim().to_string();
        if action_id.is_empty() {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            inner.menu_items.retain(|m| m.action_id != action_id);
            inner.menu_items.push(PluginMenuItem {
                action_id,
                label: label.trim().to_string(),
                icon: None,
            });
        }
    }

    /// Registers an editor right-click context menu item with an icon glyph.
    pub fn add_context_menu_item_with_icon(
        &mut self,
        action_id: String,
        label: String,
        icon: String,
    ) {
        let action_id = action_id.trim().to_string();
        if action_id.is_empty() {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            inner.menu_items.retain(|m| m.action_id != action_id);
            inner.menu_items.push(PluginMenuItem {
                action_id,
                label: label.trim().to_string(),
                icon: Some(icon.trim().to_string()),
            });
        }
    }

    /// Registers a periodic background timer in seconds.
    pub fn add_timer(&mut self, id: String, interval_seconds: i64) {
        let id = id.trim().to_string();
        if id.is_empty() || interval_seconds <= 0 {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            inner.timers.retain(|t| t.id != id);
            inner.timers.push(crate::plugins::types::PluginTimer {
                id,
                interval_seconds: interval_seconds.max(1) as u64,
            });
        }
    }

    /// Converts builder into a populated `PluginMetadata` instance.
    pub fn to_metadata(&self, id: String, file_path: Option<String>) -> PluginMetadata {
        let inner = self.inner.lock().unwrap();
        PluginMetadata {
            id: if inner.name.is_empty() {
                id
            } else {
                inner.name.to_lowercase().replace(' ', "_")
            },
            name: if inner.name.is_empty() {
                "Unnamed Plugin".to_string()
            } else {
                inner.name.clone()
            },
            version: if inner.version.is_empty() {
                "1.0.0".to_string()
            } else {
                inner.version.clone()
            },
            author: inner.author.clone(),
            description: inner.description.clone(),
            enabled: true,
            file_path,
        }
    }

    /// Clones registered header buttons.
    pub fn header_buttons(&self) -> Vec<PluginHeaderButton> {
        self.inner
            .lock()
            .map(|i| i.header_buttons.clone())
            .unwrap_or_default()
    }

    /// Clones registered shortcuts.
    pub fn shortcuts(&self) -> Vec<PluginShortcut> {
        self.inner
            .lock()
            .map(|i| i.shortcuts.clone())
            .unwrap_or_default()
    }

    /// Clones registered context menu items.
    pub fn menu_items(&self) -> Vec<PluginMenuItem> {
        self.inner
            .lock()
            .map(|i| i.menu_items.clone())
            .unwrap_or_default()
    }

    /// Clones registered background timers.
    pub fn timers(&self) -> Vec<crate::plugins::types::PluginTimer> {
        self.inner
            .lock()
            .map(|i| i.timers.clone())
            .unwrap_or_default()
    }
}
