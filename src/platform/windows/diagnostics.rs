//! Windows-specific hardware diagnostics, Win32 memory sampling, and DWM detection.

use crate::platform::diagnostics::ProcessMemory;
use std::mem::MaybeUninit;
use std::sync::Mutex;
use std::time::Instant;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

static LAST_SAMPLE: Mutex<Option<(u64, Instant, f32)>> = Mutex::new(None);

/// Fetches current process memory metrics from Windows Win32 API.
pub fn get_process_memory() -> Option<ProcessMemory> {
    unsafe {
        let mut counters: MaybeUninit<PROCESS_MEMORY_COUNTERS> = MaybeUninit::uninit();
        let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        let handle = GetCurrentProcess();
        if K32GetProcessMemoryInfo(handle, counters.as_mut_ptr(), size) != 0 {
            let counters = counters.assume_init();
            Some(ProcessMemory {
                resident_bytes: counters.WorkingSetSize as u64,
                private_heap_bytes: counters.PagefileUsage as u64,
                shared_lib_bytes: 0,
                virtual_bytes: (counters.WorkingSetSize + counters.PagefileUsage) as u64,
            })
        } else {
            None
        }
    }
}

/// Samples real-time CPU usage percentage of the current process over time on Windows.
pub fn get_process_cpu_usage() -> f32 {
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

/// Detects the active Windows display server protocol.
pub fn detect_display_server() -> &'static str {
    "Windows DWM (Desktop Window Manager)"
}
