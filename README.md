# quicky_notes

A small floating note-taking widget for Hyprland and Wayland.

I built this because I wanted a lightweight popup note widget that matches my desktop wallpaper colors and lets me quickly throw ideas down or drag text files into tabs.

![Quicky Notes](./assets/screenshot-main.png)

## Features

- Auto-syncs with Pywal & Caelestia wallpaper colors
- Multi-provider AI Copilot & Fixer (`Ctrl+Enter`) supporting Gemini, OpenAI, Claude, DeepSeek, Groq, Ollama & OpenRouter
- Real-time language syntax highlighting for code and markdown (`.rs`, `.py`, `.js`, `.ts`, `.json`, `.toml`, `.yaml`, `.c`, `.cpp`, `.sh`, etc.)
- IDE-style right-click context menu (Cut, Copy, Paste, AI Copilot, Search, Save)
- Direct external file editing & disk syncing (edits update files at their source location)
- Drag and drop text files or snippets directly from Dolphin / file managers
- Live Markdown split preview and rendered view (`Ctrl+P`)
- Custom system fonts and code monospace mode
- Tab pinning, double-click renaming, and search (`Ctrl+K`)
- Real-time glassmorphism styling (opacity, corner rounding, custom theme colors)
- Safe atomic disk writes, crash logs, and corrupt config recovery
- Interactive bottom status bar with linked file paths, sync state, and word/char/line count

![Settings](./assets/screenshot-settings.png)

## Supported Platforms

- Compositors: Hyprland, Sway, River, Wayfire (Wayland)
- File Managers: Dolphin, Nautilus, Thunar, PCManFM, Nemo
- OS: Arch Linux, Fedora, Ubuntu/Debian, NixOS, Void, Alpine

## Keybinds

- Super + N : Toggle window
- Ctrl + N : New tab
- Ctrl + W : Close active tab
- Ctrl + S : Save all notes / sync linked files to disk
- Ctrl + Enter : AI Copilot & Fixer (Selection / Cursor)
- Ctrl + K : Search notes
- Ctrl + , : Open settings & preferences
- Ctrl + P : Toggle Markdown preview mode (Edit / Split / Preview)
- Ctrl + Shift + E : Export active note
- Ctrl + 1..9 / 0 : Switch to tab index
- Ctrl + Tab / Ctrl + Shift + Tab : Next / Previous tab
- Ctrl + + / - : Adjust font size

## Building

Requires Rust and fontconfig.

```bash
cargo install --path .
```

Add to your `hyprland.lua`:

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
