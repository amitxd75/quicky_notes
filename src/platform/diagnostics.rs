//! Lightweight cross-platform hardware acceleration and process resource diagnostics.

use std::fs;
use std::sync::Mutex;
use std::time::Instant;

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
    #[cfg(windows)]
    {
        use std::mem::MaybeUninit;
        use windows_sys::Win32::System::ProcessStatus::{
            K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
        };
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        unsafe {
            let mut counters: MaybeUninit<PROCESS_MEMORY_COUNTERS> = MaybeUninit::uninit();
            let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            let handle = GetCurrentProcess();
            if K32GetProcessMemoryInfo(handle, counters.as_mut_ptr(), size) != 0 {
                let counters = counters.assume_init();
                return Some(ProcessMemory {
                    resident_bytes: counters.WorkingSetSize as u64,
                    private_heap_bytes: counters.PagefileUsage as u64,
                    shared_lib_bytes: 0,
                    virtual_bytes: (counters.WorkingSetSize + counters.PagefileUsage) as u64,
                });
            }
        }
        None
    }

    #[cfg(not(windows))]
    {
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            let mut resident_kb = 0_u64;
            let mut anon_kb = 0_u64;
            let mut file_kb = 0_u64;
            let mut vmsize_kb = 0_u64;

            for line in status.lines() {
                if let Some(val) = line.strip_prefix("VmRSS:") {
                    resident_kb = parse_status_kb(val);
                } else if let Some(val) = line.strip_prefix("RssAnon:") {
                    anon_kb = parse_status_kb(val);
                } else if let Some(val) = line.strip_prefix("RssFile:") {
                    file_kb = parse_status_kb(val);
                } else if let Some(val) = line.strip_prefix("VmSize:") {
                    vmsize_kb = parse_status_kb(val);
                }
            }

            if resident_kb > 0 {
                return Some(ProcessMemory {
                    resident_bytes: resident_kb * 1024,
                    private_heap_bytes: anon_kb * 1024,
                    shared_lib_bytes: file_kb * 1024,
                    virtual_bytes: vmsize_kb * 1024,
                });
            }
        }

        let statm = fs::read_to_string("/proc/self/statm").ok()?;
        let mut parts = statm.split_whitespace();
        let vmsize_pages: u64 = parts.next()?.parse().ok()?;
        let resident_pages: u64 = parts.next()?.parse().ok()?;
        let page_size = 4096_u64;

        Some(ProcessMemory {
            resident_bytes: resident_pages * page_size,
            private_heap_bytes: resident_pages * page_size,
            shared_lib_bytes: 0,
            virtual_bytes: vmsize_pages * page_size,
        })
    }
}

#[cfg(not(windows))]
#[inline]
fn parse_status_kb(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|num| num.parse::<u64>().ok())
        .unwrap_or(0)
}

static LAST_SAMPLE: Mutex<Option<(u64, Instant, f32)>> = Mutex::new(None);

/// Samples real-time CPU usage percentage of the current process over time.
pub fn get_process_cpu_usage() -> f32 {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::FILETIME;
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

        unsafe {
            let mut creation = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut exit = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut kernel = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut user = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let handle = GetCurrentProcess();

            if GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) != 0 {
                let kernel_ticks =
                    ((kernel.dwHighDateTime as u64) << 32) | (kernel.dwLowDateTime as u64);
                let user_ticks = ((user.dwHighDateTime as u64) << 32) | (user.dwLowDateTime as u64);
                let current_ticks = kernel_ticks + user_ticks;
                let now = Instant::now();

                let mut lock = match LAST_SAMPLE.lock() {
                    Ok(guard) => guard,
                    Err(_) => return 0.0,
                };

                if let Some((prev_ticks, prev_instant, prev_cpu)) = *lock {
                    let elapsed = now.duration_since(prev_instant);
                    if elapsed.as_millis() < 300 {
                        return prev_cpu;
                    }

                    let delta_ticks = current_ticks.saturating_sub(prev_ticks);
                    let delta_sec = delta_ticks as f64 / 10_000_000.0;
                    let elapsed_sec = elapsed.as_secs_f64();
                    let cpu_pct = (delta_sec / elapsed_sec) * 100.0;
                    let clamped = (cpu_pct as f32).clamp(0.0, 100.0);
                    *lock = Some((current_ticks, now, clamped));
                    clamped
                } else {
                    *lock = Some((current_ticks, now, 0.0));
                    0.0
                }
            } else {
                0.0
            }
        }
    }

    #[cfg(not(windows))]
    {
        let stat = match fs::read_to_string("/proc/self/stat") {
            Ok(s) => s,
            Err(_) => return 0.0,
        };

        let r_paren = match stat.rfind(')') {
            Some(idx) => idx,
            None => return 0.0,
        };
        let rest = &stat[r_paren + 1..];
        let fields: Vec<&str> = rest.split_whitespace().collect();

        if fields.len() <= 12 {
            return 0.0;
        }

        let utime: u64 = fields[11].parse().unwrap_or(0);
        let stime: u64 = fields[12].parse().unwrap_or(0);
        let current_ticks = utime + stime;
        let now = Instant::now();

        let mut lock = match LAST_SAMPLE.lock() {
            Ok(guard) => guard,
            Err(_) => return 0.0,
        };

        if let Some((prev_ticks, prev_instant, prev_cpu)) = *lock {
            let elapsed = now.duration_since(prev_instant);
            if elapsed.as_millis() < 300 {
                return prev_cpu;
            }

            let delta_ticks = current_ticks.saturating_sub(prev_ticks);
            let seconds = elapsed.as_secs_f64();
            let clock_ticks_per_sec = 100.0;
            let cpu_pct = ((delta_ticks as f64 / clock_ticks_per_sec) / seconds) * 100.0;
            let clamped = (cpu_pct as f32).clamp(0.0, 100.0);
            *lock = Some((current_ticks, now, clamped));
            clamped
        } else {
            *lock = Some((current_ticks, now, 0.0));
            0.0
        }
    }
}

/// Detects the active display server protocol (Windows DWM, Wayland, X11, or Unknown).
pub fn detect_display_server() -> &'static str {
    #[cfg(windows)]
    {
        "Windows DWM (Desktop Window Manager)"
    }
    #[cfg(not(windows))]
    {
        if let Ok(sess) = std::env::var("XDG_SESSION_TYPE") {
            if sess.eq_ignore_ascii_case("wayland") {
                return "Wayland (Native Compositor)";
            } else if sess.eq_ignore_ascii_case("x11") {
                return "X11 (X.Org Server)";
            }
        }
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            "Wayland (Native Compositor)"
        } else if std::env::var("DISPLAY").is_ok() {
            "X11 (X.Org Server)"
        } else {
            "Linux Desktop / Headless"
        }
    }
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
