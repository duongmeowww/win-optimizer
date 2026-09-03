use super::*;
use std::time::{Duration, Instant};
use std::sync::{Mutex, OnceLock};

pub mod gaming;
pub mod gaming_advanced;
pub mod diagnostics;
pub mod advanced;
pub mod debloat;
pub mod services_startup;
pub mod memory_cleaner;
pub mod app_manager;
pub mod duplicate_finder;
pub mod registry_tweaks;
pub mod advanced_hub;

#[tauri::command]
pub async fn run_benchmark() -> String {
    tauri::async_runtime::spawn_blocking(run_benchmark_inner)
        .await
        .unwrap_or_default()
}

fn run_benchmark_inner() -> String {
    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
try {
    $ramAvailMB = (Get-Counter '\Memory\Available MBytes' -MaxSamples 1 |
        Select-Object -ExpandProperty CounterSamples |
        Measure-Object -Property CookedValue -Average).Average
    $diskActivePct = (Get-Counter '\PhysicalDisk(_Total)\% Disk Time' -MaxSamples 1 |
        Select-Object -ExpandProperty CounterSamples |
        Measure-Object -Property CookedValue -Average).Average
    $cpuPct = (Get-Counter '\Processor(_Total)\% Processor Time' -MaxSamples 1 |
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

#[derive(serde::Serialize, Default, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub clock_mhz: u64,
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
    pub gpu_usage: f64,
    pub gpu_list: Vec<GpuInfo>,
    pub os_version: String,
    pub uptime_seconds: u64,
}

/// Dữ liệu tĩnh (ít thay đổi) — cache lại để không phải query PowerShell/WMI mỗi lần poll.
struct SysSnapshot {
    fetched: Instant,
    cpu_name: String,
    os_version: String,
    boot_epoch_s: u64,
    disk_total: u64,
    disk_free: u64,
    disk_info: Vec<DiskInfo>,
    gpu_name: String,
    gpu_clock_mhz: u64,
    gpu_list: Vec<GpuInfo>,
}

static SYS_SNAPSHOT: OnceLock<Mutex<SysSnapshot>> = OnceLock::new();
static SYS: OnceLock<Mutex<System>> = OnceLock::new();
const SNAPSHOT_TTL: Duration = Duration::from_secs(30);

fn fetch_snapshot() -> SysSnapshot {
    let ps = sh_timeout("powershell", &[
        "-NoProfile", "-Command", r#"
$ErrorActionPreference = 'SilentlyContinue'
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$os  = Get-CimInstance Win32_OperatingSystem
$disks = Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=3'
$gpus = Get-CimInstance Win32_VideoController | Select-Object Name, CurrentClockSpeed
$dLines = @(); $dTotal = [uint64]0; $dFree = [uint64]0
foreach ($d in $disks) {
    $t = [uint64]$d.Size; $f = [uint64]$d.FreeSpace
    $dTotal += $t; $dFree += $f
    $dLines += "$($d.DeviceID)|$t|$f"
}
$boot = [datetime]$os.LastBootUpTime
$uptimeSec = [int64]((Get-Date) - $boot).TotalSeconds
Write-Output "CPU_NAME=$($cpu.Name)"
Write-Output "OS=$($os.Caption) $($os.Version)"
Write-Output "UPTIME=$uptimeSec"
foreach ($dl in $dLines) { Write-Output "DISK=$dl" }
Write-Output "DISK_TOTAL=$dTotal"; Write-Output "DISK_FREE=$dFree"
if ($gpus) {
    foreach ($g in $gpus) { Write-Output "GPU=$($g.Name)|$($g.CurrentClockSpeed)" }
} else { Write-Output 'GPU=N/A|0' }
"#
    ], PS_TIMEOUT);

    let mut s = SysSnapshot {
        fetched: Instant::now(),
        cpu_name: String::new(),
        os_version: String::new(),
        boot_epoch_s: 0,
        disk_total: 0,
        disk_free: 0,
        disk_info: Vec::new(),
        gpu_name: String::new(),
        gpu_clock_mhz: 0,
        gpu_list: Vec::new(),
    };
    for line in ps.lines() {
        match line {
            l if l.starts_with("CPU_NAME=") => s.cpu_name = l[9..].trim().to_string(),
            l if l.starts_with("OS=") => s.os_version = l[3..].trim().to_string(),
            l if l.starts_with("UPTIME=") => {
                if let Ok(v) = l[7..].trim().parse::<u64>() {
                    s.boot_epoch_s = chrono::Utc::now().timestamp().max(0) as u64 - v;
                }
            }
            l if l.starts_with("DISK=") => {
                let p: Vec<&str> = l[5..].split('|').collect();
                if p.len() == 3 {
                    s.disk_info.push(DiskInfo {
                        name: p[0].to_string(),
                        total_bytes: p[1].parse().unwrap_or(0),
                        free_bytes: p[2].parse().unwrap_or(0),
                    });
                }
            }
            l if l.starts_with("DISK_TOTAL=") => {
                if let Some((_, v)) = l.split_once('=') {
                    s.disk_total = v.trim().parse().unwrap_or(0);
                }
            }
            l if l.starts_with("DISK_FREE=") => {
                if let Some((_, v)) = l.split_once('=') {
                    s.disk_free = v.trim().parse().unwrap_or(0);
                }
            }
            l if l.starts_with("GPU=") => {
                let p: Vec<&str> = l[4..].split('|').collect();
                let g_name = p.first().map(|x| x.trim().to_string()).unwrap_or_default();
                let g_clock = p.get(1).and_then(|x| x.trim().parse().ok()).unwrap_or(0);
                if s.gpu_name.is_empty() {
                    s.gpu_name = g_name.clone();
                    s.gpu_clock_mhz = g_clock;
                }
                s.gpu_list.push(GpuInfo {
                    name: g_name,
                    clock_mhz: g_clock,
                });
            }
            _ => {}
        }
    }
    s
}

#[tauri::command]
pub async fn get_sys_info() -> SystemInfo {
    tauri::async_runtime::spawn_blocking(|| {
        let snap_mutex = SYS_SNAPSHOT.get_or_init(|| Mutex::new(SysSnapshot {
            fetched: Instant::now() - SNAPSHOT_TTL,
            cpu_name: String::new(),
            os_version: String::new(),
            boot_epoch_s: 0,
            disk_total: 0,
            disk_free: 0,
            disk_info: Vec::new(),
            gpu_name: String::new(),
            gpu_clock_mhz: 0,
            gpu_list: Vec::new(),
        }));
        let sys_mutex = SYS.get_or_init(|| Mutex::new(System::new()));

        // Chỉ chạy PowerShell/WMI nặng khi cache hết hạn (mỗi 30s)
        let mut snap = snap_mutex.lock().unwrap();
        if snap.fetched.elapsed() > SNAPSHOT_TTL {
            *snap = fetch_snapshot();
        }

        // GPU utilization động (nhẹ, mỗi lần gọi) — Get-Counter nhanh ~100ms
        let gpu_usage = fetch_gpu_usage();

        // CPU + RAM lấy từ sysinfo (native, nhẹ) ngay mỗi lần gọi
        let mut sys = sys_mutex.lock().unwrap();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
        let ram_total = sys.total_memory();
        let ram_used = sys.used_memory();

        let uptime_seconds = chrono::Utc::now().timestamp().max(0) as u64 - snap.boot_epoch_s;
        SystemInfo {
            cpu_usage,
            cpu_name: snap.cpu_name.clone(),
            cpu_clock_mhz: 0,
            cpu_temp_c: 0.0,
            ram_total,
            ram_used,
            disk_total: snap.disk_total,
            disk_free: snap.disk_free,
            disk_info: snap.disk_info.clone(),
            gpu_name: snap.gpu_name.clone(),
            gpu_clock_mhz: snap.gpu_clock_mhz,
            gpu_temp_c: 0.0,
            gpu_usage,
            gpu_list: snap.gpu_list.clone(),
            os_version: snap.os_version.clone(),
            uptime_seconds,
        }
    })
    .await
    .unwrap_or_default()
}

/// Lấy GPU utilization tổng hợp (sum tất cả GPU engine, clamp 0-100).
/// Dùng `Get-Counter` (nhanh ~100-200ms), không phải WMI nặng.
fn fetch_gpu_usage() -> f64 {
    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
$samples = Get-Counter '\GPU Engine(*)\Utilization Percentage' -MaxSamples 1 -ErrorAction SilentlyContinue |
    Select-Object -ExpandProperty CounterSamples | Where-Object { $_.CookedValue -gt 0 }
if ($samples) {
    $total = ($samples | Measure-Object -Property CookedValue -Sum).Sum
    [math]::Min(100, [math]::Round($total, 1))
} else {
    0
}
"#;
    let out = sh_timeout("powershell", &["-NoProfile", "-Command", ps], PS_TIMEOUT);
    out.trim().parse::<f64>().unwrap_or(0.0)
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

fn scan_sync() -> DiskCleanScan {
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
pub async fn get_disk_clean_candidates() -> DiskCleanScan {
    tauri::async_runtime::spawn_blocking(scan_sync).await.unwrap_or_default()
}

#[tauri::command]
pub async fn run_disk_clean(ids: Vec<String>) -> u64 {
    tauri::async_runtime::spawn_blocking(move || {
    let scan = scan_sync();
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
    }).await.unwrap_or_default()
}

#[derive(serde::Serialize, Default)]
pub struct MemCleanResult {
    pub freed_kb: u64,
    pub before_avail_kb: u64,
    pub after_avail_kb: u64,
    pub processes_trimmed: u32,
}

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA, PROCESS_VM_OPERATION};

fn mem_available_kb() -> u64 {
    unsafe {
        let mut st = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
        if GlobalMemoryStatusEx(&mut st).is_err() {}
        let avail = st.ullAvailPhys as u64;
        avail / 1024
    }
}

/// Lấy danh sách PID có RAM > ngưỡng (MB) qua WMI — reliable, không cần psapi EnumProcesses.
fn heavy_pids(threshold_mb: u64) -> Vec<u32> {
    let ps = format!(
        r#"
$ErrorActionPreference='SilentlyContinue'
Get-Process | Where-Object {{ $_.WorkingSet64 -gt {}MB }} | ForEach-Object {{ $_.Id }}
"#,
        threshold_mb
    );
    let out = sh_timeout("powershell", &["-NoProfile", "-Command", &ps], Duration::from_secs(15));
    out.lines().filter_map(|l| l.trim().parse::<u32>().ok()).collect()
}

#[tauri::command]
pub fn clean_memory(threshold_mb: Option<u32>) -> MemCleanResult {
    let threshold = threshold_mb.unwrap_or_else(|| 50u32);
    let before = mem_available_kb();
    let pids = heavy_pids(threshold as u64);
    let mut trimmed = 0u32;
    let rights: PROCESS_ACCESS_RIGHTS =
        PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA | PROCESS_VM_OPERATION;
    for pid in pids {
        if let Ok(h) = unsafe { OpenProcess(rights, false, pid) } {
            let _ = unsafe { EmptyWorkingSet(h) };
            let _ = unsafe { CloseHandle(h) };
            trimmed += 1;
        }
    }
    // chờ một chút để kernel xử lý release trước khi đo after
    std::thread::sleep(Duration::from_millis(250));
    let after = mem_available_kb();
    let freed = after.saturating_sub(before);
    MemCleanResult { freed_kb: freed, before_avail_kb: before, after_avail_kb: after, processes_trimmed: trimmed }
}
