# Quicky Notes Plugin Ecosystem & Developer Guide

Quicky Notes features a powerful, embedded **Rhai Scripting Plugin Engine**. Plugins allow you to customize and extend Quicky Notes with custom header buttons, global hotkeys, right-click context menu tools, note buffer transformations, clipboard automations, persistent storage, pure-Rust HTTP REST requests, recurring timers, dynamic theme color overrides, collapsible bottom console panels, and native terminal launchers.

---

## Table of Contents

1. [Quick Installation & Setup](#1-quick-installation--setup)
2. [Managing Plugins in Settings](#2-managing-plugins-in-settings)
3. [Plugin Architecture & Lifecycle Hooks](#3-plugin-architecture--lifecycle-hooks)
4. [Host API Reference](#4-host-api-reference)
   - [Registration API (`plugin`)](#registration-api-plugin)
   - [Note API (`note`)](#note-api-note)
   - [UI & Bottom Panel API (`ui`)](#ui--bottom-panel-api-ui)
   - [Persistent Key-Value Storage API (`storage`)](#persistent-key-value-storage-api-storage)
   - [HTTP REST Client API (`http`)](#http-rest-client-api-http)
   - [Theme & Accent API (`theme`)](#theme--accent-api-theme)
   - [System API (`system`)](#system-api-system)
5. [Complete Practical Examples](#5-complete-practical-examples)
   - [Example 1: Pomodoro Focus Timer with Interval Ticks & Console Drawer](#example-1-pomodoro-focus-timer)
   - [Example 2: Quote of the Day HTTP REST Fetcher](#example-2-quote-of-the-day-http-rest-fetcher)
   - [Example 3: Dynamic Theme Accent Cycler](#example-3-dynamic-theme-accent-cycler)
   - [Example 4: Quick Terminal Launcher](#example-4-quick-terminal-launcher)
6. [Security & Sandboxing Quotas](#6-security--sandboxing-quotas)

---

## 1. Quick Installation & Setup

### Plugin Directory Location

All user plugin scripts reside in your standard config folder:

```text
~/.config/quicky_notes/plugins/
```

### Installing a Plugin

To install any plugin:
1. Save your script file with a `.rhai` extension (e.g. `pomodoro_timer.rhai`) directly inside `~/.config/quicky_notes/plugins/`.
2. Or create a subdirectory containing `plugin.rhai` or `main.rhai`:
   ```text
   ~/.config/quicky_notes/plugins/
   ├── pomodoro_timer.rhai
   ├── quote_of_the_day.rhai
   ├── quick_terminal.rhai
   └── my_custom_tool/
       └── plugin.rhai
   ```
3. Open Quicky Notes (or click **🔄 Reload** in Settings) — your plugin is compiled and active immediately!

---

## 2. Managing Plugins in Settings

Open **Settings & Preferences** by clicking the gear icon or pressing <kbd>Ctrl</kbd> + <kbd>,</kbd>, then switch to the **`🔌 Plugins`** tab:

- **Master Toggle (`Enable Rhai Plugin System`)**: Enables or disables the entire plugin runtime.
- **📁 Open Folder**: Opens `~/.config/quicky_notes/plugins/` in your desktop file manager.
- **🔄 Reload**: Re-scans the directory and hot-reloads all scripts in memory without restarting.
- **✨ Starter Plugins**: Generates official starter templates.
- **Plugin Cards**:
  - Enable or disable individual plugins with dedicated toggle switches.
  - View author, version, and capability badges (registered buttons, shortcuts, timers, and context menu items).
  - Inspect compilation warnings and runtime error diagnostics.

---

## 3. Plugin Architecture & Lifecycle Hooks

Plugins are written in [Rhai](https://rhai.rs/), an embedded scripting language designed specifically for Rust.

### Lifecycle Hooks

Every plugin can implement any combination of the following entrypoints:

| Hook | Parameters | Description |
| :--- | :--- | :--- |
| `init(plugin)` | `plugin` | Registration entrypoint called once on load to configure metadata, header buttons, shortcuts, timers, and context menu entries. |
| `on_header_click(button_id, note, ui, system)` | `button_id, note, ui, system` | Triggered when the user clicks a custom header bar icon registered by this plugin. |
| `on_shortcut(action_id, note, ui, system)` | `action_id, note, ui, system` | Triggered when a registered keyboard shortcut combination is pressed. |
| `on_context_menu_click(action_id, note, ui, system)` | `action_id, note, ui, system` | Triggered when a custom right-click context menu item is selected in the editor. |
| `on_timer(timer_id, note, ui, system)` | `timer_id, note, ui, system` | Triggered periodically when a background timer interval elapses. |
| `on_save(note, ui, system)` | `note, ui, system` | Triggered whenever the active note is saved to disk or SQLite. |
| `on_note_change(note, ui, system)` | `note, ui, system` | Triggered whenever note text is modified. |

> [!NOTE]
> All hook functions have global access to `storage`, `http`, `theme`, `note`, `ui`, and `system` inside their scope.

---

## 4. Host API Reference

### Registration API (`plugin` inside `init`)

| Method / Property | Description |
| :--- | :--- |
| `plugin.name = "My Plugin"` | Sets the display name of the plugin. |
| `plugin.author = "Your Name"` | Sets the author string. |
| `plugin.version = "1.0.0"` | Sets the semantic version string. |
| `plugin.description = "..."` | Sets a description shown in the Plugin Manager. |
| `plugin.add_header_button(id, icon, tooltip, position)` | Registers a header icon button. `position` can be `"right"` or `"left"`. |
| `plugin.add_shortcut(action_id, key_comb)` | Registers a global hotkey (e.g. `"Ctrl+Shift+T"`, `"Ctrl+\``", `"Alt+F12"`). |
| `plugin.add_shortcut_with_label(action_id, key_comb, label)` | Registers a shortcut with a custom display label in Settings. |
| `plugin.add_context_menu_item(action_id, label)` | Registers an item in the right-click editor context menu. |
| `plugin.add_context_menu_item_with_icon(action_id, label, icon)` | Registers an item in the right-click menu with an icon prefix (e.g. `">_"`, `"▦"`). |
| `plugin.add_timer(timer_id, interval_seconds)` | Registers a recurring background timer in seconds (e.g. `plugin.add_timer("tick", 1)`). |

---

### Note API (`note`)

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

### UI & Bottom Panel API (`ui`)

| Method | Description |
| :--- | :--- |
| `ui.toast(message)` | Displays a modern floating info toast notification. |
| `ui.toast_success(message)` | Displays a green success toast notification. |
| `ui.toast_error(message)` | Displays a red error toast notification. |
| `ui.toast_warning(message)` | Displays an amber warning toast notification. |
| `ui.set_status(message)` | Updates the bottom status bar message. |
| `ui.copy_to_clipboard(text)` | Writes text to the system clipboard. |
| `ui.get_clipboard()` | Reads the current text from the system clipboard. |
| `ui.show_panel(title, content)` | Opens the bottom output console drawer with the given title and text. |
| `ui.append_panel(text)` | Appends text to the active bottom output console drawer. |
| `ui.set_panel_content(content)` | Replaces the content in the active bottom console drawer. |
| `ui.clear_panel()` | Clears the active bottom console drawer. |
| `ui.hide_panel()` | Closes and hides the bottom console drawer. |
| `ui.request_repaint()` | Forces an immediate egui frame redraw. |

---

### Persistent Key-Value Storage API (`storage`)

Each plugin gets its own isolated JSON data store saved automatically under `~/.config/quicky_notes/plugins_data/<plugin_id>.json`:

| Method | Return Type | Description |
| :--- | :--- | :--- |
| `storage.get(key)` | `String` | Gets a value by key (or empty string if not found). |
| `storage.set(key, value)` | `()` | Sets a key-value pair and persists immediately to disk. |
| `storage.has(key)` | `bool` | Checks if a key exists in storage. |
| `storage.delete(key)` | `bool` | Deletes a key from storage. Returns `true` if the key was present. |
| `storage.clear()` | `()` | Clears all stored data for this plugin. |
| `storage.keys()` | `Array` | Returns an array of all keys stored for this plugin. |
| `storage.all()` | `Map` | Returns a key-value dictionary of all stored data. |

---

### HTTP REST Client API (`http`)

Pure-Rust synchronous and background-ready HTTP client with 10s timeouts and a 5 MB response cap:

| Method | Return Type | Description |
| :--- | :--- | :--- |
| `http.get(url)` | `String` | Performs an HTTP GET request and returns the response body. |
| `http.get_with_headers(url, headers_map)` | `String` | Performs an HTTP GET request with custom headers dictionary. |
| `http.post(url, body)` | `String` | Performs an HTTP POST request with a raw text/JSON body. |
| `http.post_json(url, body, headers_map)` | `String` | Performs an HTTP POST request with custom headers. |
| `http.status_code()` | `i64` | Returns the HTTP status code of the last request (e.g. `200`, `404`). |
| `http.last_error()` | `String` | Returns the last HTTP error message, if any. |

---

### Theme & Accent API (`theme`)

| Method | Return Type | Description |
| :--- | :--- | :--- |
| `theme.get_accent()` | `String` | Returns the current accent color as a HEX string (e.g. `"#7AA2F7"`). |
| `theme.get_bg()` | `String` | Returns the current background color as a HEX string. |
| `theme.set_accent(hex_color)` | `()` | Dynamically sets a temporary accent color override (e.g. `"#00F0FF"`). |
| `theme.reset_accent()` | `()` | Resets the accent color back to the active theme palette. |
| `theme.is_dark()` | `bool` | Returns `true` if the active theme is a dark theme. |

---

### System API (`system`)

| Method | Return Type | Description |
| :--- | :--- | :--- |
| `system.launch_terminal(dir)` | `bool` | Auto-detects installed modern terminal emulators (`$TERMINAL`, `kitty`, `alacritty`, `foot`, `ghostty`, `wezterm`, `gnome-terminal`, `konsole`, `xfce4-terminal`, `xterm`) and spawns a window in `dir`. |
| `system.exec(command, args)` | `bool` | Spawns a non-blocking background child process (e.g. `system.exec("notify-send", ["Hi"])`). |
| `system.exec_sync(command, args)` | `String` | Executes a command synchronously and returns captured stdout (capped at 1 MB). |
| `system.open_folder(path)` | `()` | Safely opens the directory in the desktop default file manager (`xdg-open`). |
| `system.open_url(url)` | `()` | Opens an `http://` or `https://` URL in the default web browser. |
| `system.home_dir()` | `String` | Returns the user's home directory path (`/home/username`). |
| `system.parent_dir(path)` | `String` | Returns the parent directory containing the given file path. |

---

## 5. Complete Practical Examples

### Example 1: Pomodoro Focus Timer

File: `~/.config/quicky_notes/plugins/pomodoro_timer.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Pomodoro Timer";
    plugin.author = "Community";
    plugin.version = "1.0.0";
    plugin.description = "25-minute focus intervals and 5-minute breaks with timers and console drawer";
    
    plugin.add_header_button("pomo_start", "🍅", "Toggle Pomodoro Timer", "right");
    plugin.add_header_button("pomo_panel", "⏱️", "Toggle Pomodoro Output Panel", "right");
    plugin.add_shortcut_with_label("pomo_toggle", "Ctrl+Alt+P", "Toggle Pomodoro Timer");
    plugin.add_context_menu_item_with_icon("pomo_reset", "Reset Pomodoro Timer", "🔄");

    // 1-second recurring background interval
    plugin.add_timer("pomo_tick", 1);
    return plugin;
}

fn on_header_click(button_id, note, ui, system) {
    if button_id == "pomo_start" { toggle_timer(ui); }
    else if button_id == "pomo_panel" { toggle_panel(ui); }
}

fn on_timer(timer_id, note, ui, system) {
    if timer_id != "pomo_tick" || storage.get("running") != "true" { return; }
    
    let remaining = storage.get("remaining").to_int();
    if remaining > 0 {
        remaining -= 1;
        storage.set("remaining", `${remaining}`);
        let mode = storage.get("mode");
        let mins = remaining / 60;
        let secs = remaining % 60;
        let sec_str = if secs < 10 { `0${secs}` } else { `${secs}` };
        ui.set_status(`🍅 [${mode}]: ${mins}:${sec_str}`);
        if storage.get("panel_open") == "true" {
            ui.set_panel_content(`=== Pomodoro Status ===\nPhase: ${mode}\nTime Left: ${mins}:${sec_str}\nStatus: Active Focusing`);
        }
    } else {
        let mode = storage.get("mode");
        if mode == "Work" {
            storage.set("mode", "Break");
            storage.set("remaining", "300");
            ui.toast_success("🎉 Work session completed! Take a 5-minute break.");
            system.exec("notify-send", ["Pomodoro", "Work session done! 5m break."]);
        } else {
            storage.set("mode", "Work");
            storage.set("remaining", "1500");
            ui.toast_success("🔔 Break finished! Ready to focus?");
            system.exec("notify-send", ["Pomodoro", "Break over! Back to work."]);
        }
    }
}

fn toggle_timer(ui) {
    let is_running = storage.get("running") == "true";
    if is_running {
        storage.set("running", "false");
        ui.toast_warning("⏸️ Pomodoro timer paused.");
        ui.set_status("Pomodoro: Paused");
    } else {
        if !storage.has("mode") {
            storage.set("mode", "Work");
            storage.set("remaining", "1500");
        }
        storage.set("running", "true");
        ui.toast_success("🍅 Pomodoro timer started (25m focus)!");
    }
}

fn toggle_panel(ui) {
    let is_open = storage.get("panel_open") == "true";
    if is_open {
        storage.set("panel_open", "false");
        ui.hide_panel();
    } else {
        storage.set("panel_open", "true");
        ui.show_panel("Pomodoro Focus Console", "=== Pomodoro Status ===\nPhase: Work\nTime Left: 25:00");
    }
}
```

---

### Example 2: Quote of the Day HTTP REST Fetcher

File: `~/.config/quicky_notes/plugins/quote_of_the_day.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Daily Wisdom Quotes";
    plugin.author = "Community";
    plugin.version = "1.0.0";
    plugin.description = "Fetches programming wisdom over HTTP with console panel output";
    
    plugin.add_header_button("fetch_quote", "💡", "Fetch Daily Programming Quote", "right");
    plugin.add_context_menu_item_with_icon("quote_insert", "Insert Programming Quote", "📝");
    return plugin;
}

fn on_header_click(button_id, note, ui, system) {
    if button_id == "fetch_quote" {
        ui.toast("Fetching quote over HTTP...");
        let body = http.get("https://dummyjson.com/quotes/random");
        if http.status_code() == 200 && !body.is_empty() {
            ui.show_panel("Daily Programming Wisdom", `💡 Developer Wisdom\n--------------------------------\n\n"${body}"`);
            ui.copy_to_clipboard(body);
            ui.toast_success("Quote loaded & copied!");
        } else {
            ui.toast_error("Failed to retrieve quote over HTTP.");
        }
    }
}
```

---

### Example 3: Dynamic Theme Accent Cycler

File: `~/.config/quicky_notes/plugins/custom_theme_cycler.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Theme Accent Cycler";
    plugin.author = "Community";
    plugin.version = "1.0.0";
    plugin.description = "Cycles UI accent highlights dynamically between Cyberpunk Cyan, Tokyo Purple, and Emerald";
    
    plugin.add_header_button("theme_cycle", "🎨", "Cycle Theme Accent Color", "left");
    plugin.add_shortcut_with_label("theme_cycle_key", "Ctrl+Alt+T", "Cycle Accent Color");
    return plugin;
}

fn on_header_click(button_id, note, ui, system) {
    if button_id == "theme_cycle" {
        let colors = ["#00F0FF", "#BB9AF7", "#2ECC71", "#FF007F", "#F59E0B"];
        let idx = if storage.has("idx") { storage.get("idx").to_int() } else { 0 };
        idx = (idx + 1) % colors.len();
        storage.set("idx", `${idx}`);
        theme.set_accent(colors[idx]);
        ui.toast_success(`🎨 Accent color updated to ${colors[idx]}`);
    }
}
```

---

### Example 4: Quick Terminal Launcher

File: `~/.config/quicky_notes/plugins/quick_terminal.rhai`

```rhai
fn init(plugin) {
    plugin.name = "Quick Terminal";
    plugin.author = "amitxd";
    plugin.version = "1.0.0";
    plugin.description = "Auto-detects and launches terminal emulator in note directory";

    plugin.add_header_button("btn_terminal", ">_", "Launch Terminal (Ctrl+`)", "right");
    plugin.add_shortcut("open_terminal", "Ctrl+`");
    plugin.add_context_menu_item_with_icon("open_terminal", "Open Terminal in Folder", ">_");
    return plugin;
}

fn on_header_click(button_id, note, ui, system, storage, http, theme) {
    if button_id == "btn_terminal" {
        launch_term(note, ui, system);
    }
}

fn on_shortcut(action_id, note, ui, system, storage, http, theme) {
    if action_id == "open_terminal" {
        launch_term(note, ui, system);
    }
}

fn on_context_menu_click(action_id, note, ui, system, storage, http, theme) {
    if action_id == "open_terminal" {
        launch_term(note, ui, system);
    }
}

fn launch_term(note, ui, system) {
    let fp = note.get_file_path();
    let target_dir = if fp != "" { system.parent_dir(fp) } else { system.home_dir() };
    let launched = system.launch_terminal(target_dir);
    if launched {
        ui.toast_success("Terminal launched in " + target_dir);
    } else {
        ui.toast_error("Could not find an installed terminal emulator.");
    }
}
```

---

## 6. Execution Model, Hardware Quotas & Security

To guarantee responsive UI performance and prevent runaway resource consumption, the Rhai runtime enforces strict execution limits:

1. **Max Operations (`500,000` steps)**: Prevents hanging script loops from blocking the UI thread.
2. **Max Call Depth (`50` frames)**: Protects against stack overflow from uncontrolled recursion.
3. **Max String Size (`5 MB`)**: Bounds heap allocations inside scripts.
4. **Max HTTP Payload (`5 MB`)**: Bounded streaming prevents oversized network responses from causing excessive memory usage.
5. **HTTP Timeout (`10 Seconds`)**: All network calls time out automatically.
6. **Synchronous Process Timeout (`5 Seconds`)**: All synchronous system commands time out and terminate automatically.
7. **Path Traversal Protection**: Folder opening, terminal spawning, and plugin persistent key-value storage enforce safe directory boundaries.
8. **Trust & Permissions**: Plugins execute with user privileges when invoking host system APIs (`system.exec`, `system.exec_sync`). Only install plugins from trusted sources.
