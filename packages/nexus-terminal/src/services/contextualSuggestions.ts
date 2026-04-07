import { invoke } from '@tauri-apps/api/core';

export interface CommandSuggestion {
  command: string;
  description: string;
  confidence: number;
  category: 'file-ops' | 'git' | 'dev' | 'system' | 'navigation' | 'search' | 'network' | 'docker';
  context: string[];
  examples?: string[];
  params?: SuggestionParam[];
  learnedFrom?: 'history' | 'patterns' | 'context' | 'ai';
}

export interface SuggestionParam {
  name: string;
  description: string;
  type: 'string' | 'number' | 'path' | 'url' | 'enum';
  required: boolean;
  defaultValue?: string;
  options?: string[];
  placeholder?: string;
}

export interface ContextInfo {
  currentDirectory: string;
  directoryContents: string[];
  gitRepository?: {
    branch: string;
    status: 'clean' | 'dirty';
    staged: string[];
    modified: string[];
    untracked: string[];
    remote?: string;
  };
  recentCommands: string[];
  workingOnFiles: string[];
  activeProcesses: Array<{ pid: number; name: string; cpu: number }>;
  environmentVars: Record<string, string>;
  shellHistory: string[];
  lastCommandResult: {
    command: string;
    exitCode: number;
    output?: string;
    duration: number;
  };
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  dayOfWeek: string;
  projectType?: 'nodejs' | 'rust' | 'python' | 'go' | 'java' | 'web' | 'other';
}

export interface SuggestionFilter {
  categories?: string[];
  minConfidence?: number;
  maxResults?: number;
  includeExamples?: boolean;
  personalizedOnly?: boolean;
}

export class ContextualSuggestionsService {
  private suggestionCache = new Map<string, CommandSuggestion[]>();
  private contextHistory: ContextInfo[] = [];
  private learningPatterns = new Map<string, number>();
  
  async getSuggestions(
    partialCommand: string = '', 
    filter?: SuggestionFilter
  ): Promise<CommandSuggestion[]> {
    const context = await this.getCurrentContext();
    const cacheKey = this.generateCacheKey(partialCommand, context, filter);
    
    // Check cache first
    const cached = this.suggestionCache.get(cacheKey);
    if (cached) {
      return this.filterSuggestions(cached, filter);
    }
    
    try {
      // Get AI-powered suggestions from backend
      const suggestions = await invoke<CommandSuggestion[]>('get_contextual_suggestions', {
        partialCommand,
        context,
        filter
      });
      
      // Enhance with local context
      const enhancedSuggestions = await this.enhanceWithLocalContext(suggestions, context);
      
      // Cache results
      this.suggestionCache.set(cacheKey, enhancedSuggestions);
      
      return this.filterSuggestions(enhancedSuggestions, filter);
    } catch (error) {
      // Fallback to rule-based suggestions
      return this.getRuleBasedSuggestions(partialCommand, context, filter);
    }
  }
  
  private async getCurrentContext(): Promise<ContextInfo> {
    try {
      const context = await invoke<ContextInfo>('get_current_context');
      
      // Store in history
      this.contextHistory.push(context);
      if (this.contextHistory.length > 1000) {
        this.contextHistory.shift();
      }
      
      return context;
    } catch (error) {
      throw new Error(`Failed to get current context: ${error}`);
    }
  }
  
  private async enhanceWithLocalContext(
    suggestions: CommandSuggestion[], 
    context: ContextInfo
  ): Promise<CommandSuggestion[]> {
    const enhanced = [...suggestions];
    
    // Add context-specific suggestions
    enhanced.push(...this.getFileBasedSuggestions(context));
    enhanced.push(...this.getGitBasedSuggestions(context));
    enhanced.push(...this.getProjectBasedSuggestions(context));
    enhanced.push(...this.getHistoryBasedSuggestions(context));
    enhanced.push(...this.getTimeBasedSuggestions(context));
    
    // Sort by confidence and relevance
    return enhanced.sort((a, b) => b.confidence - a.confidence);
  }
  
  private getFileBasedSuggestions(context: ContextInfo): CommandSuggestion[] {
    const suggestions: CommandSuggestion[] = [];
    
    // Code files detected
    const codeFiles = context.directoryContents.filter(f => 
      /\.(js|ts|py|rs|go|java|cpp|c|h)$/.test(f)
    );
    
    if (codeFiles.length > 0) {
      suggestions.push({
        command: 'find . -name "*.{ext}" -exec grep -l "pattern" {} \\;',
        description: 'Search for pattern in code files',
        confidence: 0.8,
        category: 'search',
        context: ['code-files-present'],
        params: [
          { name: 'ext', description: 'File extension', type: 'enum', required: true, 
            options: ['js', 'ts', 'py', 'rs', 'go'] },
          { name: 'pattern', description: 'Search pattern', type: 'string', required: true }
        ]
      });
    }
    
    // Package files detected
    if (context.directoryContents.includes('package.json')) {
      suggestions.push({
        command: 'npm run',
        description: 'Run npm script',
        confidence: 0.9,
        category: 'dev',
        context: ['nodejs-project'],
        learnedFrom: 'context'
      });
      
      suggestions.push({
        command: 'npm audit fix',
        description: 'Fix security vulnerabilities',
        confidence: 0.7,
        category: 'dev',
        context: ['nodejs-project']
      });
    }
    
    if (context.directoryContents.includes('Cargo.toml')) {
      suggestions.push({
        command: 'cargo build',
        description: 'Build Rust project',
        confidence: 0.9,
        category: 'dev',
        context: ['rust-project']
      });
      
      suggestions.push({
        command: 'cargo test',
        description: 'Run Rust tests',
        confidence: 0.8,
        category: 'dev',
        context: ['rust-project']
      });
    }
    
    return suggestions;
  }
  
  private getGitBasedSuggestions(context: ContextInfo): CommandSuggestion[] {
    const suggestions: CommandSuggestion[] = [];
    
    if (!context.gitRepository) return suggestions;
    
    const git = context.gitRepository;
    
    if (git.status === 'dirty') {
      if (git.modified.length > 0) {
        suggestions.push({
          command: 'git add .',
          description: 'Stage all modified files',
          confidence: 0.85,
          category: 'git',
          context: ['git-modified-files'],
          learnedFrom: 'context'
        });
        
        suggestions.push({
          command: 'git diff',
          description: 'View changes in modified files',
          confidence: 0.8,
          category: 'git',
          context: ['git-modified-files']
        });
      }
      
      if (git.staged.length > 0) {
        suggestions.push({
          command: 'git commit -m "message"',
          description: 'Commit staged changes',
          confidence: 0.9,
          category: 'git',
          context: ['git-staged-files'],
          params: [
            { name: 'message', description: 'Commit message', type: 'string', required: true }
          ]
        });
      }
      
      if (git.untracked.length > 0) {
        suggestions.push({
          command: 'git add',
          description: 'Add untracked files',
          confidence: 0.7,
          category: 'git',
          context: ['git-untracked-files']
        });
      }
    }
    
    if (git.remote) {
      suggestions.push({
        command: 'git push',
        description: 'Push changes to remote',
        confidence: 0.8,
        category: 'git',
        context: ['git-remote-configured']
      });
      
      suggestions.push({
        command: 'git pull',
        description: 'Pull latest changes',
        confidence: 0.7,
        category: 'git',
        context: ['git-remote-configured']
      });
    }
    
    return suggestions;
  }
  
  private getProjectBasedSuggestions(context: ContextInfo): CommandSuggestion[] {
    const suggestions: CommandSuggestion[] = [];
    
    switch (context.projectType) {
      case 'nodejs':
        suggestions.push(
          {
            command: 'npm start',
            description: 'Start development server',
            confidence: 0.8,
            category: 'dev',
            context: ['nodejs-project']
          },
          {
            command: 'npm test',
            description: 'Run tests',
            confidence: 0.7,
            category: 'dev',
            context: ['nodejs-project']
          },
          {
            command: 'npm run build',
            description: 'Build project',
            confidence: 0.7,
            category: 'dev',
            context: ['nodejs-project']
          }
        );
        break;
        
      case 'rust':
        suggestions.push(
          {
            command: 'cargo run',
            description: 'Run Rust application',
            confidence: 0.9,
            category: 'dev',
            context: ['rust-project']
          },
          {
            command: 'cargo check',
            description: 'Check code without building',
            confidence: 0.8,
            category: 'dev',
            context: ['rust-project']
          }
        );
        break;
        
      case 'python':
        suggestions.push(
          {
            command: 'python -m venv venv',
            description: 'Create virtual environment',
            confidence: 0.8,
            category: 'dev',
            context: ['python-project']
          },
          {
            command: 'pip install -r requirements.txt',
            description: 'Install dependencies',
            confidence: 0.9,
            category: 'dev',
            context: ['python-project']
          }
        );
        break;
    }
    
    return suggestions;
  }
  
  private getHistoryBasedSuggestions(context: ContextInfo): CommandSuggestion[] {
    const suggestions: CommandSuggestion[] = [];
    
    // Analyze recent command patterns
    const recentCommands = context.recentCommands.slice(-10);
    const commandFrequency = new Map<string, number>();
    
    recentCommands.forEach(cmd => {
      const baseCommand = cmd.split(' ')[0];
      commandFrequency.set(baseCommand, (commandFrequency.get(baseCommand) || 0) + 1);
    });
    
    // Suggest frequently used commands
    for (const [command, frequency] of commandFrequency.entries()) {
      if (frequency >= 2) {
        suggestions.push({
          command,
          description: `Frequently used: ${command}`,
          confidence: 0.6 + (frequency * 0.1),
          category: 'navigation',
          context: ['command-history'],
          learnedFrom: 'history'
        });
      }
    }
    
    // Pattern-based suggestions
    if (recentCommands.some(cmd => cmd.startsWith('docker'))) {
      suggestions.push({
        command: 'docker ps',
        description: 'List running containers',
        confidence: 0.8,
        category: 'docker',
        context: ['docker-usage-detected']
      });
    }
    
    return suggestions;
  }
  
  private getTimeBasedSuggestions(context: ContextInfo): CommandSuggestion[] {
    const suggestions: CommandSuggestion[] = [];
    
    if (context.timeOfDay === 'morning') {
      suggestions.push({
        command: 'git pull',
        description: 'Start day with latest changes',
        confidence: 0.6,
        category: 'git',
        context: ['morning-workflow']
      });
    }
    
    if (context.timeOfDay === 'evening') {
      suggestions.push({
        command: 'git status',
        description: 'Review end-of-day changes',
        confidence: 0.6,
        category: 'git',
        context: ['evening-workflow']
      });
    }
    
    return suggestions;
  }
  
  private getRuleBasedSuggestions(
    partialCommand: string,
    _context: ContextInfo,
    filter?: SuggestionFilter
  ): CommandSuggestion[] {
    const suggestions: CommandSuggestion[] = [];
    
    // Basic command completion
    const commonCommands = [
      { cmd: 'ls', desc: 'List directory contents', cat: 'navigation' as const },
      { cmd: 'cd', desc: 'Change directory', cat: 'navigation' as const },
      { cmd: 'pwd', desc: 'Print working directory', cat: 'navigation' as const },
      { cmd: 'grep', desc: 'Search text patterns', cat: 'search' as const },
      { cmd: 'find', desc: 'Find files and directories', cat: 'search' as const },
      { cmd: 'ps', desc: 'List running processes', cat: 'system' as const },
      { cmd: 'top', desc: 'Display running processes', cat: 'system' as const },
      { cmd: 'df', desc: 'Display filesystem usage', cat: 'system' as const },
      { cmd: 'du', desc: 'Display directory usage', cat: 'system' as const },
    ];
    
    for (const { cmd, desc, cat } of commonCommands) {
      if (cmd.startsWith(partialCommand)) {
        suggestions.push({
          command: cmd,
          description: desc,
          confidence: 0.7,
          category: cat,
          context: ['basic-commands'],
          learnedFrom: 'patterns'
        });
      }
    }
    
    return this.filterSuggestions(suggestions, filter);
  }
  
  private filterSuggestions(
    suggestions: CommandSuggestion[], 
    filter?: SuggestionFilter
  ): CommandSuggestion[] {
    let filtered = suggestions;
    
    if (filter) {
      if (filter.categories) {
        filtered = filtered.filter(s => filter.categories!.includes(s.category));
      }
      
      if (filter.minConfidence) {
        filtered = filtered.filter(s => s.confidence >= filter.minConfidence!);
      }
      
      if (filter.personalizedOnly) {
        filtered = filtered.filter(s => s.learnedFrom === 'history' || s.learnedFrom === 'patterns');
      }
      
      if (filter.maxResults) {
        filtered = filtered.slice(0, filter.maxResults);
      }
    }
    
    return filtered;
  }
  
  private generateCacheKey(
    partialCommand: string, 
    context: ContextInfo, 
    filter?: SuggestionFilter
  ): string {
    const contextKey = `${context.currentDirectory}_${context.projectType}_${context.timeOfDay}`;
    const filterKey = filter ? JSON.stringify(filter) : '';
    return `${partialCommand}_${contextKey}_${filterKey}`;
  }
  
  /**
   * Learn from executed commands
   */
  async learnFromCommand(command: string, successful: boolean, context?: ContextInfo): Promise<void> {
    const currentContext = context || await this.getCurrentContext();
    
    try {
      await invoke('learn_from_command', {
        command,
        successful,
        context: currentContext
      });
      
      // Update local learning patterns
      if (successful) {
        const pattern = this.extractCommandPattern(command);
        this.learningPatterns.set(pattern, (this.learningPatterns.get(pattern) || 0) + 1);
      }
      
    } catch (error) {
      console.warn('Failed to learn from command:', error);
    }
  }
  
  private extractCommandPattern(command: string): string {
    // Extract meaningful patterns from commands
    const parts = command.split(' ');
    const baseCommand = parts[0];
    const flags = parts.filter(p => p.startsWith('-')).join(' ');
    
    return `${baseCommand} ${flags}`.trim();
  }
  
  /**
   * Get suggestions for specific scenarios
   */
  async getSuggestionsForScenario(scenario: string): Promise<CommandSuggestion[]> {
    const scenarioMappings: Record<string, CommandSuggestion[]> = {
      'debugging': [
        {
          command: 'strace -p PID',
          description: 'Trace system calls of running process',
          confidence: 0.8,
          category: 'system',
          context: ['debugging'],
          params: [{ name: 'PID', description: 'Process ID', type: 'number', required: true }]
        },
        {
          command: 'gdb ./program',
          description: 'Debug with GDB debugger',
          confidence: 0.7,
          category: 'dev',
          context: ['debugging']
        }
      ],
      
      'performance-analysis': [
        {
          command: 'perf record -g ./program',
          description: 'Profile application performance',
          confidence: 0.8,
          category: 'system',
          context: ['performance']
        },
        {
          command: 'valgrind --tool=memcheck ./program',
          description: 'Check for memory leaks',
          confidence: 0.8,
          category: 'dev',
          context: ['performance']
        }
      ],
      
      'file-cleanup': [
        {
          command: 'find . -name "*.log" -mtime +7 -delete',
          description: 'Delete log files older than 7 days',
          confidence: 0.9,
          category: 'file-ops',
          context: ['cleanup']
        },
        {
          command: 'find . -type f -size +100M',
          description: 'Find large files',
          confidence: 0.8,
          category: 'file-ops',
          context: ['cleanup']
        }
      ]
    };
    
    return scenarioMappings[scenario] || [];
  }
  
  /**
   * Get smart command completions
   */
  async getSmartCompletions(partialCommand: string): Promise<CommandSuggestion[]> {
    const context = await this.getCurrentContext();
    
    // Handle different completion scenarios
    if (partialCommand.includes('cd ')) {
      return this.getDirectoryCompletions(partialCommand, context);
    }
    
    if (partialCommand.includes('git ')) {
      return this.getGitCompletions(partialCommand, context);
    }
    
    if (partialCommand.includes('npm ')) {
      return this.getNpmCompletions(partialCommand, context);
    }
    
    return this.getSuggestions(partialCommand);
  }
  
  private getDirectoryCompletions(partialCommand: string, context: ContextInfo): CommandSuggestion[] {
    const pathPart = partialCommand.replace(/.*cd\s+/, '');
    const suggestions: CommandSuggestion[] = [];
    
    // Get directory suggestions from current context
    const directories = context.directoryContents.filter(item => 
      item.endsWith('/') || !item.includes('.')
    );
    
    for (const dir of directories) {
      if (dir.startsWith(pathPart)) {
        suggestions.push({
          command: `cd ${dir}`,
          description: `Navigate to ${dir}`,
          confidence: 0.9,
          category: 'navigation',
          context: ['directory-completion']
        });
      }
    }
    
    return suggestions;
  }
  
  private getGitCompletions(partialCommand: string, _context: ContextInfo): CommandSuggestion[] {
    const suggestions: CommandSuggestion[] = [];
    
    if (partialCommand.includes('git checkout ')) {
      // Suggest branches (would need backend support for actual branch list)
      suggestions.push({
        command: 'git checkout -b feature/new-feature',
        description: 'Create and checkout new branch',
        confidence: 0.8,
        category: 'git',
        context: ['git-checkout']
      });
    }
    
    return suggestions;
  }
  
  private getNpmCompletions(partialCommand: string, _context: ContextInfo): CommandSuggestion[] {
    const suggestions: CommandSuggestion[] = [];
    
    if (partialCommand.includes('npm run ')) {
      // Would need to read package.json for actual scripts
      suggestions.push({
        command: 'npm run dev',
        description: 'Run development script',
        confidence: 0.8,
        category: 'dev',
        context: ['npm-scripts']
      });
    }
    
    return suggestions;
  }
  
  /**
   * Clear suggestion cache
   */
  clearCache(): void {
    this.suggestionCache.clear();
  }
  
  /**
   * Get suggestion analytics
   */
  getSuggestionAnalytics(): SuggestionAnalytics {
    return {
      cacheSize: this.suggestionCache.size,
      totalPatterns: this.learningPatterns.size,
      contextHistorySize: this.contextHistory.length,
      mostLearnedPatterns: Array.from(this.learningPatterns.entries())
        .sort(([,a], [,b]) => b - a)
        .slice(0, 10)
        .map(([pattern, count]) => ({ pattern, count }))
    };
  }
}

export interface SuggestionAnalytics {
  cacheSize: number;
  totalPatterns: number;
  contextHistorySize: number;
  mostLearnedPatterns: Array<{ pattern: string; count: number }>;
}

export const contextualSuggestionsService = new ContextualSuggestionsService();
