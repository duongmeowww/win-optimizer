import { QuestionCircleOutlined } from "@ant-design/icons";
import { Tooltip, Modal } from "antd";

interface HelpButtonProps {
  title: string;
  content: string;
}

export default function HelpButton({ title, content }: HelpButtonProps) {
  const showHelp = () => {
    Modal.info({
      title: `Hướng dẫn sử dụng: ${title}`,
      content: (
        <div style={{ marginTop: 12, lineHeight: 1.6, color: "rgba(255,255,255,0.85)" }}>
          {content}
        </div>
      ),
      okText: "Đã hiểu",
      width: 500,
    });
  };

  return (
    <Tooltip title={`Hướng dẫn nhanh về ${title}`}>
      <QuestionCircleOutlined
        onClick={showHelp}
        style={{
          cursor: "pointer",
          marginLeft: 8,
          color: "#1890ff",
          fontSize: 18,
          verticalAlign: "middle",
        }}
      />
    </Tooltip>
  );
}
