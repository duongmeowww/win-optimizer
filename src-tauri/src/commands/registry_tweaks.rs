use std::process::Command;
use std::os::windows::process::CommandExt;
use winreg::RegKey;
use winreg::enums::*;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RegistryTweakItem {
    pub id: String,
    pub name: String,
    pub category: String, // Performance, Privacy, Gaming, UI
    pub description: String,
    pub risk: String, // Safe, Conditional, Advanced
    pub enabled: bool,
}

#[tauri::command]
pub async fn get_registry_tweaks() -> Vec<RegistryTweakItem> {
    tauri::async_runtime::spawn_blocking(|| {
        vec![
            RegistryTweakItem {
                id: "disable_telemetry".into(),
                name: "Vô hiệu hóa Windows Telemetry".into(),
                category: "Privacy".into(),
                description: "Tắt thu thập dữ liệu chẩn đoán và telemetry nền ngầm của Microsoft.".into(),
                risk: "Safe".into(),
                enabled: check_telemetry_disabled(),
            },
            RegistryTweakItem {
                id: "disable_game_bar".into(),
                name: "Vô hiệu hóa Xbox Game Bar & Captures".into(),
                category: "Gaming".into(),
                description: "Tắt tính năng ghi hình nền ngầm giúp giảm độ trễ (input lag) khi chơi game.".into(),
                risk: "Safe".into(),
                enabled: check_game_bar_disabled(),
            },
            RegistryTweakItem {
                id: "fast_menu_show".into(),
                name: "Tăng tốc độ hiển thị menu (MenuShowDelay)".into(),
                category: "Performance".into(),
                description: "Giảm thời gian trễ khi rê chuột qua các menu xổ xuống trong Windows.".into(),
                risk: "Safe".into(),
                enabled: check_menu_delay_fast(),
            },
            RegistryTweakItem {
                id: "disable_hibernation".into(),
                name: "Vô hiệu hóa chế độ Hibernation (Ngủ đông)".into(),
                category: "Performance".into(),
                description: "Giải phóng dung lượng ổ C bằng cách xóa file hiberfil.sys (bằng kích thước RAM).".into(),
                risk: "Conditional".into(),
                enabled: false,
            },
        ]
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn apply_registry_tweak(id: String, enable: bool) -> Result<(bool, String), String> {
    tauri::async_runtime::spawn_blocking(move || {
        match id.as_str() {
            "disable_telemetry" => {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let path = r"SOFTWARE\Policies\Microsoft\Windows\DataCollection";
                if let Ok((key, _)) = hklm.create_subkey(path) {
                    let val: u32 = if enable { 0 } else { 1 };
                    let _ = key.set_value("AllowTelemetry", &val);
                    Ok((true, "Đã áp dụng thay đổi telemetry thành công.".into()))
                } else {
                    Err("Không thể truy cập Registry key.".into())
                }
            }
            "disable_game_bar" => {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let path = r"Software\Microsoft\Windows\CurrentVersion\GameDVR";
                if let Ok((key, _)) = hkcu.create_subkey(path) {
                    let val: u32 = if enable { 0 } else { 1 };
                    let _ = key.set_value("AppCaptureEnabled", &val);
                    Ok((true, "Đã cấu hình Xbox Game Bar thành công.".into()))
                } else {
                    Err("Không thể truy cập Registry key.".into())
                }
            }
            "fast_menu_show" => {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let path = r"Control Panel\Desktop";
                if let Ok((key, _)) = hkcu.create_subkey(path) {
                    let val = if enable { "10" } else { "400" };
                    let _ = key.set_value("MenuShowDelay", &val);
                    Ok((true, "Đã thay đổi tốc độ menu thành công.".into()))
                } else {
                    Err("Không thể truy cập Registry key.".into())
                }
            }
            "disable_hibernation" => {
                let status = if enable { "off" } else { "on" };
                let output = Command::new("powercfg")
                    .args(&["/hibernate", status])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW - ẩn cửa sổ cmd
                    .output();
                match output {
                    Ok(_) => Ok((true, format!("Đã chuyển chế độ Hibernation sang {}", status))),
                    Err(e) => Err(e.to_string()),
                }
            }
            _ => Err("Tweak không hợp lệ.".into()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

fn check_telemetry_disabled() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\Policies\Microsoft\Windows\DataCollection") {
        if let Ok(val) = key.get_value::<u32, _>("AllowTelemetry") {
            return val == 0;
        }
    }
    false
}

fn check_game_bar_disabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\GameDVR") {
        if let Ok(val) = key.get_value::<u32, _>("AppCaptureEnabled") {
            return val == 0;
        }
    }
    false
}

fn check_menu_delay_fast() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Control Panel\Desktop") {
        if let Ok(val) = key.get_value::<String, _>("MenuShowDelay") {
            return val == "10";
        }
    }
    false
}
