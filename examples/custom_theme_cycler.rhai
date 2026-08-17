// Dynamic Theme Accent Cycler Plugin for Quicky Notes
// Cycles accent color highlights dynamically using the Theme API.

fn init(plugin) {
    plugin.name = "Theme Accent Cycler";
    plugin.author = "Quicky Community";
    plugin.version = "1.0.0";
    plugin.description = "Cycles UI accent highlights between Cyberpunk Neon, Tokyo Night, Dracula, and Emerald";
    
    // Register header buttons
    plugin.add_header_button("theme_cycle", "🎨", "Cycle Theme Accent Color", "left");
    
    // Register shortcuts
    plugin.add_shortcut_with_label("theme_cycle_key", "Ctrl+Alt+T", "Cycle Accent Color");
    
    // Register context menu items
    plugin.add_context_menu_item_with_icon("theme_neon_cyan", "Accent: Cyberpunk Cyan", "💎");
    plugin.add_context_menu_item_with_icon("theme_tokyo_purple", "Accent: Tokyo Purple", "🔮");
    plugin.add_context_menu_item_with_icon("theme_emerald_green", "Accent: Emerald Green", "🌿");
    plugin.add_context_menu_item_with_icon("theme_reset", "Reset Accent to System Theme", "🔄");
    
    return plugin;
}

fn on_header_click(button_id, note, ui, system, storage, http, theme) {
    if button_id == "theme_cycle" {
        cycle_next_accent(ui, theme, storage);
    }
}

fn on_shortcut(action_id, note, ui, system, storage, http, theme) {
    if action_id == "theme_cycle_key" {
        cycle_next_accent(ui, theme, storage);
    }
}

fn on_context_menu_click(action_id, note, ui, system, storage, http, theme) {
    if action_id == "theme_neon_cyan" {
        theme.set_accent("#00F0FF");
        ui.toast_success("Accent set to Cyberpunk Cyan!");
    } else if action_id == "theme_tokyo_purple" {
        theme.set_accent("#BB9AF7");
        ui.toast_success("Accent set to Tokyo Night Purple!");
    } else if action_id == "theme_emerald_green" {
        theme.set_accent("#2ECC71");
        ui.toast_success("Accent set to Emerald Green!");
    } else if action_id == "theme_reset" {
        theme.reset_accent();
        ui.toast("Accent reset to base theme palette.");
    }
}

fn cycle_next_accent(ui, theme, storage) {
    let colors = [
        "#00F0FF", // Cyan
        "#BB9AF7", // Purple
        "#2ECC71", // Emerald
        "#FF007F", // Rose Neon
        "#F59E0B"  // Amber Gold
    ];
    
    let current_idx = if storage.has("color_idx") { storage.get("color_idx").to_int() } else { 0 };
    current_idx = (current_idx + 1) % colors.len();
    storage.set("color_idx", `${current_idx}`);
    
    let next_color = colors[current_idx];
    theme.set_accent(next_color);
    ui.toast_success(`🎨 Accent color updated to ${next_color}`);
    ui.set_status(`Accent: ${next_color}`);
}
