import { Suspense, lazy } from "react";
import { ConfigProvider, theme, Spin } from "antd";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Sidebar from "./components/Sidebar";

const Dashboard = lazy(() => import("./components/Dashboard/Dashboard"));
const DiskCleaner = lazy(() => import("./components/DiskCleaner/DiskCleaner"));
const Debloat = lazy(() => import("./components/Debloat/Debloat"));
const MemoryCleaner = lazy(() => import("./components/MemoryCleaner/MemoryCleaner"));
const ServicesStartup = lazy(() => import("./components/ServicesStartup/ServicesStartup"));
const Gaming = lazy(() => import("./components/Gaming/Gaming"));
const Diagnostics = lazy(() => import("./components/Diagnostics/Diagnostics"));
const AdvancedTools = lazy(() => import("./components/Settings/AdvancedTools"));
const DuplicateFinder = lazy(() => import("./components/DuplicateFinder/DuplicateFinder"));
const RegistryTweaks = lazy(() => import("./components/RegistryTweaks/RegistryTweaks"));
const AdvancedHub = lazy(() => import("./components/AdvancedHub/AdvancedHub"));

function Loading() {
  return (
    <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "60vh" }}>
      <Spin size="large" />
    </div>
  );
}

export default function App() {
  return (
    <ConfigProvider theme={{ algorithm: theme.darkAlgorithm }}>
      <Router>
        <div className="app-shell">
          <Sidebar />
          <main className="app-main">
            <Suspense fallback={<Loading />}>
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/disk" element={<DiskCleaner />} />
                <Route path="/debloat" element={<Debloat />} />
                <Route path="/memory" element={<MemoryCleaner />} />
                <Route path="/services" element={<ServicesStartup />} />
                <Route path="/gaming" element={<Gaming />} />
                <Route path="/diagnostics" element={<Diagnostics />} />
                {/* Gộp: /settings, /registry, /hub cùng trỏ về trang Tinh chỉnh & Công cụ nâng cao */}
                <Route path="/settings" element={<AdvancedTools />} />
                <Route path="/registry" element={<RegistryTweaks />} />
                <Route path="/hub" element={<AdvancedHub />} />
                {/* Gộp: /duplicates nằm trong tab Dọn dẹp hệ thống */}
                <Route path="/duplicates" element={<DuplicateFinder />} />
              </Routes>
            </Suspense>
          </main>
        </div>
      </Router>
    </ConfigProvider>
  );
}
