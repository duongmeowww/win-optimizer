import { useState, useEffect } from "react";
import { Card, Button, Typography, Table, Tag, message, Spin } from "antd";
import { RocketOutlined, CheckCircleOutlined } from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import HelpButton from "../Common/HelpButton";

const { Title, Text } = Typography;

interface HubModule {
  id: string;
  name: string;
  category: string;
  description: string;
  status: string;
}

export default function AdvancedHub() {
  const [modules, setModules] = useState<HubModule[]>([]);
  const [loading, setLoading] = useState(false);
  const [executingId, setExecutingId] = useState<string | null>(null);

  const fetchModules = async () => {
    setLoading(true);
    try {
      const res = await invoke<HubModule[]>("get_advanced_hub_modules");
      setModules(res);
    } catch (err) {
      message.error("Lỗi khi tải danh sách module: " + err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchModules();
  }, []);

  const handleExecute = async (id: string) => {
    setExecutingId(id);
    try {
      const [success, msg] = await invoke<[boolean, string]>("execute_advanced_hub_action", { id });
      if (success) {
        message.success(msg);
        fetchModules();
      } else {
        message.error(msg);
      }
    } catch (err) {
      message.error("Lỗi khi thực thi: " + err);
    } finally {
      setExecutingId(null);
    }
  };

  const columns = [
    {
      title: "Tính năng nâng cao",
      dataIndex: "name",
      key: "name",
      render: (text: string, record: HubModule) => (
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
      render: (cat: string) => <Tag color="purple">{cat}</Tag>,
    },
    {
      title: "Trạng thái",
      dataIndex: "status",
      key: "status",
      width: 180,
      render: (status: string) => <Tag icon={<CheckCircleOutlined />} color="success">{status}</Tag>,
    },
    {
      title: "Thao tác",
      key: "action",
      width: 160,
      render: (_: any, record: HubModule) => (
        <Button
          type="primary"
          ghost
          loading={executingId === record.id}
          onClick={() => handleExecute(record.id)}
        >
          Áp dụng / Chạy
        </Button>
      ),
    },
  ];

  return (
    <div style={{ padding: 24, maxWidth: 1100, margin: "0 auto" }}>
      <div style={{ marginBottom: 24 }}>
        <Title level={2} style={{ margin: 0, display: "flex", alignItems: "center", gap: 12 }}>
          <RocketOutlined style={{ color: "#fa8c16" }} /> Advanced Optimization Suite (Toàn bộ 8 tính năng) <HelpButton title="Advanced Suite" content="Tập hợp toàn bộ 8 tính năng nâng cao: Tối ưu GPU, bảo mật telemetry, quản lý Windows Update và Profile tự động."/>
        </Title>
        <Text type="secondary">
          Bao gồm GPU Scheduling, Telemetry Hardening, Smart Startup, Windows Update Controller, Benchmark và Profiles.
        </Text>
      </div>

      <Card bordered={false} style={{ background: "rgba(255,255,255,0.02)" }}>
        {loading ? (
          <div style={{ textAlign: "center", padding: 40 }}>
            <Spin size="large" tip="Đang tải các module nâng cao..." />
          </div>
        ) : (
          <Table
            dataSource={modules}
            columns={columns}
            rowKey="id"
            pagination={false}
          />
        )}
      </Card>
    </div>
  );
}