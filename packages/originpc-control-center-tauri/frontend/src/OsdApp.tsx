import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { HotkeyEventPayload } from "./types";

// Renders the Fn-hotkey popup. The Rust side shows/hides this window
// (see CONTRACT.md "hotkey-event") - this component only needs to render
// whatever the latest event said, auto-clearing after a short delay so a
// stale message doesn't linger if the window is ever left visible.
function OsdApp() {
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<HotkeyEventPayload>("hotkey-event", (event) => {
      setMessage(`${event.payload.icon}  ${event.payload.label}`);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        height: "100vh",
        fontFamily: "sans-serif",
        fontSize: "1.5rem",
        color: "white",
        background: "rgba(30, 30, 30, 0.85)",
        borderRadius: "12px",
      }}
    >
      {message ?? ""}
    </div>
  );
}

export default OsdApp;
