// Pomodoro Focus Timer Plugin for Quicky Notes
// Uses background timers, bottom panel drawer, toasts, and persistent storage.

fn init(plugin) {
    plugin.name = "Pomodoro Timer";
    plugin.author = "Quicky Community";
    plugin.version = "1.0.0";
    plugin.description = "25-minute focus intervals and 5-minute break cycles with audio/visual reminders";
    
    // Register header buttons to start, pause, and check timer
    plugin.add_header_button("pomo_start", "🍅", "Start / Toggle Pomodoro Timer", "right");
    plugin.add_header_button("pomo_panel", "⏱️", "Toggle Pomodoro Output Panel", "right");
    
    // Register global shortcuts
    plugin.add_shortcut_with_label("pomo_toggle", "Ctrl+Alt+P", "Toggle Pomodoro Timer");
    
    // Register context menu items
    plugin.add_context_menu_item_with_icon("pomo_reset", "Reset Pomodoro Timer", "🔄");

    // Register a 1-second recurring background interval timer
    plugin.add_timer("pomo_tick", 1);
    
    return plugin;
}

fn on_header_click(button_id, note, ui, system, storage, http, theme) {
    if button_id == "pomo_start" {
        toggle_timer(ui, storage);
    } else if button_id == "pomo_panel" {
        toggle_panel(ui, storage);
    }
}

fn on_shortcut(action_id, note, ui, system, storage, http, theme) {
    if action_id == "pomo_toggle" {
        toggle_timer(ui, storage);
    }
}

fn on_context_menu_click(action_id, note, ui, system, storage, http, theme) {
    if action_id == "pomo_reset" {
        reset_timer(ui, storage);
    }
}

// Background timer event hook executed on every tick
fn on_timer(timer_id, note, ui, system, storage, http, theme) {
    if timer_id != "pomo_tick" {
        return;
    }
    
    let is_running = storage.get("running") == "true";
    if !is_running {
        return;
    }
    
    let remaining = storage.get("remaining").to_int();
    if remaining > 0 {
        remaining -= 1;
        storage.set("remaining", `${remaining}`);
        
        let mode = storage.get("mode");
        let mins = remaining / 60;
        let secs = remaining % 60;
        let sec_str = if secs < 10 { `0${secs}` } else { `${secs}` };
        let status = `🍅 [${mode}]: ${mins}:${sec_str}`;
        
        ui.set_status(status);
        if storage.get("panel_open") == "true" {
            ui.set_panel_content(`=== Pomodoro Status ===\nPhase: ${mode}\nTime Left: ${mins}:${sec_str}\nStatus: Active Focusing`);
        }
    } else {
        // Cycle completed!
        let mode = storage.get("mode");
        if mode == "Work" {
            storage.set("mode", "Break");
            storage.set("remaining", "300"); // 5 minutes
            ui.toast_success("🎉 Work session completed! Take a 5-minute break.");
            system.exec("notify-send", ["Pomodoro", "Focus session finished! Time for a 5 min break."]);
        } else {
            storage.set("mode", "Work");
            storage.set("remaining", "1500"); // 25 minutes
            ui.toast_success("🔔 Break finished! Ready to focus for 25 minutes?");
            system.exec("notify-send", ["Pomodoro", "Break over! Back to work."]);
        }
    }
}

fn toggle_timer(ui, storage) {
    let is_running = storage.get("running") == "true";
    if is_running {
        storage.set("running", "false");
        ui.toast_warning("⏸️ Pomodoro timer paused.");
        ui.set_status("Pomodoro: Paused");
    } else {
        if !storage.has("mode") {
            storage.set("mode", "Work");
            storage.set("remaining", "1500"); // 25 minutes
        }
        storage.set("running", "true");
        ui.toast_success("🍅 Pomodoro timer started (25m focus)!");
    }
}

fn reset_timer(ui, storage) {
    storage.set("running", "false");
    storage.set("mode", "Work");
    storage.set("remaining", "1500");
    ui.toast("🔄 Pomodoro timer reset to 25:00.");
    ui.set_status("Pomodoro: Ready");
    if storage.get("panel_open") == "true" {
        ui.set_panel_content("=== Pomodoro Status ===\nPhase: Work\nTime Left: 25:00\nStatus: Idle");
    }
}

fn toggle_panel(ui, storage) {
    let is_open = storage.get("panel_open") == "true";
    if is_open {
        storage.set("panel_open", "false");
        ui.hide_panel();
    } else {
        storage.set("panel_open", "true");
        let mode = if storage.has("mode") { storage.get("mode") } else { "Work" };
        let remaining = if storage.has("remaining") { storage.get("remaining").to_int() } else { 1500 };
        let mins = remaining / 60;
        let secs = remaining % 60;
        let sec_str = if secs < 10 { `0${secs}` } else { `${secs}` };
        ui.show_panel("Pomodoro Focus Console", `=== Pomodoro Status ===\nPhase: ${mode}\nTime Left: ${mins}:${sec_str}\nShortcuts: Ctrl+Alt+P to toggle`);
    }
}
