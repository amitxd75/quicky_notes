//! Platform and operating system integrations.

pub mod cli;
pub mod crash;
pub mod font;

pub use cli::CliArgs;
pub use crash::install_crash_handler;
pub use font::setup_fonts_async;
