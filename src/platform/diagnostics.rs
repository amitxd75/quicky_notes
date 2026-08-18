//! Lightweight cross-platform hardware acceleration and process resource diagnostics.

/// Formats a byte count into human-readable string (B, KB, MB, GB).
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Process memory footprint information.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessMemory {
    /// Total resident set size in bytes (includes shared GPU driver libraries).
    pub resident_bytes: u64,
    /// Private anonymous heap memory allocated by the application.
    pub private_heap_bytes: u64,
    /// Shared file-mapped memory (shared libraries like libLLVM, Mesa, Vulkan).
    pub shared_lib_bytes: u64,
    /// Total virtual memory size in bytes.
    pub virtual_bytes: u64,
}

/// Fetches current process memory metrics from Linux `/proc/self/status` or Windows Win32 API.
pub fn get_process_memory() -> Option<ProcessMemory> {
    #[cfg(target_os = "linux")]
    return crate::platform::linux::diagnostics::get_process_memory();

    #[cfg(windows)]
    return crate::platform::windows::diagnostics::get_process_memory();

    #[cfg(not(any(target_os = "linux", windows)))]
    None
}

/// Samples real-time CPU usage percentage of the current process over time.
pub fn get_process_cpu_usage() -> f32 {
    #[cfg(target_os = "linux")]
    return crate::platform::linux::diagnostics::get_process_cpu_usage();

    #[cfg(windows)]
    return crate::platform::windows::diagnostics::get_process_cpu_usage();

    #[cfg(not(any(target_os = "linux", windows)))]
    0.0
}

/// Detects the active display server protocol (Windows DWM, Wayland, X11, or Unknown).
pub fn detect_display_server() -> &'static str {
    #[cfg(target_os = "linux")]
    return crate::platform::linux::diagnostics::detect_display_server();

    #[cfg(windows)]
    return crate::platform::windows::diagnostics::detect_display_server();

    #[cfg(not(any(target_os = "linux", windows)))]
    "Desktop Surface (Hardware Accelerated)"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_scaling() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(10_485_760), "10.0 MB");
        assert_eq!(format_bytes(2_147_483_648), "2.00 GB");
    }

    #[test]
    fn test_linux_diagnostics_queries() {
        let mem = get_process_memory();
        if let Some(m) = mem {
            assert!(m.resident_bytes > 0);
            assert!(m.virtual_bytes >= m.resident_bytes);
        }

        let srv = detect_display_server();
        assert!(!srv.is_empty());
    }

    #[test]
    fn test_profile_subsystems_memory() {
        println!("\n================== SUBSYSTEM MEMORY PROFILING ==================");
        let baseline = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        println!("1. Baseline Process Heap: {}", format_bytes(baseline));

        // 1. Suggestion Engine (50k words Trie)
        let before_suggest = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        let suggest_50k = crate::engine::suggest::SuggestionEngine::new_with_limit(50_000);
        let after_suggest = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        let diff_suggest = after_suggest.saturating_sub(before_suggest);
        println!(
            "2. SuggestionEngine (50,000 words): {} (Word count: {})",
            format_bytes(diff_suggest),
            suggest_50k.word_count()
        );

        // 2. Plugin Subsystem (Rhai AST & Script Engine)
        let before_plugins = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        let mut plugin_mgr = crate::plugins::PluginManager::new();
        plugin_mgr.load_plugins(&[]);
        let after_plugins = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        let diff_plugins = after_plugins.saturating_sub(before_plugins);
        println!(
            "3. Rhai Plugin Manager Engine: {}",
            format_bytes(diff_plugins)
        );

        // 3. SQLite In-Memory Database
        let before_db = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        let db = crate::storage::db::Database::in_memory().unwrap();
        let after_db = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        let diff_db = after_db.saturating_sub(before_db);
        println!("4. SQLite In-Memory Database: {}", format_bytes(diff_db));

        // 4. Notes Data Structure (100 notes with content)
        let before_notes = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        let mut notes = Vec::new();
        for i in 0..100 {
            notes.push(crate::models::note::Note::new(
                format!("Note {}", i),
                "This is a sample markdown note buffer with some text for testing memory allocation.".to_string(),
            ));
        }
        let after_notes = get_process_memory()
            .map(|m| m.private_heap_bytes)
            .unwrap_or(0);
        let diff_notes = after_notes.saturating_sub(before_notes);
        println!(
            "5. 100 Note Buffers & Metadata: {}",
            format_bytes(diff_notes)
        );

        println!("=================================================================\n");
        drop(notes);
        drop(db);
        drop(plugin_mgr);
        drop(suggest_50k);
    }
}
