import { useEffect, useState } from "react";
import { Card, Button, Tag, Alert, message, Tooltip, Row, Col, Modal } from "antd";
import {
  ReloadOutlined,
  PlayCircleOutlined,
  WarningOutlined,
  CheckCircleOutlined,
  CloseOutlined,
} from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";

interface AdvTool {
  id: string;
  label: string;
  desc: string;
  admin: boolean;
  risky: boolean;
}

export default function AdvancedTools() {
  const [tools, setTools] = useState<AdvTool[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [results, setResults] = useState<Record<string, { ok: boolean; out: string }>>({});
  const [isAdmin, setIsAdmin] = useState(false);
  const [msg, msgCtx] = message.useMessage();

  const load = async () => {
    try { setTools(await invoke<AdvTool[]>("get_advanced_tools")); } catch (e) { /* catalog nhúng, không cần */ }
    try { setIsAdmin(await invoke<boolean>("is_admin")); } catch (e) {}
  };
  useEffect(() => { load(); }, []);

  const run = async (t: AdvTool) => {
    const doRun = async () => {
      setBusy(t.id);
      try {
        const [ok, out] = await invoke<[boolean, string]>("run_advanced_tool", { id: t.id });
        setResults((r) => ({ ...r, [t.id]: { ok, out } }));
        msg.success(ok ? `Hoàn tất: ${t.label}` : `Lỗi: ${t.label}`);
      } catch (e) { msg.error("Lỗi: " + e); }
      finally { setBusy(null); }
    };
    if (t.risky) {
      Modal.confirm({
        title: `Chạy "${t.label}"?`,
        content: t.desc + " Hành động này có thể gây gián đoạn ngắn (màn hình nhấp nháy, taskbar ẩn hiện).",
        okText: "Chạy", cancelText: "Hủy", onOk: doRun,
      });
    } else doRun();
  };

  const dangerColor = (t: AdvTool) => (t.risky ? "volcano" : t.admin ? "orange" : "blue");

  return (
    <Card
      size="small" title="Công cụ nâng cao"
      extra={<Button icon={<ReloadOutlined />} onClick={load}>Làm mới</Button>}
    >
      {msgCtx}
      {!isAdmin && (
        <Alert style={{ marginBottom: 12 }} type="warning" showIcon
          message="Một số công cụ cần quyền admin (Hibernate, Restore Point, SFC). Bấm nút dưới để chạy lại app với quyền admin."
          action={<Button size="small" danger onClick={() => invoke("relaunch_admin")}>Chạy lại với quyền admin</Button>} />
      )}
      <Row gutter={[12, 12]}>
        {tools.map((t) => (
          <Col xs={24} sm={12} lg={8} key={t.id}>
            <Card size="small">
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
                <b>{t.label}</b>
                <Tag color={dangerColor(t)} icon={t.risky ? <WarningOutlined /> : undefined}>
                  {t.risky ? "Có gián đoạn" : t.admin ? "Cần admin" : "An toàn"}
                </Tag>
              </div>
              <div style={{ color: "#94a3b8", fontSize: 12, minHeight: 36 }}>{t.desc}</div>
              <div style={{ marginTop: 8, display: "flex", gap: 8, alignItems: "center" }}>
                <Button size="small" type="primary" icon={<PlayCircleOutlined />} loading={busy === t.id} disabled={busy !== null || (t.admin && !isAdmin)} onClick={() => run(t)}>
                  Chạy
                </Button>
                {results[t.id] && (
                  <Tooltip title={results[t.id].out}>
                    <Tag color={results[t.id].ok ? "success" : "error"} icon={results[t.id].ok ? <CheckCircleOutlined /> : <CloseOutlined />}>
                      {results[t.id].ok ? "OK" : "Lỗi"}
                    </Tag>
                  </Tooltip>
                )}
              </div>
            </Card>
          </Col>
        ))}
      </Row>
    </Card>
  );
}