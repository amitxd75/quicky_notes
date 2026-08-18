//! Linux-specific font path resolution and fontconfig indexer.

use std::path::Path;
use std::process::Command;

/// Queries fontconfig via `fc-match` to locate a font file on disk.
pub fn query_platform_font_path(pattern: &str) -> Option<String> {
    if let Ok(output) = Command::new("fc-match")
        .arg("-f")
        .arg("%{file}")
        .arg(pattern)
        .output()
        && output.status.success()
    {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() && Path::new(&path_str).exists() {
            Some(path_str)
        } else {
            None
        }
    } else {
        None
    }
}

/// Discovers installed font families on Linux using `fc-list`.
pub fn discover_system_ui_fonts() -> Vec<String> {
    let mut detected = Vec::new();
    if let Ok(output) = Command::new("fc-list").arg(":").arg("family").output()
        && output.status.success()
    {
        let font_str = String::from_utf8_lossy(&output.stdout);
        detected = font_str
            .lines()
            .flat_map(|line| line.split(','))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.contains(':'))
            .collect();

        detected.sort();
        detected.dedup();
    }
    detected
}

/// Discovers installed monospace font families on Linux using `fc-list :spacing=mono`.
pub fn discover_monospace_fonts() -> Vec<String> {
    let mut detected = Vec::new();
    if let Ok(output) = Command::new("fc-list")
        .arg(":spacing=mono")
        .arg("family")
        .output()
        && output.status.success()
    {
        let font_str = String::from_utf8_lossy(&output.stdout);
        detected = font_str
            .lines()
            .flat_map(|line| line.split(','))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.contains(':'))
            .collect();

        detected.sort();
        detected.dedup();
    }
    detected
}
