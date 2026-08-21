import { useState } from "react";
import { getTlpStats, setFanMode, setPowerProfile } from "../../lib/api";
import { Modal } from "../Modal";
import { formatUptime } from "../../lib/format";
import type {
  FanModeName,
  PowerInfo,
  PowerProfileName,
  SensorSnapshot,
  SystemUsage,
  TemperatureReading,
} from "../../types";

interface SystemTabProps {
  sensors: SensorSnapshot | null;
  power: PowerInfo | null;
  usage: SystemUsage | null;
  live: boolean;
}

const POWER_PROFILES: Array<{ id: PowerProfileName; label: string }> = [
  { id: "performance", label: "Performance" },
  { id: "balanced", label: "Balanced" },
  { id: "power_save", label: "Power Save" },
];

const FAN_MODES: Array<{ id: FanModeName; label: string }> = [
  { id: "auto", label: "Auto" },
  { id: "silent", label: "Silent" },
];

function TempTable({
  title,
  readings,
  unit = "°C",
}: {
  title: string;
  readings: TemperatureReading[] | undefined;
  unit?: string;
}) {
  return (
    <div className="temp-table">
      <h4>{title}</h4>
      {!readings || readings.length === 0 ? (
        <p className="panel-hint">No readings available.</p>
      ) : (
        <ul>
          {readings.map((reading) => (
            <li key={reading.label}>
              <span>{reading.label}</span>
              <span>
                {reading.celsius.toFixed(1)}
                {unit}
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function SystemTab({ sensors, power, usage, live }: SystemTabProps) {
  const [profile, setProfile] = useState<PowerProfileName>("balanced");
  const [fanMode, setFanModeState] = useState<FanModeName>("auto");
  const [status, setStatus] = useState("");
  const [showFanDetails, setShowFanDetails] = useState(false);
  const [showTlpStats, setShowTlpStats] = useState(false);
  const [tlpStatsText, setTlpStatsText] = useState<string | null>(null);
  const [tlpStatsError, setTlpStatsError] = useState<string | null>(null);

  async function openTlpStats() {
    setShowTlpStats(true);
    setTlpStatsText(null);
    setTlpStatsError(null);
    try {
      setTlpStatsText(await getTlpStats());
    } catch (err) {
      setTlpStatsError(String(err));
    }
  }

  async function applyProfile(next: PowerProfileName) {
    setProfile(next);
    try {
      await setPowerProfile(next);
      setStatus(`Power profile set to "${next}"`);
    } catch (err) {
      setStatus(`Failed to set power profile: ${String(err)}`);
    }
  }

  async function applyFanMode(next: FanModeName) {
    setFanModeState(next);
    try {
      await setFanMode(next);
      setStatus(`Fan mode set to "${next}"`);
    } catch (err) {
      setStatus(`Failed to set fan mode: ${String(err)}`);
    }
  }

  return (
    <div className="tab-panel">
      <section className="panel-card">
        <h3>
          Sensors
          {live && <span className="live-badge">LIVE</span>}
        </h3>
        <div className="temp-grid">
          <TempTable title="CPU" readings={sensors?.cpu} />
          <TempTable title="GPU" readings={sensors?.gpu} />
          <TempTable title="NVMe" readings={sensors?.nvme} />
          <TempTable title="Fans" readings={sensors?.fans_rpm} unit=" RPM" />
        </div>
        {usage && (
          <p className="panel-hint">
            Load average: {usage.load_avg.map((v) => v.toFixed(2)).join(", ")} · Uptime:{" "}
            {formatUptime(usage.uptime_secs)}
          </p>
        )}
      </section>

      <section className="panel-card">
        <h3>Power</h3>
        <dl className="stat-list">
          <div className="stat-row">
            <dt>Battery</dt>
            <dd>{power ? `${power.battery_percent.toFixed(0)}%` : "--"}</dd>
          </div>
          <div className="stat-row">
            <dt>AC Connected</dt>
            <dd>{power ? (power.ac_connected ? "Yes" : "No") : "--"}</dd>
          </div>
          <div className="stat-row">
            <dt>Status</dt>
            <dd>{power?.status ?? "--"}</dd>
          </div>
          <div className="stat-row">
            <dt>TLP</dt>
            <dd>{power ? (power.tlp_active ? "Active" : "Inactive") : "--"}</dd>
          </div>
        </dl>
      </section>

      <section className="panel-card">
        <h3>Power Profile</h3>
        <p className="panel-hint">Wraps the backend's TLP-based profile switch.</p>
        <div className="button-row">
          {POWER_PROFILES.map((p) => (
            <button
              key={p.id}
              className={profile === p.id ? "is-active" : ""}
              onClick={() => applyProfile(p.id)}
            >
              {p.label}
            </button>
          ))}
          <button className="accent-button" onClick={openTlpStats}>
            TLP Stats
          </button>
        </div>
      </section>

      <section className="panel-card">
        <h3>Fan Mode</h3>
        <p className="panel-hint">
          Applied via NBFC, matching the original app's fan-control mechanism
          (no raw EC/PWM protocol for this laptop was ever reverse-engineered).
        </p>
        <div className="button-row">
          {FAN_MODES.map((m) => (
            <button
              key={m.id}
              className={fanMode === m.id ? "is-active" : ""}
              onClick={() => applyFanMode(m.id)}
            >
              {m.label}
            </button>
          ))}
          <button className="accent-button" onClick={() => setShowFanDetails(true)}>
            Fan Details
          </button>
        </div>
      </section>

      {status && <p className="status-line">{status}</p>}

      {showFanDetails && (
        <Modal title="Fan Details" onClose={() => setShowFanDetails(false)}>
          {sensors?.fans_rpm && sensors.fans_rpm.length > 0 ? (
            <ul className="modal-list">
              {sensors.fans_rpm.map((fan) => (
                <li key={fan.label}>
                  <span>{fan.label}</span>
                  <span>{fan.celsius.toFixed(0)} RPM</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="panel-hint">
              No fan sensors detected. This laptop's fans may not expose hwmon/NBFC
              telemetry - fan mode can still be set above even without a speed readout.
            </p>
          )}
        </Modal>
      )}

      {showTlpStats && (
        <Modal title="TLP Stats" onClose={() => setShowTlpStats(false)}>
          {tlpStatsError ? (
            <p className="panel-hint">Failed to read TLP stats: {tlpStatsError}</p>
          ) : tlpStatsText ? (
            <pre className="modal-pre">{tlpStatsText}</pre>
          ) : (
            <p className="panel-hint">Loading…</p>
          )}
        </Modal>
      )}
    </div>
  );
}
