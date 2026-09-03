import { useNavigate, useLocation } from "react-router-dom";
import {
  ThunderboltOutlined,
  ClearOutlined,
  SafetyOutlined,
  ExperimentOutlined,
  SettingOutlined,
  PlayCircleOutlined,
  MedicineBoxOutlined,
  ControlOutlined,
} from "@ant-design/icons";

const NAV_ITEMS = [
  { key: "/", icon: <ThunderboltOutlined />, label: "Tối ưu hóa" },
  { key: "/disk", icon: <ClearOutlined />, label: "Dọn dẹp hệ thống" },
  { key: "/debloat", icon: <SafetyOutlined />, label: "Debloat & Ứng dụng" },
  { key: "/memory", icon: <ExperimentOutlined />, label: "Dọn RAM" },
  { key: "/services", icon: <SettingOutlined />, label: "Dịch vụ & Khởi động" },
  { key: "/gaming", icon: <PlayCircleOutlined />, label: "Tăng tốc Gaming" },
  { key: "/diagnostics", icon: <MedicineBoxOutlined />, label: "Chẩn đoán & Sửa lỗi" },
  { key: "/registry", icon: <ControlOutlined />, label: "Tinh chỉnh & Công cụ" },
];

export default function Sidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const selected = "/" + (location.pathname.split("/")[1] || "");

  return (
    <aside className="sidebar">
      <div>
        <div className="sidebar-logo">
          <div className="sidebar-logo-icon">
            <ThunderboltOutlined />
          </div>
          <span className="sidebar-logo-text">WinOptimizer</span>
        </div>
        <nav className="sidebar-nav">
          {NAV_ITEMS.map((item) => {
            const active = selected === item.key;
            return (
              <a
                key={item.key}
                href="#"
                className={`sidebar-item ${active ? "sidebar-item-active" : ""}`}
                onClick={(e) => { e.preventDefault(); navigate(item.key); }}
              >
                <div className={`sidebar-item-icon ${active ? "sidebar-item-icon-active" : ""}`}>
                  {item.icon}
                </div>
                <span>{item.label}</span>
              </a>
            );
          })}
        </nav>
      </div>
    </aside>
  );
}
