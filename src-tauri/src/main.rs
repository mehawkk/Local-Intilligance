// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Set up panic hook to log panics to a file (since windows_subsystem hides stderr)
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}\n", info);
        let log_path = std::env::temp_dir().join("local-file-intelligence-panic.log");
        let _ = std::fs::write(&log_path, &msg);
        eprintln!("{}", msg);
    }));

    local_file_intelligence_lib::run();
}
