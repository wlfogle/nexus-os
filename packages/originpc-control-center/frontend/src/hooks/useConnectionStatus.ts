import { useEffect, useState } from "react";
import { getConnectionStatus } from "../lib/api";
import type { ConnectionStatus } from "../types";

const POLL_MS = 3000;

/**
 * There's no push event for connection status in CONTRACT.md (unlike
 * sensors/power), so this just polls `get_connection_status` - cheap since
 * the backend command is a non-blocking `is_connected()` check.
 */
export function useConnectionStatus(): ConnectionStatus | null {
  const [status, setStatus] = useState<ConnectionStatus | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function poll() {
      try {
        const result = await getConnectionStatus();
        if (!cancelled) setStatus(result);
      } catch (err) {
        console.error("Failed to fetch connection status:", err);
      }
    }

    poll();
    const interval = setInterval(poll, POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  return status;
}
