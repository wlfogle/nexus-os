import { GaugeRing } from "./GaugeRing";
import { tempColor } from "../lib/tempColor";
import { formatTemp, formatUptime } from "../lib/format";
import type {
  ConnectionStatus,
  PowerInfo,
  SensorSnapshot,
  SystemUsage,
  TemperatureReading,
} from "../types";

interface SidebarProps {
  connection: ConnectionStatus | null;
  sensors: SensorSnapshot | null;
  power: PowerInfo | null;
  usage: SystemUsage | null;
  live: boolean;
}

function maxTemp(readings: TemperatureReading[] | undefined): number | undefined {
  if (!readings || readings.length === 0) return undefined;
  return Math.max(...readings.map((r) => r.celsius));
}

// Layout below matches the old app's "System Monitor" panel: CPU/Memory
// usage gauges up top, then color-coded temperature readouts, then device
// status, then the free-text "System Information" block, then power - see
// the screenshots referenced when this was ported.
export function Sidebar({ connection, sensors, power, usage, live }: SidebarProps) {
  const connected = connection?.connected ?? false;
  const cpuTemp = maxTemp(sensors?.cpu);
  const gpuTemp = maxTemp(sensors?.gpu);

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <span className="sidebar-brand-icon">⌨</span>
        <span>System Monitor</span>
      </div>

      <section className="sidebar-section">
        <div className="gauge-row">
          <GaugeRing label="CPU Usage" percent={usage?.cpu_percent ?? 0} color="var(--accent)" />
          <GaugeRing label="Memory Usage" percent={usage?.memory_percent ?? 0} color="var(--warn)" />
        </div>
      </section>

      <section className="sidebar-section">
        <h2>
          Temperature Monitoring
          {live && <span className="live-badge">LIVE</span>}
        </h2>
        <dl className="stat-list">
          <div className="stat-row">
            <dt>CPU</dt>
            <dd style={{ color: tempColor(cpuTemp) }}>
              {cpuTemp !== undefined ? formatTemp(cpuTemp) : "--"}
            </dd>
          </div>
          <div className="stat-row">
            <dt>GPU</dt>
            <dd style={{ color: tempColor(gpuTemp) }}>
              {gpuTemp !== undefined ? formatTemp(gpuTemp) : "--"}
            </dd>
          </div>
        </dl>
      </section>

      <section className="sidebar-section">
        <h2>Device</h2>
        <div className={`status-pill ${connected ? "status-ok" : "status-bad"}`}>
          <span className="status-dot" />
          {connection === null ? "Checking…" : connected ? "Connected" : "Disconnected"}
        </div>
        {connection?.device_path && <p className="sidebar-detail">{connection.device_path}</p>}
      </section>

      <section className="sidebar-section">
        <h2>System Information</h2>
        <pre className="system-info-block">
          {usage
            ? `Memory: ${usage.memory_percent.toFixed(1)}% (${usage.memory_used_gb.toFixed(1)}GB / ${usage.memory_total_gb.toFixed(1)}GB)\n` +
              `Disk: ${usage.disk_percent.toFixed(1)}% (${usage.disk_used_gb.toFixed(1)}GB / ${usage.disk_total_gb.toFixed(1)}GB)\n` +
              `Load Average: ${usage.load_avg.map((v) => v.toFixed(2)).join(", ")}\n` +
              `Uptime: ${formatUptime(usage.uptime_secs)}`
            : "Gathering system information…"}
        </pre>
      </section>

      <section className="sidebar-section">
        <h2>Power</h2>
        <dl className="stat-list">
          <div className="stat-row">
            <dt>Battery</dt>
            <dd>{power ? `${power.battery_percent.toFixed(0)}%` : "--"}</dd>
          </div>
          <div className="stat-row">
            <dt>AC</dt>
            <dd>{power ? (power.ac_connected ? "Connected" : "On battery") : "--"}</dd>
          </div>
          <div className="stat-row">
            <dt>TLP</dt>
            <dd>{power ? (power.tlp_active ? "Active" : "Inactive") : "--"}</dd>
          </div>
        </dl>
      </section>
    </aside>
  );
}
