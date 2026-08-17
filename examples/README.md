# Quicky Notes Example Plugins

This directory contains ready-to-use sample plugins written in [Rhai](https://rhai.rs/) for Quicky Notes.

---

## Included Examples

| File | Description | Capabilities |
| :--- | :--- | :--- |
| [`quick_terminal.rhai`](quick_terminal.rhai) | Auto-detects and launches your terminal in the active note folder | `>_` Header Button, `Ctrl+\`` Hotkey, Context Menu |
| [`markdown_table_formatter.rhai`](markdown_table_formatter.rhai) | Aligns Markdown table cells and delimiter rows | `▦ Format Table` Context Menu |
| [`timestamp_inserter.rhai`](timestamp_inserter.rhai) | Inserts current ISO date and time at cursor position | `Ctrl+Shift+D` Hotkey, `🕒` Context Menu |
| [`case_transformer.rhai`](case_transformer.rhai) | Converts selected text to UPPERCASE or lowercase | Context Menu actions |

---

## How to Install

Copy any `.rhai` file into your local Quicky Notes plugin directory:

```bash
# Ensure directory exists
mkdir -p ~/.config/quicky_notes/plugins

# Copy an example plugin
cp examples/quick_terminal.rhai ~/.config/quicky_notes/plugins/
cp examples/markdown_table_formatter.rhai ~/.config/quicky_notes/plugins/
```

Then in Quicky Notes, press <kbd>Ctrl</kbd> + <kbd>,</kbd>, go to **🧩 Plugins**, and click **🔄 Reload**.

For complete developer documentation and API reference, see [Plugin Ecosystem Guide](../docs/plugin_ecosystem.md).
