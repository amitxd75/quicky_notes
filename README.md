# quicky_notes

A small floating note-taking widget for Hyprland and Wayland.

I built this because I wanted a lightweight popup note widget that matches my desktop wallpaper colors and lets me quickly throw ideas down or drag text files into tabs.

![Quicky Notes](./assets/screenshot-main.png)

## Highlights

- **Real-Time Wallpaper Color Sync**: Auto-detects Pywal & Caelestia scheme updates and adapts palette accents dynamically.
- **Dolphin Drag & Drop Payload Parser**: Accepts raw byte payloads, percent-encoded URIs, and code snippets dropped from KDE Dolphin / GTK file managers.
- **Thread-Safe System Font Discovery**: Queries `fontconfig` and `fc-list` asynchronously on startup with zero UI main-thread blocking.
- **Chunked 100k+ Line Rendering**: Bypasses GPU mesh vertex clipping to handle giant files smoothly without ANR or frame freezes.
- **Wayland Window Geometry Memory**: Saves window dimensions and restore presets cleanly across desktop sessions.
- **Modal Close Confirmation**: Modal overlay intercepting tab close actions on unsaved files.

![Settings](./assets/screenshot-settings.png)

## Supported Platforms

- Compositors: Hyprland, Sway, River, Wayfire (Wayland)
- File Managers: Dolphin, Nautilus, Thunar, PCManFM, Nemo
- OS: Arch Linux, Fedora, Ubuntu/Debian, NixOS, Void, Alpine

## Keybinds

- Super + N : Toggle window
- Ctrl + N : New tab
- Ctrl + W : Close tab
- Ctrl + S : Save
- Ctrl + K : Search notes
- Ctrl + , : Open settings
- Ctrl + 1..9 : Switch tabs
- Ctrl + + / - : Adjust font size

## Building

Requires Rust and fontconfig.

```bash
cargo install --path .
```

Add to your `hyprland.lua` (or `keybinds.lua`):

```lua
-- Keybind
hl.bind("SUPER + N", hl.dsp.exec_cmd("quicky_notes"), { desc = "Toggle Quicky Notes" })

-- Window Rule
hl.rule({
    class = "quicky_notes",
    float = true,
    center = true,
    opacity = 0.85,
})
```

## License

MIT
