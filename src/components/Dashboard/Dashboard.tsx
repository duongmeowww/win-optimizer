import { useEffect, useRef, useState, useCallback } from "react";
import HelpButton from "../Common/HelpButton";
import { useLocation } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "antd";
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
  DesktopOutlined,
  WarningOutlined,
  CheckCircleOutlined,
  ThunderboltOutlined,
} from "@ant-design/icons";

interface DiskInfo {
  name: string;
  total_bytes: number;
  free_bytes: number;
}

interface GpuInfo {
  name: string;
  clock_mhz: number;
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
  gpu_list: GpuInfo[];
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
     const [optimizing, setOptimizing] = useState(false);
     const [optimizeResult, setOptimizeResult] = useState<string | null>(null);
     const busyRef = useRef(false);
  const mountedRef = useRef(true);

     const optimizeSystem = async () => {
       setOptimizing(true);
       setOptimizeResult(null);
       try {
         const [success, message] = await invoke<[boolean, string]>(
           "one_click_optimize"
         );
         if (success) {
           setOptimizeResult(`✅ Hoàn tất tối ưu hóa!\n\n${message}`);
         } else {
           setOptimizeResult(`❌ Lỗi khi tối ưu hóa:\n\n${message}`);
         }
       } catch (err) {
         setOptimizeResult(`❌ Lỗi kết nối: ${err}`);
       } finally {
         setOptimizing(false);
       }
     };

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

  const ramPct = info.ram_total ? Math.round((info.ram_used / info.ram_total) * 100) : 0;
  const diskUsed = info.disk_total - info.disk_free;
  const diskPct = info.disk_total ? Math.round((diskUsed / info.disk_total) * 100) : 0;

  // Tính điểm Sức khỏe (Health Score) động thông minh (0 - 100)
  let healthScore = 100;
  if (ramPct > 85) healthScore -= 25;
  else if (ramPct > 70) healthScore -= 12;

  if (diskPct > 90) healthScore -= 25;
  else if (diskPct > 80) healthScore -= 15;

  if (info.cpu_usage > 80) healthScore -= 20;
  else if (info.cpu_usage > 50) healthScore -= 10;
  healthScore = Math.max(20, healthScore);

  // Khuyến nghị thông minh động dựa trên phần cứng thực tế của máy
  const recommendations: { title: string; desc: string; type: "warning" | "success" | "info" }[] = [];
  
  if (ramPct > 75) {
    recommendations.push({
      title: "RAM đang bị chiếm dụng cao",
      desc: `Mức sử dụng RAM đã đạt ${ramPct}%. Hãy sử dụng tính năng Dọn RAM để giải phóng bớt bộ nhớ.`,
      type: "warning",
    });
  } else {
    recommendations.push({
      title: "Bộ nhớ RAM ổn định",
      desc: `RAM hoạt động mượt mà (${(info.ram_used / 1073741824).toFixed(1)}GB / ${(info.ram_total / 1073741824).toFixed(1)}GB).`,
      type: "success",
    });
  }

  if (diskPct > 85) {
    recommendations.push({
      title: "Ổ cứng gần đầy",
      desc: `Dung lượng ổ đĩa đã dùng ${diskPct}%. Nên dọn dẹp file rác hoặc tìm file trùng lặp để giải phóng không gian.`,
      type: "warning",
    });
  } else {
    recommendations.push({
      title: "Dung lượng ổ đĩa tối ưu",
      desc: `Còn trống ${(info.disk_free / 1073741824).toFixed(1)} GB trên tổng số ổ đĩa hệ thống.`,
      type: "success",
    });
  }

  if (info.gpu_list && info.gpu_list.length > 1) {
    recommendations.push({
      title: `Phát hiện ${info.gpu_list.length} Card đồ họa`,
      desc: `Hệ thống có đa GPU (${info.gpu_list.map(g => g.name).join(" + ")}). Hãy đảm bảo game/app nặng sử dụng GPU rời để đạt hiệu năng cao nhất.`,
      type: "info",
    });
  }

  const formatUptime = (secs: number) => {
    const d = Math.floor(secs / 86400);
    const h = Math.floor((secs % 86400) / 3600);
    const m = Math.floor((secs % 3600) / 60);
    if (d > 0) return `${d} ngày ${h} giờ`;
    return `${h} giờ ${m} phút`;
  };

  return (
    <div style={{ padding: 24, maxWidth: 1200, margin: "0 auto" }}>
      {/* Header Banner */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 24 }}>
        <div>
          <h1 style={{ margin: 0, fontSize: 26, fontWeight: 700, color: "#f8fafc" }}>
            Tổng quan hệ thống <HelpButton title="Tổng quan hệ thống" content="Hiển thị điểm sức khỏe tổng thể của máy, dung lượng ổ cứng, thông tin GPU, RAM, CPU và các khuyến nghị tối ưu thông minh được cá nhân hóa cho máy của bạn." />
          </h1>
          <p style={{ margin: "4px 0 0 0", color: "#94a3b8" }}>{info.os_version} · Hoạt động: {formatUptime(info.uptime_seconds)}</p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 16, background: "rgba(255,255,255,0.03)", padding: "12px 20px", borderRadius: 12, border: "1px solid rgba(255,255,255,0.08)" }}>
          <HeartOutlined style={{ fontSize: 28, color: healthScore > 75 ? "#52c41a" : healthScore > 50 ? "#faad14" : "#ff4d4f" }} />
          <div>
            <div style={{ fontSize: 12, color: "#94a3b8" }}>Điểm sức khỏe</div>
            <div style={{ fontSize: 20, fontWeight: 700, color: "#f8fafc" }}>{healthScore} / 100 điểm</div>
          </div>
          <Button
            type="primary"
            icon={<ThunderboltOutlined />}
            loading={optimizing}
            onClick={optimizeSystem}
            style={{ marginLeft: 16, height: 40, fontWeight: 600 }}
          >
            Tối ưu một phím
          </Button>
        </div>
      </div>

      {/* Main CPU / RAM / GPU / Disk Meters */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))", gap: 16, marginBottom: 24 }}>
        {/* CPU Card */}
        <div style={{ background: "rgba(255,255,255,0.02)", border: "1px solid rgba(255,255,255,0.06)", borderRadius: 12, padding: 18 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
            <span style={{ fontWeight: 600, color: "#cbd5e1" }}>CPU Processor</span>
            <span style={{ fontSize: 12, color: "#38bdf8" }}>{info.cpu_name}</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <Ring value={Math.round(info.cpu_usage)} color="#38bdf8">
              <div style={{ textAlign: "center", paddingTop: 38 }}>
                <div style={{ fontSize: 22, fontWeight: 700, color: "#fff" }}>{Math.round(info.cpu_usage)}%</div>
              </div>
            </Ring>
            <div style={{ flex: 1 }}>
              <Sparkline data={history.cpu} color="#38bdf8" />
            </div>
          </div>
        </div>

        {/* RAM Card */}
        <div style={{ background: "rgba(255,255,255,0.02)", border: "1px solid rgba(255,255,255,0.06)", borderRadius: 12, padding: 18 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
            <span style={{ fontWeight: 600, color: "#cbd5e1" }}>Bộ nhớ RAM</span>
            <span style={{ fontSize: 12, color: "#a855f7" }}>{fmtBytes(info.ram_used)} / {fmtBytes(info.ram_total)}</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <Ring value={ramPct} color="#a855f7">
              <div style={{ textAlign: "center", paddingTop: 38 }}>
                <div style={{ fontSize: 22, fontWeight: 700, color: "#fff" }}>{ramPct}%</div>
              </div>
            </Ring>
            <div style={{ flex: 1 }}>
              <Sparkline data={history.ram} color="#a855f7" />
            </div>
          </div>
        </div>

        {/* GPU Card */}
        <div style={{ background: "rgba(255,255,255,0.02)", border: "1px solid rgba(255,255,255,0.06)", borderRadius: 12, padding: 18 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
            <span style={{ fontWeight: 600, color: "#cbd5e1" }}>Card đồ họa (GPU)</span>
            <span style={{ fontSize: 12, color: "#10b981" }}>{info.gpu_list?.length || 1} GPU detected</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <Ring value={Math.round(info.gpu_usage)} color="#10b981">
              <div style={{ textAlign: "center", paddingTop: 38 }}>
                <div style={{ fontSize: 22, fontWeight: 700, color: "#fff" }}>{Math.round(info.gpu_usage)}%</div>
              </div>
            </Ring>
            <div style={{ flex: 1 }}>
              <Sparkline data={history.gpu} color="#10b981" />
            </div>
          </div>
        </div>
      </div>

      {/* Multi-Disk & Multi-GPU Details Section */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 24 }}>
        {/* Disks Detail */}
        <div style={{ background: "rgba(255,255,255,0.02)", border: "1px solid rgba(255,255,255,0.06)", borderRadius: 12, padding: 18 }}>
          <h3 style={{ margin: "0 0 14px 0", fontSize: 16, color: "#f8fafc", display: "flex", alignItems: "center", gap: 8 }}>
            <HddOutlined style={{ color: "#60a5fa" }} /> Chi tiết ổ đĩa ({info.disk_info?.length || 0} ổ)
          </h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {info.disk_info?.map((d, i) => {
              const used = d.total_bytes - d.free_bytes;
              const pct = d.total_bytes ? Math.round((used / d.total_bytes) * 100) : 0;
              return (
                <div key={i} style={{ background: "rgba(255,255,255,0.02)", padding: 12, borderRadius: 8, border: "1px solid rgba(255,255,255,0.04)" }}>
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 6 }}>
                    <span style={{ fontWeight: 600, color: "#cbd5e1" }}>Ổ đĩa {d.name}</span>
                    <span style={{ fontSize: 12, color: "#94a3b8" }}>Trống {fmtBytes(d.free_bytes)} / {fmtBytes(d.total_bytes)} ({pct}%)</span>
                  </div>
                  <div style={{ width: "100%", background: "rgba(255,255,255,0.1)", height: 6, borderRadius: 3, overflow: "hidden" }}>
                    <div style={{ width: `${pct}%`, background: pct > 85 ? "#ef4444" : "#3b82f6", height: "100%" }} />
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* GPUs Detail */}
        <div style={{ background: "rgba(255,255,255,0.02)", border: "1px solid rgba(255,255,255,0.06)", borderRadius: 12, padding: 18 }}>
          <h3 style={{ margin: "0 0 14px 0", fontSize: 16, color: "#f8fafc", display: "flex", alignItems: "center", gap: 8 }}>
            <DesktopOutlined style={{ color: "#34d399" }} /> Chi tiết Card đồ họa ({info.gpu_list?.length || 1} GPU)
          </h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {info.gpu_list?.map((g, i) => (
              <div key={i} style={{ background: "rgba(255,255,255,0.02)", padding: 12, borderRadius: 8, border: "1px solid rgba(255,255,255,0.04)" }}>
                <div style={{ fontWeight: 600, color: "#cbd5e1", marginBottom: 4 }}>{g.name}</div>
                <div style={{ fontSize: 12, color: "#94a3b8" }}>Clock speed: {g.clock_mhz ? `${g.clock_mhz} MHz` : "N/A"}</div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Intelligent Recommendations */}
      <div style={{ background: "rgba(255,255,255,0.02)", border: "1px solid rgba(255,255,255,0.06)", borderRadius: 12, padding: 18 }}>
        <h3 style={{ margin: "0 0 14px 0", fontSize: 16, color: "#f8fafc", display: "flex", alignItems: "center", gap: 8 }}>
          <BulbOutlined style={{ color: "#facc15" }} /> Khuyến nghị tối ưu thông minh cho máy của bạn
        </h3>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(300px, 1fr))", gap: 12 }}>
          {recommendations.map((rec, i) => (
            <div key={i} style={{ background: "rgba(255,255,255,0.02)", padding: 14, borderRadius: 8, border: "1px solid rgba(255,255,255,0.04)", display: "flex", gap: 12, alignItems: "flex-start" }}>
              {rec.type === "warning" ? <WarningOutlined style={{ fontSize: 18, color: "#f59e0b", marginTop: 2 }} /> : <CheckCircleOutlined style={{ fontSize: 18, color: "#10b981", marginTop: 2 }} />}
              <div>
                <div style={{ fontWeight: 600, color: "#f8fafc", marginBottom: 4 }}>{rec.title}</div>
                <div style={{ fontSize: 13, color: "#94a3b8", lineHeight: 1.4 }}>{rec.desc}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
