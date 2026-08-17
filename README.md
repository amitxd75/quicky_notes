# Quicky Notes

Lightweight floating glassmorphism note widget and fast code scratchpad for Hyprland, Wayland, and Linux desktops.

![Quicky Notes](./assets/screenshot-main.png)

## Core Highlights

- **Adaptive Glassmorphism & Wallpaper Sync**:
  Real-time dynamic palette extraction from active desktop wallpaper via Pywal and Caelestia caches, alongside curated presets and an AI-driven natural language theme generator.

- **Embedded Predictive Autocomplete**:
  Zero-latency dual-engine autocomplete combining an in-memory Radix Trie and Markov bigram language model for predictive ghost text and single-key Tab completions.

- **Multi-Provider AI Copilot**:
  Integrated assistance supporting Gemini, OpenAI, Claude, DeepSeek, Groq, Ollama, and OpenRouter for in-editor refactoring, summaries, explanations, and custom prompts.

- **Portable `.qn` Containers & Direct Disk Linking**:
  Standalone binary note format with embedded image attachments, plus direct filesystem file and folder workspace editing with external modification detection (`mtime`).

- **Syntax Highlighting & Soft-Wrap Aware Gutter**:
  Real-time syntax highlighting across 15+ programming languages with pixel-locked line numbers that maintain exact row alignment across wrapped lines.

![Settings](./assets/screenshot-settings.png)

## Installation

```bash
# Using the installer script (binary, desktop entry, and icons)
./scripts/install.sh

# Or using Cargo
cargo install --path .
```

## CLI Usage

```bash
# Launch note widget
quicky

# Open specific files
quicky note.md main.rs document.qn

# Open folder workspace
quicky -f ~/projects/my_project

# Launch with folder sidebar open
quicky --sidebar ~/projects/my_project
```

## Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + N` | New Note Tab |
| `Ctrl + W` | Close Active Tab |
| `Ctrl + S` | Save Notes / Linked Files |
| `Ctrl + O` | Open File Dialog |
| `Ctrl + Shift + O` | Open Folder Workspace Dialog |
| `Ctrl + F` | Search Notes |
| `Ctrl + ,` | Settings / Options |
| `Ctrl + M` | Cycle Markdown Mode (Edit / Split / Preview) |
| `Ctrl + Shift + F` | Toggle Folder Sidebar |
| `Ctrl + Space` | Open AI Copilot |
| `Tab` | Accept Ghost Autocomplete |

## Window Manager & Desktop Integration

### Hyprland (Lua)
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

## Documentation

- [Suggestion Engine Architecture](docs/suggestion_engine_architecture.md)
## License

MIT
