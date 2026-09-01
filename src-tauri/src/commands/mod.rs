use super::*;
use std::time::Duration;

#[tauri::command]
pub fn run_benchmark() -> String {
    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
try {
    $ramAvailMB = (Get-Counter '\Memory\Available MBytes' -SampleInterval 1 -MaxSamples 2 |
        Select-Object -ExpandProperty CounterSamples |
        Measure-Object -Property CookedValue -Average).Average
    $diskActivePct = (Get-Counter '\PhysicalDisk(_Total)\% Disk Time' -SampleInterval 1 -MaxSamples 2 |
        Select-Object -ExpandProperty CounterSamples |
        Measure-Object -Property CookedValue -Average).Average
    $cpuPct = (Get-Counter '\Processor(_Total)\% Processor Time' -SampleInterval 1 -MaxSamples 2 |
        Select-Object -ExpandProperty CounterSamples |
        Measure-Object -Property CookedValue -Average).Average
    $appData = "$env:LOCALAPPDATA\Microsoft\Windows"
    $openMs = (Measure-Command { Get-ChildItem $appData -Force -ErrorAction SilentlyContinue | Out-Null }).TotalMilliseconds
    $heavyProcs = (Get-Process | Where-Object { $_.WorkingSet64 -gt 500MB }).Count
    [PSCustomObject]@{
        timestamp = (Get-Date -Format 'o')
        ram_available_mb = [math]::Round($ramAvailMB,0)
        disk_active_pct = [math]::Round($diskActivePct,1)
        cpu_processor_pct = [math]::Round($cpuPct,1)
        appdata_open_ms = [math]::Round($openMs,0)
        heavy_processes = $heavyProcs
    } | ConvertTo-Json -Compress
} catch {
    $os = Get-CimInstance Win32_OperatingSystem
    [PSCustomObject]@{
        timestamp = (Get-Date -Format 'o')
        ram_available_mb = [math]::Round($os.FreePhysicalMemory,0)
        disk_active_pct = $null
        cpu_processor_pct = $null
        appdata_open_ms = $null
        heavy_processes = $null
    } | ConvertTo-Json -Compress
}
"#;
    sh_timeout("powershell", &["-NoProfile", "-Command", ps], Duration::from_secs(20))
}


const PS_TIMEOUT: Duration = Duration::from_secs(8);
const SCAN_TIMEOUT: Duration = Duration::from_secs(30);

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
    pub os_version: String,
    pub uptime_seconds: u64,
}

#[tauri::command]
pub fn get_sys_info() -> SystemInfo {
    let ps_main = sh_timeout("powershell", &[
        "-NoProfile", "-Command", r#"
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$os = Get-CimInstance Win32_OperatingSystem
$disks = Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=3'
$ram = Get-CimInstance Win32_OperatingSystem
$dLines = @()
$dTotal = [uint64]0; $dFree = [uint64]0
foreach ($d in $disks) {
    $t = [uint64]$d.Size; $f = [uint64]$d.FreeSpace
    $dTotal += $t; $dFree += $f
    $dLines += "$($d.DeviceID)|$t|$f"
}
Write-Output "CPU_NAME=$($cpu.Name)"
Write-Output "RAM_TOTAL_KB=$($ram.TotalVisibleMemorySize)"
Write-Output "RAM_FREE_KB=$($ram.FreePhysicalMemory)"
Write-Output "OS=$($os.Caption) $($os.Version)"
Write-Output "UPTIME=$($os.LastBootUpTime)"
foreach ($dl in $dLines) { Write-Output "DISK=$dl" }
Write-Output "DISK_TOTAL=$dTotal"
Write-Output "DISK_FREE=$dFree"
"#
    ], PS_TIMEOUT);
    let ps_gpu = sh_timeout("powershell", &[
        "-NoProfile", "-Command",
        r#"
            $gpus = Get-CimInstance Win32_VideoController | Select-Object Name, CurrentClockSpeed | Select-Object -First 1
            if ($gpus) { Write-Output "$($gpus.Name)|$($gpus.CurrentClockSpeed)" } else { Write-Output 'N/A|0' }
        "#,
    ], PS_TIMEOUT);
    let mut cpu_name = String::new();
    let mut ram_total_kb: u64 = 0;
    let mut ram_free_kb: u64 = 0;
    let mut uptime_seconds: u64 = 0;
    let mut disk_total: u64 = 0;
    let mut disk_free: u64 = 0;
    let mut disk_info: Vec<DiskInfo> = Vec::new();
    let mut os_version = String::new();
    for line in ps_main.lines() {
        match line {
            s if s.starts_with("CPU_NAME=") => { cpu_name = s[9..].trim().to_string(); }
            s if s.starts_with("RAM_TOTAL_KB=") => { ram_total_kb = s[12..].trim().parse().unwrap_or(0); }
            s if s.starts_with("RAM_FREE_KB=") => { ram_free_kb = s[11..].trim().parse().unwrap_or(0); }
            s if s.starts_with("UPTIME=") => { let boot = s[7..].trim(); if let Ok(d) = chrono::DateTime::parse_from_rfc3339(boot) { uptime_seconds = chrono::Utc::now().signed_duration_since(d).num_seconds() as u64; } }
            s if s.starts_with("DISK=") => { let p = s[5..].split('|').collect::<Vec<_>>(); if p.len() == 3 { disk_info.push(DiskInfo { name: p[0].to_string(), total_bytes: p[1].parse().unwrap_or(0), free_bytes: p[2].parse().unwrap_or(0) }); } }
            s if s.starts_with("DISK_TOTAL=") => { disk_total = s[10..].trim().parse().unwrap_or(0); }
            s if s.starts_with("DISK_FREE=") => { disk_free = s[9..].trim().parse().unwrap_or(0); }
            s if s.starts_with("OS=") => { os_version = s[3..].trim().to_string(); }
            _ => {}
        }
    }
    let (gpu_name, gpu_clock) = {
        let parts: Vec<&str> = ps_gpu.split('|').collect();
        (parts.first().map(|s| s.trim().to_string()).unwrap_or_default(),
         parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(0))
    };
    let ram_total = ram_total_kb * 1024;
    let ram_free = ram_free_kb * 1024;
    let ram_used = ram_total - ram_free;
    let cpu_usage_kind = sysinfo::CpuRefreshKind::new().with_cpu_usage();
    let mut sys = System::new_with_specifics(sysinfo::RefreshKind::new().with_cpu(cpu_usage_kind));
    sys.refresh_cpu_usage();
    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
    SystemInfo { cpu_usage, cpu_name, cpu_clock_mhz: 0, cpu_temp_c: 0.0, ram_total, ram_used, disk_total, disk_free, disk_info, gpu_name, gpu_clock_mhz: gpu_clock, gpu_temp_c: 0.0, os_version, uptime_seconds }
}


#[derive(serde::Serialize, Clone, Default)]
pub struct CleanCategory {
    pub id: String,
    pub label: String,
    pub path: String,
    pub size_bytes: u64,
    pub item_count: u64,
    pub exists: bool,
}

#[derive(serde::Serialize, Default)]
pub struct DiskCleanScan {
    pub categories: Vec<CleanCategory>,
    pub total_size: u64,
    pub completed: bool,
}

#[tauri::command]
pub fn get_disk_clean_candidates() -> DiskCleanScan {
    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
$cats = @()
function Get-FolderSize($p) {
    $sz = 0; $cnt = 0
    if (Test-Path $p) {
        $items = Get-ChildItem -Path $p -Force -Recurse -ErrorAction SilentlyContinue
        $cnt = ($items | Measure-Object).Count
        $r = $items | Measure-Object -Property Length -Sum
        $sz = if ($r) { $r.Sum } else { 0 }
    }
    return "$sz|$cnt"
}
$userTemp = $env:TEMP
$v = Get-FolderSize $userTemp; $cats += "TEMP_USER|$v|$userTemp"
$wt = "$env:windir\Temp"
$v = Get-FolderSize $wt; $cats += "WIN_TEMP|$v|$wt"
$rbSize = 0; $rbCount = 0
try {
    $shell = New-Object -ComObject Shell.Application
    $rb = $shell.Namespace(0xA)
    $rbItems = $rb.Items()
    $rbCount = $rbItems.Count
    foreach ($item in $rbItems) { $rbSize += $item.Size }
} catch {}
$cats += "RECYCLE_BIN|$rbSize|$rbCount|Recycle Bin"
$pf = "$env:windir\Prefetch"
$v = Get-FolderSize $pf; $cats += "PREFETCH|$v|$pf"
$chrome = "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache"
$v = Get-FolderSize $chrome; $cats += "CHROME_CACHE|$v|$chrome"
$edge = "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Default\Cache"
$v = Get-FolderSize $edge; $cats += "EDGE_CACHE|$v|$edge"
$wu = "$env:windir\SoftwareDistribution\Download"
$v = Get-FolderSize $wu; $cats += "WIN_UPDATE|$v|$wu"
$thumbs = Get-ChildItem "$env:LOCALAPPDATA\Microsoft\Windows\Explorer" -Filter "thumbcache_*.db" -Force -ErrorAction SilentlyContinue
$thSize = 0; $thCount = 0
if ($thumbs) {
    $thCount = $thumbs.Count
    $r = $thumbs | Measure-Object -Property Length -Sum
    $thSize = if ($r) { $r.Sum } else { 0 }
}
$cats += "THUMBNAILS|$thSize|$thCount|thumbcache"
$dumps = "$env:LOCALAPPDATA\CrashDumps"
$v = Get-FolderSize $dumps; $cats += "CRASH_DUMPS|$v|$dumps"
$wer = "$env:LOCALAPPDATA\Microsoft\Windows\WER"
$v = Get-FolderSize $wer; $cats += "WER|$v|$wer"
$cats
"#;
    let out = sh_timeout("powershell", &["-NoProfile", "-Command", ps], SCAN_TIMEOUT);
    let mut categories = Vec::new();
    let mut total_size: u64 = 0;
    for line in out.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 4 { continue; }
        let id = parts[0].trim();
        let size: u64 = parts[1].trim().parse().unwrap_or(0);
        let count: u64 = parts[2].trim().parse().unwrap_or(0);
        let desc = parts[3].trim();
        total_size += size;
        let label = match id {
            "TEMP_USER" => "TEMP người dùng".into(),
            "WIN_TEMP" => "Windows Temp".into(),
            "RECYCLE_BIN" => "Thùng rác (Recycle Bin)".into(),
            "PREFETCH" => "Prefetch".into(),
            "CHROME_CACHE" => "Chrome Cache".into(),
            "EDGE_CACHE" => "Edge Cache".into(),
            "WIN_UPDATE" => "Windows Update cache".into(),
            "THUMBNAILS" => "Thumbnail cache".into(),
            "CRASH_DUMPS" => "Crash Dumps".into(),
            "WER" => "Windows Error Reporting".into(),
            _ => id.into(),
        };
        let path = if id == "RECYCLE_BIN" { "Recycle Bin".into() } else if id == "THUMBNAILS" { "thumbcache files".into() } else { desc.to_string() };
        categories.push(CleanCategory { id: id.to_string(), label, path, size_bytes: size, item_count: count, exists: true });
    }
    DiskCleanScan { categories, total_size, completed: true }
}

#[tauri::command]
pub fn run_disk_clean(ids: Vec<String>) -> u64 {
    let scan = get_disk_clean_candidates();
    let mut freed: u64 = 0;
    for cat in &scan.categories {
        if !ids.contains(&cat.id) { continue; }
        if cat.path.is_empty() || cat.path == "Recycle Bin" || cat.path == "thumbcache files" { continue; }
        let escaped = cat.path.replace('\\', "\\\\").replace('\'', "\\'");
        let ps = format!(
            r#"
            $p = '{}';
            if (Test-Path $p) {{
  $before = (Get-ChildItem $p -Force -Recurse -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum; if (-not $before) {{ $before = 0 }}
  Get-ChildItem $p -Force -ErrorAction SilentlyContinue | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
  $after = (Get-ChildItem $p -Force -Recurse -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum; if (-not $after) {{ $after = 0 }}
  Write-Output ($before - $after)
}} else {{ Write-Output 0 }}
            "#,
            escaped
        );
        let out = sh_timeout("powershell", &["-NoProfile", "-Command", &ps], SCAN_TIMEOUT);
        if let Ok(v) = out.trim().parse::<u64>() { freed += v; }
    }
    freed
}
