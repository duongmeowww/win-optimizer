use super::*;
use super::gaming::{ps_quiet, reg_val};

// ============ Gaming Profile Presets ============

#[derive(serde::Serialize, Clone)]
pub struct GamingPreset {
    pub id: String,
    pub label: String,
    pub desc: String,
    pub aggressive: bool, // cần confirm
    pub tweaks: Vec<String>, // id tweak áp dụng
    pub active: bool,
}

/// Áp dụng một batch tweak = đổi từng cái qua apply_gaming_tweak (rồi trace riêng).
fn apply_ids(ids: &[String]) -> (usize, Vec<String>) {
    let mut ok = 0;
    let mut fails = Vec::new();
    for id in ids {
        if let Ok((true, _)) = super::gaming::apply_gaming_tweak(id.clone(), "apply".into()) {
            ok += 1;
        } else {
            fails.push(id.clone());
        }
    }
    (ok, fails)
}

#[tauri::command]
pub fn get_gaming_presets() -> Vec<GamingPreset> {
    let active: Vec<String> = super::gaming::get_gaming_tweaks()
        .into_iter()
        .filter(|t| t.active)
        .map(|t| t.id)
        .collect();

    vec![
        GamingPreset {
            id: "performance".into(),
            label: "Performance".into(),
            desc: "Cân bằng an toàn: HAGS, Game Mode, tắt DVR, Nagle, net throttle, SystemRes — hiệu năng tức thì, không cần reboot.".into(),
            aggressive: false,
            tweaks: vec!["hags".into(), "game_mode".into(), "game_dvr".into(), "nagle".into(), "net_throttle".into(), "sys_resp".into(), "win32prio".into(), "mmcss".into(), "mouse".into()],
            active: active.iter().all(|a| ["hags".to_string(), "game_mode".to_string(), "game_dvr".to_string(), "nagle".to_string(), "net_throttle".to_string(), "sys_resp".to_string(), "win32prio".to_string(), "mmcss".to_string(), "mouse".to_string()].contains(a)),
        },
        GamingPreset {
            id: "ultra".into(),
            label: "Ultra (Esports)".into(),
            desc: "Tối đa FPS: Performance + Ultimate plan + core parking off. Có thể cần reboot. Tăng nhiệt/điện.".into(),
            aggressive: false,
            tweaks: vec!["hags".into(), "game_mode".into(), "game_dvr".into(), "nagle".into(), "net_throttle".into(), "sys_resp".into(), "win32prio".into(), "mmcss".into(), "mouse".into(), "ult_power".into(), "core_park".into(), "tdr".into()],
            active: active.iter().all(|a| ["hags".to_string(), "game_mode".to_string(), "game_dvr".to_string(), "nagle".to_string(), "net_throttle".to_string(), "sys_resp".to_string(), "win32prio".to_string(), "mmcss".to_string(), "mouse".to_string(), "ult_power".to_string(), "core_park".to_string(), "tdr".to_string()].contains(a)),
        },
    ]
}

#[tauri::command]
pub fn apply_gaming_preset(id: String, action: String) -> Result<(bool, String), String> {
    let presets = get_gaming_presets();
    let p = presets.into_iter().find(|x| x.id == id).ok_or("Preset không tồn tại")?;
    let (ok, fails) = match action.as_str() {
        "apply" => apply_ids(&p.tweaks),
        "revert" => {
            let mut n = 0;
            let mut f = Vec::new();
            for tid in &p.tweaks {
                if let Ok((true, _)) = super::gaming::apply_gaming_tweak(tid.clone(), "revert".into()) {
                    n += 1;
                } else { f.push(tid.clone()); }
            }
            (n, f)
        }
        _ => return Err("action phải là apply/revert".into()),
    };
    let msg = if fails.is_empty() {
        format!("{} preset '{}': {} tweak xong.", if action == "apply" { "Áp" } else { "Hoàn tác" }, p.label, ok)
    } else {
        format!("{} preset '{}': {} xong, {} lỗi: {:?}", if action == "apply" { "Áp" } else { "Hoàn tác" }, p.label, ok, fails.len(), fails)
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

    // 3. Suspend background apps (tên process) — Windows 11 suspend API qua PowerShell không có, dùng Suspend-Process via NtSuspendProcess? Đơn giản: giảm priority + not suspend.
    //    Dùng PowerShell job? Để an toàn, chỉ giảm priority xuống low cho các app nặng user chỉ định.
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

const HKCU_GPU_PREFS: &str = "HKCU:\\SOFTWARE\\Microsoft\\DirectX\\UserGpuPreferences";
const HKCU_APPCOMPAT: &str = "HKCU:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers";

fn exe_name(path: &str) -> String {
    std::path::Path::new(path).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "game".into())
}

/// Lưu/đọc profile cho 1 game (dựa trên registry HKCU — per-user, không admin).
#[tauri::command]
pub fn set_game_profile(path: String, gpu_pref: u8, _fso_off: bool, run_as_admin: bool) -> Result<(bool, String), String> {
    let name = exe_name(&path);

    // 1. GPU preference (HKCU UserGpuPreferences) — "exe_path" = "GpuPreference=2;"
    if gpu_pref > 0 {
        let ps = format!(
            r#"New-Item -Path '{0}' -Force | Out-Null; Set-ItemProperty -Path '{0}' -Name '{1}' -Value 'GpuPreference={2};' -Type String"#,
            HKCU_GPU_PREFS,
            path.replace('\'', "''"),
            gpu_pref
        );
        ps_quiet(&ps, 10);
    } else {
        let ps = format!(r#"Remove-ItemProperty -Path '{0}' -Name '{1}' -ErrorAction SilentlyContinue"#, HKCU_GPU_PREFS, path.replace('\'', "''"));
        ps_quiet(&ps, 10);
    }

    // 2. FSO off: registry HKCU\System\GameConfigStore\Children — cần GUID, phức tạp. Dùng AppCompat Layers thay (đơn giản hoá): HighDPIAware + Disable fullscreen optimizations qua AppCompatFlag
    //    AppCompat Layers: ~ HIGHDPIAWARE (FSO control qua OtherOptions). Để đơn giản dùng RUNASADMIN riêng.

    // 3. Run-as-admin via AppCompat Layers (HKCU)
    let compat_val = format!("~{}", if run_as_admin { "RUNASADMIN" } else { "" });
    if run_as_admin {
        let ps = format!(r#"New-Item -Path '{0}' -Force | Out-Null; Set-ItemProperty -Path '{0}' -Name '{1}' -Value '{2}' -Type String"#, HKCU_APPCOMPAT, path.replace('\'', "''"), compat_val);
        ps_quiet(&ps, 10);
    } else {
        let ps = format!(r#"Remove-ItemProperty -Path '{0}' -Name '{1}' -ErrorAction SilentlyContinue"#, HKCU_APPCOMPAT, path.replace('\'', "''"));
        ps_quiet(&ps, 10);
    }

    Ok((true, format!("Đã lưu profile '{}': GPU={}, RunAsAdmin={}", name, if gpu_pref == 2 { "High Perf" } else if gpu_pref == 1 { "Power Saving" } else { "Default" }, run_as_admin)))
}

/// Lấy profile hiện tại của 1 game từ registry.
#[tauri::command]
pub fn get_game_profile(path: String) -> GameProfile {
    let gpu = {
        let v = reg_val(HKCU_GPU_PREFS, "", &path);
        if v.contains("GpuPreference=2") { 2 } else if v.contains("GpuPreference=1") { 1 } else { 0 }
    };
    let admin = !reg_val(HKCU_APPCOMPAT, "", &path).is_empty();
    GameProfile {
        path: path.clone(),
        name: exe_name(&path),
        gpu_pref: gpu,
        fso_off: false, // chưa hỗ trợ FSO registry phức tạp
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

/// Kiểm tra toàn bộ tweak nào đang active — pass/fail đọc lại từ hệ thống.
#[tauri::command]
pub fn verify_gaming_tweaks() -> Vec<VerifyItem> {
    super::gaming::get_gaming_tweaks()
        .into_iter()
        .map(|t| VerifyItem { id: t.id.clone(), label: t.label, ok: t.active, expected: if t.active { "hoạt động" } else { "chưa kích hoạt" }.into() })
        .collect()
}
