//! Linux-specific hardware diagnostics, /proc/ memory sampling, and Wayland/X11 detection.

use crate::platform::diagnostics::ProcessMemory;
use std::fs;
use std::sync::Mutex;
use std::time::Instant;

static LAST_SAMPLE: Mutex<Option<(u64, Instant, f32)>> = Mutex::new(None);

#[inline]
fn parse_status_kb(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|num| num.parse::<u64>().ok())
        .unwrap_or(0)
}

/// Fetches current process memory metrics from Linux `/proc/self/status` and `/proc/self/statm`.
pub fn get_process_memory() -> Option<ProcessMemory> {
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

/// Samples real-time CPU usage percentage of the current process over time on Linux.
pub fn get_process_cpu_usage() -> f32 {
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

/// Detects the active Linux display server (Wayland compositor or X11 server).
pub fn detect_display_server() -> &'static str {
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
