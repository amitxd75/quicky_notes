// Text Case Transformer Plugin for Quicky Notes
// Converts selected text to UPPERCASE or lowercase.

fn init(plugin) {
    plugin.name = "Case Transformer";
    plugin.author = "Community";
    plugin.version = "1.0.0";
    plugin.description = "Converts selected text to UPPERCASE or lowercase";

    plugin.add_context_menu_item("to_upper", "Transform: UPPERCASE");
    plugin.add_context_menu_item("to_lower", "Transform: lowercase");

    return plugin;
}

fn on_context_menu_click(action_id, note, ui, system) {
    let sel = note.get_selection();
    if sel == "" {
        ui.toast_warning("Please select text first!");
        return note;
    }

    if action_id == "to_upper" {
        note.replace_selection(sel.to_upper());
        ui.toast_success("Transformed to UPPERCASE ✓");
    } else if action_id == "to_lower" {
        note.replace_selection(sel.to_lower());
        ui.toast_success("Transformed to lowercase ✓");
    }
    return note;
}
