# WinOptimizer — Gaming Features Research

> Ngày: 2026-09-01
> Mục đích: Tham khảo các GitHub repo tối ưu Windows 11 cho gaming để thêm tính năng vào WinOptimizer (Tauri 2 + React + Rust)

---

## 📌 Tổng quan 6 repo đáng tham khảo

### 1. [jonax1337/Reclaim](https://github.com/jonax1337/Reclaim) ⭐ CÙNG STACK (Tauri 2)
- **Stack:** Tauri 2 + Svelte 5 + Rust + TypeScript
- **Điểm mạnh:** 228 reversible tweaks, 12 categories, **live state detection**, per-tweak revert, Mica UI
- **Gaming category:** 35-38 tweaks — Game Mode, MMCSS scheduling, CPU foreground priority, HAGS, TDR delay, mouse/keyboard latency, HID power off, TCP ack/no-delay, QoS bandwidth, NIC EEE/flow-control/interrupt-moderation, AFD buffers, HPET off, TSC enhanced, fullscreen-opt off, NVIDIA telemetry off
- **Esports category:** 48 tweaks (aggressive) — + core parking off, NIC tuning
- **Killer features:**
  - **Gaming Session** — snapshot system state → suspend background apps → switch power plan → pause Defender → End = revert mọi thứ (crash-safe)
  - **Per-game profiles** — HKCU `UserGpuPreferences` (GPU preference), Fullscreen Optimizations off, HighDPIAware, Run-as-admin via AppCompat `Layers` key. **HKCU only, không cần admin!**
  - **MSI mode editor** — per-PCI device MSI/MSI-X toggle
  - **NIC tuning editor** — `Get-NetAdapterAdvancedProperty` live editor, 16 latency props
  - **Latency monitor** — live ping sparklines (Steam/Riot/Epic/Cloudflare/Google/GitHub), 2/5/10s polling, 60-sample stats
  - **Headless CLI mode** — `reclaim.exe --apply-profile basics --silent` (cho sysadmin/Intune)
  - **Restore on demand** từ backup

### 2. [mojouto3/mojo-gaming-mode](https://github.com/mojouto3/mojo-gaming-mode) ⭐ UX PATTERN
- **Stack:** Electron (nhưng UX patterns rất tốt)
- **Presets:** Balanced / Performance / Esports / Custom
- **Tweaks:** Game Mode, SysMain disable, High perf power plan, Search disable, FSO override, HPET disable, MSI mode, Xbox Game Bar off, Steam overlay off, QoS off, Nagle disable
- **Killer features:**
  - GPU vendor auto-detect (NVIDIA/AMD/Intel) → theme theo vendor
  - **Overlay modes: Mini mode (always-on-top stats card) + Bar mode (thin draggable overlay in-game)**
  - **Before/after performance snapshot** khi bật
  - Import/export custom rules JSON
  - Global hotkeys: Ctrl+G toggle, Ctrl+B/P/E presets
  - **Crash recovery** — auto-revert tweaks nếu app bị force-close
  - i18n 10 languages

### 3. [maxrenke/game-optimizer](https://github.com/maxrenke/game-optimizer) ⭐ PROCESS MANAGEMENT
- **Stack:** C# .NET 10 + WinUI 3
- **Core idea:** CPU affinity management tự động
  - **Auto-pin game** vào P-cores (Intel hybrid) / all cores (AMD) + High priority
  - **Isolate media** (Firefox/VLC) sang cores riêng
  - **Demote background** (OneDrive, iCloud, AV) → BelowNormal + low I/O
  - **Suspend cloud sync** trong session game
  - **Timer resolution** `timeBeginPeriod(1)` — 15ms → 1ms scheduling jitter
  - **Win32PrioritySeparation = 26** (bỏ foreground boost penalty)
  - **Lock GPU clocks** (NVIDIA `nvidia-smi --lock-gpu-clocks`) — hết P-state oscillation
- **Killer features:**
  - WMI `Win32_ProcessStartTrace` — detect game instant, không cần polling
  - **Game library auto-scan** (Steam/Epic/GOG/Ubi/EA)
  - Per-zone CPU tracking (game / media / background)
  - Bottleneck detector (CPU-bound / GPU-bound)
  - 154 tests, `Reset-Optimizer.ps1` emergency reset

### 4. [mohammad-javad-asadi/revx](https://github.com/mohammad-javad-asadi/revx)
- **Stack:** Kernel optimizer (low-level)
- **3 profiles:** Smart Gaming / Max Performance / Balanced / RevX Cool (thermal-aware)
- **Tweaks:** micro-stutter elimination (thread priority dynamic), input lag reduction (timer resolution), network packet shaping (SMBv3, bufferbloat), MMCSS recalibration
- **Killer features:**
  - **DNS Benchmark Engine** — test Cloudflare/Google/NextDNS/Quad9 → auto-apply lowest latency
  - **Ookla Speedtest client tích hợp**
  - **Live Game Ping Diagnostic** — ICMP routes tới datacenter Steam/Riot/Epic/Blizzard

### 5. [theantipopau/windows11nontouchgamingoptimizer](https://github.com/theantipopau/windows11nontouchgamingoptimizer) ⭐ REGISTRY TWEAKS
- **Batch script menu-driven, reversible, logged**
- **Key tweaks:**
  - Ultimate Performance power plan
  - HAGS enable, Game DVR disable
  - Win32PrioritySeparation 0x26 (foreground priority)
  - System timer resolution
  - Multimedia system profile (NetworkThrottlingIndex, SystemResponsiveness)
  - Mouse acceleration off, input queue tăng
  - **20+ GPU latency tweaks** (power states)
  - Audio latency reduction
- **Killer features:**
  - **Verification tool** [V] — check tweak đang active với pass/fail
  - **Registry backup** trước khi sửa
  - **Laptop detection** — cảnh báo battery
  - Restore point tự động

### 6. [cococool13/win11-gaming-toolkit](https://github.com/cococool13/win11-gaming-toolkit)
- **Toolkit PowerShell reversible, source-cited**
- **Check scripts đáng giá:**
  - `check-input-polling` — per-HID polling rate (mouse/keyboard/gamepad)
  - `check-msi-mode` — per-device MSI (GPU + Net + NVMe)
  - `check-rebar` — 3-layer ReBAR state
  - `check-directstorage` — 4-prereq check
  - `check-ram` — DIMM + WHEA errors
  - `show-mouse-info` — polling rates
- **Toggles:** enable-hags, enable-msi-mode, force-rebar, force-p0-state, configure-vbs (⚠ anti-cheat impact: VBS disable breaks Valorant/R6S!), disable-spectre-meltdown

---

## 🎯 ĐỀ XUẤT TÍNH NĂNG CHO WINOPTIMIZER

### Ưu tiên cao (P0) — Gaming Tab mới

| Tính năng | Nguồn tham khảo | Độ khó | Mô tả |
|-----------|----------------|--------|-------|
| **1. Gaming Profile (presets)** | mojo-gaming-mode, Reclaim | Thấp | Balanced / Performance / Esports preset, mỗi preset = batch registry+service tweaks, 1-click apply/revert |
| **2. Per-game Profiles** | Reclaim, Executable-Performance-Manager | Thấp | Chọn .exe game → GPU preference (HKCU UserGpuPreferences), FSO off, HighDPIAware, Run-as-admin (AppCompat Layers). **HKCU only, không cần admin!** |
| **3. Gaming Session (Boost mode)** | Reclaim, game-optimizer | Trung bình | Snapshot state → suspend background apps → Ultimate Performance plan → pause Defender → End session = revert hết. Crash-safe snapshot |
| **4. Live Latency Monitor** | Reclaim, revx | Trung bình | Ping sparklines tới Steam/Riot/Epic/Google DNS/Cloudflare, 2/5/10s polling, min/avg/max/loss% |
| **5. Registry Tweaks Catalog (gaming)** | theantipopau, win11-gaming-toolkit | Thấp | Bảng toggle: HAGS, Win32PrioritySeparation=0x26, timer resolution, MMCSS, Game DVR off, background apps off, TDR delay, mouse accel off, Nagle off, QoS off |

### Ưu tiên trung bình (P1)

| Tính năng | Nguồn | Độ khó | Mô tả |
|-----------|-------|--------|-------|
| **6. NIC Tuning Editor** | Reclaim | Trung bình | Live editor `Get-NetAdapterAdvancedProperty` — EEE, flow control, interrupt moderation, RSS, LSO |
| **7. MSI Mode Editor** | Reclaim, win11-gaming-toolkit | Trung bình | Per-PCI device MSI/MSI-X toggle + check script |
| **8. Verification Tool** | theantipopau | Thấp | Kiểm tra tweak nào đang active (pass/fail) |
| **9. Before/After Snapshot** | mojo-gaming-mode | Trung bình | So sánh FPS/latency trước-sau khi apply |
| **10. Startup Auto-optimize** | vixygrey | Trung bình | Apply preset khi máy boot (idempotent) |

### Có thể để sau (P2)

| Tính năng | Nguồn | Mô tả |
|-----------|-------|-------|
| Tab Live Overlay (mini/bar mode) | mojo-gaming-mode | Always-on-top stats overlay in-game |
| Game library auto-scan | game-optimizer | Steam/Epic/GOG detection |
| CPU affinity auto-pin | game-optimizer | P-cores pinning (phức tạp, dễ sai) |
| DNS benchmark | revx | Test 4 DNS providers auto-apply |
| Speedtest tích hợp | revx | Ookla API |

---

## 🛡️ CẢNH BÁO AN TOÀN (quan trọng!)

1. **VBS/HVCI disable** → +5-15% FPS nhưng **break Valorant (Vanguard), R6 Siege (BattlEye)** — phải warning rõ
2. **HPET disable / bcdedit** → có thể gây boot issue — cần restore point trước
3. **Antivirus/Defender disable** → chỉ trong session, tự bật lại
4. **Core parking off** → tăng nhiệt/điện năng — chỉ cho desktop
5. **Disable SMT/HT** → heuristic flag trên Zen 5, anti-cheat có thể nhạy cảm
6. **Disable PageFile** → RAM 16GB trở xuống không nên
7. Luôn: **Registry backup + Restore point + Undo system** (WinOptimizer đã có undo system 👍)

---

## 🔑 REGISTRY KEYS CẦN BIẾT

```powershell
# HAGS (Hardware-Accelerated GPU Scheduling)
HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers /v HwSchMode = 2

# Win32PrioritySeparation (foreground boost) — 0x26 = 26 decimal
HKLM\SYSTEM\CurrentControlSet\Control\PriorityControl /v Win32PrioritySeparation

# NetworkThrottlingIndex (0 = unlimited)
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile /v NetworkThrottlingIndex
# SystemResponsiveness (100 = no throttling)
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile /v SystemResponsiveness

# MMCSS Gaming priority (68 = high)
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games /v Priority
# GPU Priority (8)
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games /v GPU Priority

# Game DVR off
HKCU\System\GameConfigStore /v GameDVR_Enabled = 0
HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR /v AppCaptureEnabled = 0

# Mouse accel off (MouseSpeed=0)
HKCU\Control Panel\Mouse /v MouseSpeed

# Nagle off (TCP no delay)
HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\{GUID} /v TcpAckFrequency = 1
HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\{GUID} /v TCPNoDelay = 1

# GPU preference per-app (HKCU, no admin!)
HKCU\SOFTWARE\Microsoft\DirectX\UserGpuPreferences /v "path.exe" = "GpuPreference=2;"

# FSO off per-app (HKCU, no admin!)
HKCU\System\GameConfigStore\Children\{GUID} /v "GameDVR_Enabled" = 0

# Run-as-admin per-app (HKCU AppCompat)
HKCU\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers /v "path.exe" = "~ RUNASADMIN"

# TDR delay (GPU timeout 0 = disabled)
HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers /v TdrDelay = 0

# Game Mode
HKCU\SOFTWARE\Microsoft\GameBar /v AutoGameModeEnabled = 1
```

---

## ✅ KHUYẾN NGHỊ TRIỂN KHAI

1. **Bắt đầu với P0 features 1, 2, 5** (Gaming Profile + Per-game Profiles + Registry Catalog) — đều là HKCU/HKLM registry, có thể làm bằng PowerShell giống hệt phần còn lại của WinOptimizer
2. **Kiến trúc:** thêm tab Gaming vào sidebar, mỗi feature = 1 command Rust (`apply_gaming_profile`, `get_gpu_prefs`, `set_game_profile`, `start_gaming_session`...), frontend React dùng Ant Design Tabs/Switch/Collapse
3. **Reuse:** WinOptimizer đã có undo system → nối Gaming tweaks vào cùng registry backup
4. **Test trên máy Dương:** FPT PC (Windows 11), test Gaming Profile + Per-game trên game bất kỳ

> ⚠️ Tất cả tweaks aggressive (Esports profile, CPU affinity, HPET off) phải nằm sau warning confirm dialog