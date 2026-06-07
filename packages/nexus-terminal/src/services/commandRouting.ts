import { invoke } from '@tauri-apps/api/core';
import { routingLogger } from '../utils/logger';

export interface CommandRoutingResult {
  isShellCommand: boolean;
  confidence: number;
  reason: string;
  suggestedAction: 'execute_shell' | 'send_to_ai' | 'ask_user';
}

// ─── Language-structure helpers ───────────────────────────────────────────────

/** English articles / prepositions that signal natural language in the middle of input. */
const NL_FILLER_WORDS = new Set([
  'a', 'an', 'the',
  'about', 'above', 'across', 'after', 'against', 'along', 'among', 'around',
  'at', 'before', 'behind', 'below', 'beneath', 'beside', 'between', 'by',
  'down', 'during', 'except', 'for', 'from', 'in', 'inside', 'into',
  'like', 'near', 'of', 'off', 'on', 'onto', 'out', 'outside', 'over',
  'past', 'since', 'than', 'through', 'throughout', 'to', 'toward', 'towards',
  'under', 'until', 'up', 'upon', 'with', 'within', 'without',
]);

/** Returns true when any word after position 0 is a clear filler word. */
function hasNaturalLanguageStructure(words: string[]): boolean {
  if (words.length < 2) return false;
  // Check words after the first
  return words.slice(1).some(w => NL_FILLER_WORDS.has(w.toLowerCase()));
}

export interface ShellCommandPattern {
  pattern: RegExp | string[];
  priority: number;
  description: string;
}

export class CommandRoutingService {
  // High priority shell command patterns - these take precedence
  private readonly highPriorityShellPatterns: ShellCommandPattern[] = [
    {
      pattern: /^(ls|ll|la|dir)(\s+.*)?$/i,
      priority: 10,
      description: 'Directory listing commands'
    },
    {
      pattern: /^(pwd|cd)\s*/i,
      priority: 10,
      description: 'Directory navigation commands'
    },
    {
      pattern: /^(ps|top|htop|kill|killall)\s*/i,
      priority: 10,
      description: 'Process management commands'
    },
    {
      pattern: /^(git|docker|kubectl|npm|yarn|cargo|pip)\s+/i,
      priority: 9,
      description: 'Development tool commands'
    }
  ];

  // Comprehensive shell command list organized by category
  private readonly shellCommands = {
    // File operations (priority 8)
    fileOps: [
      'ls', 'll', 'la', 'dir', 'pwd', 'cd', 'mkdir', 'rmdir', 'rm', 'cp', 'mv', 
      'ln', 'find', 'locate', 'touch', 'chmod', 'chown', 'chgrp', 'file', 
      'stat', 'du', 'df', 'tree', 'rsync'
    ],
    
    // Text processing (priority 7)
    textOps: [
      'cat', 'less', 'more', 'head', 'tail', 'grep', 'awk', 'sed', 'sort', 
      'uniq', 'cut', 'tr', 'wc', 'diff', 'comm', 'join', 'paste', 'split'
    ],
    
    // System info and process management (priority 9)
    systemOps: [
      'ps', 'top', 'htop', 'kill', 'killall', 'jobs', 'nohup', 'screen', 'tmux',
      'who', 'w', 'users', 'id', 'groups', 'sudo', 'su', 'whoami', 'date', 'uptime',
      'uname', 'hostname', 'dmesg', 'lscpu', 'lsmem', 'lsblk', 'lsusb', 'lspci',
      'systemctl', 'service', 'journalctl', 'systemd-analyze'
    ],
    
    // Network operations (priority 7)
    networkOps: [
      'ping', 'curl', 'wget', 'ssh', 'scp', 'rsync', 'netstat', 'ss', 'nmap',
      'iptables', 'route', 'ip', 'ifconfig', 'tcpdump', 'nc', 'ncat'
    ],
    
    // Package management (priority 8)
    packageOps: [
      'apt', 'yum', 'dnf', 'pacman', 'yay', 'paru', 'brew', 'pip', 'pip3', 
      'npm', 'yarn', 'pnpm', 'cargo', 'go', 'gem', 'composer', 'conda', 
      'snap', 'flatpak'
    ],
    
    // Development tools (priority 9)
    devOps: [
      'git', 'docker', 'docker-compose', 'kubectl', 'helm', 'terraform',
      'make', 'cmake', 'gcc', 'g++', 'clang', 'rustc', 'node', 'python', 'python3',
      'java', 'javac', 'mvn', 'gradle', 'vim', 'nano', 'emacs', 'code', 'nvim'
    ],
    
    // Archive operations (priority 6)
    archiveOps: [
      'tar', 'zip', 'unzip', 'gzip', 'gunzip', 'bzip2', 'bunzip2', '7z'
    ],
    
    // Environment and shell (priority 7)
    envOps: [
      'env', 'export', 'set', 'unset', 'alias', 'unalias', 'which', 'type', 
      'whereis', 'history', 'clear', 'reset', 'source', 'exec', 'eval'
    ]
  };

  // Shell pattern detectors
  private readonly shellPatterns: RegExp[] = [
    // Executable paths
    /^(\.\/|\/|~\/)/,
    
    // Environment variables
    /^[A-Z_][A-Z0-9_]*=/,
    
    // Command with sudo
    /^sudo\s+/,
    
    // Pipe operations
    /[|&]{1,2}/,
    
    // Redirection
    /[<>]/,
    
    // Command substitution
    /[$`]/,
    
    // File globs
    /[*?[\]]/,
    
    // Command chaining
    /[;&]/
  ];

  /**
   * Route input to shell or AI using a strict priority-ordered algorithm.
   *
   * TIER 1 — Definite shell (checked FIRST, before any NL heuristics)
   *   a. First word is a known command in our dictionary
   *   b. High-priority structural patterns (pipes, paths, redirects, globs …)
   *
   * TIER 2 — Try async PATH lookup for unknown first words
   *
   * TIER 3 — Definite natural language (question words, conversational phrases)
   *
   * TIER 4 — Structural NL heuristics (articles/prepositions, word count)
   *
   * TIER 5 — Default → AI  (safer than misfiring a shell command)
   */
  public async routeCommand(input: string): Promise<CommandRoutingResult> {
    const trimmed = input.trim();

    if (!trimmed) {
      return { isShellCommand: false, confidence: 0, reason: 'Empty input', suggestedAction: 'ask_user' };
    }

    const words  = trimmed.split(/\s+/);
    const first  = words[0].toLowerCase();

    // ── TIER 1a: known command dictionary (always wins) ───────────────────────
    const shellCommandInfo = this.findShellCommand(first);
    if (shellCommandInfo) {
      return {
        isShellCommand: true,
        confidence: 0.80 + (shellCommandInfo.priority / 100),
        reason: `Known shell command (${shellCommandInfo.category}): ${first}`,
        suggestedAction: 'execute_shell',
      };
    }

    // ── TIER 1b: high-priority regex shell patterns ───────────────────────────
    for (const sp of this.highPriorityShellPatterns) {
      const matched = Array.isArray(sp.pattern)
        ? sp.pattern.includes(first)
        : (sp.pattern as RegExp).test(trimmed);
      if (matched) {
        return {
          isShellCommand: true,
          confidence: 0.90 + (sp.priority / 100),
          reason: `High-priority shell pattern: ${sp.description}`,
          suggestedAction: 'execute_shell',
        };
      }
    }

    // ── TIER 1c: structural shell patterns (pipes, paths, redirects …) ────────
    for (const pattern of this.shellPatterns) {
      if (pattern.test(trimmed)) {
        return {
          isShellCommand: true,
          confidence: 0.85,
          reason: `Shell structural pattern: ${pattern.source}`,
          suggestedAction: 'execute_shell',
        };
      }
    }

    // ── TIER 2: async PATH lookup for unknown first words ────────────────────
    try {
      if (await this.checkIfExecutable(first)) {
        return {
          isShellCommand: true,
          confidence: 0.85,
          reason: `Executable found in PATH: ${first}`,
          suggestedAction: 'execute_shell',
        };
      }
    } catch { /* continue */ }

    // ── TIER 3: definite natural language ─────────────────────────────────────
    // 3a. Question mark anywhere → always AI
    if (trimmed.includes('?')) {
      return { isShellCommand: false, confidence: 0.98, reason: 'Contains question mark', suggestedAction: 'send_to_ai' };
    }

    // 3b. Clear question-word openers (what / how / why / when / where / who)
    if (/^(what|how|why|when|where|who)\b/i.test(trimmed)) {
      return { isShellCommand: false, confidence: 0.97, reason: 'Starts with question word', suggestedAction: 'send_to_ai' };
    }

    // 3c. Unambiguous conversational openers
    if (/^(can you|could you|would you|please help|i want to|i need to|i would like|help me\b)/i.test(trimmed)) {
      return { isShellCommand: false, confidence: 0.97, reason: 'Conversational opener', suggestedAction: 'send_to_ai' };
    }

    // 3d. Embedded NL phrases that never appear in shell commands
    if (/\b(help me|show me how|tell me how|explain to me|how do i|what is|what are|how to)\b/i.test(trimmed)) {
      return { isShellCommand: false, confidence: 0.95, reason: 'Embedded natural-language phrase', suggestedAction: 'send_to_ai' };
    }

    // 3e. AI-specific verb + object pairs (generate/write/create/etc. + NL object)
    //     Only when the object contains clear English words, not file paths or flags
    if (
      /^(generate|create|write|suggest|recommend|analyze|analyse|review|check|debug|fix|optimize|optimise|improve|explain|describe)\s+/i.test(trimmed) &&
      hasNaturalLanguageStructure(words)
    ) {
      return { isShellCommand: false, confidence: 0.90, reason: 'AI verb with natural-language object', suggestedAction: 'send_to_ai' };
    }

    // ── TIER 4: structural NL heuristics ─────────────────────────────────────
    // 4a. Contains articles or prepositions not at position 0 → likely prose
    if (hasNaturalLanguageStructure(words)) {
      return { isShellCommand: false, confidence: 0.82, reason: 'Natural-language structure (articles/prepositions)', suggestedAction: 'send_to_ai' };
    }

    // 4b. 4+ words with no known command first word → almost certainly prose
    if (words.length >= 4) {
      return { isShellCommand: false, confidence: 0.80, reason: 'Long multi-word input without command prefix', suggestedAction: 'send_to_ai' };
    }

    // 4c. Looks like a command invocation: short, lowercase, optional flags/paths
    if (
      words.length <= 3 &&
      trimmed.length < 60 &&
      /^[a-z][a-z0-9_-]*$/.test(first)
    ) {
      // If any subsequent word is a flag or path, treat as shell
      const hasFlag = words.slice(1).some(w => w.startsWith('-') || w.startsWith('/') || w.startsWith('~') || w.includes('.'));
      if (hasFlag) {
        return { isShellCommand: true, confidence: 0.72, reason: 'Command-like invocation with flags or paths', suggestedAction: 'execute_shell' };
      }

      // Single unknown word: could be an alias or custom script — treat as shell
      if (words.length === 1 && first.length < 25) {
        return { isShellCommand: true, confidence: 0.60, reason: 'Single unrecognised word — treating as potential command', suggestedAction: 'execute_shell' };
      }
    }

    // ── TIER 5: default → AI ─────────────────────────────────────────────────
    // Safer default: unclassified input is more likely natural language than
    // an unknown shell command that should silently execute.
    return {
      isShellCommand: false,
      confidence: 0.75,
      reason: 'Unclassified input — defaulting to AI (safer)',
      suggestedAction: 'send_to_ai',
    };
  }

  /**
   * Find shell command in categorized lists
   */
  private findShellCommand(command: string): { category: string; priority: number } | null {
    const categories = [
      { name: 'systemOps', priority: 9, commands: this.shellCommands.systemOps },
      { name: 'devOps', priority: 9, commands: this.shellCommands.devOps },
      { name: 'fileOps', priority: 8, commands: this.shellCommands.fileOps },
      { name: 'packageOps', priority: 8, commands: this.shellCommands.packageOps },
      { name: 'textOps', priority: 7, commands: this.shellCommands.textOps },
      { name: 'networkOps', priority: 7, commands: this.shellCommands.networkOps },
      { name: 'envOps', priority: 7, commands: this.shellCommands.envOps },
      { name: 'archiveOps', priority: 6, commands: this.shellCommands.archiveOps },
    ];

    for (const category of categories) {
      if (category.commands.includes(command)) {
        return { category: category.name, priority: category.priority };
      }
    }

    return null;
  }

  /**
   * Check if a command is an executable file
   */
  private async checkIfExecutable(command: string): Promise<boolean> {
    try {
      // Use the which command to check if executable exists
      const result = await invoke('execute_safe_system_command', { 
        command: `which ${command} 2>/dev/null || command -v ${command} 2>/dev/null` 
      }) as string;
      
      return result.trim().length > 0;
    } catch {
      return false;
    }
  }

  /**
   * Synchronous fast-path — mirrors the tier logic of routeCommand without
   * async PATH lookup (Tier 2 is skipped here).
   */
  public isShellCommand(input: string): boolean {
    const trimmed = input.trim();
    if (!trimmed) return false;

    const words = trimmed.split(/\s+/);
    const first = words[0].toLowerCase();

    // Tier 1a — known command dictionary
    if (this.findShellCommand(first)) {
      routingLogger.debug('Known shell command', 'is_shell_command', { input, first });
      return true;
    }

    // Tier 1b/c — structural patterns
    for (const sp of this.highPriorityShellPatterns) {
      const matched = Array.isArray(sp.pattern)
        ? sp.pattern.includes(first)
        : (sp.pattern as RegExp).test(trimmed);
      if (matched) return true;
    }
    for (const pattern of this.shellPatterns) {
      if (pattern.test(trimmed)) return true;
    }

    // Tier 3 — definite NL signals
    if (trimmed.includes('?')) return false;
    if (/^(what|how|why|when|where|who)\b/i.test(trimmed)) return false;
    if (/^(can you|could you|would you|please help|i want to|i need to|i would like|help me\b)/i.test(trimmed)) return false;
    if (/\b(help me|show me how|tell me how|explain to me|how do i|what is|what are|how to)\b/i.test(trimmed)) return false;
    if (
      /^(generate|create|write|suggest|recommend|analyze|analyse|review|check|debug|fix|optimize|optimise|improve|explain|describe)\s+/i.test(trimmed) &&
      hasNaturalLanguageStructure(words)
    ) return false;

    // Tier 4 — structural NL heuristics
    if (hasNaturalLanguageStructure(words)) return false;
    if (words.length >= 4) return false;

    // Short command-like input
    if (words.length <= 3 && trimmed.length < 60 && /^[a-z][a-z0-9_-]*$/.test(first)) {
      const hasFlag = words.slice(1).some(w => w.startsWith('-') || w.startsWith('/') || w.startsWith('~') || w.includes('.'));
      if (hasFlag) return true;
      if (words.length === 1) return true;
    }

    // Tier 5 — default AI
    return false;
  }

  /**
   * Get detailed analysis of command routing decision
   */
  public async analyzeCommand(input: string): Promise<{
    routing: CommandRoutingResult;
    alternatives: string[];
    explanation: string;
  }> {
    const routing = await this.routeCommand(input);
    const alternatives: string[] = [];
    
    let explanation = `Command: "${input}"\n`;

    explanation += `Decision: ${routing.isShellCommand ? 'Shell Command' : 'AI Query'}\n`;
    explanation += `Confidence: ${(routing.confidence * 100).toFixed(1)}%\n`;
    explanation += `Reason: ${routing.reason}\n`;

    // Suggest alternatives if confidence is low
    if (routing.confidence < 0.8) {
      if (routing.isShellCommand) {
        alternatives.push(`Ask AI: "help me with ${input}"`);
        alternatives.push(`Ask AI: "explain ${input}"`);
      } else {
        alternatives.push(`Execute as shell: ${input}`);
        alternatives.push(`Execute with confirmation: ${input}`);
      }
    }

    return { routing, alternatives, explanation };
  }
}

// Export singleton instance
export const commandRoutingService = new CommandRoutingService();

// Export convenience function
export const routeCommand = (input: string) => commandRoutingService.routeCommand(input);
export const isShellCommand = (input: string) => commandRoutingService.isShellCommand(input);
