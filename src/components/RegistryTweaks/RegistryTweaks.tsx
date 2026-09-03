import { useState, useEffect } from "react";
import { Card, Button, Typography, Switch, Table, Tag, message, Spin } from "antd";
import { SettingOutlined, SafetyCertificateOutlined } from "@ant-design/icons";
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

export default function RegistryTweaks() {
  const [tweaks, setTweaks] = useState<TweakItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [updatingId, setUpdatingId] = useState<string | null>(null);

  const fetchTweaks = async () => {
    setLoading(true);
    try {
      const res = await invoke<TweakItem[]>("get_registry_tweaks");
      setTweaks(res);
    } catch (err) {
      message.error("Lỗi khi tải danh sách Registry Tweaks: " + err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchTweaks();
  }, []);

  const handleToggle = async (id: string, currentEnabled: boolean) => {
    setUpdatingId(id);
    try {
      const [success, msg] = await invoke<[boolean, string]>("apply_registry_tweak", {
        id,
        enable: !currentEnabled,
      });
      if (success) {
        message.success(msg);
        fetchTweaks();
      } else {
        message.error(msg);
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
      render: (_: any, record: TweakItem) => (
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
      <div style={{ marginBottom: 24 }}>
        <Title level={2} style={{ margin: 0, display: "flex", alignItems: "center", gap: 12 }}>
          <SettingOutlined style={{ color: "#52c41a" }} /> Quản lý Registry & System Tweaks
        </Title>
        <Text type="secondary">
          Tối ưu hóa hiệu năng, quyền riêng tư (Privacy) và vô hiệu hóa các tính năng nền ngầm không cần thiết của Windows.
        </Text>
      </div>

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
