//! Default template plugins bundled with Quicky Notes for instant out-of-the-box utility.

/// Quick Terminal launcher plugin source code.
pub const QUICK_TERMINAL_TEMPLATE: &str = r#"// Quick Terminal Plugin for Quicky Notes
// Launches your favorite terminal in the current note directory.

fn init(plugin) {
    plugin.name = "Quick Terminal";
    plugin.author = "amitxd";
    plugin.version = "1.0.0";
    plugin.description = "Launches terminal emulator in note directory with header icon and shortcut";

    // 1. Add top header icon button
    plugin.add_header_button(
        "btn_terminal",
        ">_",
        "Launch Terminal (Ctrl+`)",
        "right"
    );

    // 2. Add global keyboard shortcut
    plugin.add_shortcut("open_terminal", "Ctrl+`");

    // 3. Add right-click context menu item
    plugin.add_context_menu_item_with_icon("open_terminal", "Open Terminal in Folder", ">_");

    return plugin;
}

fn on_header_click(button_id, note, ui, system) {
    if button_id == "btn_terminal" {
        launch_term(note, ui, system);
    }
    return note;
}

fn on_shortcut(action_id, note, ui, system) {
    if action_id == "open_terminal" {
        launch_term(note, ui, system);
    }
    return note;
}

fn on_context_menu_click(action_id, note, ui, system) {
    if action_id == "open_terminal" {
        launch_term(note, ui, system);
    }
    return note;
}

fn launch_term(note, ui, system) {
    let fp = note.get_file_path();
    let target_dir = if fp != "" {
        system.parent_dir(fp)
    } else {
        system.home_dir()
    };

    let launched = system.launch_terminal(target_dir);
    if launched {
        ui.toast_success("Terminal launched in " + target_dir);
    } else {
        ui.toast_error("Could not find an installed terminal (kitty, alacritty, foot, etc.)");
    }
}
"#;

/// Markdown Table Formatter plugin source code.
pub const MARKDOWN_TABLE_FORMATTER_TEMPLATE: &str = r#"// Markdown Table Formatter Plugin for Quicky Notes
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
"#;
