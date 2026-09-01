import { Menu } from "antd";
import {
  DashboardOutlined,
  ClearOutlined,
  SafetyOutlined,
  ThunderboltOutlined,
  CloudServerOutlined,
  SettingOutlined,
} from "@ant-design/icons";
import { useNavigate, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";

export default function Sidebar() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();

  const selected = "/" + (location.pathname.split("/")[1] || "");

  const items = [
    { key: "/", icon: <DashboardOutlined />, label: t("nav.dashboard") },
    { key: "/disk", icon: <ClearOutlined />, label: t("nav.disk") },
    { key: "/debloat", icon: <SafetyOutlined />, label: t("nav.debloat") },
    { key: "/memory", icon: <ThunderboltOutlined />, label: t("nav.memory") },
    { key: "/services", icon: <CloudServerOutlined />, label: t("nav.services") },
    { key: "/settings", icon: <SettingOutlined />, label: t("nav.settings") },
  ];

  return (
    <Menu
      theme="dark"
      mode="inline"
      selectedKeys={[selected]}
      items={items}
      onClick={({ key }) => navigate(key)}
      style={{ height: "100vh", background: "transparent", borderInlineEnd: "none" }}
    />
  );
}
