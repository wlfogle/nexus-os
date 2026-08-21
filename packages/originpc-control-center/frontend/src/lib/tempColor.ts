// Matches the old app's `update_system_data` color thresholds exactly
// (enhanced-professional-control-center.py): >80C red, >65C orange,
// otherwise green.
export function tempColor(celsius: number | undefined): string {
  if (celsius === undefined) return "var(--text-muted)";
  if (celsius > 80) return "var(--danger)";
  if (celsius > 65) return "var(--warn)";
  return "var(--ok)";
}
