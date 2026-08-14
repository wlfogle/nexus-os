import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionStatus, PowerInfo, SensorSnapshot } from "./types";

// Minimal, genuinely working shell: connection status + a few group-color
// buttons + a periodic stats poll. frontend-agent expands this into the
// full 4-tab layout (RGB Control, System, Effects, Key Bindings) plus the
// monitoring sidebar per CONTRACT.md - this file establishes the pattern
// (typed `invoke` calls, polling cadence) rather than being the final UI.

const PRESET_COLORS: Array<{ name: string; r: number; g: number; b: number }> = [
  { name: "Red", r: 255, g: 0, b: 0 },
  { name: "Green", r: 0, g: 255, b: 0 },
  { name: "Blue", r: 0, g: 0, b: 255 },
  { name: "Orange", r: 255, g: 102, b: 0 },
];

function App() {
  const [connection, setConnection] = useState<ConnectionStatus | null>(null);
  const [sensors, setSensors] = useState<SensorSnapshot | null>(null);
  const [power, setPower] = useState<PowerInfo | null>(null);
  const [status, setStatus] = useState<string>("");

  useEffect(() => {
    let cancelled = false;

    async function poll() {
      try {
        const [conn, sensorData, powerData] = await Promise.all([
          invoke<ConnectionStatus>("get_connection_status"),
          invoke<SensorSnapshot>("get_sensor_snapshot"),
          invoke<PowerInfo>("get_power_info"),
        ]);
        if (!cancelled) {
          setConnection(conn);
          setSensors(sensorData);
          setPower(powerData);
        }
      } catch (err) {
        if (!cancelled) setStatus(`Error polling backend: ${String(err)}`);
      }
    }

    poll();
    const interval = setInterval(poll, 2000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  async function applyGroupColor(group: string, r: number, g: number, b: number) {
    try {
      await invoke("set_group_color", { group, r, g, b });
      setStatus(`Applied color to ${group}`);
    } catch (err) {
      setStatus(`Failed to apply color: ${String(err)}`);
    }
  }

  async function clearAll() {
    try {
      await invoke("clear_all_keys");
      setStatus("All keys cleared");
    } catch (err) {
      setStatus(`Failed to clear keys: ${String(err)}`);
    }
  }

  return (
    <main style={{ fontFamily: "sans-serif", padding: "1.5rem", color: "#eee", background: "#1e1e1e", minHeight: "100vh" }}>
      <h1>OriginPC Control Center</h1>
      <p>
        RGB device:{" "}
        {connection === null
          ? "checking..."
          : connection.connected
            ? `connected (${connection.device_path ?? "unknown path"})`
            : "not connected"}
      </p>

      <section>
        <h2>Quick Colors (all keys)</h2>
        <div style={{ display: "flex", gap: "0.5rem" }}>
          {PRESET_COLORS.map((c) => (
            <button key={c.name} onClick={() => applyGroupColor("all_keys", c.r, c.g, c.b)}>
              {c.name}
            </button>
          ))}
          <button onClick={clearAll}>Clear All</button>
        </div>
      </section>

      <section>
        <h2>System</h2>
        <pre>{JSON.stringify({ sensors, power }, null, 2)}</pre>
      </section>

      {status && <p style={{ color: "#4CAF50" }}>{status}</p>}
    </main>
  );
}

export default App;
