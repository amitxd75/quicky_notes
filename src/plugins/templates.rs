//! Default template plugins bundled with Quicky Notes.
//!
//! Employs compile-time `include_str!` embedding directly from the `examples/` directory
//! to eliminate duplicate template declarations and guarantee DRY single-source-of-truth.

/// Quick Terminal launcher plugin source code.
pub const QUICK_TERMINAL_TEMPLATE: &str = include_str!("../../examples/quick_terminal.rhai");

/// Pomodoro Focus Timer plugin source code.
pub const POMODORO_TIMER_TEMPLATE: &str = include_str!("../../examples/pomodoro_timer.rhai");

/// Quote of the Day HTTP REST fetcher plugin source code.
pub const QUOTE_OF_THE_DAY_TEMPLATE: &str = include_str!("../../examples/quote_of_the_day.rhai");

/// Dynamic Theme Accent Cycler plugin source code.
pub const CUSTOM_THEME_CYCLER_TEMPLATE: &str =
    include_str!("../../examples/custom_theme_cycler.rhai");
