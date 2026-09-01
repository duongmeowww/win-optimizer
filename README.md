# WinOptimizer 🖥️

Ứng dụng tối ưu Windows — dọn rác, debloat, dọn RAM, quản lý dịch vụ & startup.
Windows Optimization App — disk cleaner, debloat, RAM cleaner, services & startup manager.

## Tech Stack

- **Frontend:** React + TypeScript + Vite + Ant Design
- **Backend:** Rust (Tauri v2)
- **i18n:** Tiếng Việt / English (react-i18next)

## Yêu cầu

- Windows 10/11
- [Rust](https://rustup.rs) (MSVC toolchain)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) — C++ workload
- Node.js 18+

## Dev

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Modules

| Module | Status |
|--------|--------|
| Dashboard (system info + health score) | ✅ hoạt động |
| Disk Cleaner (scan + dọn + benchmark) | ✅ hoạt động |
| Debloat | 🔜 Phase 4 |
| Memory Cleaner | 🔜 Phase 3 |
| Services & Startup | 🔜 Phase 5 |
| Undo system | 🔜 Phase 6 |

## Lộ trình

Xem [ROADMAP.md](./ROADMAP.md) — mỗi phase code xong đều có benchmark đo tốc độ.

### Phase hoàn thành

- **Phase 1** — Nền tảng: build sạch, benchmark script hoạt động (`scripts/benchmark.ps1`)
- **Phase 2** — Disk Cleaner nối backend: scan 10 hạng mục rác + dọn + benchmark RAM trước/sau

## Features (roadmap, inspired by)

- WinUtil (Chris Titus): preset configs, service management
- Win11Debloat: full undo/revert, telemetry & privacy toggles
- WinMemoryCleaner: native Windows API memory cleaning, tray icon
