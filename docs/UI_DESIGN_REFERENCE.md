# WinOptimizer — UI Design Reference (HyperLis-inspired)

> Ngày: 2026-09-01
> Reference: HyperLis v0.1.1 BETA — Zit's Tech Lab (Lê Minh Đức)

---

## 🎨 Design Tokens

### Theme: Glassmorphism (Kính mờ)

**Background:**
- Base: soft gradient linear-gradient(135deg, #e0eafc → #cfdef3 → #e8d5e0) — light blue → lavender → light pink
- Optional: animated floating gradient blobs or static gradient

**Cards (glass panels):**
```css
.glass-card {
  background: rgba(255, 255, 255, 0.65);       /* white 65% opacity */
  backdrop-filter: blur(12px);                    /* frosted glass */
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.4);   /* subtle white border */
  border-radius: 16px;                            /* rounded corners */
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.06);   /* soft shadow */
}
```

**Sidebar:**
```css
.sidebar {
  background: rgba(240, 244, 248, 0.75);        /* light blue glass */
  backdrop-filter: blur(16px);
  border-right: 1px solid rgba(255, 255, 255, 0.5);
  width: 200px;
  padding: 24px 12px;
}
.sidebar-item.active {
  background: rgba(255, 255, 255, 0.7);
  border-radius: 12px;
}
.sidebar-item:hover {
  background: rgba(255, 255, 255, 0.4);
}
```

**Typography:**
- Font: Inter / Segoe UI / system font stack
- Title: 24px, bold, dark (#1a1a2e)
- Section headers: 16px, semibold
- Body: 14px, regular
- Muted: 12px, gray (#666)

**Colors:**
- Primary accent: #3b82f6 (blue)
- Success/active: #22c55e (green)
- Warning: #f59e0b (amber)
- Error: #ef4444 (red)
- GPU accent: #8b5cf6 (purple)
- Text primary: #1a1a2e
- Text secondary: #64748b
- Border: rgba(255, 255, 255, 0.4)

**Spacing:**
- Card padding: 24px
- Card gap: 20px
- Sidebar item gap: 8px
- Border radius: 12-16px

---

## 📐 Layout Structure

```
┌──────────────────────────────────────────────────────┐
│ Top Bar: [Logo + App Name] [Restart btn] [⚙️] [—][×]│
├──────────┬───────────────────────────────────────────┤
│          │                                           │
│ Sidebar  │  Main Content Area                        │
│          │                                           │
│ 🏠 Dash  │  ┌─────────────────────────────────────┐  │
│ ⚙ General│  │  Section Title                      │  │
│ 🖥 Windows│  │                                     │  │
│ 📦 UWP   │  │  Content Cards (grid)               │  │
│ 🚀 Start │  │                                     │  │
│ ⬇ Down  │  │                                     │  │
│ 🗑 Clean │  └─────────────────────────────────────┘  │
│ 🌐 Net   │                                           │
│ 🔌 USB   │  ┌─────────────────────────────────────┐  │
│ ℹ HW    │  │  Changelog / Version Info             │  │
│ 🎮 Game  │  └─────────────────────────────────────┘  │
│ 🔧 Fix   │                                           │
│ 📄 WSAP  │                                           │
│          │                                           │
└──────────┴───────────────────────────────────────────┘
```

---

## 🧩 Components参考

### 1. Performance Tracker (Circular Gauges)
- 3 circular progress rings: CPU (green), RAM (blue), GPU (purple)
- Center: percentage (large, bold) + subtitle (clock speed / GB / VRAM%)
- Below: hardware name (e.g., "i5-8500T 2.10GHz", "Samsung DDR4", "Intel UHD 630")
- Dropdown for GPU selection if multiple

**Implementation:**
```tsx
// Use SVG circles for gauges (no external lib needed)
<svg viewBox="0 0 120 120">
  <circle cx="60" cy="60" r="50" stroke="#e5e7eb" strokeWidth="8" fill="none" />
  <circle cx="60" cy="60" r="50" stroke={color} strokeWidth="8" fill="none"
    strokeDasharray={`${2 * Math.PI * 50}`}
    strokeDashoffset={`${2 * Math.PI * 50 * (1 - percent / 100)}`}
    strokeLinecap="round" transform="rotate(-90 60 60)" />
</svg>
```

### 2. Quick Settings (Toggle Switches)
- Toggle switches with labels
- Grid layout: 2 columns, 3 rows
- Each toggle: Switch component + text label
- Active state: blue/green; Inactive: gray

### 3. Shortcuts (Icon Buttons)
- Row of circular icon buttons
- Each: icon + label below
- Edit button in top-right corner
- On click: navigate to that section

### 4. Sidebar
- Icon + text per item
- Active item: highlighted background (glass)
- Hover: subtle background change
- Grouped with visual separators (optional)

### 5. Top Bar
- Left: Logo icon (rocket 🚀) + app name
- Right: "Restart to apply changes" warning button, settings gear, window controls (minimize, close)
- Note: custom title bar (remove default Windows chrome)

### 6. Changelog/Footer
- Copyright, developer, tester info
- Version number
- Release summary bullets
- Glassmorphism card at bottom

---

## 🎯 WinOptimizer — Sidebar Structure (Mới)

Dựa trên HyperLis + tính năng hiện có + gaming features sắp thêm:

| # | Icon | Name | Mục |
|---|------|------|-----|
| 1 | 🏠 | Dashboard | Tổng quan hệ thống (hiện tại) |
| 2 | 🧹 | Disk Cleaner | Dọn rác ổ đĩa (hiện tại) |
| 3 | 💾 | Memory Cleaner | Tối ưu RAM (hiện tại) |
| 4 | ⚙️ | Services & Startup | Quản lý services + startup (hiện tại) |
| 5 | 🔧 | Debloat | Gỡ bloatware (hiện tại) |
| 6 | 🎮 | **Gaming** | **Tab mới — Profiles, Per-game, Tweaks** |
| 7 | 📊 | Performance | Live latency monitor, CPU/GPU stats |
| 8 | 🔄 | Undo / History | Khôi phục thay đổi (hiện tại) |
| 9 | ⚙️ | Settings | Cài đặt app, language, theme |

---

## ⚡ Custom Title Bar (Remotion window controls)

Vì HyperLis dùng custom title bar, WinOptimizer cần:
- `tauri.conf.json`: `"decorations": false` (hide native title bar)
- Custom React component: draggable area + min/max/close buttons
- `@tauri-apps/api/window` for minimize/maximize/close

---

## 📦 Ant Design Components Phù hợp

| Component | Dùng cho |
|-----------|---------|
| `Card` | Glass cards |
| `Switch` | Quick settings toggles |
| `Progress` (type="circle") | Circular gauges |
| `Menu` + `Menu.Item` | Sidebar navigation |
| `Button` | Actions |
| `Tooltip` | Hover info |
| `Badge` | Status indicators |
| `Typography` | Headers, text |
| `Space` / `Flex` | Layout |
| `Modal` | Confirm dialogs (tweak apply) |
| `message` / `notification` | Toast notifications |
| `Tag` | Status badges (active/inactive) |

> **Note:** `Progress type="circle"` của Ant Design có thể dùng trực tiếp thay vì SVG custom!
