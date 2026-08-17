//! Platform and operating system integrations.

pub mod crash;
pub mod font;

pub use crash::install_crash_handler;
pub use font::setup_fonts_async;
