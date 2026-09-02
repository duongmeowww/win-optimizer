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
                <Route path="/settings" element={<AdvancedTools />} />
              </Routes>
            </Suspense>
          </main>
        </div>
      </Router>
    </ConfigProvider>
  );
}
