use crate::commands::gaming::ps_quiet;
use tauri::{AppHandle, Emitter};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use winreg::enums::*;
use winreg::RegKey;

/// App có đang chạy với quyền admin (elevated) không?
#[tauri::command]
pub fn is_admin() -> bool {
    let out = ps_quiet("(New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)", 10);
    out.trim() == "True"
}

/// Relaunch app với quyền admin (UAC prompt), đóng instance hiện tại.
#[tauri::command]
pub fn relaunch_admin(app: AppHandle) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe.to_string_lossy().to_string();
    // runas → UAC prompt
    let _ = Command::new("powershell")
        .args(["-Command", &format!("Start-Process -FilePath '{0}' -Verb RunAs", exe_str.replace('\'', "''"))])
        .spawn()
        .map_err(|e| e.to_string())?;
    // đóng instance hiện tại sau 1s
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1200));
        app.exit(0);
    });
    Ok(())
}

// ============ Gaming Profile Presets ============

#[derive(serde::Serialize, Clone)]
pub struct GamingPreset {
    pub id: String,
    pub label: String,
    pub desc: String,
    pub aggressive: bool, // cần confirm
    pub tweaks: Vec<String>, // id tweak áp dụng
    pub active: bool,
    pub active_count: usize,
    pub total: usize,
}

#[tauri::command]
pub async fn get_gaming_presets() -> Result<Vec<GamingPreset>, String> {
    let tweaks_list = super::gaming::get_gaming_tweaks().await?;
    let active: Vec<String> = tweaks_list
        .into_iter()
        .filter(|t| t.active)
        .map(|t| t.id)
        .collect();

    let count = |ids: &[&str]| ids.iter().filter(|i| active.contains(&i.to_string())).count();

    Ok(vec![
        GamingPreset {
            id: "performance".into(),
            label: "Performance".into(),
            desc: "Cân bằng an toàn: HAGS, Game Mode, tắt DVR, Nagle, net throttle, SystemRes — hiệu năng tức thì, không cần reboot.".into(),
            aggressive: false,
            tweaks: vec!["hags".into(), "game_mode".into(), "game_dvr".into(), "nagle".into(), "net_throttle".into(), "sys_resp".into(), "win32prio".into(), "mmcss".into(), "mouse".into()],
            active: active.iter().all(|a| ["hags".to_string(), "game_mode".to_string(), "game_dvr".to_string(), "nagle".to_string(), "net_throttle".to_string(), "sys_resp".to_string(), "win32prio".to_string(), "mmcss".to_string(), "mouse".to_string()].contains(a)),
            active_count: count(&["hags", "game_mode", "game_dvr", "nagle", "net_throttle", "sys_resp", "win32prio", "mmcss", "mouse"]),
            total: 9,
        },
        GamingPreset {
            id: "ultra".into(),
            label: "Ultra (Esports)".into(),
            desc: "Tối đa FPS: Performance + Ultimate plan + core parking off. Có thể cần reboot. Tăng nhiệt/điện.".into(),
            aggressive: false,
            tweaks: vec!["hags".into(), "game_mode".into(), "game_dvr".into(), "nagle".into(), "net_throttle".into(), "sys_resp".into(), "win32prio".into(), "mmcss".into(), "mouse".into(), "ult_power".into(), "core_park".into(), "tdr".into()],
            active: active.iter().all(|a| ["hags".to_string(), "game_mode".to_string(), "game_dvr".to_string(), "nagle".to_string(), "net_throttle".to_string(), "sys_resp".to_string(), "win32prio".to_string(), "mmcss".to_string(), "mouse".to_string(), "ult_power".to_string(), "core_park".to_string(), "tdr".to_string()].contains(a)),
            active_count: count(&["hags", "game_mode", "game_dvr", "nagle", "net_throttle", "sys_resp", "win32prio", "mmcss", "mouse", "ult_power", "core_park", "tdr"]),
            total: 12,
        },
    ])
}

#[tauri::command]
pub async fn apply_gaming_preset(app: AppHandle, id: String, action: String) -> Result<(bool, String), String> {
    let presets = get_gaming_presets().await?;
    let p = presets.into_iter().find(|x| x.id == id).ok_or("Preset không tồn tại")?;
    let tweaks = p.tweaks.clone();
    let label = p.label.clone();
    let action_inner = action.clone();

    // Chạy nền — không block UI thread. Emit tiến trình từng tweak.
    let (ok, fails) = tauri::async_runtime::spawn_blocking(move || {
        let mut ok = 0;
        let mut fails: Vec<String> = Vec::new();
        let total = tweaks.len();
        for (i, tid) in tweaks.iter().enumerate() {
            let r = futures::executor::block_on(super::gaming::apply_gaming_tweak(tid.clone(), action_inner.clone()));
            let success = r.is_ok();
            
            // Get label from async context
            let t_list = futures::executor::block_on(super::gaming::get_gaming_tweaks());
            let t_label = t_list.ok()
                .and_then(|list| list.into_iter().find(|t| t.id == *tid).map(|t| t.label))
                .unwrap_or_else(|| tid.clone());

            let _ = app.emit("preset-progress", serde_json::json!({
                "index": i + 1,
                "total": total,
                "id": tid,
                "ok": success,
                "label": t_label,
                "msg": r.map(|(_, m)| m).unwrap_or_else(|e| e),
            }));
            if success { ok += 1; } else { fails.push(tid.clone()); }
        }
        (ok, fails)
    })
    .await
    .map_err(|e| e.to_string())?;

    let msg = if fails.is_empty() {
        format!("{} preset '{}': {} tweak xong.", if action == "apply" { "Áp" } else { "Hoàn tác" }, label, ok)
    } else {
        format!("{} preset '{}': {} xong, {} lỗi: {:?}", if action == "apply" { "Áp" } else { "Hoàn tác" }, label, ok, fails.len(), fails)
    };
    Ok((fails.is_empty(), msg))
}

// ============ Gaming Session (Boost mode) ============

/// Session state lưu trong memory (không cần file) — Volatile, mất khi app đóng (không revert tự động, nhưng state track).
#[derive(serde::Serialize, Clone, Default)]
pub struct SessionState {
    pub active: bool,
    pub saved_power_guid: String,
    pub suspended: Vec<String>,
    pub power_plan_switched: bool,
}

static SESSION: OnceLock<Mutex<SessionState>> = OnceLock::new();
fn session() -> &'static Mutex<SessionState> {
    SESSION.get_or_init(|| Mutex::new(SessionState::default()))
}

/// Bắt đầu Gaming Session: snapshot power plan, chuyển sang Ultimate, suspend background app nặng.
#[tauri::command]
pub fn start_gaming_session(apps: Vec<String>) -> SessionState {
    let mut st = session().lock().unwrap();
    if st.active {
        return st.clone();
    }
    // 1. Snapshot power plan hiện tại
    let out = ps_quiet("powercfg /getactivescheme", 10);
    let old_guid = out.split("GUID:").nth(1).map(|s| s.trim().split_whitespace().next().unwrap_or("").to_string()).unwrap_or_default();
    st.saved_power_guid = old_guid.clone();

    // 2. Chuyển sang Ultimate Performance
    let dup = format!(
        "$s = powercfg /list; if (-not ($s -match 'e9a42b02')) {{ powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61 | Out-Null }}; powercfg /setactive e9a42b02-d5df-448d-aa00-03f14749eb61"
    );
    ps_quiet(&dup, 15);

    // 3. Suspend background apps
    st.suspended.clear();
    for a in &apps {
        let ps = format!("Get-Process -Name '{0}' -ErrorAction SilentlyContinue | ForEach-Object {{ $_.PriorityClass = 'Idle' }}", a);
        ps_quiet(&ps, 10);
    }
    st.suspended = apps;
    st.power_plan_switched = true;
    st.active = true;
    st.clone()
}

/// Kết thúc Gaming Session: khôi phục power plan cũ + priority.
#[tauri::command]
pub fn stop_gaming_session() -> SessionState {
    let mut st = session().lock().unwrap();
    if !st.active {
        return st.clone();
    }
    if !st.saved_power_guid.is_empty() {
        let ps = format!("powercfg /setactive {}", st.saved_power_guid);
        ps_quiet(&ps, 10);
    }
    for a in &st.suspended {
        let ps = format!("Get-Process -Name '{0}' -ErrorAction SilentlyContinue | ForEach-Object {{ $_.PriorityClass = 'Normal' }}", a);
        ps_quiet(&ps, 10);
    }
    st.suspended.clear();
    st.saved_power_guid.clear();
    st.power_plan_switched = false;
    st.active = false;
    st.clone()
}

#[tauri::command]
pub fn get_gaming_session() -> SessionState {
    session().lock().unwrap().clone()
}

// ============ Per-game Profiles (HKCU, không cần admin) ============

#[derive(serde::Serialize, Clone)]
pub struct GameProfile {
    pub path: String,        // đường dẫn exe đầy đủ
    pub name: String,        // tên hiển thị
    pub gpu_pref: u8,        // 0=default, 1=power-saving, 2=high performance
    pub fso_off: bool,       // Fullscreen Optimizations off
    pub run_as_admin: bool,  // AppCompat RUNASADMIN
    pub exists: bool,
}

fn exe_name(path: &str) -> String {
    std::path::Path::new(path).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "game".into())
}

/// Đọc registry native helper cho game profile
fn reg_val_hkcu(path: &str, name: &str) -> String {
    let hk = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hk.open_subkey(path) {
        if let Ok(val) = key.get_value::<String, _>(name) {
            return val;
        }
    }
    String::new()
}

/// Lưu/đọc profile cho 1 game (dựa trên registry HKCU — per-user, không admin).
#[tauri::command]
pub fn set_game_profile(path: String, gpu_pref: u8, _fso_off: bool, run_as_admin: bool) -> Result<(bool, String), String> {
    let name = exe_name(&path);

    // 1. GPU preference
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if gpu_pref > 0 {
        if let Ok((key, _)) = hkcu.create_subkey("SOFTWARE\\Microsoft\\DirectX\\UserGpuPreferences") {
            let _ = key.set_value(&path, &format!("GpuPreference={};", gpu_pref));
        }
    } else {
        if let Ok(key) = hkcu.open_subkey("SOFTWARE\\Microsoft\\DirectX\\UserGpuPreferences") {
            let _ = key.delete_value(&path);
        }
    }

    // 2. Run-as-admin via AppCompat Layers
    let compat_val = format!("~{}", if run_as_admin { "RUNASADMIN" } else { "" });
    if run_as_admin {
        if let Ok((key, _)) = hkcu.create_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers") {
            let _ = key.set_value(&path, &compat_val);
        }
    } else {
        if let Ok(key) = hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers") {
            let _ = key.delete_value(&path);
        }
    }

    Ok((true, format!("Đã lưu profile '{}': GPU={}, RunAsAdmin={}", name, if gpu_pref == 2 { "High Perf" } else if gpu_pref == 1 { "Power Saving" } else { "Default" }, run_as_admin)))
}

/// Lấy profile hiện tại của 1 game từ registry.
#[tauri::command]
pub fn get_game_profile(path: String) -> GameProfile {
    let gpu_val = reg_val_hkcu("SOFTWARE\\Microsoft\\DirectX\\UserGpuPreferences", &path);
    let gpu = if gpu_val.contains("GpuPreference=2") {
        2
    } else if gpu_val.contains("GpuPreference=1") {
        1
    } else {
        0
    };
    
    let compat_val = reg_val_hkcu("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers", &path);
    let admin = compat_val.contains("RUNASADMIN");

    GameProfile {
        path: path.clone(),
        name: exe_name(&path),
        gpu_pref: gpu,
        fso_off: false,
        run_as_admin: admin,
        exists: gpu > 0 || admin,
    }
}

// ============ Verification Tool ============

#[derive(serde::Serialize)]
pub struct VerifyItem {
    pub id: String,
    pub label: String,
    pub ok: bool,       // pass/fail
    pub expected: String,
}

/// Kiểm tra toàn bộ tweak nào đang active (Async).
#[tauri::command]
pub async fn verify_gaming_tweaks() -> Result<Vec<VerifyItem>, String> {
    let tweaks_list = super::gaming::get_gaming_tweaks().await?;
    let items = tweaks_list
        .into_iter()
        .map(|t| VerifyItem {
            id: t.id.clone(),
            label: t.label,
            ok: t.active,
            expected: if t.active { "hoạt động" } else { "chưa kích hoạt" }.into()
        })
        .collect();
    Ok(items)
}
