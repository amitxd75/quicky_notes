//! Linux XDG autostart desktop entry synchronization.

use std::fs;
use std::path::PathBuf;

/// Generates the standard Linux `.desktop` file content for system autostart.
fn generate_desktop_entry(custom_flags: &str) -> Option<String> {
    let exe_path = std::env::current_exe().ok()?;
    let flags = custom_flags.trim();
    let exec_cmd = if flags.is_empty() {
        exe_path.to_string_lossy().to_string()
    } else {
        format!("{} {}", exe_path.display(), flags)
    };

    Some(format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Version=1.0\n\
         Name=Quicky Notes\n\
         Comment=Floating glassmorphism note widget and code scratchpad\n\
         Exec={}\n\
         Icon=quicky_notes\n\
         Terminal=false\n\
         Categories=Utility;TextEditor;\n\
         StartupNotify=false\n\
         X-GNOME-Autostart-enabled=true\n",
        exec_cmd
    ))
}

/// Returns the path to the user's autostart `.desktop` entry file.
fn get_autostart_path() -> Option<PathBuf> {
    directories::UserDirs::new().map(|u| {
        u.home_dir()
            .join(".config")
            .join("autostart")
            .join("quicky.desktop")
    })
}

/// Synchronizes the user's autostart `.desktop` file with user settings.
pub fn sync_autostart_desktop_entry(enabled: bool, custom_flags: &str) {
    if let Some(autostart_path) = get_autostart_path() {
        if enabled {
            if let Some(parent) = autostart_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Some(content) = generate_desktop_entry(custom_flags) {
                let _ = fs::write(&autostart_path, content);
            }
        } else if autostart_path.exists() {
            let _ = fs::remove_file(&autostart_path);
        }
    }
}
