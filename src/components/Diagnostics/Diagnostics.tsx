import { useState, useCallback } from "react";
import { Card, Button, Space, Alert, Row, Col, message, Typography } from "antd";
import {
  MedicineBoxOutlined,
  HddOutlined,
  FileSearchOutlined,
  DatabaseOutlined,
  ReloadOutlined,
} from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import HelpButton from "../Common/HelpButton";

const { Text } = Typography;

interface DiagResult {
  action: string;
  output: string;
  ok: boolean;
}

function ResultBox({ label, loading, result, onRun, icon, danger }: {
  label: string;
  loading: boolean;
  result: DiagResult | null;
  onRun: () => void;
  icon: React.ReactNode;
  danger?: boolean;
}) {
  return (
    <Card
      size="small"
      title={
        <Space>
          {icon}
          {label}
        </Space>
      }
      extra={
        <Button size="small" onClick={onRun} loading={loading} danger={danger}>
          Chạy
        </Button>
      }
    >
      {result ? (
        <pre
          style={{
            whiteSpace: "pre-wrap",
            background: "rgba(0,0,0,0.3)",
            padding: 10,
            borderRadius: 8,
            fontSize: 12,
            maxHeight: 320,
            overflow: "auto",
            color: result.ok ? "#a7f3d0" : "#fca5a5",
          }}
        >
          {result.output}
        </pre>
      ) : (
        <Text type="secondary">Chưa chạy. Nhấn "Chạy" để bắt đầu.</Text>
      )}
    </Card>
  );
}

export default function Diagnostics() {
  const [msg, msgCtx] = message.useMessage();
  const [repair, setRepair] = useState<DiagResult | null>(null);
  const [repairL, setRepairL] = useState(false);
  const [chkdsk, setChkdsk] = useState<DiagResult | null>(null);
  const [chkdskL, setChkdskL] = useState(false);
  const [events, setEvents] = useState<DiagResult | null>(null);
  const [eventsL, setEventsL] = useState(false);
  const [smart, setSmart] = useState<DiagResult | null>(null);
  const [smartL, setSmartL] = useState(false);

  const run = useCallback(
    async (cmd: string, setter: (r: DiagResult) => void, setL: (b: boolean) => void) => {
      setL(true);
      try {
        const r = await invoke<DiagResult>(cmd);
        setter(r);
      } catch (e) {
        msg.error("Lỗi: " + e);
      } finally {
        setL(false);
      }
    },
    [msg]
  );

  return (
    <div>
      {msgCtx}
      <Alert
        type="info"
        showIcon
        style={{ marginBottom: 12 }}
        message="Chẩn đoán & sửa chữa hệ thống — đọc-only trừ khi bạn bấm nút sửa chữa"
        description={
          <>
            <b>DISM + SFC</b> có thể mất vài phút. <b>Chkdsk /scan</b> chỉ kiểm tra online, không cần reboot.
            Tất cả cần quyền Admin.
          </>
        }
      />
      <Row gutter={[12, 12]}>
        <Col xs={24} lg={12}>
          <ResultBox
            label="Sửa file hệ thống (DISM + SFC)"
            icon={<MedicineBoxOutlined />}
            loading={repairL}
            result={repair}
            onRun={() => run("repair_system_files", setRepair, setRepairL)}
            danger
          />
        </Col>
        <Col xs={24} lg={12}>
          <ResultBox
            label="Chkdsk scan ổ đĩa"
            icon={<HddOutlined />}
            loading={chkdskL}
            result={chkdsk}
            onRun={() => run("run_chkdsk_scan", setChkdsk, setChkdskL)}
          />
        </Col>
        <Col xs={24} lg={12}>
          <ResultBox
            label="Lỗi hệ thống / crash 24h (Event Log)"
            icon={<FileSearchOutlined />}
            loading={eventsL}
            result={events}
            onRun={() => run("get_recent_events", setEvents, setEventsL)}
          />
        </Col>
        <Col xs={24} lg={12}>
          <ResultBox
            label="Sức khỏe ổ đĩa (SMART)"
            icon={<DatabaseOutlined />}
            loading={smartL}
            result={smart}
            onRun={() => run("get_drive_smart", setSmart, setSmartL)}
          />
        </Col>
      </Row>
      <div style={{ marginTop: 12 }}>
        <Button
          icon={<ReloadOutlined />}
          onClick={() => {
            setRepair(null); setChkdsk(null); setEvents(null); setSmart(null);
          }}
        >
          Xóa kết quả
        </Button>
      </div>
    </div>
  );
}