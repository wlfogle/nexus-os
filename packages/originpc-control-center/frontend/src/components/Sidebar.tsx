import type { ConnectionStatus, PowerInfo, SensorSnapshot, TemperatureReading } from "../types";

interface SidebarProps {
  connection: ConnectionStatus | null;
  sensors: SensorSnapshot | null;
  power: PowerInfo | null;
  live: boolean;
}

function maxTemp(readings: TemperatureReading[] | undefined): string {
  if (!readings || readings.length === 0) return "--";
  const max = Math.max(...readings.map((r) => r.celsius));
  return `${max.toFixed(0)}°C`;
}

export function Sidebar({ connection, sensors, power, live }: SidebarProps) {
  const connected = connection?.connected ?? false;

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <span className="sidebar-brand-icon">⌨</span>
        <span>OriginPC</span>
      </div>

      <section className="sidebar-section">
        <h2>Device</h2>
        <div className={`status-pill ${connected ? "status-ok" : "status-bad"}`}>
          <span className="status-dot" />
          {connection === null ? "Checking…" : connected ? "Connected" : "Disconnected"}
        </div>
        {connection?.device_path && <p className="sidebar-detail">{connection.device_path}</p>}
      </section>

      <section className="sidebar-section">
        <h2>
          Monitoring
          {live && <span className="live-badge">LIVE</span>}
        </h2>
        <dl className="stat-list">
          <div className="stat-row">
            <dt>CPU</dt>
            <dd>{maxTemp(sensors?.cpu)}</dd>
          </div>
          <div className="stat-row">
            <dt>GPU</dt>
            <dd>{maxTemp(sensors?.gpu)}</dd>
          </div>
          <div className="stat-row">
            <dt>NVMe</dt>
            <dd>{maxTemp(sensors?.nvme)}</dd>
          </div>
          <div className="stat-row">
            <dt>Fans</dt>
            <dd>{sensors?.fans_rpm.length ? `${sensors.fans_rpm.length} active` : "--"}</dd>
          </div>
        </dl>
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
