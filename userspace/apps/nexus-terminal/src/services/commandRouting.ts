import { invoke } from '@tauri-apps/api/core';
import { routingLogger } from '../utils/logger';

export interface CommandRoutingResult {
  isShellCommand: boolean;
  confidence: number;
  reason: string;
  suggestedAction: 'execute_shell' | 'send_to_ai' | 'ask_user';
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
      pattern: /^(ls|ll|la|dir)\s*$/i,
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

  // AI trigger patterns - these should go to AI
  private readonly aiTriggerPatterns: RegExp[] = [
    // Question words at start
    /^(what|how|why|when|where|who|can|should|would|could|will|is|are|do|does|did|explain|help|show|tell|describe|please)\s+/i,
    
    // Conversational phrases
    /\b(help me|explain|show me|tell me|what is|how do|why does|can you|could you|would you|help with|assist with)\b/i,
    
    // Questions with question marks
    /\?/,
    
    // AI request phrases
    /^(generate|create|write|suggest|recommend|analyze|review|check|debug|fix|optimize|improve)\s+(code|script|function|a)/i,
    
    // Natural language patterns
    /^(i want to|i need to|i would like|please help|can you help|how can i)\b/i
  ];

  // Shell pattern detectors
  private readonly shellPatterns: RegExp[] = [
    // Executable paths
    /^(\.\/|\/|~\/)/,
    
    // Environment variables
    /^[A-Z_][A-Z0-9_]*=/,
    
    // Command with sudo
    /^sudo\s+/,
    
    // Pipe operations
    /[\|&]{1,2}/,
    
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
   * Determines if input is a shell command or AI query
   */
  public async routeCommand(input: string): Promise<CommandRoutingResult> {
    const trimmed = input.trim();
    
    if (!trimmed) {
      return {
        isShellCommand: false,
        confidence: 0,
        reason: 'Empty input',
        suggestedAction: 'ask_user'
      };
    }

    // Check for explicit AI triggers first (highest priority)
    for (const pattern of this.aiTriggerPatterns) {
      if (pattern.test(trimmed)) {
        return {
          isShellCommand: false,
          confidence: 0.95,
          reason: `AI trigger pattern detected: ${pattern.source}`,
          suggestedAction: 'send_to_ai'
        };
      }
    }

    // Check high priority shell patterns
    for (const shellPattern of this.highPriorityShellPatterns) {
      if (Array.isArray(shellPattern.pattern)) {
        const firstWord = trimmed.split(/\s+/)[0].toLowerCase();
        if (shellPattern.pattern.includes(firstWord)) {
          return {
            isShellCommand: true,
            confidence: 0.9 + (shellPattern.priority / 100),
            reason: `High priority shell command detected: ${shellPattern.description}`,
            suggestedAction: 'execute_shell'
          };
        }
      } else if (shellPattern.pattern.test(trimmed)) {
        return {
          isShellCommand: true,
          confidence: 0.9 + (shellPattern.priority / 100),
          reason: `High priority shell pattern matched: ${shellPattern.description}`,
          suggestedAction: 'execute_shell'
        };
      }
    }

    // Check if first word is a known shell command
    const firstWord = trimmed.split(/\s+/)[0].toLowerCase();
    const shellCommandInfo = this.findShellCommand(firstWord);
    
    if (shellCommandInfo) {
      return {
        isShellCommand: true,
        confidence: 0.8 + (shellCommandInfo.priority / 100),
        reason: `Known shell command from ${shellCommandInfo.category}: ${firstWord}`,
        suggestedAction: 'execute_shell'
      };
    }

    // Check for shell patterns
    for (const pattern of this.shellPatterns) {
      if (pattern.test(trimmed)) {
        return {
          isShellCommand: true,
          confidence: 0.75,
          reason: `Shell pattern detected: ${pattern.source}`,
          suggestedAction: 'execute_shell'
        };
      }
    }

    // Check if it's an executable file
    try {
      const isExecutable = await this.checkIfExecutable(firstWord);
      if (isExecutable) {
        return {
          isShellCommand: true,
          confidence: 0.85,
          reason: `Detected executable: ${firstWord}`,
          suggestedAction: 'execute_shell'
        };
      }
    } catch (error) {
      // Ignore error and continue with other checks
    }

    // Heuristic checks for command-like inputs
    const words = trimmed.split(/\s+/);
    
    // Short, command-like inputs
    if (words.length <= 3 && trimmed.length < 40 && !trimmed.includes('?')) {
      // Check if it looks like a command with flags
      if (words.some(word => word.startsWith('-'))) {
        return {
          isShellCommand: true,
          confidence: 0.7,
          reason: 'Short input with command flags detected',
          suggestedAction: 'execute_shell'
        };
      }
      
      // Single word that might be a command
      if (words.length === 1 && firstWord.length < 20 && /^[a-z][a-z0-9-]*$/.test(firstWord)) {
        return {
          isShellCommand: true,
          confidence: 0.6,
          reason: 'Single word resembling command name',
          suggestedAction: 'execute_shell'
        };
      }
    }

    // If we reach here, it's likely an AI query
    return {
      isShellCommand: false,
      confidence: 0.8,
      reason: 'Natural language query detected',
      suggestedAction: 'send_to_ai'
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
   * Simple synchronous check for backwards compatibility
   */
  public isShellCommand(input: string): boolean {
    const trimmed = input.trim();
    
    if (!trimmed) {
      routingLogger.debug('Empty input provided', 'is_shell_command', { input });
      return false;
    }

    // Quick check for AI triggers
    for (const pattern of this.aiTriggerPatterns) {
      if (pattern.test(trimmed)) {
        routingLogger.debug('AI trigger pattern detected', 'is_shell_command', { input, pattern: pattern.source });
        return false;
      }
    }

    // Quick check for known shell commands
    const firstWord = trimmed.split(/\s+/)[0].toLowerCase();
    const allShellCommands = [
      ...this.shellCommands.fileOps,
      ...this.shellCommands.textOps,
      ...this.shellCommands.systemOps,
      ...this.shellCommands.networkOps,
      ...this.shellCommands.packageOps,
      ...this.shellCommands.devOps,
      ...this.shellCommands.archiveOps,
      ...this.shellCommands.envOps
    ];

    if (allShellCommands.includes(firstWord)) {
      routingLogger.debug('Known shell command detected', 'is_shell_command', { input, firstWord });
      return true;
    }

    // Quick pattern checks
    for (const pattern of this.shellPatterns) {
      if (pattern.test(trimmed)) {
        routingLogger.debug('Shell pattern detected', 'is_shell_command', { input, pattern: pattern.source });
        return true;
      }
    }

    // Default heuristic for short inputs
    const words = trimmed.split(/\s+/);
    const isShort = words.length <= 3 && trimmed.length < 40 && !trimmed.includes('?');
    
    routingLogger.debug(
      `Heuristic analysis: ${isShort ? 'shell' : 'AI'}`, 
      'is_shell_command', 
      { input, wordCount: words.length, length: trimmed.length, hasQuestion: trimmed.includes('?') }
    );
    
    return isShort;
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
