// Mirrors hw/src/usage.rs's `format_uptime` ("Nd HHh MMm").
export function formatUptime(totalSecs: number): string {
  const days = Math.floor(totalSecs / 86400);
  const hours = Math.floor((totalSecs % 86400) / 3600);
  const minutes = Math.floor((totalSecs % 3600) / 60);
  return `${days}d ${hours}h ${minutes}m`;
}

// Backend/threshold logic (see tempColor.ts) stays in Celsius throughout -
// this only converts at the final display step, per user preference for
// Fahrenheit readouts.
export function celsiusToFahrenheit(celsius: number): number {
  return (celsius * 9) / 5 + 32;
}

export function formatTemp(celsius: number): string {
  return `${celsiusToFahrenheit(celsius).toFixed(1)}\u00b0F`;
}
