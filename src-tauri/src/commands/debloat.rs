use crate::commands::gaming::ps_quiet;
use tauri::async_runtime::spawn_blocking;

/// Một hạng mục debloat. Mỗi hạng mục có id, label, desc, category, admin (cần quyền admin?),
/// và `enabled` = trạng thái hiện tại (tính từ thời điểm gọi).
#[derive(serde::Serialize, Clone)]
pub struct DebloatItem {
    pub id: String,
    pub label: String,
    pub desc: String,
    pub category: String, // "Bloatware" | "Telemetry" | "Service" | "Privacy"
    pub admin: bool,
    pub enabled: bool, // đang bật? (true = app/feature đang hiện diện hoặc chưa tắt)
}

/// Trả về danh sách hạng mục debloat + trạng thái hiện tại.
#[tauri::command]
pub async fn get_debloat_items() -> Vec<DebloatItem> {
    spawn_blocking(move || {
    let appx = installed_appx_names();
    let sysmain = reg_dword(r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management", "PrefetchParameters", "EnableSuperfetch");
    let hibernate_off = ps_quiet("(Test-Path \"$env:SystemDrive\\hiberfil.sys\") -eq $false -or ((Get-Item \"$env:SystemDrive\\hiberfil.sys\" -ErrorAction SilentlyContinue).Length -eq 0)", 10).trim() == "True";

    vec![
        // ============ IoT Bloatware (AppX) ============
        DebloatItem {
            id: "appx_spotify".into(),
            label: "Spotify Music".into(),
            desc: "Gỡ bỏ gói Spotify Music (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "Spotify"),
        },
        DebloatItem {
            id: "appx_skype".into(),
            label: "Skype".into(),
            desc: "Gỡ bỏ gói Skype (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "SkypeApp"),
        },
        DebloatItem {
            id: "appx_onedrive".into(),
            label: "OneDrive".into(),
            desc: "Gỡ bỏ OneDrive (AppX + ẩn khỏi File Explorer)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "OneDriveSync") || contains_appx(&appx, "Microsoft.OneDrive"),
        },
        DebloatItem {
            id: "appx_tiktok".into(),
            label: "TikTok".into(),
            desc: "Gỡ bỏ gói TikTok (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "TikTok"),
        },
        DebloatItem {
            id: "appx_clipchamp".into(),
            label: "Clipchamp".into(),
            desc: "Gỡ bỏ gói Clipchamp (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "Clipchamp"),
        },
        DebloatItem {
            id: "appx_teams".into(),
            label: "Microsoft Teams".into(),
            desc: "Gỡ bỏ gói Microsoft Teams (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "Teams") && !contains_appx(&appx, "enSpace"),
        },
        DebloatItem {
            id: "appx_news".into(),
            label: "Bing News / Tin tức".into(),
            desc: "Gỡ bỏ gói Bing News (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "BingNews"),
        },
        DebloatItem {
            id: "appx_maps".into(),
            label: "Windows Maps".into(),
            desc: "Gỡ bỏ gói Windows Maps (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "WindowsMaps") || contains_appx(&appx, "Maps"),
        },
        DebloatItem {
            id: "appx_candycrush".into(),
            label: "Candy Crush".into(),
            desc: "Gỡ bỏ gói Candy Crush Saga (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "CandyCrush"),
        },
        DebloatItem {
            id: "appx_solitaire".into(),
            label: "Microsoft Solitaire".into(),
            desc: "Gỡ bỏ gói Microsoft Solitaire Collection (AppX)".into(),
            category: "Bloatware".into(),
            admin: false,
            enabled: contains_appx(&appx, "Solitaire"),
        },
        // ============ Telemetry / Privacy ============
        DebloatItem {
            id: "telemetry_diag".into(),
            label: "Tắt Telemetry (DiagTrack)".into(),
            desc: "Vô hiệu hóa dịch vụ Diagnostics Tracking Service (DiagTrack).".into(),
            category: "Telemetry".into(),
            admin: true,
            enabled: service_exists_running("DiagTrack"),
        },
        DebloatItem {
            id: "telemetry_ceip".into(),
            label: "Tắt CEIP / Data Collection".into(),
            desc: "Tắt chương trình cải thiện trải nghiệm khách hàng (CEIP) qua registry.".into(),
            category: "Telemetry".into(),
            admin: true,
            enabled: reg_dword(r"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "", "AllowTelemetry") != 0,
        },
        DebloatItem {
            id: "telemetry_wipe".into(),
            label: "Xóa lịch sử Telemetry".into(),
            desc: "Xóa các file/event log telemetry đã thu thập (compat, WER).".into(),
            category: "Telemetry".into(),
            admin: true,
            enabled: true,
        },
        // ============ Services (nhẹ, không ảnh hưởng hệ thống) ============
        DebloatItem {
            id: "svc_wsearch".into(),
            label: "Tắt Windows Search Indexer".into(),
            desc: "Vô hiệu hóa dịch vụ WSearch (index). Search vẫn chạy nhưng không index nền.".into(),
            category: "Service".into(),
            admin: true,
            enabled: service_start_type("WSearch") != "Disabled",
        },
        DebloatItem {
            id: "svc_bits".into(),
            label: "Tắt BITS (Background Transfer)".into(),
            desc: "Vô hiệu hóa dịch vụ BITS dùng để tải cập nhật nền.".into(),
            category: "Service".into(),
            admin: true,
            enabled: service_start_type("BITS") != "Disabled",
        },
        // ============ System tweaks ============
        DebloatItem {
            id: "sys_hibernate".into(),
            label: "Tắt Hibernate".into(),
            desc: "powercfg /hibernate off — xóa hiberfil.sys (thường 3-6GB), tắt chế độ ngủ đông.".into(),
            category: "System".into(),
            admin: true,
            enabled: !hibernate_off,
        },
        DebloatItem {
            id: "sys_sysmain".into(),
            label: "Tắt SysMain (Superfetch)".into(),
            desc: "Tắt dịch vụ SysMain (prefetch các app thường dùng).".into(),
            category: "System".into(),
            admin: true,
            enabled: sysmain != 0,
        },
    ]
    }).await.unwrap_or_default()
}

/// Trạng thái enabled / action. action = "apply" (tắt) | "revert" (bật lại).
/// Trả (ok, message).
#[tauri::command]
pub fn apply_debloat(id: String, action: String) -> Result<(bool, String), String> {
    let script: Result<String, String> = match (id.as_str(), action.as_str()) {
        ("appx_spotify", "apply") => Ok(appx_remove("Spotify")),
        ("appx_spotify", "revert") => Err("Không thể cài lại Spotify — dùng Microsoft Store.".into()),
        ("appx_skype", "apply") => Ok(appx_remove("SkypeApp")),
        ("appx_skype", "revert") => Err("Không thể cài lại Skype — dùng Microsoft Store.".into()),
        ("appx_onedrive", "apply") => Ok(appx_remove("OneDriveSync") + r#"
# Ẩn OneDrive khỏi File Explorer
New-Item -Path 'HKCU:\Software\Classes\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}' -Force | Out-Null
Set-ItemProperty -Path 'HKCU:\Software\Classes\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}' -Name 'System.IsPinnedToNameSpaceTree' -Value 0 -Type DWord
"#),
        ("appx_onedrive", "revert") => Err("Không thể cài lại OneDrive — dùng OneDrive setup.".into()),
        ("appx_tiktok", "apply") => Ok(appx_remove("TikTok")),
        ("appx_tiktok", "revert") => Err("Không thể cài lại TikTok — dùng Microsoft Store.".into()),
        ("appx_clipchamp", "apply") => Ok(appx_remove("Clipchamp")),
        ("appx_clipchamp", "revert") => Err("Không thể cài lại Clipchamp — dùng Microsoft Store.".into()),
        ("appx_teams", "apply") => Ok(appx_remove("Teams")),
        ("appx_teams", "revert") => Err("Không thể cài lại Teams — dùng Microsoft Store.".into()),
        ("appx_news", "apply") => Ok(appx_remove("BingNews")),
        ("appx_news", "revert") => Err("Không thể cài lại Bing News — dùng Microsoft Store.".into()),
        ("appx_maps", "apply") => Ok(appx_remove("WindowsMaps")),
        ("appx_maps", "revert") => Err("Không thể cài lại Maps — dùng Microsoft Store.".into()),
        ("appx_candycrush", "apply") => Ok(appx_remove("CandyCrush")),
        ("appx_candycrush", "revert") => Err("Không thể cài lại Candy Crush — dùng Microsoft Store.".into()),
        ("appx_solitaire", "apply") => Ok(appx_remove("Solitaire")),
        ("appx_solitaire", "revert") => Err("Không thể cài lại Solitaire — dùng Microsoft Store.".into()),

        ("telemetry_diag", "apply") => Ok(r#"
sc.exe stop DiagTrack -ErrorAction SilentlyContinue | Out-Null
sc.exe config DiagTrack start= disabled | Out-Null
"#.into()),
        ("telemetry_diag", "revert") => Ok(r#"
sc.exe config DiagTrack start= demand | Out-Null
sc.exe start DiagTrack -ErrorAction SilentlyContinue | Out-Null
"#.into()),
        ("telemetry_ceip", "apply") => Ok(r#"
New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection' -Force | Out-Null
Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection' -Name 'AllowTelemetry' -Value 0 -Type DWord
Set-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection' -Name 'AllowTelemetry' -Value 0 -Type DWord -ErrorAction SilentlyContinue
"#.into()),
        ("telemetry_ceip", "revert") => Ok(r#"
Remove-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection' -Name 'AllowTelemetry' -ErrorAction SilentlyContinue
Remove-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection' -Name 'AllowTelemetry' -ErrorAction SilentlyContinue
"#.into()),
        ("telemetry_wipe", "apply") => Ok(r#"
$paths = @(
  'C:\ProgramData\Microsoft\Diagnosis',
  'C:\ProgramData\Microsoft\Windows\WER',
  "$env:LOCALAPPDATA\Microsoft\Windows\WER"
)
foreach ($p in $paths) {
  if (Test-Path $p) { Get-ChildItem $p -Recurse -Force -ErrorAction SilentlyContinue | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue }
}
"#.into()),
        ("telemetry_wipe", "revert") => Ok("Không thể khôi phục telemetry đã xóa (dữ liệu đã mất).".into()),

        ("svc_wsearch", "apply") => Ok(r#"
sc.exe config WSearch start= disabled | Out-Null
sc.exe stop WSearch -ErrorAction SilentlyContinue | Out-Null
"#.into()),
        ("svc_wsearch", "revert") => Ok(r#"
sc.exe config WSearch start= auto | Out-Null
sc.exe start WSearch -ErrorAction SilentlyContinue | Out-Null
"#.into()),
        ("svc_bits", "apply") => Ok(r#"
sc.exe config BITS start= disabled | Out-Null
sc.exe stop BITS -ErrorAction SilentlyContinue | Out-Null
"#.into()),
        ("svc_bits", "revert") => Ok(r#"
sc.exe config BITS start= demand | Out-Null
sc.exe start BITS -ErrorAction SilentlyContinue | Out-Null
"#.into()),

        ("sys_hibernate", "apply") => Ok(r#"powercfg /hibernate off 2>&1 | Out-String"#.into()),
        ("sys_hibernate", "revert") => Ok(r#"powercfg /hibernate on 2>&1 | Out-String"#.into()),
        ("sys_sysmain", "apply") => Ok(r#"
Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management' -Name 'EnableSuperfetch' -Value 0 -Type DWord
sc.exe config SysMain start= disabled | Out-Null
sc.exe stop SysMain -ErrorAction SilentlyContinue | Out-Null
"#.into()),
        ("sys_sysmain", "revert") => Ok(r#"
Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management' -Name 'EnableSuperfetch' -Value 3 -Type DWord
sc.exe config SysMain start= auto | Out-Null
sc.exe start SysMain -ErrorAction SilentlyContinue | Out-Null
"#.into()),

        _ => return Err(format!("Hạng mục/action không hợp lệ: {} {}", id, action)),
    };

    let script = script?;
    // Các hạng mục admin cần elevated; nếu đang chạy non-admin thì lệnh HKLM/sc sẽ fail.
    let out = ps_quiet(&script, 60);
    Ok((!out.to_lowercase().contains("access denied"), out))
}

/// Áp một loạt hạng mục (bao gồm cả apply lẫn revert). Trả về map id -> (ok, msg).
#[tauri::command]
pub async fn apply_debloat_batch(items: Vec<(String, String)>) -> Vec<(String, bool, String)> {
    spawn_blocking(move || {
        items.into_iter().map(|(id, action)| {
            let r = apply_debloat(id.clone(), action.clone());
            match r {
                Ok((ok, msg)) => (id, ok, msg),
                Err(e) => (id, false, e),
            }
        }).collect()
    }).await.unwrap_or_default()
}

// ============ helpers ============

/// Trả về danh sách tên AppX đã cài đặt (non-admin — chỉ đọc user packages).
fn installed_appx_names() -> Vec<String> {
    let ps = r#"
Get-AppxPackage | Select-Object -ExpandProperty Name -ErrorAction SilentlyContinue
"#;
    let out = ps_quiet(ps, 15);
    out.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}
fn contains_appx(names: &[String], q: &str) -> bool {
    names.iter().any(|n| n == q)
}

/// Remove một AppX package by family name (non-admin, user-scoped).
fn appx_remove(family: &str) -> String {
    format!(
        r#"Get-AppxPackage -Name "*{0}*" | Remove-AppxPackage -ErrorAction SilentlyContinue
Write-Output "Removed {0}"
"#,
        family
    )
}

/// Đọc DWORD từ registry (HKEY_LOCAL_MACHINE). Trả về 0 nếu không tồn tại.
fn reg_dword(key: &str, subkey: &str, name: &str) -> u32 {
    let ps = format!(
        r#"
try {{
  (Get-ItemProperty -Path 'HKLM:\{0}\{1}' -Name '{2}' -ErrorAction Stop).'{2}'
}} catch {{
  0
}}
"#,
        key, subkey, name
    );
    let val = ps_quiet(&ps, 10).trim().to_string();
    val.parse().unwrap_or(0)
}

/// Kiểm tra service có đang chạy không.
fn service_exists_running(name: &str) -> bool {
    let out = ps_quiet(
        &format!("(Get-Service -Name '{}' -ErrorAction SilentlyContinue).Status", name),
        10,
    );
    out.trim().to_lowercase() == "running"
}

/// Lấy start type của service: "Auto" | "Demand" | "Disabled" | "" nếu không tồn tại.
fn service_start_type(name: &str) -> String {
    let out = ps_quiet(
        &format!(
            r#"
(Get-CimInstance Win32_Service -Filter "Name='{}'").StartMode
"#,
            name
        ),
        10,
    );
    out.trim().to_string()
}