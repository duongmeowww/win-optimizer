import { useState } from "react";
import { Card, Button, Input, Slider, Typography, message, Spin, Table, Tag } from "antd";
import { FileSearchOutlined, DeleteOutlined, FolderOpenOutlined } from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import HelpButton from "../Common/HelpButton";

const { Title, Text } = Typography;

export default function DuplicateFinder() {
  const [targetPath, setTargetPath] = useState("C:\\Users");
  const [minSizeMb, setMinSizeMb] = useState(10);
  const [loading, setLoading] = useState(false);
  const [resultText, setResultText] = useState<string | null>(null);

  const handleScan = async () => {
    if (!targetPath) {
      message.error("Vui lòng nhập đường dẫn thư mục cần quét!");
      return;
    }
    setLoading(true);
    setResultText(null);
    try {
      const res = await invoke<string>("find_duplicate_files", {
        paths: [targetPath],
        minSizeKb: minSizeMb * 1024,
      });
      setResultText(res);
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

      {resultText && !loading && (
        <Card title="Kết quả quét" bordered={false} style={{ background: "rgba(255,255,255,0.02)" }}>
          <pre style={{ whiteSpace: "pre-wrap", wordBreak: "break-all", fontFamily: "monospace", fontSize: 13, background: "rgba(0,0,0,0.2)", padding: 16, borderRadius: 8 }}>
            {resultText}
          </pre>
        </Card>
      )}
    </div>
  );
}