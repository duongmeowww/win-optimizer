import { Suspense } from "react";
import { ConfigProvider, theme, Spin } from "antd";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import Dashboard from "./components/Dashboard/Dashboard";
import DiskCleaner from "./components/DiskCleaner/DiskCleaner";
import Debloat from "./components/Debloat/Debloat";
import MemoryCleaner from "./components/MemoryCleaner/MemoryCleaner";
import ServicesStartup from "./components/ServicesStartup/ServicesStartup";
import Gaming from "./components/Gaming/Gaming";
import Diagnostics from "./components/Diagnostics/Diagnostics";
import Settings from "./components/Settings/Settings";

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
                <Route path="/settings" element={<Settings />} />
              </Routes>
            </Suspense>
          </main>
        </div>
      </Router>
    </ConfigProvider>
  );
}
