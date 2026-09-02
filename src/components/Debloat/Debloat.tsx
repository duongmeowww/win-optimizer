import { Card, Empty, List, Switch, Space, Button, Modal, Tag, Divider, Tooltip } from "antd";
import { ReloadOutlined, DeleteOutlined, UndoOutlined, InfoCircleOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface DebloatItem {
  id: string;
  label: string;
  desc: string;
  category: string;
  admin: boolean;
  enabled: boolean;
}

interface BatchResult {
  id: string;
  ok: boolean;
  message: string;
}

const CATEGORY_COLORS: Record<string, string> = {
  Bloatware: "red",
  Telemetry: "orange",
  Service: "blue",
  Privacy: "purple",
  System: "geekblue",
};

const CATEGORIES = ["Bloatware", "Telemetry", "Service", "Privacy", "System"] as const;

let cachedItems: DebloatItem[] | null = null;
let cachedAt = 0;
const CACHE_TTL = 30_000; // 30s cache to avoid refetch on tab switch

export default function Debloat() {
  const { t } = useTranslation();
  const [items, setItems] = useState<DebloatItem[]>(() => cachedItems ?? []);
  const [loading, setLoading] = useState(false);
  const [operating, setOperating] = useState<Record<string, boolean>>({});
  const [batchModal, setBatchModal] = useState<{ visible: boolean; items: {id: string; action: string}[] }>({ visible: false, items: [] });
  const [batchResults, setBatchResults] = useState<BatchResult[]>([]);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  const loadItems = useCallback(async (force = false) => {
    // Use cache if fresh and not forced
    if (!force && cachedItems && Date.now() - cachedAt < CACHE_TTL) {
      setItems(cachedItems);
      return;
    }
    setLoading(true);
    try {
      const data = await invoke<DebloatItem[]>("get_debloat_items");
      if (!mountedRef.current) return;
      cachedItems = data;
      cachedAt = Date.now();
      setItems(data);
    } catch (e) {
      console.error("Failed to load debloat items:", e);
    } finally {
      if (mountedRef.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    // Only fetch if no cached data
    if (cachedItems && Date.now() - cachedAt < CACHE_TTL) {
      setItems(cachedItems);
    } else {
      loadItems();
    }
  }, [loadItems]);

  const handleToggle = useCallback((item: DebloatItem, action: string) => {
    setOperating(prev => ({ ...prev, [item.id]: true }));

    invoke<[boolean, string]>("apply_debloat", { id: item.id, action })
      .then(([ok, msg]) => {
        if (!mountedRef.current) return;
        // Invalidate cache and reload
        cachedAt = 0;
        loadItems(true);
        setOperating(prev => {
          const next = { ...prev };
          delete next[item.id];
          return next;
        });
        if (!ok) {
          Modal.error({ title: item.label, content: msg });
        }
      })
      .catch((e: unknown) => {
        if (!mountedRef.current) return;
        setOperating(prev => {
          const next = { ...prev };
          delete next[item.id];
          return next;
        });
        Modal.error({ title: item.label, content: String(e) });
      });
  }, [loadItems]);

  const handleBatch = useCallback((itemsToProcess: { id: string; action: string }[]) => {
    setBatchModal({ visible: false, items: [] });
    setBatchResults([]);
    setOperating({});

    invoke<[string, boolean, string][]>("apply_debloat_batch", { items: itemsToProcess })
      .then((results) => {
        if (!mountedRef.current) return;
        const mapped = results.map(([id, ok, message]) => ({ id, ok, message }));
        setBatchResults(mapped);
        cachedAt = 0;
        loadItems(true);
      })
      .catch((e: unknown) => {
        if (!mountedRef.current) return;
        setBatchResults([]);
        Modal.error({ title: "Batch lỗi", content: String(e) });
      });
  }, [loadItems]);

  const itemsByCategory = useMemo(() => {
    const map = new Map<string, DebloatItem[]>();
    for (const cat of CATEGORIES) map.set(cat, []);
    for (const item of items) {
      const arr = map.get(item.category);
      if (arr) arr.push(item);
      else map.set(item.category, [item]);
    }
    return map;
  }, [items]);

  const renderCategory = useCallback((cat: string) => {
    const catItems = itemsByCategory.get(cat) ?? [];
    if (catItems.length === 0) return null;

    return (
      <Card
        key={cat}
        title={
          <Space>
            <Tag color={CATEGORY_COLORS[cat] || "default"}>{cat}</Tag>
            <span>{cat}</span>
          </Space>
        }
        size="small"
        style={{ marginBottom: 16 }}
      >
        <List
          loading={false}
          dataSource={catItems}
          size="small"
          renderItem={(item) => {
            const isOp = !!operating[item.id];
            return (
              <List.Item
                key={item.id}
                actions={[
                  <Tooltip
                    key="admin-tip"
                    title={item.admin ? "Cần quyền Admin để thực thi" : "Không cần quyền Admin"}
                  >
                    <Tag
                      key="admin"
                      color={item.admin ? "warning" : "success"}
                      style={{ margin: 0, cursor: "help", fontSize: 11, padding: "0 6px" }}
                    >
                      {item.admin ? "Admin" : "User"}
                    </Tag>
                  </Tooltip>,
                  <Tooltip
                    key="cat-tip"
                    title={`Category: ${item.category}`}
                  >
                    <Tag
                      key="cat"
                      color={CATEGORY_COLORS[item.category] || "default"}
                      style={{ margin: 0, fontSize: 11, padding: "0 6px" }}
                    >
                      {item.category}
                    </Tag>
                  </Tooltip>,
                  <Switch
                    key="switch"
                    size="small"
                    loading={isOp}
                    checkedChildren="Bật"
                    unCheckedChildren="Tắt"
                    checked={item.enabled}
                    disabled={isOp}
                    onChange={(checked) => {
                      const action = checked ? "revert" : "apply";
                      const itemLabel = item.label;
                      Modal.confirm({
                        title: `${checked ? "Bật lại" : "Vô hiệu hóa"}: ${itemLabel}`,
                        content: (
                          <Space direction="vertical" size="small">
                            <small>{item.desc}</small>
                            {item.admin && (
                              <Tag color="warning" style={{ margin: 0 }}>
                                Yêu cầu quyền Admin — app sẽ tự động yêu cầu thẩm quyền nếu chưa có.
                              </Tag>
                            )}
                          </Space>
                        ),
                        okText: checked ? "Bật lại" : "Tắt ngay",
                        okType: checked ? "default" : "danger",
                        onOk: () => handleToggle(item, action),
                      });
                    }}
                  />,
                ]}
              >
                <List.Item.Meta
                  title={
                    <Space>
                      <span>{item.label}</span>
                      <Tooltip title={item.desc}>
                        <InfoCircleOutlined
                          style={{ color: "rgba(255,255,255,0.45)", fontSize: 12 }}
                        />
                      </Tooltip>
                    </Space>
                  }
                  description={
                    <Space size={44}>
                      <Tag
                        color={item.admin ? "warning" : "success"}
                        style={{ fontSize: 11, padding: "0 6px" }}
                      >
                        {item.admin ? "Admin" : "User"}
                      </Tag>
                      <Tag
                        color={CATEGORY_COLORS[item.category] || "default"}
                        style={{ fontSize: 11, padding: "0 6px" }}
                      >
                        {item.category}
                      </Tag>
                    </Space>
                  }
                />
              </List.Item>
            );
          }}
        />
      </Card>
    );
  }, [itemsByCategory, operating, handleToggle]);

  return (
    <Card
      title={t("nav.debloat", { defaultValue: "Debloat" })}
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
            icon={<DeleteOutlined />}
            onClick={() => {
              const itemsToProcess = items
                .filter((i) => i.enabled)
                .map((i) => ({
                  id: i.id,
                  action: "apply",
                }));
              if (itemsToProcess.length === 0) {
                Modal.warning({
                  title: "Không có gì để xóa",
                  content: "Tất cả hạng mục đã được tắt.",
                });
                return;
              }
              Modal.confirm({
                title: "Xác nhận xóa bớt?",
                content: `Bạn chắc muốn tắt ${itemsToProcess.length} hạng mục? Hành động này không thể hoàn tác một số mục (AppX, Telemetry wipe).`,
                okText: "Xóa ngay",
                okType: "danger",
                onOk: () => handleBatch(itemsToProcess),
              });
            }}
          >
            Xóa tất cả
          </Button>
          <Button
            type="primary"
            icon={<UndoOutlined />}
            onClick={() => {
              const itemsToProcess = items
                .filter((i) => !i.enabled)
                .map((i) => ({
                  id: i.id,
                  action: "revert",
                }));
              if (itemsToProcess.length === 0) {
                Modal.info({
                  title: "Không có gì để hoàn tác",
                  content: "Tất cả hạng mục đã được bật.",
                });
                return;
              }
              Modal.confirm({
                title: "Bật lại tất cả?",
                content: `Bạn muốn bật lại ${itemsToProcess.length} hạng mục đã tắt?`,
                okText: "Bật lại",
                onOk: () => handleBatch(itemsToProcess),
              });
            }}
          >
            Bật lại tất cả
          </Button>
        </Space>
      }
    >
      {items.length === 0 && !loading ? (
        <Empty description="Chưa có dữ liệu. Nhấn Làm mới." />
      ) : (
        CATEGORIES.map((cat) => renderCategory(cat))
      )}

      <Divider>
        {loading
          ? "Đang xử lý..."
          : `Hoàn tất ${batchResults.filter((r) => r.ok).length}/${batchResults.length}`}
      </Divider>

      {batchResults.length > 0 && (
        <List
          size="small"
          dataSource={batchResults}
          renderItem={(r) => (
            <List.Item>
              <Space direction="vertical" size="small" style={{ width: "100%" }}>
                <div>
                  <Tag color={r.ok ? "green" : "red"}>
                    {r.ok ? "OK" : "Lỗi"}
                  </Tag>
                  <strong>{r.id}</strong>
                </div>
                <small style={{ color: "#888" }}>{r.message}</small>
              </Space>
            </List.Item>
          )}
        />
      )}

      <Modal
        title="Xác nhận"
        open={batchModal.visible}
        onCancel={() => setBatchModal({ visible: false, items: [] })}
        onOk={() => handleBatch(batchModal.items)}
        okText="Xác nhận"
      >
        <p>Bạn có muốn tiếp tục xử lý {batchModal.items.length} hạng mục?</p>
      </Modal>
    </Card>
  );
}
