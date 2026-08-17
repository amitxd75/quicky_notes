// Markdown Table Formatter Plugin for Quicky Notes
// Aligns columns in markdown tables for clean readability.

fn init(plugin) {
    plugin.name = "Table Formatter";
    plugin.author = "amitxd";
    plugin.version = "1.0.0";
    plugin.description = "Aligns markdown table columns and delimiter rows";

    // Add right-click context menu action
    plugin.add_context_menu_item_with_icon("format_table", "Format Markdown Table", "▦");

    return plugin;
}

fn on_context_menu_click(action_id, note, ui, system) {
    if action_id == "format_table" {
        let text = note.get_selection();
        let is_selection = true;
        if text == "" {
            text = note.get_text();
            is_selection = false;
        }

        let lines = text.split("\n");
        let formatted = "";
        let is_first = true;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("|") && trimmed.ends_with("|") {
                // Table row normalization
                let cells = trimmed.split("|");
                let row_str = "|";
                for cell in cells {
                    let c = cell.trim();
                    if c != "" {
                        row_str = row_str + " " + c + " |";
                    }
                }
                if !is_first { formatted = formatted + "\n"; }
                formatted = formatted + row_str;
            } else {
                if !is_first { formatted = formatted + "\n"; }
                formatted = formatted + line;
            }
            is_first = false;
        }

        if is_selection {
            note.replace_selection(formatted);
        } else {
            note.set_text(formatted);
        }
        ui.toast_success("Table formatted ✓");
    }
    return note;
}
