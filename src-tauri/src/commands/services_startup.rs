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

/// Lấy trạng thái runtime của một service (Running/Stopped/Pending/Terminating...).
fn service_status(name: &str) -> String {
    let out = ps_quiet(
        &format!(
            r#"
$ErrorActionPreference = 'SilentlyContinue'
(Get-Service -Name '{}' -ErrorAction SilentlyContinue).Status
"#,
            name
        ),
        10,
    );
    let s = out.trim().to_string();
    if s.is_empty() { "Unknown".to_string() } else { s }
}

/// Lấy StartType của một service (Auto/Demand/Disabled).
fn service_start_type(name: &str) -> String {
    let out = ps_quiet(
        &format!(
            r#"
$ErrorActionPreference = 'SilentlyContinue'
$svc = Get-CimInstance Win32_Service -Filter "Name='{0}'"
if ($svc) {{ $svc.StartMode }} else {{ 'Unknown' }}
"#,
            name
        ),
        10,
    );
    let s = out.trim().to_string();
    if s.is_empty() { "Unknown".to_string() } else { s }
}

/// Lấy DisplayName của một service (dành cho UI thân thiện).
fn service_display_name(name: &str) -> String {
    let out = ps_quiet(
        &format!(
            r#"
$ErrorActionPreference = 'SilentlyContinue'
$svc = Get-CimInstance Win32_Service -Filter "Name='{0}'"
if ($svc) {{ $svc.DisplayName }} else {{ '{0}' }}
"#,
            name
        ),
        10,
    );
    let s = out.trim().to_string();
    if s.is_empty() { name.to_string() } else { s }
}

/// Trả về danh sách dịch vụ được monitor + trạng thái startup hiện tại.
#[tauri::command]
pub async fn get_service_startup_items() -> Vec<ServiceStartupItem> {
    spawn_blocking(|| {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (name, category, desc) in SERVICE_CATEGORIES.iter() {
            if seen.contains(name) { continue; }
            seen.insert(name);
            let display_name = service_display_name(name);
            let status = service_status(name);
            let start_type = service_start_type(name);
            let id = format!("svc_{}", name.to_lowercase());

            result.push(ServiceStartupItem {
                id,
                name: name.to_string(),
                display_name,
                status,
                start_type,
                category: category.to_string(),
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

