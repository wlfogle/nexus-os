import { useEffect, useRef, useState } from "react";
import { getPowerInfo, getSensorSnapshot, getSystemUsage, listenSystemStats } from "../lib/api";
import type { PowerInfo, SensorSnapshot, SystemUsage } from "../types";

interface SystemStatsState {
  sensors: SensorSnapshot | null;
  power: PowerInfo | null;
  usage: SystemUsage | null;
  /** True while readings are arriving via the "system-stats" push event. */
  live: boolean;
}

const POLL_CHECK_MS = 2500;
// If no push event has arrived within this window, assume backend-agent's
// "system-stats" emitter either hasn't landed yet or isn't running, and
// fall back to polling the on-demand commands - per CONTRACT.md's note:
// "fall back to polling get_sensor_snapshot/get_power_info if that lands
// later". Once events resume, polling stops again automatically.
const EVENT_STALE_MS = 4000;

/**
 * Single shared subscription to system telemetry, used by both the sidebar
 * and the System tab so they never issue duplicate polls/listeners.
 */
export function useSystemStats(): SystemStatsState {
  const [sensors, setSensors] = useState<SensorSnapshot | null>(null);
  const [power, setPower] = useState<PowerInfo | null>(null);
  const [usage, setUsage] = useState<SystemUsage | null>(null);
  const [live, setLive] = useState(false);
  const lastEventAt = useRef(0);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    async function pollOnce() {
      try {
        const [sensorData, powerData, usageData] = await Promise.all([
          getSensorSnapshot(),
          getPowerInfo(),
          getSystemUsage(),
        ]);
        if (!cancelled) {
          setSensors(sensorData);
          setPower(powerData);
          setUsage(usageData);
        }
      } catch (err) {
        console.error("Failed to poll system stats:", err);
      }
    }

    listenSystemStats((payload) => {
      if (cancelled) return;
      lastEventAt.current = Date.now();
      setSensors(payload.sensors);
      setPower(payload.power);
      setUsage(payload.usage);
      setLive(true);
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    // Prime the view immediately, then periodically re-check whether the
    // push event has gone quiet and poll on-demand if so.
    pollOnce();
    const interval = setInterval(() => {
      if (Date.now() - lastEventAt.current > EVENT_STALE_MS) {
        setLive(false);
        pollOnce();
      }
    }, POLL_CHECK_MS);

    return () => {
      cancelled = true;
      clearInterval(interval);
      unlisten?.();
    };
  }, []);

  return { sensors, power, usage, live };
}
