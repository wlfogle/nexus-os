import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// Debounce function for performance optimization
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => func.apply(null, args), delay);
  };
}

// Throttle function for performance optimization
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func.apply(null, args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

// Format bytes to human readable format
export function formatBytes(bytes: number, decimals = 2): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

// Format time duration
export function formatDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d ${hours % 24}h`;
  if (hours > 0) return `${hours}h ${minutes % 60}m`;
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
  return `${seconds}s`;
}

// Generate unique ID
export function generateId(prefix = 'id'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

// Copy text to clipboard
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (error) {
    console.error('Failed to copy to clipboard:', error);
    // Fallback method
    try {
      const textArea = document.createElement('textarea');
      textArea.value = text;
      document.body.appendChild(textArea);
      textArea.select();
      document.execCommand('copy');
      document.body.removeChild(textArea);
      return true;
    } catch (fallbackError) {
      console.error('Fallback copy method also failed:', fallbackError);
      return false;
    }
  }
}

// Check if string is valid JSON
export function isValidJSON(str: string): boolean {
  try {
    JSON.parse(str);
    return true;
  } catch {
    return false;
  }
}

// Truncate string with ellipsis
export function truncateString(str: string, maxLength: number): string {
  if (str.length <= maxLength) return str;
  return str.substring(0, maxLength - 3) + '...';
}

// Parse command line arguments
export function parseCommandLine(command: string): { cmd: string; args: string[] } {
  const parts = command.trim().split(/\s+/);
  const cmd = parts[0] || '';
  const args = parts.slice(1);
  return { cmd, args };
}

// Escape HTML entities
export function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Convert ANSI escape codes to HTML
export function ansiToHtml(text: string): string {
  // Basic ANSI color code conversion
  const ansiMap: Record<string, string> = {
    '\u001b[30m': '<span style="color: #000000">', // Black
    '\u001b[31m': '<span style="color: #ff0000">', // Red
    '\u001b[32m': '<span style="color: #00ff00">', // Green
    '\u001b[33m': '<span style="color: #ffff00">', // Yellow
    '\u001b[34m': '<span style="color: #0000ff">', // Blue
    '\u001b[35m': '<span style="color: #ff00ff">', // Magenta
    '\u001b[36m': '<span style="color: #00ffff">', // Cyan
    '\u001b[37m': '<span style="color: #ffffff">', // White
    '\u001b[0m': '</span>', // Reset
  };

  let result = escapeHtml(text);
  Object.entries(ansiMap).forEach(([code, html]) => {
    result = result.split(code).join(html);
  });
  
  return result;
}

// Validate shell command safety
export function isCommandSafe(command: string): boolean {
  const dangerousPatterns = [
    /rm\s+-rf\s+\//, // rm -rf /
    /:\(\)\{\s*:\|:\&\s*\}/, // Fork bomb
    /sudo\s+rm/, // sudo rm commands
    /\>\s*\/dev\/sd/, // Writing to disk devices
    /dd\s+if=/, // dd commands
    /mkfs/, // Format filesystem
    /fdisk/, // Disk partitioning
    /chmod\s+777/, // Dangerous permissions
  ];

  return !dangerousPatterns.some(pattern => pattern.test(command));
}

// Format terminal prompt
export function formatPrompt(user: string, host: string, cwd: string, shell: string): string {
  const shortCwd = cwd.replace(/^\/home\/[^/]+/, '~');
  
  switch (shell) {
    case 'zsh':
      return `%n@%m ${shortCwd} %# `;
    case 'fish':
      return `${user}@${host} ${shortCwd} $ `;
    case 'powershell':
      return `PS ${shortCwd}> `;
    default: // bash
      return `${user}@${host}:${shortCwd}$ `;
  }
}
