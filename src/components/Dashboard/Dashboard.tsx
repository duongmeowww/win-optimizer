import { useEffect, useRef, useState, useCallback } from "react";
import { useLocation } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
  InfoCircleOutlined,
  ClockCircleOutlined,
  ControlOutlined,
  DeploymentUnitOutlined,
  HddOutlined,
  ExpandOutlined,
  HeartOutlined,
  BarChartOutlined,
  BulbOutlined,
  ClearOutlined,
  ExperimentOutlined,
  SafetyOutlined,
} from "@ant-design/icons";

interface DiskInfo {
  name: string;
  total_bytes: number;
  free_bytes: number;
}

interface SystemInfo {
  cpu_usage: number;
  cpu_name: string;
  cpu_clock_mhz: number;
  cpu_temp_c: number;
  ram_total: number;
  ram_used: number;
  disk_total: number;
  disk_free: number;
  disk_info: DiskInfo[];
  gpu_name: string;
  gpu_clock_mhz: number;
  gpu_temp_c: number;
  gpu_usage: number;
  os_version: string;
  uptime_seconds: number;
}

function fmtBytes(b: number): string {
  const gb = b / 1073741824;
  if (gb >= 1) return `${gb.toFixed(1)} GB`;
  return `${(b / 1048576).toFixed(0)} MB`;
}

function fmtGHz(mhz: number): string {
  if (!mhz) return "N/A";
  return `${(mhz / 1000).toFixed(2)} GHz`;
}

const MAX_POINTS = 24;

/* Smooth SVG sparkline */
function Sparkline({ data, color }: { data: number[]; color: string }) {
  const w = 150;
  const h = 46;
  if (data.length < 2) return <div style={{ height: h }} />;
  const max = Math.max(100, ...data);
  const step = w / (data.length - 1);
  const pts = data
    .map((v, i) => `${(i * step).toFixed(1)},${(h - (v / max) * (h - 8) - 4).toFixed(1)}`)
    .join(" ");
  const area = `0,${h} ${pts} ${w},${h}`;
  const gid = `sg-${color.replace("#", "")}`;
  return (
    <svg width="100%" height={h} viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" style={{ display: "block" }}>
      <defs>
        <linearGradient id={gid} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.25" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <polygon points={area} fill={`url(#${gid})`} />
      <polyline
        points={pts}
        fill="none"
        stroke={color}
        strokeWidth={2}
        strokeLinejoin="round"
        strokeLinecap="round"
      />
    </svg>
  );
}

/* Circular progress ring */
function Ring({ value, size = 128, stroke = 10, color, children }: any) {
  const pct = Math.max(0, Math.min(100, value));
  const r = (size - stroke) / 2;
  const c = 2 * Math.PI * r;
  const offset = c * (1 - pct / 100);
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
      <defs>
        <linearGradient id={`ring-${color.replace("#", "")}`} x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor={color} />
          <stop offset="100%" stopColor={color} stopOpacity="0.6" />
        </linearGradient>
      </defs>
      <circle
        cx={size / 2}
        cy={size / 2}
        r={r}
        fill="none"
        stroke="rgba(148,163,184,0.18)"
        strokeWidth={stroke}
      />
      <circle
        className="ring-arc"
        cx={size / 2}
        cy={size / 2}
        r={r}
        fill="none"
        stroke={`url(#ring-${color.replace("#", "")})`}
        strokeWidth={stroke}
        strokeLinecap="round"
        strokeDasharray={c}
        strokeDashoffset={offset}
        transform={`rotate(-90 ${size / 2} ${size / 2})`}
      />
      <foreignObject x={0} y={0} width={size} height={size}>
        <div className="ring-label">{children}</div>
      </foreignObject>
    </svg>
  );
}

export default function Dashboard() {
  const location = useLocation();
  const isActive = location.pathname === "/" || location.pathname === "/dashboard";
  const [info, setInfo] = useState<SystemInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<{ cpu: number[]; ram: number[]; disk: number[]; gpu: number[] }>({
    cpu: [],
    ram: [],
    disk: [],
    gpu: [],
  });
  const busyRef = useRef(false);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  const load = useCallback(async (showSpinner: boolean) => {
    if (busyRef.current) return;
    busyRef.current = true;
    if (showSpinner) setInfo(null);
    try {
      if (!("__TAURI_INTERNALS__" in window)) {
        setError("Chạy `npm run tauri dev` để khởi động backend.");
        return;
      }
      const data = await invoke<SystemInfo>("get_sys_info");
      if (mountedRef.current) {
        setInfo(data);
        const diskPct = data.disk_total
          ? Math.round(((data.disk_total - data.disk_free) / data.disk_total) * 100)
          : 0;
        setHistory((h) => ({
          cpu: [...h.cpu, data.cpu_usage].slice(-MAX_POINTS),
          ram: [
            ...h.ram,
            data.ram_total ? Math.round((data.ram_used / data.ram_total) * 100) : 0,
          ].slice(-MAX_POINTS),
          disk: [...h.disk, diskPct].slice(-MAX_POINTS),
          gpu: [...h.gpu, Math.min(100, Math.round(data.gpu_usage || 0))].slice(-MAX_POINTS),
        }));
      }
    } catch (e) {
      if (mountedRef.current) setError(String(e));
    } finally {
      busyRef.current = false;
    }
  }, []);

  useEffect(() => {
    if (!isActive) return;
    load(true);
    const id = setInterval(() => load(false), 5000);
    return () => clearInterval(id);
  }, [isActive, load]);

  if (error) {
    return (
      <div className="dash-error">
        <InfoCircleOutlined style={{ fontSize: 34, color: "#22d3ee" }} />
        <p style={{ color: "#94a3b8" }}>{error}</p>
      </div>
    );
  }
  if (!info) {
    return (
      <div className="dash-loading"><div className="dash-spinner" /></div>
    );
  }

  // Derived metrics
  const ramPct = info.ram_total ? Math.round((info.ram_used / info.ram_total) * 100) : 0;
  const diskUsed = info.disk_total - info.disk_free;
  const diskPct = info.disk_total ? Math.round((diskUsed / info.disk_total) * 100) : 0;
  const gpuPct = Math.min(100, Math.round(info.gpu_usage || 0));
  const health = Math.max(
    0,
    Math.min(100, 100 - Math.min(60, ramPct) - Math.max(0, diskPct - 70))
  );
  const uptimeH = Math.floor(info.uptime_seconds / 3600);
  const uptimeM = Math.floor((info.uptime_seconds % 3600) / 60);

  const cpuColor = "#38bdf8";
  const ramColor = "#a78bfa";
  const diskColor = "#fbbf24";
  const gpuColor = "#34d399";
  const healthColor = health > 70 ? "#34d399" : health > 40 ? "#fbbf24" : "#f87171";

  // Recommendations (based on finished data)
  const recommendations: { icon: React.ReactNode; title: string; desc: string; tag: string; tagStyle: any }[] = [];
  if (diskPct >= 80) {
    recommendations.push({
      icon: <ClearOutlined />,
      title: "Dọn dẹp file rác",
      desc: `Ổ đĩa đang đầy (${diskPct}%). Xóa file tạm & rác hệ thống.`,
      tag: "Khuyến nghị",
      tagStyle: { background: "rgba(34,211,238,0.12)", color: "#22d3ee", border: "1px solid rgba(34,211,238,0.3)" },
    });
  }
  if (ramPct >= 80) {
    recommendations.push({
      icon: <ExperimentOutlined />,
      title: "Giải phóng RAM",
      desc: `RAM đang sử dụng ${ramPct}%. Giải phóng bộ nhớ hoạt động.`,
      tag: "Khuyến nghị",
      tagStyle: { background: "rgba(167,139,250,0.12)", color: "#a78bfa", border: "1px solid rgba(167,139,250,0.3)" },
    });
  }
  if (recommendations.length === 0) {
    recommendations.push({
      icon: <SafetyOutlined />,
      title: "Hệ thống khỏe mạnh",
      desc: "Chạy quét nhanh để kiểm tra toàn diện phần mềm không cần thiết.",
      tag: "Ổn định",
      tagStyle: { background: "rgba(52,211,153,0.12)", color: "#34d399", border: "1px solid rgba(52,211,153,0.3)" },
    });
  }

  return (
    <div className="dash-root">
      {/* Header */}
      <div className="dash-header">
        <div>
          <h1 className="dash-title">Bảng điều khiển</h1>
          <p className="dash-subtitle">{info.os_version}</p>
        </div>
        <div className="dash-uptime-badge">
        <ClockCircleOutlined />
          Hoạt động: {uptimeH}h {uptimeM}m
        </div>
      </div>

      {/* Stats cards grid */}
      <div className="stats-grid">
        {/* CPU */}
        <div className="stat-card stat-card-cpu">
          <div className="stat-head">
            <span className="stat-icon"><ControlOutlined /></span>
            <span className="stat-name">CPU</span>
          </div>
          <div className="stat-value">{info.cpu_usage.toFixed(1)}%</div>
          <div className="stat-sub">{info.cpu_name || "N/A"}</div>
          <div className="stat-chart"><Sparkline data={history.cpu} color={cpuColor} /></div>
        </div>

        {/* RAM */}
        <div className="stat-card">
          <div className="stat-head">
            <span className="stat-icon"><DeploymentUnitOutlined /></span>
            <span className="stat-name">RAM</span>
          </div>
          <div className="ring-wrap">
            <Ring value={ramPct} color={ramColor}>
              <div className="ring-num">{ramPct}%</div>
              <div className="ring-cap">{fmtBytes(info.ram_used)}</div>
            </Ring>
          </div>
          <div className="stat-cap">{fmtBytes(info.ram_used)} / {fmtBytes(info.ram_total)}</div>
        </div>

        {/* Disk */}
        <div className="stat-card">
          <div className="stat-head">
            <span className="stat-icon"><HddOutlined /></span>
            <span className="stat-name">Ổ Đĩa C:</span>
          </div>
          <div className="ring-wrap">
            <Ring value={diskPct} color={diskColor}>
              <div className="ring-num">{diskPct}%</div>
              <div className="ring-cap">{fmtBytes(diskUsed)} đã dùng</div>
            </Ring>
          </div>
          <div className="stat-cap">💾 Trống {fmtBytes(info.disk_free)}</div>
        </div>

        {/* GPU */}
        <div className="stat-card stat-card-gpu">
          <div className="stat-head">
            <span className="stat-icon"><ExpandOutlined /></span>
            <span className="stat-name">GPU</span>
          </div>
          <div className="stat-value">{gpuPct}%</div>
          <div className="stat-sub">{info.gpu_name || "N/A"}</div>
          <div className="stat-chart"><Sparkline data={history.gpu} color={gpuColor} /></div>
        </div>

        {/* Health */}
        <div className="stat-card stat-card-health">
          <div className="stat-head">
            <span className="stat-icon"><HeartOutlined /></span>
            <span className="stat-name">Điểm Sức Khỏe</span>
          </div>
          <div className="ring-wrap">
            <Ring value={health} color={healthColor}>
              <div className="ring-num">{health}</div>
              <div className="ring-cap">/100</div>
            </Ring>
          </div>
          <div className="stat-cap">{health > 70 ? "Tuyệt vời" : health > 40 ? "Khá ổn" : "Cần tối ưu"}</div>
        </div>
      </div>

      {/* Bottom: report + tools */}
      <div className="bottom-grid">
        {/* Báo cáo chi tiết */}
        <div className="panel">
          <h3 className="panel-title"><BarChartOutlined style={{ color: "#22d3ee" }} /> Báo cáo chi tiết</h3>
          <div className="report-grid">
            <div className="report-item">
              <span className="report-label">CPU tần số</span>
              <span className="report-value">{fmtGHz(info.cpu_clock_mhz)}</span>
            </div>
            <div className="report-item">
              <span className="report-label">CPU nhiệt độ</span>
              <span className="report-value">{info.cpu_temp_c > 0 ? `${info.cpu_temp_c.toFixed(0)}°C` : "N/A"}</span>
            </div>
            <div className="report-item">
              <span className="report-label">GPU</span>
              <span className="report-value">{info.gpu_name || "N/A"}</span>
            </div>
            <div className="report-item">
              <span className="report-label">GPU tần số</span>
              <span className="report-value">{fmtGHz(info.gpu_clock_mhz)}</span>
            </div>
            <div className="report-item">
              <span className="report-label">Hệ điều hành</span>
              <span className="report-value">{info.os_version}</span>
            </div>
            <div className="report-item">
              <span className="report-label">Uptime</span>
              <span className="report-value">{uptimeH}h {uptimeM}m</span>
            </div>
          </div>
        </div>

        {/* Công cụ khuyến nghị */}
        <div className="panel">
          <h3 className="panel-title"><BulbOutlined style={{ color: "#fbbf24" }} /> Công cụ khuyến nghị</h3>
          <div className="rec-list">
            {recommendations.slice(0, 3).map((r, i) => (
              <div key={i} className="rec-item">
                <div className="rec-icon">{r.icon}</div>
                <div className="rec-body">
                  <div className="rec-top">
                    <span className="rec-title">{r.title}</span>
                    <span className="rec-tag" style={r.tagStyle}>{r.tag}</span>
                  </div>
                  <p className="rec-desc">{r.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
