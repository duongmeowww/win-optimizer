# WinOptimizer — Lộ Trình Tối Ưu Win Có Đo Lường 🎯

> **Nguyên tắc cốt lõi:** Mỗi phase CODE XONG → chạy **BENCHMARK** đo được → chỉ khi số đo cải thiện/ổn định mới chuyển sang phase tiếp theo.
> "Code đến đâu, tối ưu tốc độ đến đó." Không làm 2 phase cùng lúc, không khoán trắng.

---

## 🧪 Benchmark Chuẩn (dùng cho mọi phase)

Mỗi phase sẽ đo bằng script benchmark này trước & sau:

| Chỉ số | Cách đo | Công cụ |
|--------|---------|---------|
| **RAM trống** | Free memory (GB) | `Get-Counter '\Memory\Available MBytes'` |
| **Boot time** | Thời gian khởi động Windows | Event log 100.1/100.2 (hẹn giờ lại) |
| **Disk 100%** | % Active time ổ C | `Get-Counter '\PhysicalDisk(_Total)\% Disk Time'` |
| **CPU idle** | % CPU khi nghỉ | `Get-Counter '\Processor(_Total)\% Processor Time'` |
| **File mở chậm** | Thời gian mở thư mục app | PowerShell `Measure-Command` |

**Script:** `scripts/benchmark.ps1` — chạy 1 lệnh, output JSON để app hiển thị được.

---

## 📋 Lộ Trình Theo Phase

### 🔴 Phase 1 — Nền Tảng Cứng (không thêm feature, chỉ làm chuẩn)
**Mục tiêu:** App chạy được, UI tàu nhanh, scan chính xác.
- [x] Dọn project sạch (xóa `fix_*.py`, cấu trúc lại)
- [x] Build runs thành công (`npm run tauri build`)
- [x] Dashboard hiển thị SystemInfo đúng, refresh < 3s
- [x] Benchmark script hoạt động, có file log kết quả

**Đo:** App khởi động, UI đổ dữ liệu không giật, scan RAM/disk < 5s — ✅ PASS

> **Baseline ghi nhận 2026-09-01:** RAM available 8426MB · Disk 0.3% · CPU 6.9% · mở AppData 54ms · 3 tiến trình nặng.

### 🟠 Phase 2 — Dọn Rác Có Số Liệu (Disk Cleaner)
**Mục tiêu:** Scan + dọn rác, NHƯNG mỗi lần dọn phải báo: giải phóng X GB, tốc độ cải thiện.
- [x] Scan 10 hạng mục rác (đã có trong backend)
- [x] UI hiển thị từng mục + tổng dung lượng (GB hiển thị chuẩn, không MB lẫn GB)
- [x] Nút "Dọn" chạy từng mục, log freed bytes → hiển thị kết quả
- [x] Benchmark RAM trước/sau dọn (hiển thị "Trước X GB → Sau Y GB")

**Đo:** Dọn xong báo được dung lượng cụ thể + RAM trước/sau. — ✅ code xong, chờ test trực quan trong app

### 🟡 Phase 3 — Giải Phóng RAM (Memory Cleaner)
**Mục tiêu:** Lấy lại RAM ngay lập tức bằng API Windows.
- [ ] Nút "Dọn RAM" — gọi **EmptyWorkingSet** (Windows API, không kill tiến trình)
- [ ] Hiển thị RAM trước → sau khi dọn (MB/GB)
- [ ] Auto-clean khi RAM thấp (threshold config được)

**Đo:** RAM trống tăng ≥ 500 MB ngay sau khi bấm, app không crash.

### 🟢 Phase 4 — Debloat Có Hoàn Tác (Debloat + Undo)
**Mục tiêu:** Tắt nhạc nền Windows/telemetry/service NHƯNG phải có nút hoàn tác.
- [ ] Danh sách checkbox các hạng mục debloat (Windows bloatware, telemetry, OneDrive, Edge chạy nền, etc.)
- [ ] Mỗi hạng mục bật/tắt, ghi log để UNDO được
- [ ] Nút "Hoàn tác tất cả" — restore lại registry/service như cũ

**Đo:** Bật/tắt 1 hạng mục → check lại đúng; Undo → về đúng trạng thái ban đầu.

### 🔵 Phase 5 — Services & Startup Manager
**Mục tiêu:** Control service + startup, biết cái nào ảnh hưởng boot.
- [ ] Liệt kê service (name, status, startup type) + khuyên nên tắt cái nào an toàn
- [ ] Liệt kê startup items (registry + startup folder)
- [ ] Bật/tắt, có chế độ "an toàn" (chỉ khuyên, không tự ý tắt cái hệ thống cần)

**Đo:** Bật/tắt service/startup đúng, máy boot lại không lỗi.

### 🟣 Phase 6 — Tổng Hợp + Undo Toàn Diện
**Mục tiêu:** Tất cả tác động đều log, undo tất cả về trạng thái gốc.
- [ ] 1 nút "FULL OPTIMIZE" chạy all phases
- [ ] Log mọi thay đổi vào file (JSON) để undo
- [ ] Bảng "Lịch sử thay đổi" trong Settings
- [ ] Export/Import config

**Đo:** Full optimize chạy 1 lần, undo 100% trả về nguyên trạng.

---

## 📊 Quy Tắc Mỗi Phase

1. **Code xong → chạy benchmark** lưu vào `benchmarks/phase<N>.json`
2. **Số đo phải cải thiện hoặc giải thích được** (ví dụ: dọn rác giải phóng 2GB → RAM trống tăng)
3. Không sang phase sau khi phase hiện chưa đạt mục tiêu đo
4. UI phải hiển thị số liệu (người dùng thấy được tác động) — không im lặng chạy ngầm

---

## 🧹 Dọn Dẹp Hiện Tại (trước Phase 1)

- [ ] Xóa toàn bộ `fix_*.py` (đã vá xong, không cần nữa)
- [ ] Xóa `write_mod.py`, `preprend_sys_info.py`, `make_icon.py` (script sinh 1 lần, không thuộc source)
- [ ] Giữ `src/`, `src-tauri/`, `index.html` (khung Tauri)
- [ ] Giữ `docs/` nếu có nội dung còn dùng
