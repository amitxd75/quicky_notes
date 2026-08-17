// Quote of the Day Plugin for Quicky Notes
// Fetches random programming quotes and wisdom via HTTP REST API.

fn init(plugin) {
    plugin.name = "Daily Wisdom Quotes";
    plugin.author = "Quicky Community";
    plugin.version = "1.0.0";
    plugin.description = "Fetches programming wisdom, tech quotes, and jokes over HTTP with console panel output";
    
    // Register header buttons
    plugin.add_header_button("fetch_quote", "💡", "Fetch Daily Programming Quote", "right");
    
    // Register context menu items
    plugin.add_context_menu_item_with_icon("quote_insert", "Insert Programming Quote at Cursor", "📝");
    plugin.add_context_menu_item_with_icon("quote_panel", "View Quote in Bottom Panel", "📟");
    
    // Register global shortcut
    plugin.add_shortcut_with_label("quote_fetch", "Ctrl+Alt+Q", "Fetch Daily Quote");
    
    return plugin;
}

fn on_header_click(button_id, note, ui, system, storage, http, theme) {
    if button_id == "fetch_quote" {
        fetch_and_show_quote(ui, http, storage);
    }
}

fn on_shortcut(action_id, note, ui, system, storage, http, theme) {
    if action_id == "quote_fetch" {
        fetch_and_show_quote(ui, http, storage);
    }
}

fn on_context_menu_click(action_id, note, ui, system, storage, http, theme) {
    if action_id == "quote_insert" {
        let quote = get_fresh_quote(http, storage);
        if !quote.is_empty() {
            note.insert_at_cursor(`\n> "${quote}"\n`);
            ui.toast_success("Quote inserted into note!");
        } else {
            ui.toast_error("Failed to retrieve quote over HTTP.");
        }
    } else if action_id == "quote_panel" {
        fetch_and_show_quote(ui, http, storage);
    }
}

fn fetch_and_show_quote(ui, http, storage) {
    ui.toast("Fetching quote over HTTP...");
    let quote = get_fresh_quote(http, storage);
    if !quote.is_empty() {
        ui.show_panel("Daily Programming Wisdom", `💡 Inspiring Developer Wisdom\n----------------------------------------\n\n"${quote}"\n\n[Copied to clipboard / ready to paste]`);
        ui.copy_to_clipboard(`"${quote}"`);
        ui.toast_success("Quote loaded!");
    } else {
        let err = http.last_error();
        if !err.is_empty() {
            ui.toast_error(`HTTP Error: ${err}`);
        } else {
            ui.toast_error("Could not fetch quote: check internet connection.");
        }
    }
}

fn get_fresh_quote(http, storage) {
    let url = "https://dummyjson.com/quotes/random";
    let body = http.get(url);
    if http.status_code() == 200 && !body.is_empty() {
        // Parse JSON response
        let data = parse_json(body);
        let quote_text = "";
        if data.contains("quote") && data.contains("author") {
            quote_text = `${data["quote"]} — ${data["author"]}`;
        } else {
            quote_text = body;
        }
        // Cache last quote in persistent storage
        storage.set("last_quote", quote_text);
        return quote_text;
    }
    
    // Fallback if offline
    if storage.has("last_quote") {
        return storage.get("last_quote");
    }
    return "Simplicity is prerequisite for reliability. — Edsger W. Dijkstra";
}
