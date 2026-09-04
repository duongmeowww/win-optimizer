import { useState, useEffect, useCallback } from "react";
import { Card, Button, Typography, Switch, Table, Tag, message, Spin, Badge, Modal } from "antd";
import { SettingOutlined, DownloadOutlined } from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";

const { Title, Text } = Typography;

interface TweakItem {
  id: string;
  name: string;
  category: string;
  description: string;
  risk: string;
  enabled: boolean;
}

interface UpdateInfo {
  current_version: string;
  latest_version: string;
  has_update: boolean;
  release_url: string;
  release_notes: string;
}

export default function RegistryTweaks() {
  const [tweaks, setTweaks] = useState<TweakItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [updatingId, setUpdatingId] = useState<string | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);

  const fetchTweaks = useCallback(async () => {
    setLoading(true);
    try {
      const res = await invoke<TweakItem[]>("get_registry_tweaks");
      setTweaks(res);
    } catch (err) {
      message.error("Lỗi khi tải danh sách Registry Tweaks: " + err);
    } finally {
      setLoading(false);
    }
  }, []);

  const checkUpdate = useCallback(async () => {
    setCheckingUpdate(true);
    try {
      const res = await invoke<UpdateInfo>("check_for_update");
      setUpdateInfo(res);
      if (res.has_update) {
        message.success(`Có phiên bản mới: ${res.latest_version}`);
      } else {
        message.info("Bạn đang dùng phiên bản mới nhất!");
      }
    } catch (err) {
      message.error("Không thể kiểm tra bản cập nhật: " + err);
    } finally {
      setCheckingUpdate(false);
    }
  }, []);

  useEffect(() => {
    fetchTweaks();
    checkUpdate();
  }, [fetchTweaks, checkUpdate]);

  const handleToggle = async (id: string, currentEnabled: boolean) => {
    setUpdatingId(id);
    try {
      const [success, msgText] = await invoke<[boolean, string]>("apply_registry_tweak", {
        id,
        enable: !currentEnabled,
      });
      if (success) {
        message.success(msgText);
        fetchTweaks();
      } else {
        message.error(msgText);
      }
    } catch (err) {
      message.error("Lỗi khi áp dụng tweak: " + err);
    } finally {
      setUpdatingId(null);
    }
  };

  const columns = [
    {
      title: "Tên Tweak",
      dataIndex: "name",
      key: "name",
      render: (text: string, record: TweakItem) => (
        <div>
          <Text strong style={{ fontSize: 15 }}>{text}</Text>
          <div style={{ fontSize: 12, color: "rgba(255,255,255,0.45)" }}>{record.description}</div>
        </div>
      ),
    },
    {
      title: "Danh mục",
      dataIndex: "category",
      key: "category",
      width: 120,
      render: (cat: string) => <Tag color="blue">{cat}</Tag>,
    },
    {
      title: "Mức độ rủi ro",
      dataIndex: "risk",
      key: "risk",
      width: 130,
      render: (risk: string) => {
        const color = risk === "Safe" ? "green" : risk === "Conditional" ? "orange" : "red";
        return <Tag color={color}>{risk}</Tag>;
      },
    },
    {
      title: "Trạng thái / Thao tác",
      key: "action",
      width: 150,
      render: (_: unknown, record: TweakItem) => (
        <Switch
          checked={record.enabled}
          loading={updatingId === record.id}
          onChange={() => handleToggle(record.id, record.enabled)}
          checkedChildren="Bật"
          unCheckedChildren="Tắt"
        />
      ),
    },
  ];

  return (
    <div style={{ padding: 24, maxWidth: 1100, margin: "0 auto" }}>
      <div style={{ marginBottom: 24, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <Title level={2} style={{ margin: 0, display: "flex", alignItems: "center", gap: 12 }}>
            <SettingOutlined style={{ color: "#52c41a" }} /> Tinh chỉnh & Công cụ
          </Title>
          <Text type="secondary">
            Tối ưu hiệu năng, quyền riêng tư và kiểm tra các bản cập nhật phần mềm.
          </Text>
        </div>
        <Button
          type="primary"
          icon={<DownloadOutlined />}
          loading={checkingUpdate}
          onClick={checkUpdate}
        >
          Kiểm tra cập nhật
        </Button>
      </div>

      {/* Update Status Card */}
      {updateInfo && (
        <Card
          size="small"
          style={{ marginBottom: 16, border: updateInfo.has_update ? "1px solid #faad14" : "1px solid #52c41a", background: "rgba(255,255,255,0.02)" }}
        >
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 16, flexWrap: "wrap" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
              {updateInfo.has_update ? (
                <Badge status="warning" />
              ) : (
                <Badge status="success" />
              )}
              <div>
                <div style={{ fontWeight: 600, fontSize: 15 }}>
                  {updateInfo.has_update ? `Có phiên bản mới: v${updateInfo.latest_version}` : `Đang dùng phiên bản mới nhất (v${updateInfo.current_version})`}
                </div>
                <div style={{ fontSize: 12, color: "#94a3b8" }}>
                  Hiện tại: v{updateInfo.current_version} · Mới nhất: v{updateInfo.latest_version}
                </div>
              </div>
            </div>
            {updateInfo.has_update && (
              <Button
                type="primary"
                size="small"
                onClick={() => {
                  Modal.confirm({
                    title: "Tải bản cập nhật mới?",
                    content: `Bạn sẽ được đưa đến trang tải xuống phiên bản v${updateInfo.latest_version}.`,
                    okText: "Tải ngay",
                    cancelText: "Để sau",
                    onOk: () => { window.open(updateInfo.release_url, "_blank"); },
                  });
                }}
              >
                Tải xuống v{updateInfo.latest_version}
              </Button>
            )}
          </div>
        </Card>
      )}

      {/* Registry Tweaks Table */}
      <Card bordered={false} style={{ background: "rgba(255,255,255,0.02)" }}>
        {loading ? (
          <div style={{ textAlign: "center", padding: 40 }}>
            <Spin size="large" tip="Đang đọc trạng thái hệ thống..." />
          </div>
        ) : (
          <Table
            dataSource={tweaks}
            columns={columns}
            rowKey="id"
            pagination={false}
          />
        )}
      </Card>
    </div>
  );
}
