//! API binding module registering Rust host types and functions into Rhai Engine.

pub mod builder;
pub mod note_api;
pub mod system_api;
pub mod ui_api;

pub use builder::PluginBuilder;
pub use note_api::NoteHandle;
pub use system_api::SystemHandle;
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
}
