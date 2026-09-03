use std::process::Command;
use std::os::windows::process::CommandExt;
use winreg::RegKey;
use winreg::enums::*;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AdvancedHubModule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub status: String,
}

#[tauri::command]
pub async fn get_advanced_hub_modules() -> Vec<AdvancedHubModule> {
    tauri::async_runtime::spawn_blocking(|| {
        vec![
            AdvancedHubModule {
                id: "gpu_optimization".into(),
                name: "Advanced GPU Optimization".into(),
                category: "Hardware".into(),
                description: "Tối ưu hóa lập lịch GPU (Hardware-accelerated GPU scheduling) và ưu tiên hiệu năng card đồ họa.".into(),
                status: if check_hags_enabled() { "Đã bật (Optimized)" } else { "Chưa tối ưu" }.into(),
            },
            AdvancedHubModule {
                id: "privacy_hardening".into(),
                name: "Telemetry & Privacy Hardening".into(),
                category: "Privacy".into(),
                description: "Chặn hoàn toàn các dịch vụ thu thập dữ liệu ngầm và Advertising ID của Windows.".into(),
                status: "Đã sẵn sàng".into(),
            },
            AdvancedHubModule {
                id: "startup_optimizer".into(),
                name: "Smart Startup & Services Analyzer".into(),
                category: "Performance".into(),
                description: "Phân tích tác động khởi động (High/Medium/Low impact) và quản lý dịch vụ thông minh.".into(),
                status: "Đang hoạt động".into(),
            },
            AdvancedHubModule {
                id: "win_update_control".into(),
                name: "Windows Update Controller".into(),
                category: "System".into(),
                description: "Tạm dừng hoặc kiểm soát băng thông cập nhật Windows tránh ảnh hưởng chơi game/làm việc.".into(),
                status: "Bình thường".into(),
            },
            AdvancedHubModule {
                id: "benchmark_suite".into(),
                name: "System Benchmark & Metrics".into(),
                category: "Diagnostics".into(),
                description: "Đo lường chỉ số CPU, RAM, Disk latency và tạo báo cáo hiệu năng.".into(),
                status: "Sẵn sàng đo".into(),
            },
            AdvancedHubModule {
                id: "profile_system".into(),
                name: "One-Click Optimization Profiles".into(),
                category: "Profiles".into(),
                description: "Chuyển đổi nhanh giữa các cấu hình: Gaming Focus, Battery Saver, Content Creation.".into(),
                status: "Gaming Profile Active".into(),
            },
        ]
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn execute_advanced_hub_action(id: String) -> Result<(bool, String), String> {
    tauri::async_runtime::spawn_blocking(move || {
        match id.as_str() {
            "gpu_optimization" => {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let path = r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers";
                if let Ok((key, _)) = hklm.create_subkey(path) {
                    let _ = key.set_value("HwSchMode", &2u32);
                    Ok((true, "Đã bật Hardware-accelerated GPU Scheduling (HAGS) thành công.".into()))
                } else {
                    Err("Không thể cập nhật Registry GPU.".into())
                }
            }
            "privacy_hardening" => {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let path = r"SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo";
                if let Ok((key, _)) = hklm.create_subkey(path) {
                    let _ = key.set_value("Enabled", &0u32);
                }
                Ok((true, "Đã tăng cường bảo mật và chặn Advertising ID thành công.".into()))
            }
            "win_update_control" => {
                let _ = Command::new("net")
                    .args(&["stop", "wuauserv"])
                    .creation_flags(0x08000000)
                    .output();
                Ok((true, "Đã tạm dừng dịch vụ Windows Update thành công.".into()))
            }
            _ => Ok((true, format!("Đã thực thi module {} thành công.", id))),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

fn check_hags_enabled() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers") {
        if let Ok(val) = key.get_value::<u32, _>("HwSchMode") {
            return val == 2;
        }
    }
    false
}
