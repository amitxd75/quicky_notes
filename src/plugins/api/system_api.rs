//! System operations, terminal launching, and OS integrations exposed to Rhai.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

/// Maximum allowed output capture in bytes (1 MB) to prevent memory exhaustion.
pub const MAX_EXEC_OUTPUT_BYTES: usize = 1_048_576;

/// Candidate terminal emulator binary names ordered by platform desktop popularity.
pub const TERMINAL_CANDIDATES: &[&str] = &[
    "wt",
    "powershell",
    "cmd",
    "kitty",
    "alacritty",
    "foot",
    "ghostty",
    "wezterm",
    "wezterm-gui",
    "gnome-terminal",
    "konsole",
    "xfce4-terminal",
    "tilix",
    "terminator",
    "xterm",
    "urxvt",
];

/// System integration proxy exposed to Rhai scripts as `system`.
#[derive(Debug, Clone)]
pub struct SystemHandle {
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

impl Default for SystemHandle {
    fn default() -> Self {
        Self {
            timeout_ms: 2000,
            max_output_bytes: MAX_EXEC_OUTPUT_BYTES,
        }
    }
}

impl SystemHandle {
    /// Creates a new system handle instance with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new system handle with custom timeout and buffer limits.
    pub fn with_config(timeout_ms: u64, max_output_bytes: usize) -> Self {
        Self {
            timeout_ms: timeout_ms.clamp(500, 30_000),
            max_output_bytes: max_output_bytes.clamp(64_000, 10_485_760),
        }
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

        // 2. Iterate through common platform terminal emulators
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

    /// Executes a command synchronously, capturing stdout up to max_output_bytes (enforces timeout and child kill).
    pub fn exec_sync(&mut self, command: String, args: rhai::Array) -> String {
        let cmd = command.trim();
        if cmd.is_empty() {
            return String::new();
        }

        let mut cmd_builder = Command::new(cmd);
        for arg in args {
            if let Ok(s) = arg.into_string() {
                cmd_builder.arg(s);
            }
        }

        cmd_builder
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let child = match cmd_builder.spawn() {
            Ok(c) => c,
            Err(_) => return String::new(),
        };

        let child_arc = Arc::new(Mutex::new(Some(child)));
        let child_clone = Arc::clone(&child_arc);

        let timeout_duration = std::time::Duration::from_millis(self.timeout_ms);
        let (done_tx, done_rx) = std::sync::mpsc::channel();

        // Supervisor thread to kill child on timeout
        std::thread::spawn(move || {
            if done_rx.recv_timeout(timeout_duration).is_err()
                && let Ok(mut guard) = child_clone.lock()
                && let Some(mut c) = guard.take()
            {
                let _ = c.kill();
                let _ = c.wait();
            }
        });

        // Worker thread to read bounded stdout
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let child_worker = Arc::clone(&child_arc);
        let max_bytes = self.max_output_bytes;

        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let mut stdout_opt = None;
            if let Ok(mut guard) = child_worker.lock()
                && let Some(ref mut c) = *guard
            {
                stdout_opt = c.stdout.take();
            }

            if let Some(mut stream) = stdout_opt {
                use std::io::Read;
                let _ = stream.by_ref().take(max_bytes as u64).read_to_end(&mut buf);
            }

            if let Ok(mut guard) = child_worker.lock()
                && let Some(mut c) = guard.take()
            {
                let _ = c.wait();
            }

            let _ = done_tx.send(());
            let _ = result_tx.send(buf);
        });

        match result_rx.recv_timeout(timeout_duration + std::time::Duration::from_millis(300)) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).trim().to_string(),
            Err(_) => {
                if let Ok(mut guard) = child_arc.lock()
                    && let Some(mut c) = guard.take()
                {
                    let _ = c.kill();
                    let _ = c.wait();
                }
                String::new()
            }
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
            #[cfg(windows)]
            {
                let _ = Command::new("cmd")
                    .args(["/c", "start", "", trimmed])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            }
            #[cfg(not(windows))]
            {
                let _ = Command::new("xdg-open")
                    .arg(trimmed)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            }
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
        "wt" | "wt.exe" => {
            cmd.arg("-d").arg(dir);
        }
        "powershell" | "powershell.exe" => {
            cmd.arg("-NoExit")
                .arg("-Command")
                .arg(format!("Set-Location '{}'", dir.display()));
        }
        "cmd" | "cmd.exe" => {
            cmd.arg("/K").arg(format!("cd /d \"{}\"", dir.display()));
        }
        "kitty" => {
            cmd.arg("--directory").arg(dir);
        }
        "alacritty" => {
            cmd.arg("--working-directory").arg(dir);
        }
        "foot" => {
            cmd.arg("-D").arg(dir);
        }
        "wezterm" | "wezterm-gui" => {
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
            #[cfg(windows)]
            {
                if dir.join(format!("{}.exe", binary)).is_file()
                    || dir.join(format!("{}.cmd", binary)).is_file()
                    || dir.join(format!("{}.bat", binary)).is_file()
                {
                    return true;
                }
            }
        }
    }
    false
}
