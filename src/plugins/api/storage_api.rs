//! Persistent key-value storage API for plugins saved to ~/.config/quicky_notes/plugins_data/.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Validates and constructs a safe path within `storage_dir` for a plugin's data store.
pub fn safe_storage_path(storage_dir: &Path, plugin_id: &str) -> Option<PathBuf> {
    let trimmed = plugin_id.trim();
    if trimmed.is_empty()
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains("..")
        || trimmed.starts_with('.')
        || !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }

    let target = storage_dir.join(format!("{}.json", trimmed));
    if target.parent() == Some(storage_dir) {
        Some(target)
    } else {
        None
    }
}

/// Internal state for a plugin's key-value store.
#[derive(Debug, Clone, Default)]
pub struct StorageHandleInner {
    pub plugin_id: String,
    pub data: HashMap<String, String>,
    pub dirty: bool,
    pub storage_dir: PathBuf,
}

/// Storage handle proxy object exposed to Rhai scripts as `storage`.
#[derive(Debug, Clone, Default)]
pub struct StorageHandle {
    inner: Arc<Mutex<StorageHandleInner>>,
}

impl StorageHandle {
    /// Creates a new StorageHandle for a specific plugin ID, loading any existing JSON file from disk.
    pub fn new(plugin_id: &str) -> Self {
        let storage_dir = default_storage_dir();
        crate::storage::AppData::ensure_private_dir(&storage_dir);

        let data = if let Some(file_path) = safe_storage_path(&storage_dir, plugin_id) {
            if file_path.is_file() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    serde_json::from_str::<HashMap<String, String>>(&content).unwrap_or_default()
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        Self {
            inner: Arc::new(Mutex::new(StorageHandleInner {
                plugin_id: plugin_id.to_string(),
                data,
                dirty: false,
                storage_dir,
            })),
        }
    }

    /// Gets a value by key. Returns empty string if not found.
    pub fn get(&mut self, key: String) -> String {
        self.inner
            .lock()
            .map(|i| i.data.get(&key).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Sets a key-value pair and persists immediately to disk.
    pub fn set(&mut self, key: String, value: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.data.insert(key, value);
            inner.dirty = true;
            persist_storage(&inner);
        }
    }

    /// Returns true if the key exists in storage.
    pub fn has(&mut self, key: String) -> bool {
        self.inner
            .lock()
            .map(|i| i.data.contains_key(&key))
            .unwrap_or(false)
    }

    /// Deletes a key from storage. Returns true if the key was present.
    pub fn delete(&mut self, key: String) -> bool {
        if let Ok(mut inner) = self.inner.lock() {
            let removed = inner.data.remove(&key).is_some();
            if removed {
                inner.dirty = true;
                persist_storage(&inner);
            }
            removed
        } else {
            false
        }
    }

    /// Clears all keys for this plugin from storage and persists to disk.
    pub fn clear(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.data.clear();
            inner.dirty = true;
            persist_storage(&inner);
        }
    }

    /// Returns an array of all keys stored for this plugin.
    pub fn keys(&mut self) -> rhai::Array {
        self.inner
            .lock()
            .map(|i| {
                i.data
                    .keys()
                    .cloned()
                    .map(rhai::Dynamic::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    /// Returns a map of all key-value entries.
    pub fn all(&mut self) -> rhai::Map {
        let mut map = rhai::Map::new();
        if let Ok(inner) = self.inner.lock() {
            for (k, v) in &inner.data {
                map.insert(k.clone().into(), rhai::Dynamic::from(v.clone()));
            }
        }
        map
    }
}

/// Helper to persist plugin storage JSON atomically to disk.
fn persist_storage(inner: &StorageHandleInner) {
    if inner.plugin_id.is_empty() {
        return;
    }

    let Some(target_path) = safe_storage_path(&inner.storage_dir, &inner.plugin_id) else {
        return;
    };

    crate::storage::AppData::ensure_private_dir(&inner.storage_dir);

    if let Ok(json_str) = serde_json::to_string_pretty(&inner.data) {
        let clean_id: String = inner
            .plugin_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        let temp_path = inner
            .storage_dir
            .join(format!("{}.tmp.{}", clean_id, std::process::id()));

        #[cfg(unix)]
        let write_res = {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&temp_path)
                .and_then(|mut f| f.write_all(json_str.as_bytes()))
        };
        #[cfg(not(unix))]
        let write_res = fs::write(&temp_path, &json_str);

        if write_res.is_ok() {
            let _ = fs::rename(&temp_path, &target_path);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&target_path, fs::Permissions::from_mode(0o600));
            }
        }
    }
}

/// Computes the default directory for plugin persistent data storage.
fn default_storage_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "quicky", "quicky_notes")
        .map(|dirs| dirs.config_dir().join("plugins_data"))
        .unwrap_or_else(|| PathBuf::from(".config/quicky_notes/plugins_data"))
}
