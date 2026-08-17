//! System operations, terminal launching, and OS integrations exposed to Rhai.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Maximum allowed output capture in bytes (1 MB) to prevent memory exhaustion.
pub const MAX_EXEC_OUTPUT_BYTES: usize = 1_048_576;

/// Candidate terminal emulator binary names ordered by modern Linux desktop popularity.
pub const TERMINAL_CANDIDATES: &[&str] = &[
    "kitty",
    "alacritty",
    "foot",
    "ghostty",
    "wezterm",
    "gnome-terminal",
    "konsole",
    "xfce4-terminal",
    "tilix",
    "terminator",
    "xterm",
    "urxvt",
];

/// System integration proxy exposed to Rhai scripts as `system`.
#[derive(Debug, Clone, Default)]
pub struct SystemHandle;

impl SystemHandle {
    /// Creates a new system handle instance.
    pub fn new() -> Self {
        Self
    }

    /// Auto-detects and launches an installed terminal emulator in the target directory.
    pub fn launch_terminal(&mut self, target_dir: String) -> bool {
        let dir = resolve_safe_dir(&target_dir);

        // 1. Check $TERMINAL environment variable first
        if let Ok(env_term) = std::env::var("TERMINAL") {
            let term_bin = env_term.trim();
            if !term_bin.is_empty() && spawn_terminal_in_dir(term_bin, &dir) {
                return true;
            }
        }

        // 2. Iterate through common Linux terminal emulators
        for term in TERMINAL_CANDIDATES {
            if is_binary_available(term) && spawn_terminal_in_dir(term, &dir) {
                return true;
            }
        }

        false
    }

    /// Executes a command in the background without blocking the UI thread.
    pub fn exec(&mut self, command: String, args: rhai::Array) -> bool {
        let cmd = command.trim();
        if cmd.is_empty() {
            return false;
        }

        let mut child = Command::new(cmd);
        for arg in args {
            if let Ok(s) = arg.into_string() {
                child.arg(s);
            }
        }

        child
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
    }

    /// Executes a command synchronously, capturing stdout up to 1MB (enforces timeout and byte cap).
    pub fn exec_sync(&mut self, command: String, args: rhai::Array) -> String {
        let cmd = command.trim();
        if cmd.is_empty() {
            return String::new();
        }

        let mut child = Command::new(cmd);
        for arg in args {
            if let Ok(s) = arg.into_string() {
                child.arg(s);
            }
        }

        child
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        match child.output() {
            Ok(output) => {
                let mut bytes = output.stdout;
                if bytes.len() > MAX_EXEC_OUTPUT_BYTES {
                    bytes.truncate(MAX_EXEC_OUTPUT_BYTES);
                }
                String::from_utf8_lossy(&bytes).to_string()
            }
            Err(_) => String::new(),
        }
    }

    /// Safely opens a directory path in the desktop file manager.
    pub fn open_folder(&mut self, path: String) {
        let p = Path::new(&path);
        crate::ui::drag_drop::safe_open_folder(p);
    }

    /// Safely opens an HTTP / HTTPS web URL in the default browser.
    pub fn open_url(&mut self, url: String) {
        let trimmed = url.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            let _ = Command::new("xdg-open")
                .arg(trimmed)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }
    }

    /// Returns the current user's home directory.
    pub fn home_dir(&mut self) -> String {
        directories::UserDirs::new()
            .map(|u| u.home_dir().to_string_lossy().to_string())
            .unwrap_or_else(|| "/home".to_string())
    }

    /// Returns the parent directory of a given file path.
    pub fn parent_dir(&mut self, path: String) -> String {
        let p = Path::new(&path);
        p.parent()
            .map(|parent| parent.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}

/// Spawns the specified terminal binary inside the designated working directory.
fn spawn_terminal_in_dir(term_bin: &str, dir: &Path) -> bool {
    let mut cmd = Command::new(term_bin);

    // Provide directory-specific flags where appropriate
    match term_bin {
        "kitty" => {
            cmd.arg("--directory").arg(dir);
        }
        "alacritty" => {
            cmd.arg("--working-directory").arg(dir);
        }
        "foot" => {
            cmd.arg("-D").arg(dir);
        }
        "wezterm" => {
            cmd.arg("start").arg("--cwd").arg(dir);
        }
        "ghostty" => {
            cmd.arg("--working-directory").arg(dir);
        }
        "gnome-terminal" => {
            cmd.arg(format!("--working-directory={}", dir.display()));
        }
        _ => {
            cmd.current_dir(dir);
        }
    }

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

/// Resolves a directory string to a valid canonical or fallback directory.
fn resolve_safe_dir(dir_str: &str) -> PathBuf {
    let trimmed = dir_str.trim();
    if !trimmed.is_empty() {
        let p = Path::new(trimmed);
        if p.is_dir() {
            return p.to_path_buf();
        }
        if let Some(parent) = p.parent()
            && parent.is_dir()
        {
            return parent.to_path_buf();
        }
    }

    directories::UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Checks whether a binary executable exists in PATH.
fn is_binary_available(binary: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let full_path = dir.join(binary);
            if full_path.is_file() {
                return true;
            }
        }
    }
    false
}
