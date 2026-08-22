import type { CSSProperties } from "react";

interface GaugeRingProps {
  label: string;
  percent: number;
  color: string;
}

/**
 * Circular percentage gauge rendered with a CSS `conic-gradient` - no
 * charting library needed for a single ring. Matches the old app's CPU
 * Usage / Memory Usage donut gauges (dark track, colored arc, bold
 * percentage centered inside).
 */
export function GaugeRing({ label, percent, color }: GaugeRingProps) {
  const clamped = Math.max(0, Math.min(100, percent));
  const style = {
    "--gauge-color": color,
    "--gauge-percent": `${clamped}`,
  } as CSSProperties;

  return (
    <div className="gauge">
      <div className="gauge-ring" style={style}>
        <div className="gauge-value">{Math.round(clamped)}%</div>
      </div>
      <div className="gauge-label">{label}</div>
    </div>
  );
}
