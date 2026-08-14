# quicky_notes

A small floating note-taking widget for Hyprland and Wayland.

I built this because I wanted a lightweight popup note widget that matches my desktop wallpaper colors and lets me quickly throw ideas down or drag text files into tabs.

![Quicky Notes](./assets/screenshot-main.png)

## Features

- Dynamic wallpaper color auto-sync (Pywal and Caelestia)
- 8 built-in color themes (Violet, Emerald, Frost, Pink, OLED, etc.)
- Drag and drop text files or text snippets from Dolphin / file managers
- System font selector (FiraCode, JetBrains Mono, Inter, etc.)
- Tab pinning and renaming
- Hyprland window resize presets (saves window size on exit)
- Full-text note search (Ctrl+K)
- Unsaved tab close confirmation dialog

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
