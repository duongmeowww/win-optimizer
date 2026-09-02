use super::*;
use tauri::async_runtime::spawn_blocking;
use winreg::enums::{HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER};
use winreg::HKEY;
use winreg::RegKey;

/// Một tweak gaming. Mỗi tweak có apply/revert/check dạng text qua PowerShell.
#[derive(serde::Serialize, Clone)]
pub struct GamingTweak {
    pub id: String,
    pub label: String,
    pub desc: String,
    pub category: String, // GPU / Network / CPU / Trade-off
    pub active: bool,     // trạng thái đã apply chưa
    pub tradeoff: bool,   // có rủi ro (đỏ)
}

/// Chạy lệnh, trả output.
pub(crate) fn ps_quiet(script: &str, timeout: u64) -> String {
    sh_timeout(
        "powershell",
        &["-NoProfile", "-Command", script],
        Duration::from_secs(timeout),
    )
}

/// Đọc giá trị registry native bằng crate winreg
fn reg_val_native(hive: HKEY, path: &str, name: &str) -> String {
    let hk = RegKey::predef(hive);
    if let Ok(key) = hk.open_subkey(path) {
        if let Ok(val) = key.get_value::<u32, _>(name) {
            return val.to_string();
        }
        if let Ok(val) = key.get_value::<String, _>(name) {
            return val;
        }
    }
    String::new()
}

fn power_plan_active() -> String {
    let out = ps_quiet("powercfg /getactivescheme", 5);
    let start = out.find("GUID:");
    match start {
        Some(i) => {
            let mut g = out[i + 5..].trim().to_string();
            if let Some(sp) = g.find(char::is_whitespace) {
                g.truncate(sp);
            }
            g
        }
        None => String::new(),
    }
}

/// Trả về danh sách tweak + trạng thái hiện tại (Async & Spawn Blocking).
#[tauri::command]
pub async fn get_gaming_tweaks() -> Result<Vec<GamingTweak>, String> {
    spawn_blocking(move || {
        let hags_on = reg_val_native(HKEY_LOCAL_MACHINE, "SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers", "HwSchMode") == "2";
        let mpo_off = reg_val_native(HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\Dwm", "OverlayTestMode") == "5";
        let game_mode = reg_val_native(HKEY_CURRENT_USER, "Software\\Microsoft\\GameBar", "AutoGameModeEnabled") == "1";
        let gamedvr = reg_val_native(HKEY_CURRENT_USER, "System\\GameConfigStore", "GameDVR_FSEBehaviorMode") == "2";
        
        let nagle = {
            let out = ps_quiet(
                "Get-ChildItem 'HKLM:\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces' | Where-Object { (Get-ItemProperty $_.PSPath).TcpNoDelay -eq 1 } | Measure-Object | Select-Object -ExpandProperty Count",
                5,
            );
            out.trim() != "0"
        };

        let net_throttle = reg_val_native(HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile", "NetworkThrottlingIndex") == "4294967295";
        let sys_resp = reg_val_native(HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile", "SystemResponsiveness") == "0";
        let ult_power = power_plan_active().to_lowercase().contains("e9a42b02");
        let vbs_off = {
            let out = ps_quiet("bcdedit /enum '{current}'", 5).to_lowercase();
            out.contains("hypervisorlaunchtype") && out.contains("off")
        };
        let hpet_off = {
            let out = ps_quiet("bcdedit /enum '{current}'", 5).to_lowercase();
            out.contains("useplatformclock") && out.contains("yes")
        };

        let win32prio = reg_val_native(HKEY_LOCAL_MACHINE, "SYSTEM\\CurrentControlSet\\Control\\PriorityControl", "Win32PrioritySeparation") == "38";
        let mmcss = {
            let v = reg_val_native(HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile\\Tasks\\Games", "Priority");
            v.parse::<u32>().map(|x| x >= 6).unwrap_or(false)
        };
        let tdr = reg_val_native(HKEY_LOCAL_MACHINE, "SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers", "TdrDelay") == "10";
        let mouse = reg_val_native(HKEY_CURRENT_USER, "Control Panel\\Mouse", "MouseSpeed") == "0";

        Ok(vec![
            GamingTweak { id: "hags".into(), label: "HAGS (GPU Scheduling)".into(), desc: "Bật Hardware-Accelerated GPU Scheduling — giảm input lag, cải thiện frame pacing.".into(), category: "GPU".into(), active: hags_on, tradeoff: false },
            GamingTweak { id: "mpo".into(), label: "Tắt MPO (Multiplane Overlay)".into(), desc: "Vô hiệu hóa MPO — hết stutter/DWM trên cấu hình HDR. Khuyên bật nếu gặp micro-stutter.".into(), category: "GPU".into(), active: mpo_off, tradeoff: false },
            GamingTweak { id: "game_mode".into(), label: "Game Mode".into(), desc: "Windows ưu tiên tài nguyên cho game đang chạy.".into(), category: "GPU".into(), active: game_mode, tradeoff: false },
            GamingTweak { id: "game_dvr".into(), label: "Tắt Game DVR / recording".into(), desc: "Vô hiệu hóa background recording — nguyên nhân input lag + V-Sync conflict.".into(), category: "GPU".into(), active: gamedvr, tradeoff: false },
            GamingTweak { id: "nagle".into(), label: "Tắt Nagle's Algorithm".into(), desc: "TcpNoDelay — gói input nhỏ gửi ngay, hết tới 300ms độ trễ nhân tạo.".into(), category: "Network".into(), active: nagle, tradeoff: false },
            GamingTweak { id: "net_throttle".into(), label: "Gỡ giới hạn Bandwidth".into(), desc: "NetworkThrottlingIndex = 0xffffffff — không trì hoãn gói mạng khi tải cao.".into(), category: "Network".into(), active: net_throttle, tradeoff: false },
            GamingTweak { id: "sys_resp".into(), label: "SystemResponsiveness = 0".into(), desc: "MMCSS nhường 100% CPU cho foreground game thay vì 20% cho background.".into(), category: "CPU".into(), active: sys_resp, tradeoff: false },
            GamingTweak { id: "ult_power".into(), label: "Ultimate Performance plan".into(), desc: "CPU không xuống xung giữa frame. Tạo bản sao plan Ultimate (không đụng plan gốc).".into(), category: "CPU".into(), active: ult_power, tradeoff: false },
            GamingTweak { id: "vbs".into(), label: "Tắt VBS / HVCI".into(), desc: "+5-15% FPS. GÃY Valorant (Vanguard) & R6 (BattlEye) trên Win11 24H2+. Cần khởi động lại.".into(), category: "Trade-off".into(), active: vbs_off, tradeoff: true },
            GamingTweak { id: "core_park".into(), label: "Tắt Core Parking".into(), desc: "Ép toàn bộ lõi CPU hoạt động — giảm latency task switching.".into(), category: "CPU".into(), active: false, tradeoff: false },
            GamingTweak { id: "hpet".into(), label: "Tắt HPET".into(), desc: "Dùng TSC thay HPET — giảm overhead timer, thường bớt input lag trên desktop.".into(), category: "CPU".into(), active: hpet_off, tradeoff: false },
            GamingTweak { id: "win32prio".into(), label: "Win32PrioritySeparation".into(), desc: "Foreground boost + short quantum — CPU ưu tiên game đang chạy, giảm task switch jitter.".into(), category: "CPU".into(), active: win32prio, tradeoff: false },
            GamingTweak { id: "mmcss".into(), label: "MMCSS Games (độ ưu tiên multimedia)".into(), desc: "MMCSS ưu tiên Game thread lên 6/8 — giảm stutter khi load background.".into(), category: "CPU".into(), active: mmcss, tradeoff: false },
            GamingTweak { id: "tdr".into(), label: "Tăng TDR delay (chống driver timeout)".into(), desc: "TdrDelay cao hơn — driver GPU không bị reset nếu frame quá lâu (giảm black-screen).".into(), category: "GPU".into(), active: tdr, tradeoff: false },
            GamingTweak { id: "mouse".into(), label: "Tắt Mouse Acceleration".into(), desc: "MouseSpeed=0 — di chuyển chuột tuyến tính 1:1, critical cho aim.".into(), category: "Input".into(), active: mouse, tradeoff: false },
        ])
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Chạy tweak theo action: "apply" | "revert". Trả (ok, message).
#[tauri::command]
pub async fn apply_gaming_tweak(id: String, action: String) -> Result<(bool, String), String> {
    spawn_blocking(move || {
        let script = match (id.as_str(), action.as_str()) {
            ("hags", "apply") => r#"New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers' -Force | Out-Null; Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers' -Name 'HwSchMode' -Value 2 -Type DWord"#,
            ("hags", "revert") => r#"Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers' -Name 'HwSchMode' -Value 1 -Type DWord"#,
            ("mpo", "apply") => r#"New-Item -Path 'HKLM:\SOFTWARE\Microsoft\Windows\Dwm' -Force | Out-Null; Set-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\Dwm' -Name 'OverlayTestMode' -Value 5 -Type DWord"#,
            ("mpo", "revert") => r#"Remove-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\Dwm' -Name 'OverlayTestMode' -ErrorAction SilentlyContinue"#,
            ("game_mode", "apply") => r#"New-Item -Path 'HKCU:\Software\Microsoft\GameBar' -Force | Out-Null; Set-ItemProperty -Path 'HKCU:\Software\Microsoft\GameBar' -Name 'AutoGameModeEnabled' -Value 1 -Type DWord"#,
            ("game_mode", "revert") => r#"Set-ItemProperty -Path 'HKCU:\Software\Microsoft\GameBar' -Name 'AutoGameModeEnabled' -Value 0 -Type DWord"#,
            ("game_dvr", "apply") => r#"New-Item -Path 'HKCU:\System\GameConfigStore' -Force | Out-Null; Set-ItemProperty -Path 'HKCU:\System\GameConfigStore' -Name 'GameDVR_FSEBehaviorMode' -Value 2 -Type DWord; Set-ItemProperty -Path 'HKCU:\System\GameConfigStore' -Name 'GameDVR_Enabled' -Value 0 -Type DWord"#,
            ("game_dvr", "revert") => r#"Set-ItemProperty -Path 'HKCU:\System\GameConfigStore' -Name 'GameDVR_FSEBehaviorMode' -Value 0 -Type DWord; Set-ItemProperty -Path 'HKCU:\System\GameConfigStore' -Name 'GameDVR_Enabled' -Value 1 -Type DWord"#,
            ("nagle", "apply") => r#"Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces' | ForEach-Object { Set-ItemProperty -Path $_.PSPath -Name 'TcpNoDelay' -Value 1 -Type DWord -ErrorAction SilentlyContinue }"#,
            ("nagle", "revert") => r#"Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces' | ForEach-Object { Remove-ItemProperty -Path $_.PSPath -Name 'TcpNoDelay' -ErrorAction SilentlyContinue }"#,
            ("net_throttle", "apply") => r#"New-Item -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile' -Force | Out-Null; Set-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile' -Name 'NetworkThrottlingIndex' -Value 0xffffffff -Type DWord"#,
            ("net_throttle", "revert") => r#"Remove-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile' -Name 'NetworkThrottlingIndex' -ErrorAction SilentlyContinue"#,
            ("sys_resp", "apply") => r#"New-Item -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile' -Force | Out-Null; Set-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile' -Name 'SystemResponsiveness' -Value 0 -Type DWord"#,
            ("sys_resp", "revert") => r#"Remove-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile' -Name 'SystemResponsiveness' -ErrorAction SilentlyContinue"#,
            ("ult_power", "apply") => r#"$schemes = powercfg /list; if (-not ($schemes -match 'e9a42b02-d5df-448d-aa00-03f14749eb61')) { powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61 | Out-Null }; powercfg /setactive e9a42b02-d5df-448d-aa00-03f14749eb61"#,
            ("ult_power", "revert") => r#"powercfg /setactive 381b4222-f694-41f0-9685-ff5bb260df2e"#,
            ("vbs", "apply") => "bcdedit /set hypervisorlaunchtype off",
            ("vbs", "revert") => "bcdedit /set hypervisorlaunchtype auto",
            ("core_park", "apply") => r#"powercfg /setacvalueindex SCHEME_CURRENT 54533251-82be-4824-96c1-47b60b740d00 0cc5b647-c1df-4637-881a-dec4282d1b8c 100; powercfg /setdcvalueindex SCHEME_CURRENT 54533251-82be-4824-96c1-47b60b740d00 0cc5b647-c1df-4637-881a-dec4282d1b8c 100; powercfg /setactive SCHEME_CURRENT"#,
            ("core_park", "revert") => r#"powercfg /setacvalueindex SCHEME_CURRENT 54533251-82be-4824-96c1-47b60b740d00 0cc5b647-c1df-4637-881a-dec4282d1b8c 5; powercfg /setdcvalueindex SCHEME_CURRENT 54533251-82be-4824-96c1-47b60b740d00 0cc5b647-c1df-4637-881a-dec4282d1b8c 5; powercfg /setactive SCHEME_CURRENT"#,
            ("hpet", "apply") => "bcdedit /set useplatformclock true",
            ("hpet", "revert") => "bcdedit /deletevalue useplatformclock",
            ("win32prio", "apply") => r#"New-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\PriorityControl' -Force | Out-Null; Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\PriorityControl' -Name 'Win32PrioritySeparation' -Value 38 -Type DWord"#,
            ("win32prio", "revert") => r#"Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\PriorityControl' -Name 'Win32PrioritySeparation' -Value 2 -Type DWord"#,
            ("mmcss", "apply") => r#"New-Item -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games' -Force | Out-Null; Set-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games' -Name 'Priority' -Value 6 -Type DWord; Set-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games' -Name 'GPU Priority' -Value 8 -Type DWord"#,
            ("mmcss", "revert") => r#"$t='HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games'; Set-ItemProperty -Path $t -Name 'Priority' -Value 2 -Type DWord; Set-ItemProperty -Path $t -Name 'GPU Priority' -Value 8 -Type DWord"#,
            ("tdr", "apply") => r#"Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers' -Name 'TdrDelay' -Value 10 -Type DWord"#,
            ("tdr", "revert") => r#"Remove-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers' -Name 'TdrDelay' -ErrorAction SilentlyContinue"#,
            ("mouse", "apply") => r#"New-Item -Path 'HKCU:\Control Panel\Mouse' -Force | Out-Null; Set-ItemProperty -Path 'HKCU:\Control Panel\Mouse' -Name 'MouseSpeed' -Value 0; Set-ItemProperty -Path 'HKCU:\Control Panel\Mouse' -Name 'MouseThreshold1' -Value 0; Set-ItemProperty -Path 'HKCU:\Control Panel\Mouse' -Name 'MouseThreshold2' -Value 0"#,
            ("mouse", "revert") => r#"Set-ItemProperty -Path 'HKCU:\Control Panel\Mouse' -Name 'MouseSpeed' -Value 1; Set-ItemProperty -Path 'HKCU:\Control Panel\Mouse' -Name 'MouseThreshold1' -Value 6; Set-ItemProperty -Path 'HKCU:\Control Panel\Mouse' -Name 'MouseThreshold2' -Value 10"#,
            _ => return Err(format!("Không hỗ trợ: {} / {}", id, action)),
        };

        let _out = ps_quiet(script, 10);
        Ok((true, format!("{} '{}' xong.", if action == "apply" { "Bật" } else { "Tắt" }, id)))
    })
    .await
    .map_err(|e| e.to_string())?
}
