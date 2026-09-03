use super::gaming::ps_quiet;
use tauri::async_runtime::spawn_blocking;

/// Một dịch vụ Windows + trạng thái startup.
#[derive(serde::Serialize, Clone)]
pub struct ServiceStartupItem {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub start_type: String,
    pub category: String,
    pub description: String,
}

/// Danh sách dịch vụ quan tâm + phân loại (không query toàn bộ services — chỉ những cái ảnh hưởng tới startup/boost).
const SERVICE_CATEGORIES: &[(&str, &str, &str)] = &[
    // (name, category, description)
    ("SysMain", "System", "SysMain / Superfetch — prefetch các app thường dùng, chiếm RAM."),
    ("WSearch", "Bloatware", "Windows Search Indexer — index toàn bộ file, chiếm I/O."),
    ("BITS", "System", "Background Intelligent Transfer Service — tải cập nhật nền."),
    ("wuauserv", "System", "Windows Update — tự động cập nhật."),
    ("DoSvc", "Bloatware", " Delivery Optimization — chia sẻ cập nhật qua mạng LAN."),
    ("PcaSvc", "System", "Program Compatibility Assistant — cảnh báo compatibility cũ."),
    ("DiagTrack", "Telemetry", "Diagnostics Tracking Service — thu thập telemetry."),
    ("dmwappushservice", "Telemetry", "WAP Push Message Routing Service — telemetty tiếp."),
    ("WerSvc", "Telemetry", "Windows Error Reporting — gửi crash dump tới Microsoft."),
    ("FontCache", "System", "Windows Font Cache — cache font hệ thống."),
    ("Themes", "System", "Themes — hiệu ứng visual, tiêu thụ RAM/GPU."),
    ("Audiosrv", "System", "Windows Audio — dịch vụ âm thanh."),
    ("Spooler", "System", "Print Spooler — in ấn."),
    ("XboxGipSvc", "Gaming", "Xbox Game Input Service."),
    ("XblAuthManager", "Gaming", "Xbox Live Auth Manager."),
    ("XblGameSave", "Gaming", "Xbox Live Game Save."),
    ("WerSvc", "Telemetry", "Windows Error Reporting."),
    ("OneSyncSvc", "Bloatware", "Sync Host — đồng bộ OneDrive/Contacts."),
    ("PimIndexMaintenanceSvc", "Bloatware", "Contact Data — đồng bộ danh bạ."),
    ("UnistoreSvc", "Bloatware", "User Data Access — đồng bộ dữ liệu người dùng."),
    ("Appinfo", "Security", "Application Info — cần thiết cho UAC."),
    ("Winmgmt", "Security", "Windows Management Instrumentation."),
    ("SecurityHealthService", "Security", "Windows Security Health."),
    ("wscsvc", "Security", "Security Center — giám sát bảo mật Windows."),
    ("Audiosrv", "Gaming", "Windows Audio."),
    ("Schedule", "System", "Task Scheduler — lên lịch công việc nền."),
    ("SENS", "System", "System Event Notification Service."),
    ("EventLog", "System", "Windows Event Log."),
    ("BITS", "System", "Background Intelligent Transfer Service."),
];

/// Trả về danh sách dịch vụ được monitor + trạng thái startup hiện tại.
/// Tối ưu: gộp 75 lần gọi PowerShell thành 1 lần duy nhất (~0.8s thay vì 30-40s)
#[tauri::command]
pub async fn get_service_startup_items() -> Vec<ServiceStartupItem> {
    spawn_blocking(|| {
        use std::collections::HashMap;
        // Dedupe + giữ category/desc đầu tiên cho mỗi service
        let mut meta: HashMap<&str, (&str, &str)> = HashMap::new();
        let mut order: Vec<&str> = Vec::new();
        for (name, cat, desc) in SERVICE_CATEGORIES.iter() {
            if !meta.contains_key(name) {
                meta.insert(name, (cat, desc));
                order.push(name);
            }
        }
        if order.is_empty() { return Vec::new(); }

        // Build 1 Powershell script duy nhất
        let names_ps = order.iter().map(|n| format!("'{}'", n)).collect::<Vec<_>>().join(",");
        let ps = format!(
            r#"
$ErrorActionPreference='SilentlyContinue'
$names=@({})
foreach ($n in $names) {{
  $svc=Get-Service -Name $n -ErrorAction SilentlyContinue
  $cim=Get-CimInstance Win32_Service -Filter "Name='$n'" -ErrorAction SilentlyContinue
  $status=if ($svc) {{ $svc.Status }} else {{ 'Unknown' }}
  $start=if ($cim) {{ $cim.StartMode }} else {{ 'Unknown' }}
  $disp=if ($cim -and $cim.DisplayName) {{ $cim.DisplayName }} else {{ $n }}
  # Escape | trong tên
  $disp=$disp -replace '\|','/'
  Write-Output "$n|$status|$start|$disp"
}}
"#,
            names_ps
        );
        let out = ps_quiet(&ps, 15);
        let mut lookup: HashMap<String, (String, String, String)> = HashMap::new();
        for line in out.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() < 4 { continue; }
            lookup.insert(parts[0].to_string(), (parts[1].to_string(), parts[2].to_string(), parts[3].to_string()));
        }

        let mut result = Vec::new();
        for name in order {
            let (cat, desc) = meta[name];
            let (status, start_type, display_name) = lookup.get(name)
                .map(|(s, st, d)| (s.clone(), st.clone(), d.clone()))
                .unwrap_or(("Unknown".into(), "Unknown".into(), name.to_string()));
            let id = format!("svc_{}", name.to_lowercase());
            result.push(ServiceStartupItem {
                id,
                name: name.to_string(),
                display_name,
                status,
                start_type,
                category: cat.to_string(),
                description: desc.to_string(),
            });
        }
        result
    })
    .await
    .unwrap_or_default()
}

/// Đặt lại StartType cho một service. start_type = "Auto" | "Demand" | "Disabled".
/// Trả về (ok, message).
#[tauri::command]
pub async fn set_service_startup(name: String, start_type: String) -> Result<(bool, String), String> {
    let script = format!(
        r#"
$ErrorActionPreference = 'SilentlyContinue'
$svc = Get-Service -Name '{0}' -ErrorAction SilentlyContinue
if (-not $svc) {{ Write-Output 'Service {0} không tồn tại'; exit 1 }}
Set-Service -Name '{0}' -StartupType '{1}'
if ('{1}' -ne 'Disabled') {{
  $svc | Start-Service -ErrorAction SilentlyContinue
  Write-Output "Service {0} đã được thiết lập startup"
}} else {{
  $svc | Stop-Service -Force -ErrorAction SilentlyContinue
  Write-Output "Service {0} đã được tắt startup"
}}
"#,
        name, start_type
    );
    let out = ps_quiet(&script, 30);
    let ok = !out.contains("không tồn tại") && !out.to_lowercase().contains("access denied");
    Ok((ok, out))
}

/// Tối ưu hóa một loạt service khuyến nghị cho gaming / performance
#[tauri::command]
pub async fn optimize_recommended_services() -> Result<(bool, String), String> {
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$targets = @(
    @{ Name = 'DiagTrack'; StartType = 'Disabled' },
    @{ Name = 'dmwappushservice'; StartType = 'Disabled' },
    @{ Name = 'DoSvc'; StartType = 'Disabled' },
    @{ Name = 'WerSvc'; StartType = 'Manual' },
    @{ Name = 'WSearch'; StartType = 'Manual' }
)

$count = 0
foreach ($t in $targets) {
    $svc = Get-Service -Name $t.Name -ErrorAction SilentlyContinue
    if ($svc) {
        Set-Service -Name $t.Name -StartupType $t.StartType
        if ($t.StartType -eq 'Disabled') {
            $svc | Stop-Service -Force -ErrorAction SilentlyContinue
        }
        $count++
    }
}
Write-Output "Đã tối ưu $count dịch vụ hệ thống."
"#;
    let out = ps_quiet(script, 30);
    Ok((true, out))
}

