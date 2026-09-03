import { Card, Empty, Table, Switch, Space, Tag, Typography, Button, Modal } from "antd";
import { ReloadOutlined, ThunderboltOutlined, ExclamationCircleOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import HelpButton from "../Common/HelpButton";

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

let cachedServices: ServiceStartupItem[] | null = null;
let cachedAt = 0;
const CACHE_TTL = 30_000;

export default function ServicesStartup() {
  const { t } = useTranslation();
  const [items, setItems] = useState<ServiceStartupItem[]>(() => cachedServices ?? []);
  const [loading, setLoading] = useState(false);
  const [operating, setOperating] = useState<Record<string, boolean>>({});
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  const loadItems = useCallback(async (force = false) => {
    if (!force && cachedServices && Date.now() - cachedAt < CACHE_TTL) {
      setItems(cachedServices);
      return;
    }
    setLoading(true);
    try {
      const data = await invoke<ServiceStartupItem[]>("get_service_startup_items");
      if (!mountedRef.current) return;
      cachedServices = data;
      cachedAt = Date.now();
      setItems(data);
    } catch (e: unknown) {
      console.error("Service startup not available:", e);
      if (mountedRef.current) {
        Modal.warning({
          title: "Dịch vụ chưa sẵn sàng",
          content: "Backend service không khả dụng. Hãy khởi động lại ứng dụng với quyền Admin.",
        });
      }
    } finally {
      if (mountedRef.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (cachedServices && Date.now() - cachedAt < CACHE_TTL) {
      setItems(cachedServices);
    } else {
      loadItems();
    }
  }, [loadItems]);

  const handleToggle = useCallback((item: ServiceStartupItem, startType: string) => {
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
        setOperating(prev => ({ ...prev, [item.id]: true }));
        invoke<[boolean, string]>("set_service_startup", { name: item.name, startType })
          .then(() => {
            if (!mountedRef.current) return;
            cachedAt = 0;
            loadItems(true);
            setOperating(prev => {
              const n = { ...prev };
              delete n[item.id];
              return n;
            });
          })
          .catch((e: unknown) => {
            if (!mountedRef.current) return;
            setOperating(prev => {
              const n = { ...prev };
              delete n[item.id];
              return n;
            });
            Modal.error({ title: item.displayName, content: String(e) });
          });
      },
    });
  }, [loadItems]);

  const handleOptimize = useCallback(async () => {
    setLoading(true);
    try {
      await invoke<[boolean, string]>("optimize_recommended_services");
      cachedAt = 0;
      await loadItems(true);
      Modal.success({ title: "Đã tối ưu", content: "Đã tối ưu các dịch vụ đề xuất cho hiệu năng." });
    } catch (e: unknown) {
      Modal.error({ title: "Lỗi tối ưu", content: String(e) });
    } finally {
      if (mountedRef.current) setLoading(false);
    }
  }, [loadItems]);

  const columns = useMemo(() => [
    {
      title: "Tên dịch vụ",
      dataIndex: "name",
      key: "name",
      render: (name: string, rec: ServiceStartupItem) => (
        <Space direction="vertical" size={2}>
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
        const isOp = !!operating[rec.id];
        return (
          <Space>
            <Tag color={color} style={{ margin: 0, fontSize: 11, padding: "2px 8px" }}>{st || "N/A"}</Tag>
            {!isOp && st !== "Disabled" && (
              <Switch
                size="small"
                loading={isOp}
                checkedChildren="Auto"
                unCheckedChildren="Off"
                checked={st === "Auto"}
                onChange={(checked) => handleToggle(rec, checked ? "Auto" : "Disabled")}
              />
            )}
            {!isOp && st === "Disabled" && (
              <Button
                size="small"
                type="primary"
                onClick={() => handleToggle(rec, "Auto")}
              >
                Bật
              </Button>
            )}
            {isOp && <Tag color="processing">...</Tag>}
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
  ], [operating, handleToggle]);

  return (
    <Card
      title={<Space>{t("nav.services", { defaultValue: "Services & Startup" })} <HelpButton title="Services & Startup" content="Quản lý dịch vụ Windows và ứng dụng khởi động. Tắt các service không cần thiết giúp khởi động máy nhanh hơn."/></Space>}
      extra={
        <Space>
          <Button
            type="primary"
            icon={<ReloadOutlined />}
            onClick={() => loadItems(true)}
            loading={loading}
          >
            Làm mới
          </Button>
          <Button
            type="primary"
            danger
            icon={<ExclamationCircleOutlined />}
            onClick={handleOptimize}
            loading={loading}
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
          columns={columns as never}
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