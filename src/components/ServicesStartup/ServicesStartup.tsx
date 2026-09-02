import { Card, Empty, Table, Switch, Space, Tag, Typography, Button, Modal } from "antd";
import { ReloadOutlined, ThunderboltOutlined, ExclamationCircleOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ServiceStartupItem {
  id: string;
  name: string;
  displayName: string;
  status: string;
  startType: string;
  category: string;
  description: string;
}

const CATEGORY_COLORS: Record<string, string> = {
  System: "blue",
  Bloatware: "red",
  Telemetry: "orange",
  Gaming: "green",
  Security: "purple",
  Privacy: "volcano",
};

export default function ServicesStartup() {
  const { t } = useTranslation();
  const [items, setItems] = useState<ServiceStartupItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [operating, setOperating] = useState<Record<string, boolean>>({});

  useEffect(() => {
    loadItems();
  }, []);

  async function loadItems() {
    setLoading(true);
    try {
      const data = await invoke<ServiceStartupItem[]>("get_service_startup_items");
      setItems(data);
    } catch (e: any) {
      console.error("Service startup not available:", e);
      Modal.warning({
        title: "Dịch vụ chưa sẵn sàng",
        content: "Backend service không khả dụng. Hãy khởi động lại ứng dụng với quyền Admin.",
      });
    }
    setLoading(false);
  }

  function handleToggle(item: ServiceStartupItem, startType: string) {
    const newOp = { ...operating, [item.id]: true };
    setOperating(newOp);

    const isDisable = startType === "Disabled";
    Modal.confirm({
      title: `${isDisable ? "Vô hiệu hóa" : "Bật lại"}: ${item.displayName || item.name}`,
      content: (
        <Space direction="vertical" size="small">
          <small>{item.description}</small>
          <Tag color={isDisable ? "red" : "green"}>
            {isDisable ? "Tắt hoàn toàn" : "Tự động khởi động"}
          </Tag>
        </Space>
      ),
      okText: isDisable ? "Tắt ngay" : "Bật lại",
      okType: isDisable ? "danger" : "default",
      onOk: () => {
        invoke("set_service_startup", { name: item.name, startType })
          .then((_r: any) => {
            loadItems();
            setOperating({ ...operating });
          })
          .catch((e: any) => {
            setOperating({ ...operating });
            Modal.error({ title: item.displayName, content: String(e) });
          });
      },
    });
  }

  const columns = [
    {
      title: "Tên dịch vụ",
      dataIndex: "name",
      key: "name",
      render: (name: string, rec: ServiceStartupItem) => (
        <Space direction="vertical" size="small">
          <Typography.Text strong style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <ThunderboltOutlined style={{ fontSize: 12, color: "rgba(255,255,255,0.45)" }} />
            {rec.displayName || name}
          </Typography.Text>
          <Typography.Text type="secondary" style={{ fontSize: 11 }}>
            {name}
          </Typography.Text>
        </Space>
      ),
    },
    {
      title: "Trạng thái",
      dataIndex: "status",
      key: "status",
      width: 110,
      render: (st: string) => {
        let color = "default";
        let txt = st || "N/A";
        if (st === "Running") { color = "green"; txt = "Đang chạy"; }
        else if (st === "Stopped") { color = "red"; txt = "Dừng"; }
        else if (st === "StartPending") { color = "orange"; txt = "Đang khởi động"; }
        else if (st === "StopPending") { color = "orange"; txt = "Đang dừng"; }
        return <Tag color={color} style={{ margin: 0, padding: "2px 8px", fontSize: 11 }}>{txt}</Tag>;
      },
    },
    {
      title: "Khởi động",
      dataIndex: "startType",
      key: "startType",
      width: 180,
      render: (st: string, rec: ServiceStartupItem) => {
        let color: string = "default";
        if (st === "Auto") color = "green";
        else if (st === "Manual") color = "orange";
        else if (st === "Disabled") color = "red";

        return (
          <Space>
            <Tag color={color} style={{ margin: 0, fontSize: 11, padding: "2px 8px" }}>{st || "N/A"}</Tag>
            {!operating[rec.id] && st !== "Disabled" && (
              <Switch
                size="small"
                loading={operating[rec.id]}
                checkedChildren="Auto"
                unCheckedChildren="Off"
                checked={st === "Auto"}
                onChange={(checked) => handleToggle(rec, checked ? "Auto" : "Disabled")}
              />
            )}
            {!operating[rec.id] && st === "Disabled" && (
              <Button
                size="small"
                type="primary"
                onClick={() => handleToggle(rec, "Auto")}
              >
                Bật
              </Button>
            )}
          </Space>
        );
      },
    },
    {
      title: "Danh mục",
      dataIndex: "category",
      key: "category",
      width: 100,
      render: (cat: string) => {
        const color = CATEGORY_COLORS[cat] || "blue";
        return (
          <Tag
            color={color}
            style={{ margin: 0, fontSize: 11, padding: "2px 8px" }}
          >
            {cat}
          </Tag>
        );
      },
    },
    {
      title: "Mô tả",
      dataIndex: "description",
      key: "description",
      ellipsis: true,
    },
  ];

  

  return (
    <Card
      title={t("nav.services", { defaultValue: "Services & Startup" })}
      extra={
        <Space>
          <Button
            type="primary"
            icon={<ReloadOutlined />}
            onClick={() => loadItems()}
            loading={loading}
          >
            Làm mới
          </Button>
          <Button
            type="primary"
            danger
            icon={<ExclamationCircleOutlined />}
            onClick={async () => {
              await invoke("optimize_recommended_services");
              await loadItems();
            }}
          >
            Tối ưu đề xuất
          </Button>
        </Space>
      }
    >
      {items.length === 0 && !loading ? (
        <Empty description="Chạy ứng dụng với quyền Admin để xem danh sách dịch vụ." />
      ) : (
        <Table
          columns={columns}
          dataSource={items}
          loading={loading}
          rowKey="id"
          scroll={{ y: 500 }}
          pagination={{ pageSize: 30, size: "small" }}
          expandable={{
            expandedRowRender: (record: ServiceStartupItem) => (
              <Typography.Text>{record.description}</Typography.Text>
            ),
          }}
          rowClassName={(record) =>
            record.category === "Security" ? "highlight-row" : ""
          }
        />
      )}
    </Card>
  );
}
