use super::*;

#[derive(serde::Serialize, Default)]
pub struct DiagResult {
    pub action: String,
    pub output: String,
    pub ok: bool,
}

/// Logic sửa chữa: DISM /RestoreHealth rồi SFC /scannow. Chạy lâu — cần timeout lớn.
#[tauri::command]
pub fn repair_system_files(_verbose: bool) -> DiagResult {
    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
$out = @()
$out += "=== DISM /RestoreHealth (có thể mất vài phút) ==="
$d = dism /Online /Cleanup-Image /RestoreHealth 2>&1 | Out-String
$out += $d.Trim()
$out += "`n=== SFC /scannow ==="
$s = sfc /scannow 2>&1 | Out-String
$out += $s.Trim()
$out -join "`n"
"#;
    let out = sh_timeout("powershell", &["-NoProfile", "-Command", ps], Duration::from_secs(600));
    DiagResult { action: "repair".into(), output: out.clone(), ok: !out.trim().is_empty() }
}

/// Chkdsk online scan (không cần reboot).
#[tauri::command]
pub fn run_chkdsk_scan() -> DiagResult {
    let ps = "chkdsk /scan 2>&1 | Out-String";
    let out = sh_timeout("powershell", &["-NoProfile", "-Command", ps], Duration::from_secs(120));
    DiagResult { action: "chkdsk".into(), output: out.clone(), ok: !out.trim().is_empty() }
}

/// Đọc event log gần đây: crash app, service lỗi, DCOM — trả top N dòng.
#[tauri::command]
pub fn get_recent_events(limit: Option<u32>) -> DiagResult {
    let _n = limit.unwrap_or(20).min(50);
    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
$rows = @()
# Application errors (crash) 24h
Get-WinEvent -FilterHashtable @{LogName='Application'; Level=2; StartTime=(Get-Date).AddHours(-24)} -MaxEvents 20 -ErrorAction SilentlyContinue |
    Select-Object TimeCreated, Id, ProviderName, @{n='Msg';e={ ($_.Message -split "`n")[0] }} |
    ForEach-Object { $rows += "[ERR] $($_.TimeCreated.ToString('HH:mm')) #$($_.Id) $($_.ProviderName): $($_.Msg)" }
# System errors 24h
Get-WinEvent -FilterHashtable @{LogName='System'; Level=2; StartTime=(Get-Date).AddHours(-24)} -MaxEvents 20 -ErrorAction SilentlyContinue |
    Select-Object TimeCreated, Id, ProviderName, @{n='Msg';e={ ($_.Message -split "`n")[0] }} |
    ForEach-Object { $rows += "[SYS] $($_.TimeCreated.ToString('HH:mm')) #$($_.Id) $($_.ProviderName): $($_.Msg)" }
if ($rows.Count -eq 0) { 'Không có lỗi nghiêm trọng nào trong 24h.' } else { $rows | Select-Object -First 30 }
"#;
    let out = sh_timeout("powershell", &["-NoProfile", "-Command", ps], Duration::from_secs(60));
    DiagResult { action: "events".into(), output: out.clone(), ok: !out.trim().is_empty() }
}

/// Drive SMART health qua WMI.
#[tauri::command]
pub fn get_drive_smart() -> DiagResult {
    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
Get-PhysicalDisk | Select-Object FriendlyName, MediaType, HealthStatus, OperationalStatus, @{n='SizeGB';e={[math]::Round($_.Size/1GB,1)}} |
    ForEach-Object { "$($_.FriendlyName) | $($_.MediaType) | $($_.HealthStatus) | $($_.OperationalStatus) | $($_.SizeGB)GB" }
"#;
    let out = sh_timeout("powershell", &["-NoProfile", "-Command", ps], Duration::from_secs(30));
    DiagResult { action: "smart".into(), output: out.clone(), ok: !out.trim().is_empty() }
}
