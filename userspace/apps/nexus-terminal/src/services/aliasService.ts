import { invoke } from '@tauri-apps/api/core';
// import { v4 as uuid } from 'uuid';
const uuid = () => Math.random().toString(36).substring(2) + Date.now().toString(36);

export interface SmartAlias {
  id: string;
  trigger: string;
  expansion: string;
  context?: string; // only work in specific directories
  parameters?: string[]; // support variables like $1, $2
  frequency: number; // learn from usage
  lastUsed: Date;
  createdDate: Date;
  description?: string;
}

export interface AliasStats {
  totalAliases: number;
  mostUsedAlias: SmartAlias | null;
  timesSaved: number; // characters saved by using aliases
  suggestionCount: number;
}

export class AliasService {
  private aliases: SmartAlias[] = [];
  private commandFrequency: Map<string, number> = new Map();
  private contextPatterns: Map<string, string[]> = new Map(); // directory -> common commands
  
  constructor() {
    this.loadAliases();
    this.loadCommandFrequency();
  }

  /**
   * Learn from command usage and suggest aliases
   */
  async learnFromCommand(command: string, context: string): Promise<SmartAlias | null> {
    // Update frequency tracking
    const currentFreq = this.commandFrequency.get(command) || 0;
    this.commandFrequency.set(command, currentFreq + 1);
    
    // Track context patterns
    if (!this.contextPatterns.has(context)) {
      this.contextPatterns.set(context, []);
    }
    const contextCommands = this.contextPatterns.get(context)!;
    if (!contextCommands.includes(command)) {
      contextCommands.push(command);
    }
    
    // Suggest alias if command is frequent enough
    if (currentFreq >= 5 && !this.hasAliasForCommand(command)) {
      return this.suggestAlias(command, context);
    }
    
    return null;
  }

  /**
   * Suggest an alias for a frequently used command
   */
  private async suggestAlias(command: string, context: string): Promise<SmartAlias> {
    const trigger = this.generateShortcut(command);
    const alias: SmartAlias = {
      id: uuid(),
      trigger,
      expansion: command,
      context: this.isDirectorySpecific(command, context) ? context : undefined,
      parameters: this.extractParameters(command),
      frequency: this.commandFrequency.get(command) || 0,
      lastUsed: new Date(),
      createdDate: new Date(),
      description: `Auto-generated alias for "${command}"`
    };
    
    return alias;
  }

  /**
   * Generate a short trigger for a command
   */
  private generateShortcut(command: string): string {
    const parts = command.split(' ');
    const baseCommand = parts[0];
    
    // Common shortcuts
    const shortcuts: Record<string, string> = {
      'git status': 'gs',
      'git add': 'ga',
      'git commit': 'gc',
      'git push': 'gp',
      'git pull': 'gl',
      'git checkout': 'gco',
      'npm run': 'nr',
      'npm install': 'ni',
      'npm test': 'nt',
      'docker ps': 'dps',
      'docker images': 'di',
      'docker run': 'dr',
      'kubectl get': 'kg',
      'kubectl describe': 'kd',
      'cd ..': '..',
      'cd ../..': '...'
    };
    
    // Check for exact matches first
    if (shortcuts[command]) {
      return shortcuts[command];
    }
    
    // Generate based on command structure
    if (parts.length === 1) {
      return baseCommand.substring(0, 2);
    } else if (parts.length === 2) {
      return baseCommand.charAt(0) + parts[1].charAt(0);
    } else {
      return baseCommand.charAt(0) + parts[1].charAt(0) + parts[2].charAt(0);
    }
  }

  /**
   * Check if command is directory-specific
   */
  private isDirectorySpecific(command: string, _context: string): boolean {
    const directorySpecificPatterns = [
      /^npm /,
      /^yarn /,
      /^cargo /,
      /^make /,
      /^gradle /,
      /^mvn /,
    ];
    
    return directorySpecificPatterns.some(pattern => pattern.test(command));
  }

  /**
   * Extract parameters from command template
   */
  private extractParameters(command: string): string[] {
    const parts = command.split(' ');
    const parameters: string[] = [];
    
    // Look for common parameter patterns
    for (let i = 1; i < parts.length; i++) {
      const part = parts[i];
      if (part.startsWith('-') || part.includes('=')) {
        // This might be a parameter we can templatize
        if (part.includes('=')) {
          const [key] = part.split('=');
          parameters.push(`${key}=$${parameters.length + 1}`);
        } else if (i + 1 < parts.length && !parts[i + 1].startsWith('-')) {
          parameters.push(`${part} $${parameters.length + 1}`);
          i++; // Skip the next part as it's the value
        }
      }
    }
    
    return parameters;
  }

  /**
   * Expand alias with parameters
   */
  expandAlias(input: string, context: string): string {
    const parts = input.split(' ');
    const trigger = parts[0];
    const args = parts.slice(1);
    
    // Find matching alias
    const alias = this.findMatchingAlias(trigger, context);
    if (!alias) return input;
    
    // Update usage stats
    alias.frequency++;
    alias.lastUsed = new Date();
    
    let expanded = alias.expansion;
    
    // Replace parameters if any
    if (alias.parameters && alias.parameters.length > 0) {
      alias.parameters.forEach((_param, index) => {
        const value = args[index] || '';
        expanded = expanded.replace(`$${index + 1}`, value);
      });
    } else if (args.length > 0) {
      // Simple append remaining arguments
      expanded += ' ' + args.join(' ');
    }
    
    return expanded;
  }

  /**
   * Find matching alias considering context
   */
  private findMatchingAlias(trigger: string, context: string): SmartAlias | null {
    // First try to find context-specific alias
    const contextSpecific = this.aliases.find(
      alias => alias.trigger === trigger && alias.context === context
    );
    
    if (contextSpecific) return contextSpecific;
    
    // Fall back to global alias
    return this.aliases.find(
      alias => alias.trigger === trigger && !alias.context
    ) || null;
  }

  /**
   * Check if command already has an alias
   */
  private hasAliasForCommand(command: string): boolean {
    return this.aliases.some(alias => alias.expansion === command);
  }

  /**
   * Add a new alias
   */
  async addAlias(alias: SmartAlias): Promise<void> {
    this.aliases.push(alias);
    await this.saveAliases();
  }

  /**
   * Remove an alias
   */
  async removeAlias(aliasId: string): Promise<boolean> {
    const index = this.aliases.findIndex(alias => alias.id === aliasId);
    if (index === -1) return false;
    
    this.aliases.splice(index, 1);
    await this.saveAliases();
    return true;
  }

  /**
   * Get all aliases
   */
  getAllAliases(): SmartAlias[] {
    return [...this.aliases].sort((a, b) => b.frequency - a.frequency);
  }

  /**
   * Get aliases for specific context
   */
  getAliasesForContext(context: string): SmartAlias[] {
    return this.aliases.filter(alias => 
      alias.context === context || !alias.context
    ).sort((a, b) => b.frequency - a.frequency);
  }

  /**
   * Get alias statistics
   */
  getStats(): AliasStats {
    const mostUsed = this.aliases.reduce((max, alias) => 
      alias.frequency > (max?.frequency || 0) ? alias : max
    , null as SmartAlias | null);
    
    const timesSaved = this.aliases.reduce((total, alias) => 
      total + (alias.expansion.length - alias.trigger.length) * alias.frequency
    , 0);
    
    return {
      totalAliases: this.aliases.length,
      mostUsedAlias: mostUsed,
      timesSaved,
      suggestionCount: Array.from(this.commandFrequency.values())
        .filter(freq => freq >= 5).length
    };
  }

  /**
   * Get command suggestions for auto-completion
   */
  getCommandSuggestions(partial: string, context: string, limit: number = 5): string[] {
    const suggestions: string[] = [];
    
    // Get aliases that match partial
    const matchingAliases = this.getAliasesForContext(context)
      .filter(alias => alias.trigger.startsWith(partial))
      .slice(0, limit);
    
    suggestions.push(...matchingAliases.map(alias => 
      `${alias.trigger} → ${alias.expansion}`
    ));
    
    // Get frequent commands that match partial
    const frequentCommands = Array.from(this.commandFrequency.entries())
      .filter(([command]) => command.startsWith(partial))
      .sort((a, b) => b[1] - a[1])
      .slice(0, limit - suggestions.length);
    
    suggestions.push(...frequentCommands.map(([command]) => command));
    
    return suggestions;
  }

  /**
   * Import aliases from file
   */
  async importAliases(aliasData: SmartAlias[]): Promise<number> {
    let importCount = 0;
    
    for (const alias of aliasData) {
      if (!this.aliases.some(existing => existing.trigger === alias.trigger)) {
        this.aliases.push({
          ...alias,
          id: alias.id || uuid(),
          createdDate: new Date(alias.createdDate),
          lastUsed: new Date(alias.lastUsed)
        });
        importCount++;
      }
    }
    
    await this.saveAliases();
    return importCount;
  }

  /**
   * Export aliases to JSON
   */
  exportAliases(): string {
    return JSON.stringify(this.aliases, null, 2);
  }

  /**
   * Reset all aliases and learning data
   */
  async reset(): Promise<void> {
    this.aliases = [];
    this.commandFrequency.clear();
    this.contextPatterns.clear();
    
    await this.saveAliases();
    await this.saveCommandFrequency();
  }

  /**
   * Load aliases from storage
   */
  private async loadAliases(): Promise<void> {
    try {
      const stored = await invoke<SmartAlias[]>('load_aliases');
      this.aliases = stored.map(alias => ({
        ...alias,
        createdDate: new Date(alias.createdDate),
        lastUsed: new Date(alias.lastUsed)
      }));
    } catch (error) {
      console.warn('Failed to load aliases:', error);
      this.aliases = [];
    }
  }

  /**
   * Save aliases to storage
   */
  private async saveAliases(): Promise<void> {
    try {
      await invoke('save_aliases', { aliases: this.aliases });
    } catch (error) {
      console.error('Failed to save aliases:', error);
    }
  }

  /**
   * Load command frequency data
   */
  private async loadCommandFrequency(): Promise<void> {
    try {
      const stored = await invoke<Record<string, number>>('load_command_frequency');
      this.commandFrequency = new Map(Object.entries(stored));
    } catch (error) {
      console.warn('Failed to load command frequency:', error);
      this.commandFrequency = new Map();
    }
  }

  /**
   * Save command frequency data
   */
  private async saveCommandFrequency(): Promise<void> {
    try {
      const data = Object.fromEntries(this.commandFrequency);
      await invoke('save_command_frequency', { frequency: data });
    } catch (error) {
      console.error('Failed to save command frequency:', error);
    }
  }
}

// Global instance
export const aliasService = new AliasService();
