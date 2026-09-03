import { useEffect, useState, useCallback } from "react";
import {
  Card,
  Button,
  Space,
  Alert,
  Statistic,
  Row,
  Col,
  Slider,
  message,
  Switch,
  Tooltip,
} from "antd";
import {
  ThunderboltOutlined,
  ReloadOutlined,
  ClockCircleOutlined,
} from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import HelpButton from "../Common/HelpButton";

export interface MemCleanResult {
  freed_kb: number;
  before_avail_kb: number;
  after_avail_kb: number;
  processes_trimmed: number;
}

export interface BenchResult {
  timestamp: string;
  ram_available_mb: number;
  disk_active_pct: number | null;
  cpu_processor_pct: number | null;
  appdata_open_ms: number | null;
  heavy_processes: number | null;
}

function fmtMB(kb: number): string {
  const mb = kb / 1024;
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`;
  return `${mb.toFixed(0)} MB`;
}

export default function MemoryCleaner() {
  const [lastResult, setLastResult] = useState<MemCleanResult | null>(null);
  const [bench, setBench] = useState<BenchResult | null>(null);
  const [cleaning, setCleaning] = useState(false);
  const [benchLoading, setBenchLoading] = useState(false);
  const [cleaningTemp, setCleaningTemp] = useState(false);
  const [autoClean, setAutoClean] = useState(false);
  const [threshold, setThreshold] = useState(2048); // MB
  const [msg, msgCtx] = message.useMessage();

  const runBench = useCallback(async () => {
    setBenchLoading(true);
    try {
      const out = await invoke<string>("run_benchmark");
      const parsed = JSON.parse(out) as BenchResult;
      setBench(parsed);
    } catch (e) {
      msg.warning("Benchmark: " + e);
    } finally {
      setBenchLoading(false);
    }
  }, [msg]);

  const doClean = useCallback(async () => {
    setCleaning(true);
    try {
      const r = await invoke<MemCleanResult>("clean_memory", {
        threshold_mb: threshold,
      });
      const mb = r.freed_kb / 1024;
      let freedStr: string;
      if (mb >= 1024) freedStr = `${(mb / 1024).toFixed(2)} GB`;
      else freedStr = `${mb.toFixed(0)} MB`;
      setLastResult(r);
      if (r.freed_kb > 0) {
        msg.success(`Giải phóng ${freedStr} · ${r.processes_trimmed} process`);
      } else {
        msg.info("RAM đã tối ưu, chưa có process nào để dọn.");
      }
      runBench();
    } catch (e) {
      msg.error("Lỗi dọn RAM: " + e);
    } finally {
      setCleaning(false);
    }
  }, [threshold, msg, runBench]);

  const doCleanTemp = useCallback(async () => {
    setCleaningTemp(true);
    try {
      const [ok, out] = await invoke<[boolean, string]>("run_memory_cleaner");
      msg.success(ok ? "Dọn Temp/Prefetch xong" : "Lỗi: " + out);
    } catch (e) {
      msg.error("Lỗi dọn Temp: " + e);
    } finally {
      setCleaningTemp(false);
    }
  }, [msg]);

  // Load initial benchmark once
  useEffect(() => { runBench(); }, [runBench]);

  // Auto-clean every 10 seconds if enabled and RAM below threshold
  useEffect(() => {
    if (!autoClean) return;
    const t = setInterval(() => {
      if (bench && bench.ram_available_mb < threshold / 1024) {
        doClean();
      }
      runBench();
    }, 10000);
    return () => clearInterval(t);
  }, [autoClean, bench, threshold, doClean, runBench]);

  return (
    <div>
      {msgCtx}
      <Card
        size="small"
        title={
          <Space>
            <ThunderboltOutlined />
            Dọn RAM Windows
            <HelpButton title="Memory Cleaner" content="Giải phóng RAM trống, kết thúc các tiến trình chiếm dụng nhiều bộ nhớ, thiết lập tự động dọn RAM định kỳ."/>
          </Space>
        }
        extra={
          <Space>
            <Tooltip title="Tự động dọn mỗi 10s khi RAM available < ngưỡng">
              <Switch size="small" checked={autoClean} onChange={setAutoClean} />
            </Tooltip>
          </Space>
        }
      >
        {/* RAM stats */}
        <Row gutter={[14, 14]} style={{ marginBottom: 16 }}>
          <Col span={8}>
            <Card size="small" hoverable onClick={() => runBench()} style={{ cursor: "zoom-in" }}>
              <Statistic
                title={
                  <Space>
                    <ReloadOutlined />
                    RAM available hiện tại
                  </Space>
                }
                value={bench ? bench.ram_available_mb * 1024 * 1024 : 0}
                formatter={(v) => fmtMB(Number(v) / 1024)}
                loading={benchLoading}
                valueStyle={{ color: bench && bench.ram_available_mb > 1024 ? "#34d399" : "#f87171" }}
              />
            </Card>
          </Col>
          <Col span={8}>
            <Card size="small">
              <Statistic
                title={
                  <Space>
                    <ClockCircleOutlined />
                    Giải phóng (lần trước)
                  </Space>
                }
                value={lastResult ? lastResult.freed_kb : 0}
                formatter={(v) => fmtMB(Number(v))}
                valueStyle={{ color: "#fbbf24" }}
              />
              {lastResult && lastResult.processes_trimmed > 0 && (
                <div style={{ fontSize: 12, color: "#94a3b8" }}>
                  {lastResult.processes_trimmed} process · trước: {fmtMB(lastResult.before_avail_kb)} → sau: {fmtMB(lastResult.after_avail_kb)}
                </div>
              )}
            </Card>
          </Col>
          <Col span={8}>
            <Card size="small">
              <Statistic
                title="Ngưỡng tự động (MB)"
                value={threshold}
                formatter={(v) => `${Math.round(Number(v) / 1024)} GB`}
              />
              <Slider
                min={256}
                max={8192}
                step={256}
                value={threshold}
                onChange={setThreshold}
                tooltip={{ formatter: (v) => `${v} MB` }}
              />
            </Card>
          </Col>
        </Row>

        {/* Action buttons */}
        <Row gutter={[14, 14]} style={{ marginBottom: 16 }}>
          <Col span={12}>
            <Space direction="vertical" size="small" style={{ width: "100%" }}>
              <Alert
                type="info"
                showIcon
                message="EmptyWorkingSet + Temp/Prefetch"
                description="Windows API buông page RAM process > ngưỡng. Nút 'Dọn Temp' xóa %TEMP% + Prefetch. An toàn — không kill process nào."
              />
              <Space style={{ marginTop: 8 }}>
                <Button
                  size="small"
                  icon={<ReloadOutlined />}
                  loading={cleaningTemp}
                  onClick={doCleanTemp}
                >
                  Dọn Temp
                </Button>
                <Tooltip title="Tự động chạy mỗi 10s khi RAM < ngưỡng">
                  <Switch size="small" checked={autoClean} onChange={setAutoClean} />
                </Tooltip>
              </Space>
            </Space>
          </Col>
          <Col span={12} style={{ textAlign: "right" }}>
            <Space>
              <Button icon={<ReloadOutlined />} onClick={runBench} loading={benchLoading}>
                Refresh RAM
              </Button>
              <Button
                type="primary"
                danger
                icon={<ThunderboltOutlined />}
                onClick={doClean}
                loading={cleaning}
                size="large"
              >
                {cleaning ? "Đang dọn..." : "Dọn RAM ngay"}
              </Button>
            </Space>
          </Col>
        </Row>

        {/* Benchmark full */}
        {bench && (
          <Card size="small" title="Benchmark chi tiết" loading={benchLoading}>
            <Row gutter={14}>
              <Col span={6}>CPU: {bench.cpu_processor_pct ?? "N/A"}%</Col>
              <Col span={6}>Disk active: {bench.disk_active_pct ?? "N/A"}%</Col>
              <Col span={6}>Mở AppData: {bench.appdata_open_ms ?? "N/A"}ms</Col>
              <Col span={6}>Tiến trình nặng: {bench.heavy_processes ?? 0}</Col>
            </Row>
          </Card>
        )}
      </Card>
    </div>
  );
}