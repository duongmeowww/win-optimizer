import { useEffect, useState, useCallback } from "react";
import {
  Card,
  Switch,
  Button,
  Space,
  Tag,
  Alert,
  Tooltip,
  Modal,
  message,
  Row,
  Col,
  Tabs,
  List,
  Input,
  Select,
  Statistic,
  Empty,
  Progress,
} from "antd";
import {
  ThunderboltOutlined,
  ReloadOutlined,
  UndoOutlined,
  CheckCircleOutlined,
  WarningOutlined,
  PlayCircleOutlined,
  StopOutlined,
  CloseOutlined,
  SafetyCertificateOutlined,
  RocketOutlined,
  SaveOutlined,
} from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface GamingTweak { id: string; label: string; desc: string; category: string; active: boolean; tradeoff: boolean; }
interface GamingPreset { id: string; label: string; desc: string; aggressive: boolean; tweaks: string[]; active: boolean; active_count: number; total: number; }
interface PresetStep { index: number; total: number; id: string; ok: boolean; label: string; msg: string; }
interface SessionState { active: boolean; saved_power_guid: string; suspended: string[]; power_plan_switched: boolean; }
interface GameProfile { path: string; name: string; gpu_pref: number; fso_off: boolean; run_as_admin: boolean; exists: boolean; }
interface VerifyItem { id: string; label: string; ok: boolean; expected: string; }

const CATEGORY_COLORS: Record<string, string> = {
  GPU: "cyan", Network: "geekblue", CPU: "purple", Input: "lime", "Trade-off": "volcano",
};

export default function Gaming() {
  return (
    <Tabs
      items={[
        { key: "tweaks", label: <Space><ThunderboltOutlined />Tweaks</Space>, children: <TweaksTab /> },
        { key: "presets", label: <Space><RocketOutlined />Gaming Profiles</Space>, children: <PresetsTab /> },
        { key: "session", label: <Space><PlayCircleOutlined />Boost Session</Space>, children: <SessionTab /> },
        { key: "pergame", label: <Space><SaveOutlined />Per-game</Space>, children: <PerGameTab /> },
        { key: "verify", label: <Space><SafetyCertificateOutlined />Kiểm tra</Space>, children: <VerifyTab /> },
      ]}
    />
  );
}

// ================= Tweaks =================
function TweaksTab() {
  const [tweaks, setTweaks] = useState<GamingTweak[]>([]);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [confirmTarget, setConfirmTarget] = useState<GamingTweak | null>(null);
  const [isAdmin, setIsAdmin] = useState(false);
  const [msg, msgCtx] = message.useMessage();

  const ADMIN_NEEDED = ["hags", "sys_resp", "mmcss"]; // HKLM protected keys

  const load = useCallback(async () => {
    setLoading(true);
    try { setTweaks(await invoke<GamingTweak[]>("get_gaming_tweaks")); }
    catch (e) { msg.error("Lỗi tải tweaks: " + e); }
    finally { setLoading(false); }
  }, [msg]);
  useEffect(() => { load(); }, [load]);
  useEffect(() => { invoke<boolean>("is_admin").then(setIsAdmin).catch(() => {}); }, []);

  const toggle = useCallback(async (t: GamingTweak, action: "apply" | "revert") => {
    setBusy(t.id);
    try {
      const [ok, out] = await invoke<[boolean, string]>("apply_gaming_tweak", { id: t.id, action });
      if (action === "apply" && t.tradeoff) {
        Modal.warning({ title: "Cảnh báo trade-off", content: "Đã áp dụng. Một số tweak cần KHỞI ĐỘNG LẠI. VBS/HVCI tắt có thể làm gãy Valorant/R6." });
      }
      msg.success(ok ? out : "Thất bại: " + out);
      await load();
    } catch (e) { msg.error("Lỗi: " + e); }
    finally { setBusy(null); }
  }, [msg, load]);

  const handleSwitch = (t: GamingTweak, checked: boolean) => {
    const action = checked ? "apply" : "revert";
    if (action === "apply" && t.tradeoff) { setConfirmTarget(t); return; }
    toggle(t, action);
  };

  const groups: Record<string, GamingTweak[]> = {};
  for (const t of tweaks) (groups[t.category] ??= []).push(t);
  const order = ["GPU", "Network", "CPU", "Input", "Trade-off"];

  return (
    <Card
      size="small" title="Registry Tweaks Catalog"
      extra={
        <Space>
          <Alert type="info" showIcon message="Mọi tweak đều reversible" style={{ marginRight: 8 }} />
          <Button icon={<ReloadOutlined />} onClick={load} loading={loading}>Làm mới</Button>
          <Button icon={<UndoOutlined />} onClick={() => {
            Modal.confirm({ title: "Revert tất cả về mặc định?", content: "Tất cả tweak đang bật sẽ bị tắt. Power plan về Balanced.", onOk: async () => { for (const t of tweaks.filter((x) => x.active)) await toggle(t, "revert"); } });
          }}>Hoàn tác hết</Button>
        </Space>
      }
    >
      {msgCtx}
      {!isAdmin && (
        <Alert style={{ marginBottom: 12 }} type="warning" showIcon
          message="Đang chạy không có quyền admin — 3 tweak (HAGS, SystemResponsiveness, MMCSS) cần admin để sửa registry HKLM protected."
          action={<Button size="small" danger onClick={() => invoke("relaunch_admin")}>Chạy lại với quyền admin</Button>} />
      )}
      {order.map((cat) => {
        const items = groups[cat] ?? [];
        if (!items.length) return null;
        return (
          <div key={cat} style={{ marginBottom: 18 }}>
            <div style={{ marginBottom: 8 }}>
              <Tag color={CATEGORY_COLORS[cat] || "default"}>
                {cat === "Trade-off" ? <Space size={4}><WarningOutlined /> {cat} — có rủi ro</Space> : cat}
              </Tag>
            </div>
            <Row gutter={[12, 12]}>
              {items.map((t) => (
                <Col xs={24} sm={12} lg={8} key={t.id}>
                  <Card
                    size="small"
                    style={{ borderColor: t.active ? "#22d3ee" : undefined, background: t.active ? "rgba(34,211,238,0.06)" : "rgba(255,255,255,0.03)" }}
                  >
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                      <b>{t.label}</b>
                      <Switch checked={t.active} loading={busy === t.id} disabled={busy !== null || (!isAdmin && ADMIN_NEEDED.includes(t.id))} onChange={(c) => handleSwitch(t, c)} />
                    </div>
                    <div style={{ color: "#94a3b8", fontSize: 12, marginTop: 6, minHeight: 48 }}>{t.desc}</div>
                    <div style={{ marginTop: 8, display: "flex", gap: 6, alignItems: "center" }}>
                      <Tag color={t.active ? "success" : "default"} icon={t.active ? <CheckCircleOutlined /> : undefined}>
                        {t.active ? "Đang bật" : "Tắt"}
                      </Tag>
                      {!isAdmin && ADMIN_NEEDED.includes(t.id) && <Tooltip title="Cần chạy app với quyền admin để sửa registry này"><Tag color="orange">Cần admin</Tag></Tooltip>}
                      {t.tradeoff && <Tooltip title="Đánh đổi bảo mật lấy hiệu năng"><Tag color="volcano">Trade-off</Tag></Tooltip>}
                    </div>
                  </Card>
                </Col>
              ))}
            </Row>
          </div>
        );
      })}
      <Modal
        open={!!confirmTarget} title="Xác nhận tweak rủi ro"
        okText="Tôi hiểu rủi ro, áp dụng" cancelText="Hủy" okButtonProps={{ danger: true }}
        onOk={() => { if (confirmTarget) toggle(confirmTarget, "apply"); setConfirmTarget(null); }}
        onCancel={() => setConfirmTarget(null)}
      >
        <Alert type="warning" showIcon message={confirmTarget?.label}
          description={<><p>{confirmTarget?.desc}</p><p style={{ marginBottom: 0 }}><b>Rủi ro:</b> giảm bảo mật. Có thể cần reboot. Một số game anti-cheat (Valorant, R6) có thể <b>từ chối chạy</b>. Revert bất cứ lúc nào.</p></>} />
      </Modal>
    </Card>
  );
}

// ================= Presets =================
function PresetsTab() {
  const [presets, setPresets] = useState<GamingPreset[]>([]);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [msg, msgCtx] = message.useMessage();
  const [steps, setSteps] = useState<PresetStep[]>([]);
  const [running, setRunning] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try { setPresets(await invoke<GamingPreset[]>(`get_gaming_presets`)); }
    catch (e) { msg.error("Lỗi: " + e); }
    finally { setLoading(false); }
  }, [msg]);
  useEffect(() => { load(); }, [load]);

  useEffect(() => {
    const un = listen<PresetStep>("preset-progress", (ev) => {
      setSteps((prev) => [...prev, ev.payload]);
    });
    return () => { un.then((f) => f()); };
  }, []);

  const run = useCallback(async (id: string, action: "apply" | "revert") => {
    setBusy(id);
    setRunning(true);
    setSteps([]);
    try {
      const [ok, out] = await invoke<[boolean, string]>("apply_gaming_preset", { id, action });
      if (ok) msg.success(out); else msg.warning(out);
      await load();
    } catch (e) { msg.error("Lỗi: " + e); }
    finally { setBusy(null); setRunning(false); }
  }, [msg, load]);

  return (
    <Card size="small" title="Gaming Profiles (presets)" extra={<Button icon={<ReloadOutlined />} loading={loading} onClick={load}>Làm mới</Button>}>
      {msgCtx}
      <List
        dataSource={presets}
        locale={{ emptyText: <Empty description="Chưa có preset" /> }}
        renderItem={(p) => (
          <List.Item
            actions={[
              <Space key="a">
                <Button type="primary" icon={<PlayCircleOutlined />} loading={busy === p.id}
                  onClick={() => run(p.id, "apply")}>
                  Áp dụng
                </Button>
                <Button icon={<UndoOutlined />} onClick={() => run(p.id, "revert")}>Hoàn tác</Button>
              </Space>,
            ]}
          >
            <List.Item.Meta
              title={
                <Space>
                  {p.label}
                  {p.active ? (
                    <Tag color="success">Đang bật ({p.active_count}/{p.total})</Tag>
                  ) : p.active_count > 0 ? (
                    <Tag color="processing">Đang bật {p.active_count}/{p.total}</Tag>
                  ) : (
                    <Tag color="default">Đang tắt</Tag>
                  )}
                </Space>
              }
              description={
                <Space direction="vertical" size={0}>
                  <span>{p.desc}</span>
                  <Progress percent={Math.round((p.active_count / p.total) * 100)} size="small" format={() => `${p.active_count}/${p.total} tweak hoạt động`} />
                </Space>
              }
            />
          </List.Item>
        )}
      />
      {(running || steps.length > 0) && (
        <Card size="small" style={{ marginTop: 16 }} title={
          <Space>Quá trình chạy <Tag color={running ? "processing" : (steps.every((s) => s.ok) ? "success" : "warning")}>{steps.filter((s) => s.ok).length}/{steps.length}</Tag>
            {!running && <Button size="small" type="text" icon={<CloseOutlined />} onClick={() => setSteps([])} />}
          </Space>
        }>
          {running && <Progress percent={Math.round((steps.filter((s) => s.ok).length / (steps[steps.length - 1]?.total || 1)) * 100)} />}
          <List
            size="small"
            dataSource={steps}
            renderItem={(s) => (
              <List.Item style={{ padding: "4px 0" }}>
                <Space>
                  {s.ok ? <CheckCircleOutlined style={{ color: "#52c41a" }} /> : <WarningOutlined style={{ color: "#faad14" }} />}
                  <span>{s.label}</span>
                  <span style={{ color: s.ok ? "#52c41a" : "#faad14" }}>{s.ok ? "Đang bật" : "Lỗi"}</span>
                </Space>
              </List.Item>
            )}
          />
        </Card>
      )}
    </Card>
  );
}

// ================= Session =================
function SessionTab() {
  const [session, setSession] = useState<SessionState>({ active: false, saved_power_guid: "", suspended: [], power_plan_switched: false });
  const [apps, setApps] = useState("chrome,spotify,discord,OneDrive");
  const [busy, setBusy] = useState(false);
  const [msg, msgCtx] = message.useMessage();

  const load = useCallback(async () => {
    try { setSession(await invoke<SessionState>("get_gaming_session")); }
    catch (e) { msg.error("Lỗi: " + e); }
  }, [msg]);
  useEffect(() => { load(); }, [load]);

  const start = async () => {
    setBusy(true);
    try {
      const list = apps.split(",").map((s) => s.trim()).filter(Boolean);
      setSession(await invoke<SessionState>("start_gaming_session", { apps: list }));
      msg.success("Boost session BẮT ĐẦU. Power plan → Ultimate, app nền → Idle priority.");
    } catch (e) { msg.error("Lỗi: " + e); }
    finally { setBusy(false); }
  };
  const stop = async () => {
    setBusy(true);
    try {
      setSession(await invoke<SessionState>("stop_gaming_session"));
      msg.success("Boost session KẾT THÚC. Power plan đã khôi phục.");
    } catch (e) { msg.error("Lỗi: " + e); }
    finally { setBusy(false); }
  };

  return (
    <Card size="small" title="Gaming Boost Session" extra={<Button icon={<ReloadOutlined />} onClick={load}>Làm mới</Button>}>
      {msgCtx}
      <Row gutter={[16, 16]}>
        <Col span={8}>
          <Card size="small">
            <Statistic title="Trạng thái" value={session.active ? "ĐANG BOOST" : "Không hoạt động"} valueStyle={{ color: session.active ? "#22d3ee" : "#94a3b8" }} />
          </Card>
        </Col>
        <Col span={8}><Card size="small"><Statistic title="Suspended apps" value={session.suspended.length} /></Card></Col>
        <Col span={8}><Card size="small"><Statistic title="Power plan switched" value={session.power_plan_switched ? "Yes" : "No"} /></Card></Col>
      </Row>
      <div style={{ marginTop: 16 }}>
        <Alert
          type="info" showIcon
          message="Gaming Session: chuyển sang Ultimate Performance plan + hạ priority app nền xuống Idle để CPU/GPU dồn cho game."
          description="Nhập tên process (không .exe) cách nhau bởi dấu phẩy để hạ priority khi boost."
          style={{ marginBottom: 12 }}
        />
        <Space.Compact style={{ width: "100%", marginBottom: 12 }}>
          <Input value={apps} onChange={(e) => setApps(e.target.value)} placeholder="chrome,spotify,discord" />
        </Space.Compact>
        {session.active ? (
          <Button danger type="primary" icon={<StopOutlined />} loading={busy} onClick={stop} block>Kết thúc Boost Session (khôi phục)</Button>
        ) : (
          <Button type="primary" icon={<PlayCircleOutlined />} loading={busy} onClick={start} block>Bắt đầu Boost Session</Button>
        )}
      </div>
    </Card>
  );
}

// ================= Per-game =================
function PerGameTab() {
  const [path, setPath] = useState("");
  const [gpuPref, setGpuPref] = useState(0);
  const [runAdmin, setRunAdmin] = useState(false);
  const [profile, setProfile] = useState<GameProfile | null>(null);
  const [busy, setBusy] = useState(false);
  const [msg, msgCtx] = message.useMessage();

  const loadProfile = useCallback(async (p: string) => {
    setBusy(true);
    try { setProfile(await invoke<GameProfile>("get_game_profile", { path: p })); }
    catch (e) { msg.error("Lỗi: " + e); }
    finally { setBusy(false); }
  }, [msg]);

  const save = async () => {
    if (!path) return msg.warning("Nhập đường dẫn .exe");
    setBusy(true);
    try {
      const [ok, out] = await invoke<[boolean, string]>("set_game_profile", { path, gpuPref, fsoOff: false, runAsAdmin: runAdmin });
      if (ok) msg.success(out); else msg.warning(out);
      await loadProfile(path);
    } catch (e) { msg.error("Lỗi: " + e); }
    finally { setBusy(false); }
  };

  return (
    <Card size="small" title="Per-game Profiles (HKCU, không cần admin)"
      extra={<Button icon={<ReloadOutlined />} onClick={() => path && loadProfile(path)}>Làm mới</Button>}>
      {msgCtx}
      <Alert type="info" showIcon
        message="Cấu hình GPU preference + Run-as-admin cho từng .exe game. Lưu trong HKCU — chỉ áp dụng cho user này, không cần quyền admin."
        style={{ marginBottom: 12 }} />
      <Space direction="vertical" style={{ width: "100%" }}>
        <Input placeholder="Đường dẫn game, ví dụ: C:\Games\cs2\cs2.exe" value={path}
          onChange={(e) => { setPath(e.target.value); setProfile(null); }}
          onPressEnter={() => path && loadProfile(path)} />
        <Space wrap>
          <Select
            style={{ width: 260 }} value={gpuPref}
            onChange={(v) => setGpuPref(v)}
            options={[
              { value: 0, label: "GPU mặc định (Windows chọn)" },
              { value: 1, label: "Power Saving (tích hợp)" },
              { value: 2, label: "High Performance (rời) — khuyên cho game" },
            ]}
          />
          <Switch checked={runAdmin} onChange={setRunAdmin} checkedChildren="Run as admin" unCheckedChildren="Không admin" />
          <Button type="primary" icon={<SaveOutlined />} loading={busy} onClick={save}>Lưu profile</Button>
        </Space>
      </Space>
      {profile && (
        <Card size="small" style={{ marginTop: 12 }}>
          <Space direction="vertical">
            <b>Profile hiện tại: {profile.name}</b>
            <Space wrap>
              <Tag color={profile.gpu_pref === 2 ? "purple" : profile.gpu_pref === 1 ? "gold" : "default"}>
                GPU: {profile.gpu_pref === 2 ? "High Performance" : profile.gpu_pref === 1 ? "Power Saving" : "Default"}
              </Tag>
              <Tag color={profile.run_as_admin ? "red" : "default"}>{profile.run_as_admin ? "Run as admin" : "Không admin"}</Tag>
            </Space>
          </Space>
        </Card>
      )}
    </Card>
  );
}

// ================= Verify =================
function VerifyTab() {
  const [items, setItems] = useState<VerifyItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [msg, msgCtx] = message.useMessage();

  const load = useCallback(async () => {
    setLoading(true);
    try { setItems(await invoke<VerifyItem[]>("verify_gaming_tweaks")); }
    catch (e) { msg.error("L?i: " + e); }
    finally { setLoading(false); }
  }, [msg]);
  useEffect(() => { load(); }, [load]);

  const passed = items.filter((i) => i.ok).length;
  const failed = items.length - passed;

  return (
    <Card size="small" title="Verification Tool — tweak nào đang hoạt động"
      extra={<Button icon={<ReloadOutlined />} onClick={load} loading={loading}>Kiểm tra lại</Button>}>
      {msgCtx}
      <Row gutter={[16, 16]} style={{ marginBottom: 16 }}>
        <Col span={8}><Card size="small"><Statistic title="Đang bật" value={passed} valueStyle={{ color: "#22d3ee" }} /></Card></Col>
        <Col span={8}><Card size="small"><Statistic title="Đang tắt" value={failed} valueStyle={{ color: "#94a3b8" }} /></Card></Col>
        <Col span={8}><Card size="small"><Statistic title="Tổng" value={items.length} /></Card></Col>
      </Row>
      <List
        size="small"
        dataSource={items}
        locale={{ emptyText: <Empty description="Chưa có dữ liệu" /> }}
        renderItem={(it) => (
          <List.Item>
            <List.Item.Meta
              title={it.label}
              description={it.expected}
            />
            <Tag color={it.ok ? "success" : "default"} icon={it.ok ? <CheckCircleOutlined /> : <WarningOutlined />}>
              {it.ok ? "PASS" : "FAIL"}
            </Tag>
          </List.Item>
        )}
      />
    </Card>
  );
}
