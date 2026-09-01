import { useEffect, useState, useCallback } from "react";
import {
  Card,
  Table,
  Tag,
  Button,
  Space,
  Alert,
  Progress,
  Statistic,
  Row,
  Col,
  message,
  Checkbox,
} from "antd";
import {
  ClearOutlined,
  FundOutlined,
  ThunderboltOutlined,
  ReloadOutlined,
} from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";

interface CleanCategory {
  id: string;
  label: string;
  path: string;
  size_bytes: number;
  item_count: number;
  exists: boolean;
}

interface DiskCleanScan {
  categories: CleanCategory[];
  total_size: number;
  completed: boolean;
}

interface BenchResult {
  ram_available_mb: number;
  disk_active_pct: number | null;
  cpu_processor_pct: number | null;
}

function fmtBytes(b: number): string {
  if (!b) return "0 MB";
  if (b >= 1073741824) return `${(b / 1073741824).toFixed(2)} GB`;
  if (b >= 1048576) return `${(b / 1048576).toFixed(0)} MB`;
  if (b >= 1024) return `${(b / 1024).toFixed(0)} KB`;
  return `${b} B`;
}

export default function DiskCleaner() {
  const [scan, setScan] = useState<DiskCleanScan | null>(null);
  const [loading, setLoading] = useState(false);
  const [cleaning, setCleaning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string[]>([]);
  const [bench, setBench] = useState<BenchResult | null>(null);
  const [benchLoading, setBenchLoading] = useState(false);
  const [msg, msgCtx] = message.useMessage();

  const loadScan = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<DiskCleanScan>("get_disk_clean_candidates");
      setScan(data);
      setSelected(data.categories.map((c) => c.id));
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadScan();
  }, [loadScan]);

  const runBench = useCallback(async () => {
    setBenchLoading(true);
    try {
      const out = await invoke<string>("run_benchmark");
      const parsed = JSON.parse(out) as BenchResult;
      setBench(parsed);
    } catch (e) {
      msg.warning("Benchmark lỗi: " + e);
    } finally {
      setBenchLoading(false);
    }
  }, [msg]);

  const doClean = useCallback(async () => {
    if (!selected.length) {
      msg.warning("Chọn ít nhất 1 hạng mục để dọn");
      return;
    }
    setCleaning(true);
    try {
      const freedBytes = await invoke<number>("run_disk_clean", { ids: selected });
      msg.success(`Đã dọn xong! Giải phóng ${fmtBytes(freedBytes)}`);
      await loadScan();
      runBench();
    } catch (e) {
      msg.error("Dọn lỗi: " + e);
    } finally {
      setCleaning(false);
    }
  }, [selected, msg, loadScan, runBench]);

  if (error) return <Alert type="error" showIcon message="Lỗi tải scan" description={error} />;

  const columns = [
    {
      title: "Hạng mục",
      dataIndex: "label",
      key: "label",
      render: (_: unknown, r: CleanCategory) => <b>{r.label}</b>,
    },
    {
      title: "Dung lượng",
      dataIndex: "size_bytes",
      key: "size",
      align: "right" as const,
      render: (v: number) => (
        <Tag color={v >= 104857600 ? "red" : v > 0 ? "orange" : "default"}>{fmtBytes(v)}</Tag>
      ),
    },
    {
      title: "Số mục",
      dataIndex: "item_count",
      key: "count",
      align: "right" as const,
      render: (v: number) => (v ? v.toLocaleString() : "—"),
    },
    {
      title: "Đường dẫn",
      dataIndex: "path",
      key: "path",
      ellipsis: true,
      render: (v: string) => <span style={{ color: "#94a3b8", fontSize: 12 }}>{v}</span>,
    },
  ];

  const total = scan?.total_size ?? 0;
  const totalPct = total ? Math.min(100, (total / (10 * 1073741824)) * 100) : 0;

  return (
    <div>
      {msgCtx}
      <Card
        size="small"
        title={
          <Space>
            <ClearOutlined />
            Dọn rác Windows
          </Space>
        }
        extra={
          <Space>
            <Button icon={<ReloadOutlined />} onClick={loadScan} loading={loading}>
              Quét lại
            </Button>
          </Space>
        }
      >
        <Row gutter={[14, 14]} style={{ marginBottom: 16 }}>
          <Col span={6}>
            <Card size="small">
              <Statistic
                title="Tổng rác tìm thấy"
                value={total}
                formatter={(v) => fmtBytes(Number(v))}
                valueStyle={{ color: totalPct > 60 ? "#f87171" : "#fbbf24" }}
              />
            </Card>
          </Col>
          <Col span={6}>
            <Card size="small">
              <Statistic
                title="Hạng mục"
                value={scan?.categories.length ?? 0}
                suffix="mục"
                valueStyle={{ color: "#60a5fa" }}
              />
            </Card>
          </Col>
          <Col span={12}>
            <Card size="small">
              <div style={{ marginBottom: 6 }}>Mức độ đầy (quy ước ~10GB tối đa)</div>
              <Progress
                percent={Math.round(totalPct)}
                status={totalPct > 60 ? "exception" : totalPct > 0 ? "active" : "success"}
              />
            </Card>
          </Col>
        </Row>

        <div style={{ marginBottom: 12 }}>
          <Checkbox
            checked={selected.length === scan?.categories.length}
            indeterminate={
              !!selected.length && selected.length !== scan?.categories.length
            }
            onChange={(e) =>
              setSelected(e.target.checked ? (scan?.categories.map((c) => c.id) ?? []) : [])
            }
          >
            Chọn tất cả
          </Checkbox>
          <Space style={{ float: "right" }}>
            <Button
              icon={<FundOutlined />}
              onClick={runBench}
              loading={benchLoading}
            >
              {bench ? "Benchmark lại RAM" : "Benchmark RAM hiện tại"}
            </Button>
            <Button
              type="primary"
              danger
              icon={<ThunderboltOutlined />}
              onClick={doClean}
              loading={cleaning}
            >
              Dọn {selected.length ? `(${selected.length} mục)` : ""}
            </Button>
          </Space>
        </div>

        {bench && (
          <Alert
            type="success"
            showIcon
            style={{ marginBottom: 12 }}
            message={`RAM available: ${(bench.ram_available_mb / 1024).toFixed(2)} GB`}
            description={
              bench.cpu_processor_pct != null
                ? `CPU: ${bench.cpu_processor_pct}% · Disk active: ${
                    bench.disk_active_pct ?? "N/A"
                  }% · ${new Date().toLocaleTimeString()}`
                : `CPU: ${bench.cpu_processor_pct}% · ${new Date().toLocaleTimeString()}`
            }
          />
        )}

        <Table
          size="small"
          rowKey="id"
          loading={loading}
          columns={columns}
          dataSource={scan?.categories ?? []}
          pagination={false}
          rowSelection={{
            selectedRowKeys: selected,
            onChange: (keys) => setSelected(keys as string[]),
          }}
        />
      </Card>
    </div>
  );
}
