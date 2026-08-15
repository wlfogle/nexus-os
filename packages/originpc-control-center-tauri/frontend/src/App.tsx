import { useState } from "react";
import "./App.css";
import { Sidebar } from "./components/Sidebar";
import { RgbControlTab } from "./components/tabs/RgbControlTab";
import { SystemTab } from "./components/tabs/SystemTab";
import { EffectsTab } from "./components/tabs/EffectsTab";
import { KeyBindingsTab } from "./components/tabs/KeyBindingsTab";
import { useConnectionStatus } from "./hooks/useConnectionStatus";
import { useSystemStats } from "./hooks/useSystemStats";

// Full 4-tab layout (RGB Control, System, Effects, Key Bindings) plus the
// monitoring sidebar, per CONTRACT.md's "frontend-agent: UI to add" section.
// The typed `invoke` pattern established by the original minimal shell now
// lives in `lib/api.ts`; every command call in the tabs below goes through
// that layer instead of calling `invoke` directly.

type TabId = "rgb" | "system" | "effects" | "bindings";

const TABS: Array<{ id: TabId; label: string }> = [
  { id: "rgb", label: "RGB Control" },
  { id: "system", label: "System" },
  { id: "effects", label: "Effects" },
  { id: "bindings", label: "Key Bindings" },
];

function App() {
  const [activeTab, setActiveTab] = useState<TabId>("rgb");
  const connection = useConnectionStatus();
  const { sensors, power, live } = useSystemStats();

  return (
    <div className="app-shell">
      <Sidebar connection={connection} sensors={sensors} power={power} live={live} />
      <main className="main-area">
        <header className="app-header">
          <h1>OriginPC Control Center</h1>
        </header>
        <nav className="tab-nav">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className={`tab-button ${activeTab === tab.id ? "tab-button-active" : ""}`}
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </nav>
        <div className="tab-content">
          {activeTab === "rgb" && <RgbControlTab />}
          {activeTab === "system" && <SystemTab sensors={sensors} power={power} live={live} />}
          {activeTab === "effects" && <EffectsTab />}
          {activeTab === "bindings" && <KeyBindingsTab />}
        </div>
      </main>
    </div>
  );
}

export default App;
