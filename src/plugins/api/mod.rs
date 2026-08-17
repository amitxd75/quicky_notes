//! API binding module registering Rust host types and functions into Rhai Engine.

pub mod builder;
pub mod http_api;
pub mod note_api;
pub mod storage_api;
pub mod system_api;
pub mod theme_api;
pub mod ui_api;

pub use builder::PluginBuilder;
pub use http_api::HttpHandle;
pub use note_api::NoteHandle;
pub use storage_api::StorageHandle;
pub use system_api::SystemHandle;
pub use theme_api::ThemeHandle;
pub use ui_api::UiHandle;

use rhai::Engine;

/// Registers all Quicky Notes plugin APIs and host objects into the given Rhai engine.
pub fn register_apis(engine: &mut Engine) {
    // ─── 1. PluginBuilder API ───
    engine.register_type_with_name::<PluginBuilder>("Plugin");
    engine.register_get_set("name", PluginBuilder::get_name, PluginBuilder::set_name);
    engine.register_get_set(
        "author",
        PluginBuilder::get_author,
        PluginBuilder::set_author,
    );
    engine.register_get_set(
        "version",
        PluginBuilder::get_version,
        PluginBuilder::set_version,
    );
    engine.register_get_set(
        "description",
        PluginBuilder::get_description,
        PluginBuilder::set_description,
    );
    engine.register_fn("add_header_button", PluginBuilder::add_header_button);
    engine.register_fn("add_shortcut", PluginBuilder::add_shortcut);
    engine.register_fn(
        "add_shortcut_with_label",
        PluginBuilder::add_shortcut_with_label,
    );
    engine.register_fn(
        "add_context_menu_item",
        PluginBuilder::add_context_menu_item,
    );
    engine.register_fn(
        "add_context_menu_item_with_icon",
        PluginBuilder::add_context_menu_item_with_icon,
    );
    engine.register_fn("add_timer", PluginBuilder::add_timer);

    // ─── 2. NoteHandle API ───
    engine.register_type_with_name::<NoteHandle>("Note");
    engine.register_fn("get_text", NoteHandle::get_text);
    engine.register_fn("set_text", NoteHandle::set_text);
    engine.register_fn("get_title", NoteHandle::get_title);
    engine.register_fn("set_title", NoteHandle::set_title);
    engine.register_fn("get_selection", NoteHandle::get_selection);
    engine.register_fn("replace_selection", NoteHandle::replace_selection);
    engine.register_fn("insert_at_cursor", NoteHandle::insert_at_cursor);
    engine.register_fn("get_file_path", NoteHandle::get_file_path);
    engine.register_fn("get_id", NoteHandle::get_id);
    engine.register_fn("is_markdown", NoteHandle::is_markdown);
    engine.register_fn("get_line_count", NoteHandle::get_line_count);
    engine.register_fn("get_word_count", NoteHandle::get_word_count);

    // ─── 3. UiHandle API ───
    engine.register_type_with_name::<UiHandle>("Ui");
    engine.register_fn("toast", UiHandle::toast);
    engine.register_fn("toast_success", UiHandle::toast_success);
    engine.register_fn("toast_error", UiHandle::toast_error);
    engine.register_fn("toast_warning", UiHandle::toast_warning);
    engine.register_fn("set_status", UiHandle::set_status);
    engine.register_fn("copy_to_clipboard", UiHandle::copy_to_clipboard);
    engine.register_fn("get_clipboard", UiHandle::get_clipboard);
    engine.register_fn("show_panel", UiHandle::show_panel);
    engine.register_fn("append_panel", UiHandle::append_panel);
    engine.register_fn("set_panel_content", UiHandle::set_panel_content);
    engine.register_fn("clear_panel", UiHandle::clear_panel);
    engine.register_fn("hide_panel", UiHandle::hide_panel);
    engine.register_fn("request_repaint", UiHandle::request_repaint);

    // ─── 4. SystemHandle API ───
    engine.register_type_with_name::<SystemHandle>("System");
    engine.register_fn("launch_terminal", SystemHandle::launch_terminal);
    engine.register_fn("exec", SystemHandle::exec);
    engine.register_fn("exec_sync", SystemHandle::exec_sync);
    engine.register_fn("open_folder", SystemHandle::open_folder);
    engine.register_fn("open_url", SystemHandle::open_url);
    engine.register_fn("home_dir", SystemHandle::home_dir);
    engine.register_fn("parent_dir", SystemHandle::parent_dir);

    // ─── 5. StorageHandle API ───
    engine.register_type_with_name::<StorageHandle>("Storage");
    engine.register_fn("get", StorageHandle::get);
    engine.register_fn("set", StorageHandle::set);
    engine.register_fn("has", StorageHandle::has);
    engine.register_fn("delete", StorageHandle::delete);
    engine.register_fn("clear", StorageHandle::clear);
    engine.register_fn("keys", StorageHandle::keys);
    engine.register_fn("all", StorageHandle::all);

    // ─── 6. HttpHandle API ───
    engine.register_type_with_name::<HttpHandle>("Http");
    engine.register_fn("get", HttpHandle::get);
    engine.register_fn("get_with_headers", HttpHandle::get_with_headers);
    engine.register_fn("post", HttpHandle::post);
    engine.register_fn("post_json", HttpHandle::post_json);
    engine.register_fn("status_code", HttpHandle::status_code);
    engine.register_fn("last_error", HttpHandle::last_error);

    // ─── 7. ThemeHandle API ───
    engine.register_type_with_name::<ThemeHandle>("Theme");
    engine.register_fn("get_accent", ThemeHandle::get_accent);
    engine.register_fn("get_bg", ThemeHandle::get_bg);
    engine.register_fn("set_accent", ThemeHandle::set_accent);
    engine.register_fn("reset_accent", ThemeHandle::reset_accent);
    engine.register_fn("is_dark", ThemeHandle::is_dark);

    // ─── 8. Global JSON Utilities ───
    engine.register_fn("parse_json", |json_str: &str| -> rhai::Dynamic {
        serde_json::from_str::<serde_json::Value>(json_str)
            .map(|v| json_to_dynamic(&v))
            .unwrap_or(rhai::Dynamic::UNIT)
    });
    engine.register_fn("to_json", |val: rhai::Dynamic| -> String {
        let json_val = dynamic_to_json(&val);
        serde_json::to_string(&json_val).unwrap_or_default()
    });

    // ─── 9. Numeric Conversion Helpers ───
    engine.register_fn("to_int", |s: &str| -> i64 {
        s.trim().parse::<i64>().unwrap_or(0)
    });
    engine.register_fn("to_int", |s: rhai::ImmutableString| -> i64 {
        s.trim().parse::<i64>().unwrap_or(0)
    });
    engine.register_fn("to_int", |s: String| -> i64 {
        s.trim().parse::<i64>().unwrap_or(0)
    });
    engine.register_fn("to_int", |n: i64| -> i64 { n });
    engine.register_fn("to_int", |f: f64| -> i64 { f as i64 });

    engine.register_fn("to_float", |s: &str| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });
    engine.register_fn("to_float", |s: rhai::ImmutableString| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });
    engine.register_fn("to_float", |s: String| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });
    engine.register_fn("to_float", |n: i64| -> f64 { n as f64 });
    engine.register_fn("to_float", |f: f64| -> f64 { f });

    engine.register_fn("parse_int", |s: &str| -> i64 {
        s.trim().parse::<i64>().unwrap_or(0)
    });
    engine.register_fn("parse_int", |s: rhai::ImmutableString| -> i64 {
        s.trim().parse::<i64>().unwrap_or(0)
    });
}

/// Converts a `serde_json::Value` into a `rhai::Dynamic` tree.
pub fn json_to_dynamic(value: &serde_json::Value) -> rhai::Dynamic {
    match value {
        serde_json::Value::Null => rhai::Dynamic::UNIT,
        serde_json::Value::Bool(b) => rhai::Dynamic::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rhai::Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                rhai::Dynamic::from(f)
            } else {
                rhai::Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => rhai::Dynamic::from(s.clone()),
        serde_json::Value::Array(arr) => {
            let list: rhai::Array = arr.iter().map(json_to_dynamic).collect();
            rhai::Dynamic::from(list)
        }
        serde_json::Value::Object(map) => {
            let mut rhai_map = rhai::Map::new();
            for (k, v) in map {
                rhai_map.insert(k.clone().into(), json_to_dynamic(v));
            }
            rhai::Dynamic::from(rhai_map)
        }
    }
}

/// Converts a `rhai::Dynamic` into a `serde_json::Value`.
pub fn dynamic_to_json(d: &rhai::Dynamic) -> serde_json::Value {
    if d.is_unit() {
        serde_json::Value::Null
    } else if let Some(b) = d.clone().try_cast::<bool>() {
        serde_json::Value::Bool(b)
    } else if let Some(i) = d.clone().try_cast::<i64>() {
        serde_json::Value::Number(i.into())
    } else if let Some(f) = d.clone().try_cast::<f64>() {
        serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null)
    } else if let Some(s) = d.clone().try_cast::<String>() {
        serde_json::Value::String(s)
    } else if let Some(arr) = d.clone().try_cast::<rhai::Array>() {
        serde_json::Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if let Some(map) = d.clone().try_cast::<rhai::Map>() {
        let mut obj = serde_json::Map::new();
        for (k, v) in map {
            obj.insert(k.to_string(), dynamic_to_json(&v));
        }
        serde_json::Value::Object(obj)
    } else {
        serde_json::Value::String(d.to_string())
    }
}
