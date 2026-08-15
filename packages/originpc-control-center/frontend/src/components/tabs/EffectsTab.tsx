import { useState } from "react";
import { startEffect, stopEffect } from "../../lib/api";
import type { EffectName } from "../../types";

const EFFECTS: Array<{ id: EffectName; label: string; description: string }> = [
  { id: "wave", label: "Wave", description: "Color wave sweeping across the keyboard." },
  { id: "breathing", label: "Breathing", description: "Whole keyboard fades in and out." },
  { id: "rainbow", label: "Rainbow", description: "Cycles through the full hue spectrum." },
];

export function EffectsTab() {
  const [selected, setSelected] = useState<EffectName>("wave");
  const [speed, setSpeed] = useState(5);
  const [running, setRunning] = useState(false);
  const [status, setStatus] = useState("");

  async function handleStart() {
    try {
      await startEffect(selected, speed);
      setRunning(true);
      setStatus(`Started "${selected}" at speed ${speed}`);
    } catch (err) {
      setStatus(`Failed to start effect: ${String(err)}`);
    }
  }

  async function handleStop() {
    try {
      await stopEffect();
      setRunning(false);
      setStatus("Effect stopped");
    } catch (err) {
      setStatus(`Failed to stop effect: ${String(err)}`);
    }
  }

  return (
    <div className="tab-panel">
      <section className="panel-card">
        <h3>Lighting Effect</h3>
        <div className="effect-grid">
          {EFFECTS.map((effect) => (
            <button
              key={effect.id}
              className={`effect-option ${selected === effect.id ? "is-active" : ""}`}
              onClick={() => setSelected(effect.id)}
            >
              <strong>{effect.label}</strong>
              <span>{effect.description}</span>
            </button>
          ))}
        </div>

        <label className="slider-row">
          Speed: {speed}
          <input
            type="range"
            min={1}
            max={10}
            value={speed}
            onChange={(e) => setSpeed(Number(e.target.value))}
          />
        </label>

        <div className="button-row">
          <button onClick={handleStart} disabled={running}>
            Start Effect
          </button>
          <button className="danger-button" onClick={handleStop} disabled={!running}>
            Stop Effect
          </button>
        </div>
        {running && (
          <p className="live-note">
            Effect running - "{selected}" at speed {speed}.
          </p>
        )}
      </section>

      {status && <p className="status-line">{status}</p>}
    </div>
  );
}
