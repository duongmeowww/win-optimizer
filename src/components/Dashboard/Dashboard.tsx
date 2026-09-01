import { useEffect, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import { Row, Col, Card, Spin, Alert } from "antd";
import {
  DesktopOutlined,
  HddOutlined,
  ThunderboltOutlined,
  ApiOutlined,
  RiseOutlined,
  ClockCircleOutlined,
  WindowsOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

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
  os_version: string;
  uptime_seconds: number;
}

function fmtGHz(mhz: number): string {
  if (!mhz) return "N/A";
  return `${(mhz / 1000).toFixed(2)} GHz`;
}

const MAX_POINTS = 30;

/* Mini sparkline — pure SVG, dark friendly */
function Sparkline({ data, color }: { data: number[]; color: string }) {
  const w = 120;
  const h = 32;
  if (data.length < 2) return <div style={{ height: h }} />;
  const max = Math.max(100, ...data);
  const step = w / (data.length - 1);
  const points = data.map((v, i) => `${(i * step).toFixed(1)},${(h - (v / max) * (h - 4) - 2).toFixed(1)}`).join(" ");
  const area = `0,${h} ${points} ${w},${h}`;
  return (
    <svg width={w} height={h} style={{ display: "block", marginTop: 8 }}>
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth={1.5}
        strokeLinejoin="round"
        strokeLinecap="round"
        opacity={0.9}
      />
      <polygon points={area} fill={color} opacity={0.08} />
    </svg>
  );
}

/* Semi-circle gauge */
function SemiGauge({ value, color }: { value: number; color: string }) {
  const pct = Math.max(0, Math.min(100, value)) / 100;
  const r = 40;
  const c = Math.PI * r; // half circumference
  const offset = c * (1 - pct);
  return (
    <svg width={100} height={56} viewBox="0 0 100 56" style={{ display: "block", margin: "0 auto" }}>
      <path
        d="M 10 50 A 40 40 0 0 1 90 50"
        fill="none"
        stroke="rgba(148,163,184,0.2)"
        strokeWidth={7}
        strokeLinecap="round"
      />
      <path
        d="M 10 50 A 40 40 0 0 1 90 50"
        fill="none"
        stroke={color}
        strokeWidth={7}
        strokeLinecap="round"
        strokeDasharray={c}
        strokeDashoffset={offset}
        style={{ transition: "stroke-dashoffset 0.6s ease" }}
      />
      <text x={50} y={46} textAnchor="middle" fill="#f1f5f9" fontSize={16} fontWeight={700}>
        {Math.round(value)}
      </text>
    </svg>
  );
}

export default function Dashboard() {
  const { t } = useTranslation();
  const location = useLocation();
  const isActive = location.pathname === "/" || location.pathname === "/dashboard";
  const [info, setInfo] = useState<SystemInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<{ cpu: number[]; ram: number[]; gpu: number[] }>({
    cpu: [],
    ram: [],
    gpu: [],
  });
  const busyRef = useRef(false);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    if (!isActive) return;
    async function load(showSpinner: boolean) {
      if (busyRef.current) return;
      busyRef.current = true;
      if (showSpinner) setInfo(null);
      try {
        const isTauri = "__TAURI_INTERNALS__" in window;
        if (!isTauri) {
          setError("Tauri backend not running — run `npm run tauri dev`");
          return;
        }
        const data = await invoke<SystemInfo>("get_sys_info");
        if (mountedRef.current) {
          setInfo(data);
          setHistory((h) => ({
            cpu: [...h.cpu, data.cpu_usage].slice(-MAX_POINTS),
            ram: [...h.ram, Math.round((data.ram_used / data.ram_total) * 100)].slice(-MAX_POINTS),
            gpu: [...h.gpu, data.gpu_clock_mhz ? Math.min(100, 60) : 0].slice(-MAX_POINTS),
          }));
        }
      } catch (e) {
        if (mountedRef.current) setError(String(e));
      } finally {
        busyRef.current = false;
      }
    }
    load(true);
    const id = setInterval(() => load(false), 5000);
    return () => clearInterval(id);
  }, [isActive]);

  if (error) return <Alert type="info" showIcon message={error} />;
  if (!info) return <Spin size="large" style={{ display: "block", margin: "40px auto" }} />;

  const ramPct = Math.round((info.ram_used / info.ram_total) * 100);
  const diskPct = Math.round(((info.disk_total - info.disk_free) / info.disk_total) * 100);
  const health = Math.max(0, 100 - Math.min(80, ramPct) - Math.max(0, diskPct - 70));
  const cpuTempText = info.cpu_temp_c > 0 ? `${info.cpu_temp_c.toFixed(0)}°C` : "N/A";
  const gpuTempText = info.gpu_temp_c > 0 ? `${info.gpu_temp_c.toFixed(0)}°C` : "N/A";
  const cpuColor = info.cpu_usage > 85 ? "#f87171" : "#60a5fa";
  const ramColor = ramPct > 85 ? "#f87171" : "#34d399";
  const diskColor = diskPct > 85 ? "#f87171" : "#fbbf24";
  const healthColor = health > 70 ? "#34d399" : health > 40 ? "#fbbf24" : "#f87171";

  const cardStyle = { height: "100%" };

  return (
    <div>
      <Row gutter={[14, 14]}>
        {/* CPU */}
        <Col span={5}>
          <Card size="small" style={cardStyle} styles={{ body: { padding: 16 } }}>
            <div className="dash-card-title">
              <DesktopOutlined style={{ color: cpuColor }} />
              {t("dashboard.cpu")}
            </div>
            <div className="dash-card-value" style={{ color: cpuColor }}>
              {info.cpu_usage.toFixed(1)}%
            </div>
            <div className="dash-card-sub">{info.cpu_name || "N/A"}</div>
            <div className="dash-card-sub">
              ⚡ {fmtGHz(info.cpu_clock_mhz)} · 🌡 {cpuTempText}
            </div>
            <Sparkline data={history.cpu} color={cpuColor} />
          </Card>
        </Col>
        {/* RAM */}
        <Col span={4}>
          <Card size="small" style={cardStyle} styles={{ body: { padding: 16 } }}>
            <div className="dash-card-title">
              <ThunderboltOutlined style={{ color: ramColor }} />
              {t("dashboard.ram")}
            </div>
            <div className="dash-card-value" style={{ color: ramColor }}>
              {(info.ram_used / 1073741824).toFixed(1)}
              <span style={{ fontSize: 13, color: "#94a3b8", fontWeight: 500 }}>
                {" "}
                / {(info.ram_total / 1073741824).toFixed(1)} GB
              </span>
            </div>
            <div className="dash-card-sub">{ramPct}% đang sử dụng</div>
            <Sparkline data={history.ram} color={ramColor} />
          </Card>
        </Col>
        {/* Disk */}
        <Col span={5}>
          <Card size="small" style={cardStyle} styles={{ body: { padding: 16 } }}>
            <div className="dash-card-title">
              <HddOutlined style={{ color: diskColor }} />
              {t("dashboard.disk")}
            </div>
            <div className="dash-card-value" style={{ color: diskColor }}>
              {(info.disk_free / 1073741824).toFixed(1)}
              <span style={{ fontSize: 13, color: "#94a3b8", fontWeight: 500 }}>
                {" "}
                / {(info.disk_total / 1073741824).toFixed(1)} GB
              </span>
            </div>
            <div className="dash-card-sub">
              {info.disk_info.slice(0, 2).map((d) => (
                <div key={d.name}>
                  {d.name}: {(d.free_bytes / 1073741824).toFixed(0)} GB trống
                </div>
              ))}
            </div>
            <Sparkline data={history.ram} color={diskColor} />
          </Card>
        </Col>
        {/* GPU */}
        <Col span={5}>
          <Card size="small" style={cardStyle} styles={{ body: { padding: 16 } }}>
            <div className="dash-card-title">
              <RiseOutlined style={{ color: "#c084fc" }} />
              {t("dashboard.gpu")}
            </div>
            <div className="dash-card-value" style={{ color: "#c084fc" }}>
              {info.gpu_clock_mhz ? fmtGHz(info.gpu_clock_mhz) : "—"}
            </div>
            <div className="dash-card-sub">{info.gpu_name || "N/A"}</div>
            <div className="dash-card-sub">🌡 {gpuTempText}</div>
            <Sparkline data={history.gpu} color="#c084fc" />
          </Card>
        </Col>
        {/* Health */}
        <Col span={5}>
          <Card size="small" style={cardStyle} styles={{ body: { padding: 16 } }}>
            <div className="dash-card-title">
              <ApiOutlined style={{ color: healthColor }} />
              {t("dashboard.health")}
            </div>
            <SemiGauge value={health} color={healthColor} />
            <div className="dash-card-sub" style={{ textAlign: "center" }}>
              {health > 70 ? "Tuyệt vời" : health > 40 ? "Khá ổn" : "Cần tối ưu"}
            </div>
          </Card>
        </Col>
      </Row>
      <Row gutter={[14, 14]} style={{ marginTop: 14 }}>
        <Col span={12}>
          <Card size="small" style={cardStyle} styles={{ body: { padding: 16 } }}>
            <div className="dash-card-title">
              <WindowsOutlined style={{ color: "#60a5fa" }} />
              {t("dashboard.os")}
            </div>
            <div style={{ color: "#e2e8f0", fontSize: 14 }}>{info.os_version}</div>
          </Card>
        </Col>
        <Col span={12}>
          <Card size="small" style={cardStyle} styles={{ body: { padding: 16 } }}>
            <div className="dash-card-title">
              <ClockCircleOutlined style={{ color: "#34d399" }} />
              {t("dashboard.uptime")}
            </div>
            <div style={{ color: "#e2e8f0", fontSize: 14 }}>
              {Math.floor(info.uptime_seconds / 3600)}h{" "}
              {Math.floor((info.uptime_seconds % 3600) / 60)}m
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  );
}