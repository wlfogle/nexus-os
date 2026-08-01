import { invoke } from '@tauri-apps/api/core';
import { routingLogger } from '../utils/logger';

export interface CommandRoutingResult {
  isShellCommand: boolean;
  confidence: number;
  reason: string;
  suggestedAction: 'execute_shell' | 'send_to_ai' | 'ask_user';
}

// ─── NL → Shell direct translation map ───────────────────────────────────────
// Common natural language requests that map to exact shell commands.
// These NEVER go to the AI model — they execute immediately.
const NL_TO_SHELL: Array<{ patterns: RegExp; command: string }> = [
  // File listing
  { patterns: /^(show|list|ls|see|what(?:'s| is| are)|display).*(files?|dir|directory|folder|contents?)/i, command: 'ls -la' },
  { patterns: /^(show me|list|what(?:'s| is)).*(here|in this (dir|folder|directory))/i, command: 'ls -la' },
  // Current directory
  { patterns: /^(where am i|what dir|current dir|show path|pwd)/i, command: 'pwd' },
  // Running processes
  { patterns: /^(what(?:'s| is) running|show.*(process|procs?)|list.*(process|procs?)|running process)/i, command: 'ps aux' },
  // Disk usage
  { patterns: /^(check|show|display).*(disk|storage|space|df)/i, command: 'df -h' },
  { patterns: /^(how much|disk usage|storage usage)/i, command: 'df -h' },
  // Memory
  { patterns: /^(check|show|display).*(memory|mem|ram)/i, command: 'free -h' },
  { patterns: /^(how much|memory usage|ram usage)/i, command: 'free -h' },
  // Network
  { patterns: /^(show|display|check).*(network|ip|interfaces?|addr)/i, command: 'ip -brief addr' },
  { patterns: /^(what(?:'s| is) my ip|show ip)/i, command: 'ip -brief addr' },
  // Git
  { patterns: /^(git status|show.*(git|changes)|what.*(changed|modified|uncommitted))/i, command: 'git status --short --branch' },
  { patterns: /^(recent commits|git log|show.*(commits|history))/i, command: 'git --no-pager log --oneline -10' },
  // Services
  { patterns: /^(what.*(services?|running)|show.*(services?|systemd))/i, command: 'systemctl list-units --type=service --state=running --no-pager --no-legend | head -20' },
  // Docker
  { patterns: /^(show|list|what).*(containers?|docker)/i, command: 'docker ps' },
  // CPU / uptime
  { patterns: /^(cpu|load|uptime|system load)/i, command: 'uptime' },
  { patterns: /^(top processes|most cpu|high cpu)/i, command: 'ps aux --sort=-%cpu | head -10' },
  // Environment
  { patterns: /^(show|list|print).*(env|environment|variables?)/i, command: 'env | sort' },
  // History
  { patterns: /^(show|command).*(history)/i, command: 'history | tail -20' },
];

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`Timed out after ${ms}ms`)), ms);

    promise.then(
      (value) => { clearTimeout(timer); resolve(value); },
      (error) => { clearTimeout(timer); reject(error); },
    );
  });
}

/**
 * Attempts to translate a natural-language phrase directly to a shell command.
 * Returns the shell command string if matched, null otherwise.
 */
export function translateNLToShell(input: string): string | null {
  const trimmed = input.trim();
  for (const entry of NL_TO_SHELL) {
    if (entry.patterns.test(trimmed)) {
      return entry.command;
    }
  }
  return null;
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
   * Ported from Warp's HeuristicClassifier (AGPL-3.0).
   *
   * TIER 1 — Definite shell: command dictionary → regex patterns → structural signals
   * TIER 2 — Async PATH lookup for unknown first words
   * TIER 3 — Definite NL: question marks, question words, conversational phrases
   * TIER 4 — Structural NL heuristics: articles/prepositions, word count
   * TIER 5 — Default → AI (safer than accidental shell execution)
   */
  public async routeCommand(input: string): Promise<CommandRoutingResult> {
    const trimmed = input.trim();
    if (!trimmed) {
      return { isShellCommand: false, confidence: 0, reason: 'Empty input', suggestedAction: 'ask_user' };
    }

    const words = trimmed.split(/\s+/);
    const first = words[0].toLowerCase();

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
          reason: `Shell structural pattern`,
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
    if (trimmed.includes('?')) {
      return { isShellCommand: false, confidence: 0.98, reason: 'Contains question mark', suggestedAction: 'send_to_ai' };
    }
    if (/^(what|how|why|when|where|who)\b/i.test(trimmed)) {
      return { isShellCommand: false, confidence: 0.97, reason: 'Starts with question word', suggestedAction: 'send_to_ai' };
    }
    if (/^(can you|could you|would you|please help|i want to|i need to|i would like|help me\b)/i.test(trimmed)) {
      return { isShellCommand: false, confidence: 0.97, reason: 'Conversational opener', suggestedAction: 'send_to_ai' };
    }
    if (/\b(help me|show me how|tell me how|explain to me|how do i|what is|what are|how to)\b/i.test(trimmed)) {
      return { isShellCommand: false, confidence: 0.95, reason: 'Embedded natural-language phrase', suggestedAction: 'send_to_ai' };
    }
    if (
      /^(generate|create|write|suggest|recommend|analyze|analyse|review|check|debug|fix|optimize|optimise|improve|explain|describe)\s+/i.test(trimmed) &&
      hasNaturalLanguageStructure(words)
    ) {
      return { isShellCommand: false, confidence: 0.90, reason: 'AI verb with natural-language object', suggestedAction: 'send_to_ai' };
    }

    // ── TIER 4: structural NL heuristics ─────────────────────────────────────
    if (hasNaturalLanguageStructure(words)) {
      return { isShellCommand: false, confidence: 0.82, reason: 'Natural-language structure (articles/prepositions)', suggestedAction: 'send_to_ai' };
    }
    if (words.length >= 4) {
      return { isShellCommand: false, confidence: 0.80, reason: 'Long multi-word input without command prefix', suggestedAction: 'send_to_ai' };
    }
    if (words.length <= 3 && trimmed.length < 60 && /^[a-z][a-z0-9_-]*$/.test(first)) {
      // NOTE: only an actual CLI flag ("-x", "--foo") or "~" (home-dir shorthand, which
      // essentially never appears in natural language) counts as strong shell evidence here.
      // A bare "/" path or a dot in the second word used to count too, but that's far too
      // weak a signal: "read /path/to/file.md", "explain HARDWARE.md", and similar
      // natural-language file references match it just as well as real shell invocations
      // do, and would incorrectly short-circuit before ever reaching the PATH lookup / ONNX
      // fallback tiers below.
      const hasFlag = words.slice(1).some(w => w.startsWith('-') || w.startsWith('~'));
      if (hasFlag) {
        return { isShellCommand: true, confidence: 0.72, reason: 'Command-like invocation with a CLI flag', suggestedAction: 'execute_shell' };
      }
      if (words.length === 1 && first.length < 25) {
        return { isShellCommand: true, confidence: 0.60, reason: 'Single unrecognised word — treating as potential command', suggestedAction: 'execute_shell' };
      }
    }

    // ── TIER 5: ML fallback ───────────────────────────────────────────────
    // Heuristic tiers are genuinely inconclusive. Mirrors Warp's real two-stage design:
    // defer to the ONNX model (bert_tiny) instead of blindly defaulting. Bounded by a short
    // timeout — bert_tiny is a tiny CPU model (millisecond-scale), not a reasoning LLM, so a
    // generous timeout isn't needed here.
    try {
      const mlResult = await withTimeout(
        invoke<{ p_shell: number; p_ai: number }>('classify_input_onnx', { input: trimmed }),
        800,
      );

      return {
        isShellCommand: mlResult.p_shell > mlResult.p_ai,
        confidence: Math.max(mlResult.p_shell, mlResult.p_ai),
        reason: `ONNX classifier (p_shell=${mlResult.p_shell.toFixed(3)}, p_ai=${mlResult.p_ai.toFixed(3)})`,
        suggestedAction: mlResult.p_shell > mlResult.p_ai ? 'execute_shell' : 'send_to_ai',
      };
    } catch (error) {
      routingLogger.warn('ONNX classifier fallback failed or timed out, defaulting to AI', error as Error, 'onnx_fallback_failed', { input: trimmed });
    }

    // ── TIER 6: default → AI (ONNX fallback unavailable) ───────────────────────
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

    // Strip trailing sentence punctuation before pattern matching.
    // Warp's tokenizer treats ?,!., as WordDelimiter::Separator — they are
    // stripped from tokens entirely and never reach the glob/syntax check.
    // Without this, "what is running?" and "?" match /[*?[\]]/ (glob pattern)
    // and are wrongly classified as shell commands.
    // Reference: /tmp/warp-src/crates/input_classifier/src/parser.rs line 32
    const forPatterns = trimmed.replace(/[?!.,]+$/, '').trim();

    // A bare punctuation-only input (e.g. "?") is definitively AI after stripping.
    if (!forPatterns) return false;

    // Tier 1b/c — structural patterns (run against stripped form)
    for (const sp of this.highPriorityShellPatterns) {
      const matched = Array.isArray(sp.pattern)
        ? sp.pattern.includes(first)
        : (sp.pattern as RegExp).test(forPatterns);
      if (matched) return true;
    }
    for (const pattern of this.shellPatterns) {
      if (pattern.test(forPatterns)) return true;
    }

    // Tier 3 — definite NL signals (check original trimmed for embedded ?)
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

    // Short command-like input. See the matching comment in routeCommand() above: only an
    // actual CLI flag or "~" counts as strong shell evidence — a bare path/dot in the second
    // word is too weak a signal and misfires on natural-language file references.
    if (words.length <= 3 && trimmed.length < 60 && /^[a-z][a-z0-9_-]*$/.test(first)) {
      const hasFlag = words.slice(1).some(w => w.startsWith('-') || w.startsWith('~'));
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
