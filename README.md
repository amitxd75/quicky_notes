# Quicky Notes

A cross-platform, lightweight floating glassmorphism note widget and fast code scratchpad for **Linux** (Wayland & X11) and **Windows** (10 & 11).

> **Note**: This branch (`feat/windows-support`) exists to demonstrate that porting Quicky Notes to Windows is fully functional and viable. Because the maintainer does not primarily use Windows, this branch may become outdated over time.

![Quicky Notes](./assets/screenshot-main.png)

## Core Highlights

- **Cross-Platform Glassmorphism**: Native translucent UI with hardware-accelerated rendering, borderless drag-moving, and 8-way border/corner resizing on Linux (Wayland/X11) and Windows 10/11.
- **Embedded Typography & Nerd Icons**: Zero external font dependencies and instant startup with bundled **Inter** (proportional UI) and **FiraCode Nerd Font Mono** (code buffer & symbols).
- **Adaptive Wallpaper Sync**: Real-time palette extraction from your desktop wallpaper using Pywal and Caelestia, plus custom RGB themes and AI-driven palette generation.
- **Embedded Predictive Autocomplete**: Zero-latency ghost text powered by a native statistical dual-engine (Radix Trie + Markov bigram model).
- **Multi-Provider AI Copilot**: Native integration with Gemini, OpenAI, Claude, DeepSeek, Groq, Ollama, and OpenRouter for instant editing, formatting, and summaries.
- **Portable `.qn` Containers & Direct Disk Linking**: Binary note format with embedded image attachments, plus live filesystem editing with automatic external modification sync.
- **Syntax Highlighting & Soft-Wrap Aware Gutter**: Real-time highlighting for 15+ languages with stable, pixel-locked line numbering and markdown split-preview.
- **Embedded Rhai Scripting Engine**: A sandboxed Rust scripting engine for custom header buttons, hotkeys, timers, and hot-reloadable community plugins.

![Settings](./assets/screenshot-settings.png)
![Plugin Manager](./assets/screenshot-plugin.png)

## Installation

### Linux
```bash
# Using the installer script (binary, desktop entry, and icons)
./scripts/install.sh

# Or using Cargo
cargo install --path .
```

### Windows
```powershell
# Build optimized release executable (with embedded icon resource)
cargo build --release

# Run
.\target\release\quicky_notes.exe
```

## CLI Usage

```bash
# Launch note widget
quicky

# Open specific files into separate tabs
quicky note.md main.rs document.qn

# Open folder workspace explorer
quicky -F ~/projects/my_project

# Launch directly with a clean new tab
quicky -n

# View help and version
quicky --help
quicky --version
```

## Default Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + N` | New Note Tab |
| `Ctrl + W` | Close Active Tab |
| `Ctrl + S` | Save Notes / Linked Files to Disk |
| `Ctrl + O` | Open File via Native File Picker |
| `Ctrl + Shift + O` | Open Folder Workspace Dialog |
| `Ctrl + F` | Search Notes Drawer |
| `Ctrl + ,` | Settings & Preferences Drawer |
| `Ctrl + M` | Cycle Markdown Mode (Edit / Split / View) |
| `Ctrl + Shift + F` | Toggle Folder Workspace Sidebar |
| `Ctrl + Space` | Open AI Copilot Modal |
| `Tab` | Accept Ghost Predictive Autocomplete |

## Desktop & Window Manager Integration

### Linux (Hyprland Lua Example)
```lua
-- ~/.config/hypr/hyprland.lua
hl.bind("SUPER + N", hl.dsp.exec_cmd("quicky"), { desc = "Toggle Quicky Notes" })
hl.rule({
    class = "quicky_notes",
    float = true,
    center = true,
    size = { 860, 600 },
})
```

### Windows 10 / 11
- Automatically launches silently into the GUI without opening a console terminal.
- Launch on Startup can be toggled in **Settings (Ctrl + ,) > General > Autostart**.

## Documentation

- [Plugin Ecosystem & Developer Guide](docs/plugin_ecosystem.md)
- [Example Plugins (Rhai)](examples/)
- [Suggestion Engine Architecture](docs/suggestion_engine_architecture.md)

## License

MIT
