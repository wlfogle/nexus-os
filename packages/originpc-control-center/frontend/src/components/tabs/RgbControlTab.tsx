import { useState } from "react";
import { clearAllKeys, setGroupColor, setKeyColor } from "../../lib/api";
import { KEY_GROUPS, KEY_NAMES } from "../../lib/keyboardData";

const PRESETS: Array<{ name: string; hex: string }> = [
  { name: "Red", hex: "#ff0000" },
  { name: "Green", hex: "#00ff00" },
  { name: "Blue", hex: "#0066ff" },
  { name: "Orange", hex: "#ff6600" },
  { name: "Purple", hex: "#9d00ff" },
  { name: "Cyan", hex: "#00ffff" },
  { name: "White", hex: "#ffffff" },
];

function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace("#", "");
  return [
    parseInt(clean.slice(0, 2), 16),
    parseInt(clean.slice(2, 4), 16),
    parseInt(clean.slice(4, 6), 16),
  ];
}

// Matches the old app's preset-button styling exactly: full-color fill,
// black text on light colors and white text on dark ones
// (`'black' if sum(color) > 400 else 'white'` in enhanced-professional-
// control-center.py's `create_control_panel_content`).
function contrastingTextColor(hex: string): string {
  const [r, g, b] = hexToRgb(hex);
  return r + g + b > 400 ? "#000" : "#fff";
}

export function RgbControlTab() {
  const [color, setColor] = useState("#00c8ff");
  const [selectedKey, setSelectedKey] = useState(KEY_NAMES[0]);
  const [status, setStatus] = useState("");

  const [r, g, b] = hexToRgb(color);

  async function applyToGroup(groupId: string, hex: string) {
    const [rr, gg, bb] = hexToRgb(hex);
    try {
      await setGroupColor(groupId, rr, gg, bb);
      setStatus(`Applied ${hex} to "${groupId}"`);
    } catch (err) {
      setStatus(`Failed to set group color: ${String(err)}`);
    }
  }

  async function applyToKey() {
    try {
      await setKeyColor(selectedKey, r, g, b);
      setStatus(`Applied ${color} to key "${selectedKey}"`);
    } catch (err) {
      setStatus(`Failed to set key color: ${String(err)}`);
    }
  }

  async function handleClear() {
    try {
      await clearAllKeys();
      setStatus("All keys cleared");
    } catch (err) {
      setStatus(`Failed to clear keys: ${String(err)}`);
    }
  }

  return (
    <div className="tab-panel">
      <section className="panel-card">
        <h3>Color</h3>
        <div className="color-row">
          <input
            type="color"
            value={color}
            onChange={(e) => setColor(e.target.value)}
            aria-label="Color picker"
          />
          <span className="rgb-readout">
            {color.toUpperCase()} · rgb({r}, {g}, {b})
          </span>
        </div>
        <div className="preset-row">
          {PRESETS.map((p) => (
            <button
              key={p.name}
              className="preset-swatch"
              style={{ backgroundColor: p.hex, color: contrastingTextColor(p.hex) }}
              onClick={() => {
                setColor(p.hex);
                void applyToGroup("all_keys", p.hex);
              }}
            >
              {p.name}
            </button>
          ))}
        </div>
      </section>

      <section className="panel-card">
        <h3>Key Groups</h3>
        <p className="panel-hint">Apply the current color to every key in a group.</p>
        <div className="group-grid">
          {KEY_GROUPS.map((group) => (
            <button key={group.id} onClick={() => applyToGroup(group.id, color)}>
              {group.label}
            </button>
          ))}
        </div>
        <button className="danger-button" onClick={handleClear}>
          Clear All Keys
        </button>
      </section>

      <section className="panel-card">
        <h3>Per-Key Color</h3>
        <p className="panel-hint">Apply the current color to a single named key.</p>
        <div className="key-picker-row">
          <select value={selectedKey} onChange={(e) => setSelectedKey(e.target.value)}>
            {KEY_NAMES.map((key) => (
              <option key={key} value={key}>
                {key}
              </option>
            ))}
          </select>
          <button onClick={applyToKey}>Apply to Key</button>
        </div>
      </section>

      {status && <p className="status-line">{status}</p>}
    </div>
  );
}
