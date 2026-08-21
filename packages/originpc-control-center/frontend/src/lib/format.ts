// Mirrors hw/src/usage.rs's `format_uptime` ("Nd HHh MMm").
export function formatUptime(totalSecs: number): string {
  const days = Math.floor(totalSecs / 86400);
  const hours = Math.floor((totalSecs % 86400) / 3600);
  const minutes = Math.floor((totalSecs % 3600) / 60);
  return `${days}d ${hours}h ${minutes}m`;
}
