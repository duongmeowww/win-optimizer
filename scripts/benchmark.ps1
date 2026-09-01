# Benchmark chuẩn cho WinOptimizer
# Dùng chung cho mọi phase để đo trước/sau khi tối ưu.
# Output: JSON (dễ cho app/frontend hiển thị).

$ErrorActionPreference = 'SilentlyContinue'

try {
    # 1. RAM available (MB)
    $ramAvailMB = (Get-Counter '\Memory\Available MBytes' -SampleInterval 1 -MaxSamples 2 |
        Select-Object -ExpandProperty CounterSamples |
        Measure-Object -Property CookedValue -Average).Average

    # 2. Disk % active time (oC)
    $diskActivePct = (Get-Counter '\PhysicalDisk(_Total)\% Disk Time' -SampleInterval 1 -MaxSamples 2 |
        Select-Object -ExpandProperty CounterSamples |
        Measure-Object -Property CookedValue -Average).Average

    # 3. CPU idle th?c (processor time cang thap cang toi uu)
    $cpuPct = (Get-Counter '\Processor(_Total)\% Processor Time' -SampleInterval 1 -MaxSamples 2 |
        Select-Object -ExpandProperty CounterSamples |
        Measure-Object -Property CookedValue -Average).Average

    # 4. Th?i gian m? AppData (do c?m giac "mem app nhanh")
    $appData = "$env:LOCALAPPDATA\Microsoft\Windows"
    $openMs = (Measure-Command { Get-ChildItem $appData -Force -ErrorAction SilentlyContinue | Out-Null }).TotalMilliseconds

    # 5. So luong tien trinh nang (README/background)
    $heavyProcs = (Get-Process | Where-Object { $_.WorkingSet64 -gt 500MB }).Count

    $result = [PSCustomObject]@{
        timestamp        = (Get-Date -Format 'o')
        ram_available_mb = [math]::Round($ramAvailMB, 0)
        disk_active_pct  = [math]::Round($diskActivePct, 1)
        cpu_processor_pct= [math]::Round($cpuPct, 1)
        appdata_open_ms  = [math]::Round($openMs, 0)
        heavy_processes  = $heavyProcs
    }

    $result | ConvertTo-Json
}
catch {
    # Neu khong co quyen doc counter, fallback sang cach khac
    $os = Get-CimInstance Win32_OperatingSystem
    $freeGB = [math]::Round($os.FreePhysicalMemory / 1MB, 2)
    $result = [PSCustomObject]@{
        timestamp         = (Get-Date -Format 'o')
        ram_available_mb  = [math]::Round($os.FreePhysicalMemory, 0)
        disk_active_pct   = $null
        cpu_processor_pct = $null
        appdata_open_ms   = $null
        heavy_processes   = $null
        note              = "counter-permission-denied, ram=free_mb"
    }
    $result | ConvertTo-Json
}
