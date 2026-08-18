//! Platform and operating system integrations.

pub mod cli;
pub mod crash;
pub mod diagnostics;
pub mod font;

pub use cli::CliArgs;
pub use crash::install_crash_handler;
pub use font::{
    apply_system_font, apply_system_fonts, get_installed_monospace_fonts,
    get_installed_system_fonts, setup_fonts_async,
};

use std::path::PathBuf;

/// Syncs the user's autostart file (Linux ~/.config/autostart/quicky.desktop or Windows Startup folder).
pub fn sync_autostart_desktop_file(enabled: bool) -> Result<(), std::io::Error> {
    #[cfg(windows)]
    {
        if let Some(base_dirs) = directories::BaseDirs::new() {
            let app_data = base_dirs.config_dir(); // Points to AppData\Roaming on Windows
            let startup_dir = app_data
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup");
            let script_path = startup_dir.join("quicky_notes.bat");
            if enabled {
                let _ = std::fs::create_dir_all(&startup_dir);
                let exe_path =
                    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("quicky_notes.exe"));
                let content = format!(
                    "@echo off\r\nstart \"\" \"{}\"\r\n",
                    exe_path.to_string_lossy()
                );
                std::fs::write(&script_path, content)?;
            } else if script_path.exists() {
                let _ = std::fs::remove_file(&script_path);
            }
        }
    }

    #[cfg(not(windows))]
    {
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
    }

    Ok(())
}
