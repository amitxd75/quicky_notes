# Quicky Notes Example Plugins

This directory contains ready-to-use flagship plugins written in [Rhai](https://rhai.rs/) for Quicky Notes.

---

## Included Flagship Plugins

| File | Description | Capabilities |
| :--- | :--- | :--- |
| [`pomodoro_timer.rhai`](pomodoro_timer.rhai) | Focus timer with 25m work / 5m break intervals, background timer ticks, audio/visual alerts, and bottom console drawer | `🍅` Header Button, `Ctrl+Alt+P` Hotkey, Background Interval Timer, Drawer |
| [`quote_of_the_day.rhai`](quote_of_the_day.rhai) | Fetches inspirational programming quotes over HTTP REST, parses JSON, and displays in console drawer | `💡` Header Button, `Ctrl+Alt+Q` Hotkey, HTTP REST Client, Drawer |
| [`custom_theme_cycler.rhai`](custom_theme_cycler.rhai) | Dynamically cycles accent highlights between Cyberpunk Cyan, Tokyo Purple, and Emerald in real time | `🎨` Header Button, `Ctrl+Alt+T` Hotkey, Dynamic Theme API |
| [`quick_terminal.rhai`](quick_terminal.rhai) | Auto-detects and launches your native terminal emulator in the active note folder | `>_` Header Button, `Ctrl+\`` Hotkey, Context Menu |

---

## How to Install

Copy any `.rhai` file into your local Quicky Notes plugin directory:

```bash
# Ensure directory exists
mkdir -p ~/.config/quicky_notes/plugins

# Copy plugins
cp examples/pomodoro_timer.rhai ~/.config/quicky_notes/plugins/
cp examples/quote_of_the_day.rhai ~/.config/quicky_notes/plugins/
cp examples/custom_theme_cycler.rhai ~/.config/quicky_notes/plugins/
cp examples/quick_terminal.rhai ~/.config/quicky_notes/plugins/
```

Then in Quicky Notes, press <kbd>Ctrl</kbd> + <kbd>,</kbd>, go to **🔌 Plugins**, and click **🔄 Reload**.

For complete developer documentation and API reference, see [Plugin Ecosystem Guide](../docs/plugin_ecosystem.md).
