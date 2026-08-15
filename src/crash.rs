//! Crash reporting module that intercepts panics and dumps detailed diagnostic logs
//! to `~/.local/share/quicky_notes/logs/log_<timestamp>.log` (or `logs/log_<timestamp>.log`).

use std::fs::File;
use std::io::Write;
use std::panic;

/// Installs custom panic hook to dump crash logs with timestamp, OS details, panic message, and backtrace.
pub fn install_crash_handler() {
    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let log_dir = crate::storage::AppData::logs_dir();
        let log_file = log_dir.join(format!("log_{}.log", timestamp));

        let mut report = String::new();
        report.push_str("====================================================\n");
        report.push_str("             QUICKY NOTES CRASH REPORT              \n");
        report.push_str("====================================================\n\n");
        report.push_str(&format!(
            "Time: {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S %Z")
        ));
        report.push_str(&format!("Version: {}\n", env!("CARGO_PKG_VERSION")));
        report.push_str(&format!(
            "OS / Target: {} {}\n\n",
            std::env::consts::OS,
            std::env::consts::ARCH
        ));

        if let Some(location) = panic_info.location() {
            report.push_str(&format!(
                "Panic Location: {}:{}:{}\n",
                location.file(),
                location.line(),
                location.column()
            ));
        } else {
            report.push_str("Panic Location: Unknown\n");
        }

        let payload_msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<Any> without string representation".to_string()
        };

        report.push_str(&format!("Panic Payload: {}\n\n", payload_msg));
        report.push_str("-------------------- BACKTRACE --------------------\n");
        report.push_str(&format!("{:?}\n", std::backtrace::Backtrace::capture()));
        report.push_str("====================================================\n");

        if let Ok(mut f) = File::create(&log_file) {
            let _ = f.write_all(report.as_bytes());
            let _ = f.flush();
            eprintln!(
                "\n[Quicky Notes] CRASH DETECTED! Diagnostic log written to:\n  {:?}\n",
                log_file
            );
        }

        default_hook(panic_info);
    }));
}
