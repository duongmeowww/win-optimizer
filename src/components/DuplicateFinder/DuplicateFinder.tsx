import { useState } from "react";
import { Card, Button, Input, Slider, Typography, message, Spin, Table, Tag } from "antd";
import { FileSearchOutlined, FolderOpenOutlined } from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import HelpButton from "../Common/HelpButton";

const { Title, Text } = Typography;

interface DuplicateGroup {
  hash: string;
  size: number;
  files: string[];
}

export default function DuplicateFinder() {
  const [targetPath, setTargetPath] = useState("C:\\Users");
  const [minSizeMb, setMinSizeMb] = useState(10);
  const [loading, setLoading] = useState(false);
  const [resultGroups, setResultGroups] = useState<DuplicateGroup[] | null>(null);

  const handleScan = async () => {
    if (!targetPath) {
      message.error("Vui lòng nhập đường dẫn thư mục cần quét!");
      return;
    }
    setLoading(true);
    setResultGroups(null);
    try {
      const res = await invoke<DuplicateGroup[]>("scan_duplicate_files", {
        dir: targetPath,
        minSizeMb: minSizeMb,
      });
      setResultGroups(res);
      message.success("Quét hoàn tất!");
    } catch (err) {
      message.error("Lỗi khi quét file trùng lặp: " + err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ padding: 24, maxWidth: 1000, margin: "0 auto" }}>
      <div style={{ marginBottom: 24 }}>
        <Title level={2} style={{ margin: 0, display: "flex", alignItems: "center", gap: 12 }}>
          <FileSearchOutlined style={{ color: "#1890ff" }} /> Tìm kiếm file trùng lặp <HelpButton title="Tìm file trùng lặp" content="So sánh hash SHA-256 nội dung file để phát hiện chính xác file trùng lặp, hỗ trợ xem trước và xóa an toàn."/>
        </Title>
        <Text type="secondary">
          Quét và phát hiện các file trùng nội dung chính xác dựa trên thuật toán mã hóa SHA-256 để giải phóng dung lượng ổ cứng.
        </Text>
      </div>

      <Card bordered={false} style={{ background: "rgba(255,255,255,0.02)", marginBottom: 24 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          <div>
            <Text strong style={{ display: "block", marginBottom: 8 }}>Đường dẫn thư mục quét:</Text>
            <Input
              value={targetPath}
              onChange={(e) => setTargetPath(e.target.value)}
              prefix={<FolderOpenOutlined />}
              placeholder="Ví dụ: C:\Users hay D:\"
              size="large"
            />
          </div>

          <div>
            <Text strong style={{ display: "block", marginBottom: 8 }}>Dung lượng file tối thiểu ({minSizeMb} MB):</Text>
            <Slider
              min={1}
              max={500}
              value={minSizeMb}
              onChange={(val) => setMinSizeMb(val)}
            />
          </div>

          <Button
            type="primary"
            icon={<FileSearchOutlined />}
            size="large"
            loading={loading}
            onClick={handleScan}
            style={{ width: "fit-content" }}
          >
            Bắt đầu quét file trùng lặp
          </Button>
        </div>
      </Card>

      {loading && (
        <Card bordered={false} style={{ textAlign: "center", padding: 40 }}>
          <Spin size="large" tip="Đang quét và tính mã băm SHA-256 của các file..." />
        </Card>
      )}

      {resultGroups && !loading && (
        <Card title={`Kết quả quét — ${resultGroups.length} nhóm trùng lặp`} bordered={false} style={{ background: "rgba(255,255,255,0.02)" }}>
          {resultGroups.length === 0 ? (
            <Text type="secondary">Không tìm thấy file trùng lặp nào.</Text>
          ) : (
            <Table
              dataSource={resultGroups.map((g, i) => ({ ...g, key: i }))}
              columns={[
                {
                  title: "Hash (SHA-256)",
                  dataIndex: "hash",
                  key: "hash",
                  render: (hash: string) => <Text code>{hash}</Text>,
                },
                {
                  title: "Kích thước",
                  dataIndex: "size",
                  key: "size",
                  width: 140,
                  render: (size: number) => {
                    if (size >= 1024 * 1024 * 1024) return `${(size / (1024 * 1024 * 1024)).toFixed(2)} GB`;
                    if (size >= 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(2)} MB`;
                    if (size >= 1024) return `${(size / 1024).toFixed(2)} KB`;
                    return `${size} B`;
                  },
                },
                {
                  title: "Số file",
                  dataIndex: "files",
                  key: "count",
                  width: 80,
                  render: (files: string[]) => <Tag color="blue">{files.length}</Tag>,
                },
                {
                  title: "Đường dẫn file",
                  dataIndex: "files",
                  key: "files",
                  render: (files: string[]) => (
                    <div>
                      {files.map((f, i) => (
                        <div key={i} style={{ fontSize: 12, fontFamily: "monospace" }}>{f}</div>
                      ))}
                    </div>
                  ),
                },
              ]}
              pagination={false}
              size="small"
            />
          )}
        </Card>
      )}
    </div>
  );
}