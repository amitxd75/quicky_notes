//! Windows Startup folder autostart script synchronization.

use std::fs;
use std::path::PathBuf;

/// Path to the Windows Startup folder launcher script.
fn get_startup_script_path() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|u| {
        let appdata = std::env::var("APPDATA")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| u.home_dir().join("AppData").join("Roaming"));
        let startup = appdata
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup");
        Some(startup.join("quicky_notes.bat"))
    })
}

/// Synchronizes the Windows Startup folder batch script with user autostart settings.
pub fn sync_autostart_desktop_entry(enabled: bool, custom_flags: &str) {
    if let Some(script_path) = get_startup_script_path() {
        if enabled {
            if let Some(parent) = script_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(exe_path) = std::env::current_exe() {
                let flags = custom_flags.trim();
                let content = if flags.is_empty() {
                    format!("@echo off\r\nstart \"\" \"{}\"\r\n", exe_path.display())
                } else {
                    format!(
                        "@echo off\r\nstart \"\" \"{}\" {}\r\n",
                        exe_path.display(),
                        flags
                    )
                };
                let _ = fs::write(&script_path, content);
            }
        } else if script_path.exists() {
            let _ = fs::remove_file(&script_path);
        }
    }
}
