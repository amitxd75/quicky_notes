//! Platform and operating system integrations.

pub mod cli;
pub mod crash;
pub mod font;

pub use cli::CliArgs;
pub use crash::install_crash_handler;
pub use font::{
    apply_system_font, apply_system_fonts, get_installed_monospace_fonts,
    get_installed_system_fonts, setup_fonts_async,
};

use std::path::PathBuf;

/// Syncs the user's autostart desktop file in ~/.config/autostart/quicky.desktop.
pub fn sync_autostart_desktop_file(enabled: bool) -> Result<(), std::io::Error> {
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "quicky", "quicky_notes")
        && let Some(config_dir) = proj_dirs.config_dir().parent()
    {
        let autostart_dir = config_dir.join("autostart");
        let desktop_path = autostart_dir.join("quicky.desktop");
        if enabled {
            let _ = std::fs::create_dir_all(&autostart_dir);
            let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("quicky"));
            let content = format!(
                "[Desktop Entry]\nType=Application\nName=Quicky Notes\nExec={}\nIcon=quicky\nComment=Lightweight scratchpad & quick notes\nTerminal=false\nCategories=Utility;TextEditor;\nX-GNOME-Autostart-enabled=true\n",
                exe_path.to_string_lossy()
            );
            std::fs::write(&desktop_path, content)?;
        } else if desktop_path.exists() {
            let _ = std::fs::remove_file(&desktop_path);
        }
    }
    Ok(())
}
