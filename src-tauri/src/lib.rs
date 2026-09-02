use std::process::Command as StdCommand;
use std::time::Duration;
use sysinfo::System;

mod commands;

#[derive(serde::Serialize, Default, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(serde::Serialize, Default)]
pub struct SystemInfo {
    pub cpu_usage: f64,
    pub cpu_name: String,
    pub cpu_clock_mhz: u64,
    pub cpu_temp_c: f64,
    pub ram_total: u64,
    pub ram_used: u64,
    pub disk_total: u64,
    pub disk_free: u64,
    pub disk_info: Vec<DiskInfo>,
    pub gpu_name: String,
    pub gpu_clock_mhz: u64,
    pub gpu_temp_c: f64,
    pub gpu_usage: f64,
    pub os_version: String,
    pub uptime_seconds: u64,
}

fn sh_timeout(cmd: &str, args: &[&str], timeout: Duration) -> String {
    use std::io::Read;
    use std::process::Stdio;
    use std::sync::mpsc;

    let mut child = match StdCommand::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let mut stdout = match child.stdout.take() {
        Some(s) => s,
        None => return String::new(),
    };

    let (tx, rx) = mpsc::channel::<String>();
    let reader = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stdout.read_to_string(&mut buf);
        let _ = tx.send(buf);
    });

    let start = std::time::Instant::now();
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = reader.join();
            return rx.recv().unwrap_or_default();
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = reader.join();
    rx.recv().unwrap_or_default().trim().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![commands::get_sys_info, commands::get_disk_clean_candidates, commands::run_disk_clean, commands::run_benchmark, commands::clean_memory, commands::gaming::get_gaming_tweaks, commands::gaming::apply_gaming_tweak, commands::gaming_advanced::get_gaming_presets, commands::gaming_advanced::apply_gaming_preset, commands::gaming_advanced::start_gaming_session, commands::gaming_advanced::stop_gaming_session, commands::gaming_advanced::get_gaming_session, commands::gaming_advanced::set_game_profile, commands::gaming_advanced::get_game_profile, commands::gaming_advanced::verify_gaming_tweaks, commands::diagnostics::repair_system_files, commands::diagnostics::run_chkdsk_scan, commands::diagnostics::get_recent_events, commands::diagnostics::get_drive_smart])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
