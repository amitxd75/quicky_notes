# quicky_notes

A lightweight floating glassmorphism note-taking widget for Hyprland and Wayland desktops.

![Quicky Notes](./assets/screenshot-main.png)

## Features

- Wallpaper Color Auto-Sync (Pywal and Caelestia)
- Native Statistical Autocomplete ([Architecture Docs](docs/suggestion_engine_architecture.md))
- Multi-provider AI Copilot (Gemini, OpenAI, Claude, DeepSeek, Groq, Ollama, OpenRouter)
- Direct disk file linking and atomic saving
- Live Markdown split preview and syntax highlighting
- Customizable keybindings and theme settings

![Settings](./assets/screenshot-settings.png)

## Keybindings (Default)

All shortcuts can be customized or rebound in **Settings (`Ctrl + ,`) -> Shortcuts**.

- `Super + N` : Toggle window
- `Ctrl + N` : New tab
- `Ctrl + W` : Close active tab
- `Ctrl + S` : Save notes and sync linked files
- `Ctrl + Enter` : AI Copilot
- `Ctrl + K` : Search notes
- `Ctrl + ,` : Settings
- `Ctrl + P` : Toggle Markdown preview mode
- `Ctrl + Shift + E` : Export note
- `Ctrl + Tab` / `Ctrl + Shift + Tab` : Next / Previous tab
- `Ctrl + 1..9` / `Ctrl + 0` : Switch to tab / Last tab
- `Ctrl + =` / `Ctrl + -` : Increase / Decrease font size
- `Tab` : Accept autocomplete suggestion

## Installation

```bash
cargo install --path .
```

### Hyprland Setup

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

## Documentation

- [Suggestion Engine Architecture](docs/suggestion_engine_architecture.md)

## License

MIT
