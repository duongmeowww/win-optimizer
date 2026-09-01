import { Suspense } from "react";
import { ConfigProvider, Layout, theme, Select, Spin } from "antd";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Sidebar from "./components/Sidebar";
import Dashboard from "./components/Dashboard/Dashboard";
import DiskCleaner from "./components/DiskCleaner/DiskCleaner";
import Debloat from "./components/Debloat/Debloat";
import MemoryCleaner from "./components/MemoryCleaner/MemoryCleaner";
import ServicesStartup from "./components/ServicesStartup/ServicesStartup";
import Settings from "./components/Settings/Settings";

const { Sider, Content, Header } = Layout;

const Loading = () => (
  <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "60vh" }}>
    <Spin size="large" />
  </div>
);

export default function App() {
  const { i18n } = useTranslation();
  return (
    <ConfigProvider theme={{ algorithm: theme.darkAlgorithm }}>
      <Router>
        <Layout style={{ minHeight: "100vh" }}>
          <Sider theme="light" width={220} style={{ background: "transparent" }}>
            <Sidebar />
          </Sider>
          <Layout>
            <Header
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                paddingInline: 24,
              }}
            >
              <span style={{ fontWeight: 600, fontSize: 16 }}>WinOptimizer</span>
              <Select
                value={i18n.language}
                style={{ width: 120 }}
                onChange={(v) => i18n.changeLanguage(v)}
                options={[
                  { value: "en", label: "English" },
                  { value: "vi", label: "Tiếng Việt" },
                ]}
              />
            </Header>
            <Content style={{ margin: 24 }}>
              <Suspense fallback={<Loading />}>
                <Routes>
                  <Route path="/" element={<Dashboard />} />
                  <Route path="/dashboard" element={<Dashboard />} />
                  <Route path="/disk" element={<DiskCleaner />} />
                  <Route path="/debloat" element={<Debloat />} />
                  <Route path="/memory" element={<MemoryCleaner />} />
                  <Route path="/services" element={<ServicesStartup />} />
                  <Route path="/settings" element={<Settings />} />
                </Routes>
              </Suspense>
            </Content>
          </Layout>
        </Layout>
      </Router>
    </ConfigProvider>
  );
}
