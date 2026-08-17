// Timestamp Inserter Plugin for Quicky Notes
// Inserts current ISO date and time into the editor.

fn init(plugin) {
    plugin.name = "Timestamp Inserter";
    plugin.author = "Community";
    plugin.version = "1.0.0";
    plugin.description = "Inserts current ISO timestamp at cursor position";

    plugin.add_shortcut_with_label("insert_timestamp", "Ctrl+Shift+D", "Insert Timestamp");
    plugin.add_context_menu_item_with_icon("insert_timestamp", "Insert Current Timestamp", "🕒");

    return plugin;
}

fn on_shortcut(action_id, note, ui, system) {
    if action_id == "insert_timestamp" {
        insert_time(note, ui, system);
    }
    return note;
}

fn on_context_menu_click(action_id, note, ui, system) {
    if action_id == "insert_timestamp" {
        insert_time(note, ui, system);
    }
    return note;
}

fn insert_time(note, ui, system) {
    let output = system.exec_sync("date", ["+%Y-%m-%d %H:%M:%S"]);
    let trimmed = output.trim();
    if trimmed != "" {
        note.insert_at_cursor(trimmed);
        ui.toast_success("Inserted timestamp ✓");
    }
}
