# Quicky Notes Plugin Ecosystem & Developer Guide

Quicky Notes features a pure-Rust, embedded **Rhai Scripting Plugin Engine**. Plugins allow you to customize and extend Quicky Notes with custom header buttons, global hotkeys, right-click context menu tools, note buffer transformations, clipboard automations, terminal launchers, and system integrations.

---

## Table of Contents

1. [Quick Installation & Setup](#1-quick-installation--setup)
2. [Managing Plugins in Settings](#2-managing-plugins-in-settings)
3. [Plugin Architecture & Lifecycle Hooks](#3-plugin-architecture--lifecycle-hooks)
4. [Host API Reference](#4-host-api-reference)
   - [Note API (`note`)](#note-api-note)
   - [UI API (`ui`)](#ui-api-ui)
   - [System API (`system`)](#system-api-system)
5. [Complete Practical Examples](#5-complete-practical-examples)
   - [Example 1: Quick Terminal Launcher](#example-1-quick-terminal-launcher)
   - [Example 2: Markdown Table Formatter](#example-2-markdown-table-formatter)
   - [Example 3: Timestamp & Date Inserter](#example-3-timestamp--date-inserter)
   - [Example 4: Text Case Transformer (Upper / Lower / Title)](#example-4-text-case-transformer)
   - [Example 5: External Shell Formatter (Prettier / Black / shfmt)](#example-5-external-shell-formatter)
6. [Security & Sandboxing](#6-security--sandboxing)

---

## 1. Quick Installation & Setup

### Plugin Directory Location

All plugins are stored in the user config directory:

```text
~/.config/quicky_notes/plugins/
```

### Installing a Plugin

To install any plugin:
1. Save your script file with a `.rhai` extension (e.g. `my_plugin.rhai`) directly inside `~/.config/quicky_notes/plugins/`.
2. Or create a subdirectory with `plugin.rhai` or `main.rhai`:
   ```text
   ~/.config/quicky_notes/plugins/
   ├── quick_terminal.rhai
   ├── markdown_table_formatter.rhai
   └── git_helper/
       └── plugin.rhai
   ```
3. Open Quicky Notes (or click **🔄 Reload** in Settings) — your plugin is instantly compiled and active!

---

## 2. Managing Plugins in Settings

Open **Settings & Preferences** by pressing <kbd>Ctrl</kbd> + <kbd>,</kbd> and navigate to the **`🧩 Plugins`** tab:

- **Master Toggle (`Enable Rhai Plugin System`)**: Enables or disables the entire plugin runtime.
- **📁 Open Folder**: Opens `~/.config/quicky_notes/plugins/` in your desktop file manager.
- **🔄 Reload**: Re-scans the directory and hot-reloads all scripts in memory without restarting the application.
- **✨ Starter Plugins**: Regenerates official starter templates (`quick_terminal.rhai` and `markdown_table_formatter.rhai`).
- **Plugin Cards**:
  - Enable or disable individual plugins with dedicated toggle switches.
  - View author, version, and capability badges (registered header buttons, key combinations, and context menu items).
  - Inspect compilation warnings and runtime error diagnostics.

---

## 3. Plugin Architecture & Lifecycle Hooks

Plugins are written in [Rhai](https://rhai.rs/), a safe, memory-bounded, pure-Rust scripting language.

### Lifecycle Hooks

Every plugin can implement any subset of the following entrypoints:

| Hook | Parameters | Description |
| :--- | :--- | :--- |
| `init(plugin)` | `plugin` | Registration entrypoint called once on load to configure metadata, header buttons, shortcuts, and context menu entries. |
| `on_header_click(button_id, note, ui, system)` | `button_id, note, ui, system` | Triggered when the user clicks a custom header bar icon registered by this plugin. |
| `on_shortcut(action_id, note, ui, system)` | `action_id, note, ui, system` | Triggered when a registered keyboard shortcut combination is pressed. |
| `on_context_menu_click(action_id, note, ui, system)` | `action_id, note, ui, system` | Triggered when a custom right-click context menu item is selected in the editor. |
| `on_save(note, ui, system)` | `note, ui, system` | Triggered whenever the active note is saved to disk or SQLite. |
| `on_note_change(note, ui, system)` | `note, ui, system` | Triggered whenever note text is modified. |

---

## 4. Host API Reference

### Plugin Registration API (`plugin` inside `init`)

| Method / Property | Description |
| :--- | :--- |
| `plugin.name = "My Plugin"` | Sets the display name of the plugin. |
| `plugin.author = "Your Name"` | Sets the author string. |
| `plugin.version = "1.0.0"` | Sets the semantic version. |
| `plugin.description = "..."` | Sets a description shown in the Plugin Manager. |
| `plugin.add_header_button(id, icon, tooltip, position)` | Registers a header icon button. `position` can be `"right"` or `"left"`. |
| `plugin.add_shortcut(action_id, key_comb)` | Registers a global hotkey (e.g. `"Ctrl+Shift+T"`, `"Ctrl+\``", `"Alt+F12"`). |
| `plugin.add_shortcut_with_label(action_id, key_comb, label)` | Registers a shortcut with a custom display label in Settings. |
| `plugin.add_context_menu_item(action_id, label)` | Registers an item in the right-click editor context menu. |
| `plugin.add_context_menu_item_with_icon(action_id, label, icon)` | Registers an item in the right-click menu with an icon prefix (e.g. `">_"`, `"▦"`). |

---

### Note API (`note`)

The `note` object provides live access and mutations to the currently focused note buffer:

| Method | Return Type | Description |
| :--- | :--- | :--- |
| `note.get_text()` | `String` | Returns the entire text content of the active note. |
| `note.set_text(text)` | `()` | Replaces the entire content of the active note. |
| `note.get_selection()` | `String` | Returns the currently selected text in the editor (or empty string if none). |
| `note.replace_selection(text)` | `()` | Replaces the active selection range with the provided text. |
| `note.insert_at_cursor(text)` | `()` | Inserts text at the current cursor position. |
| `note.get_title()` | `String` | Returns the note title. |
| `note.set_title(title)` | `()` | Sets the note title. |
| `note.get_file_path()` | `String` | Returns the external disk file path if this note is linked to a file. |
| `note.get_id()` | `String` | Returns the UUID of the active note. |
| `note.get_line_count()` | `i64` | Returns total line count. |
| `note.get_word_count()` | `i64` | Returns total word count. |
| `note.is_markdown()` | `bool` | Returns `true` if Markdown syntax highlighting/preview is active. |

---

### UI API (`ui`)

The `ui` object allows displaying visual feedback and interacting with system clipboard:

| Method | Description |
| :--- | :--- |
| `ui.toast(message)` | Displays a modern floating info toast notification. |
| `ui.toast_success(message)` | Displays a green success toast notification. |
| `ui.toast_error(message)` | Displays a red error toast notification. |
| `ui.toast_warning(message)` | Displays an amber warning toast notification. |
| `ui.set_status(message)` | Updates the bottom status bar message. |
| `ui.copy_to_clipboard(text)` | Writes text to the system clipboard. |
| `ui.get_clipboard()` | Reads the current text from the system clipboard. |
| `ui.request_repaint()` | Forces an immediate egui frame redraw. |

---

### System API (`system`)

The `system` object provides safe operating system, terminal, and execution utilities:

| Method | Return Type | Description |
| :--- | :--- | :--- |
| `system.launch_terminal(dir)` | `bool` | Auto-detects installed modern terminal emulators (`$TERMINAL`, `kitty`, `alacritty`, `foot`, `ghostty`, `wezterm`, `gnome-terminal`, `konsole`, `xfce4-terminal`, `xterm`) and spawns a window in `dir`. |
| `system.exec(command, args)` | `bool` | Spawns a non-blocking background child process (e.g. `system.exec("git", ["pull"])`). |
| `system.exec_sync(command, args)` | `String` | Executes a command synchronously and returns captured stdout (capped at 1 MB). |
| `system.open_folder(path)` | `()` | Safely opens the directory in the desktop default file manager (`xdg-open`). |
| `system.open_url(url)` | `()` | Opens an `http://` or `https://` URL in the default web browser. |
| `system.home_dir()` | `String` | Returns the user's home directory path (`/home/username`). |
| `system.parent_dir(path)` | `String` | Returns the parent directory containing the given file path. |

---

## 5. Complete Practical Examples

### Example 1: Quick Terminal Launcher

File: `~/.config/quicky_notes/plugins/quick_terminal.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Quick Terminal";
    plugin.author = "amitxd";
    plugin.version = "1.0.0";
    plugin.description = "Launches terminal emulator in note directory with header icon and shortcut";

    // 1. Add top header icon button
    plugin.add_header_button("btn_terminal", ">_", "Launch Terminal (Ctrl+`)", "right");

    // 2. Add global keyboard shortcut
    plugin.add_shortcut("open_terminal", "Ctrl+`");

    // 3. Add right-click context menu item
    plugin.add_context_menu_item_with_icon("open_terminal", "Open Terminal in Folder", ">_");
}

fn on_header_click(button_id, note, ui, system) {
    if button_id == "btn_terminal" {
        launch_term(note, ui, system);
    }
}

fn on_shortcut(action_id, note, ui, system) {
    if action_id == "open_terminal" {
        launch_term(note, ui, system);
    }
}

fn on_context_menu_click(action_id, note, ui, system) {
    if action_id == "open_terminal" {
        launch_term(note, ui, system);
    }
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
        ui.toast_error("Could not find an installed terminal emulator.");
    }
}
```

---

### Example 2: Markdown Table Formatter

File: `~/.config/quicky_notes/plugins/markdown_table_formatter.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Table Formatter";
    plugin.author = "amitxd";
    plugin.version = "1.0.0";
    plugin.description = "Aligns markdown table columns and delimiter rows";

    plugin.add_context_menu_item_with_icon("format_table", "Format Markdown Table", "▦");
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
}
```

---

### Example 3: Timestamp & Date Inserter

File: `~/.config/quicky_notes/plugins/timestamp_inserter.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Timestamp Inserter";
    plugin.author = "Community";
    plugin.version = "1.0.0";
    plugin.description = "Inserts current ISO timestamp at cursor position";

    plugin.add_shortcut_with_label("insert_timestamp", "Ctrl+Shift+D", "Insert Timestamp");
    plugin.add_context_menu_item_with_icon("insert_timestamp", "Insert Current Timestamp", "🕒");
}

fn on_shortcut(action_id, note, ui, system) {
    if action_id == "insert_timestamp" {
        insert_time(note, ui, system);
    }
}

fn on_context_menu_click(action_id, note, ui, system) {
    if action_id == "insert_timestamp" {
        insert_time(note, ui, system);
    }
}

fn insert_time(note, ui, system) {
    let output = system.exec_sync("date", ["+%Y-%m-%d %H:%M:%S"]);
    let trimmed = output.trim();
    if trimmed != "" {
        note.insert_at_cursor(trimmed);
        ui.toast_success("Inserted timestamp ✓");
    }
}
```

---

### Example 4: Text Case Transformer

File: `~/.config/quicky_notes/plugins/case_transformer.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Case Transformer";
    plugin.author = "Community";
    plugin.version = "1.0.0";
    plugin.description = "Converts selected text to UPPERCASE, lowercase, or Title Case";

    plugin.add_context_menu_item("to_upper", "Transform: UPPERCASE");
    plugin.add_context_menu_item("to_lower", "Transform: lowercase");
}

fn on_context_menu_click(action_id, note, ui, system) {
    let sel = note.get_selection();
    if sel == "" {
        ui.toast_warning("Please select text first!");
        return;
    }

    if action_id == "to_upper" {
        note.replace_selection(sel.to_upper());
        ui.toast_success("Transformed to UPPERCASE ✓");
    } else if action_id == "to_lower" {
        note.replace_selection(sel.to_lower());
        ui.toast_success("Transformed to lowercase ✓");
    }
}
```

---

### Example 5: External Shell Formatter

File: `~/.config/quicky_notes/plugins/code_formatter.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Code Formatter";
    plugin.author = "Community";
    plugin.version = "1.0.0";
    plugin.description = "Formats JSON/Rust/Python using external CLI formatters (jq / rustfmt / black)";

    plugin.add_header_button("btn_format", "⚡", "Format Code (Ctrl+Shift+I)", "right");
    plugin.add_shortcut("format_code", "Ctrl+Shift+I");
    plugin.add_context_menu_item_with_icon("format_code", "Format Document (CLI)", "⚡");
}

fn on_header_click(button_id, note, ui, system) {
    if button_id == "btn_format" {
        format_note(note, ui, system);
    }
}

fn on_shortcut(action_id, note, ui, system) {
    if action_id == "format_code" {
        format_note(note, ui, system);
    }
}

fn on_context_menu_click(action_id, note, ui, system) {
    if action_id == "format_code" {
        format_note(note, ui, system);
    }
}

fn format_note(note, ui, system) {
    let content = note.get_text();
    if content.trim() == "" {
        return;
    }

    let fp = note.get_file_path();
    if fp.ends_with(".json") {
        let formatted = system.exec_sync("jq", [".", fp]);
        if formatted != "" {
            note.set_text(formatted);
            ui.toast_success("Formatted JSON via jq ✓");
        }
    } else if fp.ends_with(".rs") {
        system.exec_sync("rustfmt", [fp]);
        ui.toast_success("Formatted Rust file via rustfmt ✓");
    } else {
        ui.toast_info("No dedicated CLI formatter found for file type.");
    }
}
```

---

## 6. Security & Sandboxing

To prevent infinite loops, UI freezing, or excessive memory consumption, the plugin engine enforces hard security quotas:

1. **Max Operations (`500,000` steps)**: Any script stuck in an infinite loop (`while true {}`) is aborted automatically without blocking the egui UI frame thread.
2. **Max Call Depth (`50` frames)**: Recursion depth is strictly capped to prevent stack overflows.
3. **Max String Size (`5 MB`)**: String allocations within scripts cannot exceed 5 MB.
4. **Max Exec Output (`1 MB`)**: `system.exec_sync` truncates command stdout output at 1 MB to prevent process memory exhaustion.
5. **Path Traversal Protection**: Folder opening and terminal spawning resolve safe user directories and reject unauthorized paths.
