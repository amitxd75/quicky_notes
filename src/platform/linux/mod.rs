//! Linux platform integrations, /proc/ diagnostics, and fontconfig discovery.

pub mod autostart;
pub mod diagnostics;
pub mod font;

pub use autostart::sync_autostart_desktop_entry;
pub use diagnostics::{detect_display_server, get_process_cpu_usage, get_process_memory};
pub use font::{discover_monospace_fonts, discover_system_ui_fonts, query_platform_font_path};
