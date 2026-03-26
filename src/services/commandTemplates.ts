import { invoke } from '@tauri-apps/api/core';

export interface CommandTemplate {
  id: string;
  name: string;
  description: string;
  template: string;
  parameters: TemplateParameter[];
  category: 'git' | 'docker' | 'file-ops' | 'dev' | 'system' | 'network' | 'custom';
  tags: string[];
  usage: {
    count: number;
    lastUsed: Date;
    averageExecutionTime: number;
  };
  author?: string;
  version: string;
  isPublic: boolean;
  isFavorite: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface TemplateParameter {
  name: string;
  displayName: string;
  description: string;
  type: 'string' | 'number' | 'boolean' | 'path' | 'url' | 'enum' | 'multi-enum';
  required: boolean;
  defaultValue?: string;
  options?: string[];
  validation?: {
    pattern?: string;
    min?: number;
    max?: number;
    minLength?: number;
    maxLength?: number;
  };
  placeholder?: string;
  helpText?: string;
  dependsOn?: string; // Name of another parameter this depends on
  showIf?: string; // Condition for when to show this parameter
}

export interface TemplateExecution {
  id: string;
  templateId: string;
  command: string;
  parameters: Record<string, string>;
  executionTime: number;
  exitCode: number;
  output: string;
  error?: string;
  timestamp: Date;
  context: {
    workingDirectory: string;
    environment: Record<string, string>;
  };
}

export interface TemplateCollection {
  id: string;
  name: string;
  description: string;
  templateIds: string[];
  tags: string[];
  isPublic: boolean;
  author?: string;
  createdAt: Date;
}

export class CommandTemplateService {
  private templates = new Map<string, CommandTemplate>();
  private collections = new Map<string, TemplateCollection>();
  private executionHistory: TemplateExecution[] = [];
  private parameterCache = new Map<string, Record<string, string>>();
  
  /**
   * Create a new template
   */
  createTemplate(
    name: string,
    description: string,
    template: string,
    parameters: TemplateParameter[],
    options: {
      category?: CommandTemplate['category'];
      tags?: string[];
      author?: string;
      isPublic?: boolean;
    } = {}
  ): string {
    const templateId = this.generateTemplateId();
    
    const commandTemplate: CommandTemplate = {
      id: templateId,
      name,
      description,
      template,
      parameters,
      category: options.category || 'custom',
      tags: options.tags || [],
      usage: {
        count: 0,
        lastUsed: new Date(),
        averageExecutionTime: 0
      },
      author: options.author,
      version: '1.0.0',
      isPublic: options.isPublic || false,
      isFavorite: false,
      createdAt: new Date(),
      updatedAt: new Date()
    };
    
    this.templates.set(templateId, commandTemplate);
    return templateId;
  }
  
  /**
   * Update existing template
   */
  updateTemplate(templateId: string, updates: Partial<CommandTemplate>): void {
    const template = this.templates.get(templateId);
    if (!template) throw new Error('Template not found');
    
    Object.assign(template, updates);
    template.updatedAt = new Date();
    
    // Increment version for template changes
    if (updates.template || updates.parameters) {
      const [major, minor, patch] = template.version.split('.').map(Number);
      template.version = `${major}.${minor}.${patch + 1}`;
    }
    
    this.templates.set(templateId, template);
  }
  
  /**
   * Delete template
   */
  deleteTemplate(templateId: string): void {
    this.templates.delete(templateId);
    
    // Remove from collections
    for (const collection of this.collections.values()) {
      collection.templateIds = collection.templateIds.filter(id => id !== templateId);
    }
  }
  
  /**
   * Get template by ID
   */
  getTemplate(templateId: string): CommandTemplate | null {
    return this.templates.get(templateId) || null;
  }
  
  /**
   * Search templates
   */
  searchTemplates(query: string, options: {
    category?: string;
    tags?: string[];
    includePublic?: boolean;
    sortBy?: 'name' | 'usage' | 'created' | 'updated';
  } = {}): CommandTemplate[] {
    const templates = Array.from(this.templates.values());
    
    let filtered = templates.filter(template => {
      // Text search
      const searchText = `${template.name} ${template.description} ${template.tags.join(' ')}`.toLowerCase();
      const queryMatch = query.toLowerCase().split(' ').every(term => searchText.includes(term));
      
      if (!queryMatch) return false;
      
      // Category filter
      if (options.category && template.category !== options.category) return false;
      
      // Tags filter
      if (options.tags && !options.tags.some(tag => template.tags.includes(tag))) return false;
      
      // Public filter
      if (options.includePublic === false && template.isPublic) return false;
      
      return true;
    });
    
    // Sort results
    if (options.sortBy) {
      filtered.sort((a, b) => {
        switch (options.sortBy) {
          case 'name': return a.name.localeCompare(b.name);
          case 'usage': return b.usage.count - a.usage.count;
          case 'created': return b.createdAt.getTime() - a.createdAt.getTime();
          case 'updated': return b.updatedAt.getTime() - a.updatedAt.getTime();
          default: return 0;
        }
      });
    }
    
    return filtered;
  }
  
  /**
   * Execute template with parameters
   */
  async executeTemplate(
    templateId: string, 
    parameters: Record<string, string>,
    options: {
      dryRun?: boolean;
      saveExecution?: boolean;
      workingDirectory?: string;
    } = {}
  ): Promise<TemplateExecutionResult> {
    const template = this.templates.get(templateId);
    if (!template) throw new Error('Template not found');
    
    // Validate parameters
    const validation = this.validateParameters(template.parameters, parameters);
    if (!validation.valid) {
      throw new Error(`Parameter validation failed: ${validation.errors.join(', ')}`);
    }
    
    // Build command from template
    const command = this.buildCommand(template.template, parameters);
    
    if (options.dryRun) {
      return {
        command,
        parameters,
        dryRun: true
      };
    }
    
    const startTime = Date.now();
    
    try {
      // Execute command
      const result = await invoke<{output: string; exitCode: number}>('execute_template_command', {
        command,
        workingDirectory: options.workingDirectory
      });
      
      const executionTime = Date.now() - startTime;
      
      // Update template usage
      template.usage.count++;
      template.usage.lastUsed = new Date();
      template.usage.averageExecutionTime = 
        (template.usage.averageExecutionTime + executionTime) / 2;
      
      // Save execution history
      if (options.saveExecution !== false) {
        const execution: TemplateExecution = {
          id: this.generateExecutionId(),
          templateId,
          command,
          parameters,
          executionTime,
          exitCode: result.exitCode,
          output: result.output,
          timestamp: new Date(),
          context: {
            workingDirectory: options.workingDirectory || process.cwd(),
            environment: process.env as Record<string, string>
          }
        };
        
        this.executionHistory.push(execution);
        if (this.executionHistory.length > 1000) {
          this.executionHistory.shift();
        }
      }
      
      // Cache parameters for future use
      this.parameterCache.set(templateId, parameters);
      
      return {
        command,
        parameters,
        output: result.output,
        exitCode: result.exitCode,
        executionTime,
        dryRun: false
      };
    } catch (error) {
      throw new Error(`Template execution failed: ${error}`);
    }
  }
  
  /**
   * Get parameter suggestions based on history
   */
  getParameterSuggestions(templateId: string): Record<string, string[]> {
    const suggestions: Record<string, string[]> = {};
    
    // Get suggestions from execution history
    const executions = this.executionHistory
      .filter(e => e.templateId === templateId)
      .slice(-50); // Recent 50 executions
    
    for (const execution of executions) {
      for (const [param, value] of Object.entries(execution.parameters)) {
        if (!suggestions[param]) suggestions[param] = [];
        if (!suggestions[param].includes(value)) {
          suggestions[param].push(value);
        }
      }
    }
    
    // Limit suggestions per parameter
    for (const param in suggestions) {
      suggestions[param] = suggestions[param].slice(0, 10);
    }
    
    return suggestions;
  }
  
  private validateParameters(
    templateParams: TemplateParameter[], 
    providedParams: Record<string, string>
  ): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    for (const param of templateParams) {
      const value = providedParams[param.name];
      
      // Check required parameters
      if (param.required && !value) {
        errors.push(`Required parameter '${param.name}' is missing`);
        continue;
      }
      
      if (!value) continue;
      
      // Type validation
      if (param.type === 'number' && isNaN(Number(value))) {
        errors.push(`Parameter '${param.name}' must be a number`);
      }
      
      if (param.type === 'boolean' && !['true', 'false', '1', '0'].includes(value.toLowerCase())) {
        errors.push(`Parameter '${param.name}' must be a boolean`);
      }
      
      if (param.type === 'url') {
        try {
          new URL(value);
        } catch {
          errors.push(`Parameter '${param.name}' must be a valid URL`);
        }
      }
      
      if (param.type === 'enum' && param.options && !param.options.includes(value)) {
        errors.push(`Parameter '${param.name}' must be one of: ${param.options.join(', ')}`);
      }
      
      // Validation rules
      if (param.validation) {
        const val = param.validation;
        
        if (val.pattern) {
          try {
            const regex = new RegExp(val.pattern);
            if (!regex.test(value)) {
              errors.push(`Parameter '${param.name}' doesn't match required pattern`);
            }
          } catch {
            errors.push(`Invalid pattern for parameter '${param.name}'`);
          }
        }
        
        if (val.minLength && value.length < val.minLength) {
          errors.push(`Parameter '${param.name}' must be at least ${val.minLength} characters`);
        }
        
        if (val.maxLength && value.length > val.maxLength) {
          errors.push(`Parameter '${param.name}' must be at most ${val.maxLength} characters`);
        }
        
        if (param.type === 'number') {
          const numValue = Number(value);
          if (val.min !== undefined && numValue < val.min) {
            errors.push(`Parameter '${param.name}' must be at least ${val.min}`);
          }
          if (val.max !== undefined && numValue > val.max) {
            errors.push(`Parameter '${param.name}' must be at most ${val.max}`);
          }
        }
      }
    }
    
    return { valid: errors.length === 0, errors };
  }
  
  private buildCommand(template: string, parameters: Record<string, string>): string {
    let command = template;
    
    // Replace parameter placeholders
    for (const [name, value] of Object.entries(parameters)) {
      const placeholderRegex = new RegExp(`\\{\\{${name}\\}\\}`, 'g');
      command = command.replace(placeholderRegex, value);
    }
    
    // Check for unreplaced placeholders
    const unreplacedMatches = command.match(/\{\{[^}]+\}\}/g);
    if (unreplacedMatches) {
      console.warn('Unreplaced placeholders found:', unreplacedMatches);
    }
    
    return command;
  }
  
  /**
   * Create template collection
   */
  createCollection(name: string, description: string, templateIds: string[] = []): string {
    const collectionId = this.generateCollectionId();
    
    const collection: TemplateCollection = {
      id: collectionId,
      name,
      description,
      templateIds,
      tags: [],
      isPublic: false,
      createdAt: new Date()
    };
    
    this.collections.set(collectionId, collection);
    return collectionId;
  }
  
  /**
   * Get predefined templates
   */
  createPredefinedTemplates(): void {
    // Git templates
    this.createTemplate(
      'Git Commit with Message',
      'Stage all changes and commit with message',
      'git add . && git commit -m "{{message}}"',
      [
        {
          name: 'message',
          displayName: 'Commit Message',
          description: 'Description of the changes',
          type: 'string',
          required: true,
          placeholder: 'Brief description of changes',
          validation: { minLength: 3, maxLength: 72 }
        }
      ],
      { category: 'git', tags: ['git', 'commit'] }
    );
    
    this.createTemplate(
      'Create Feature Branch',
      'Create and checkout a new feature branch',
      'git checkout -b feature/{{feature-name}}',
      [
        {
          name: 'feature-name',
          displayName: 'Feature Name',
          description: 'Name of the feature (kebab-case)',
          type: 'string',
          required: true,
          placeholder: 'user-authentication',
          validation: { pattern: '^[a-z0-9-]+$' }
        }
      ],
      { category: 'git', tags: ['git', 'branch'] }
    );
    
    // Docker templates
    this.createTemplate(
      'Run Docker Container',
      'Run a Docker container with port mapping',
      'docker run {{detached}} {{port-mapping}} {{image}} {{command}}',
      [
        {
          name: 'image',
          displayName: 'Docker Image',
          description: 'Docker image name',
          type: 'string',
          required: true,
          placeholder: 'nginx:latest'
        },
        {
          name: 'port-mapping',
          displayName: 'Port Mapping',
          description: 'Port mapping (host:container)',
          type: 'string',
          required: false,
          placeholder: '-p 8080:80',
          defaultValue: ''
        },
        {
          name: 'detached',
          displayName: 'Run Detached',
          description: 'Run container in background',
          type: 'boolean',
          required: false,
          defaultValue: 'true'
        },
        {
          name: 'command',
          displayName: 'Command',
          description: 'Command to run in container',
          type: 'string',
          required: false,
          defaultValue: ''
        }
      ],
      { category: 'docker', tags: ['docker', 'container'] }
    );
    
    // File operations
    this.createTemplate(
      'Find and Replace in Files',
      'Find and replace text in files using sed',
      'find {{directory}} -name "{{file-pattern}}" -type f -exec sed -i "s/{{search}}/{{replace}}/g" {} +',
      [
        {
          name: 'directory',
          displayName: 'Directory',
          description: 'Directory to search in',
          type: 'path',
          required: true,
          defaultValue: '.'
        },
        {
          name: 'file-pattern',
          displayName: 'File Pattern',
          description: 'File name pattern to match',
          type: 'string',
          required: true,
          placeholder: '*.txt'
        },
        {
          name: 'search',
          displayName: 'Search Text',
          description: 'Text to find',
          type: 'string',
          required: true
        },
        {
          name: 'replace',
          displayName: 'Replace Text',
          description: 'Replacement text',
          type: 'string',
          required: true
        }
      ],
      { category: 'file-ops', tags: ['find', 'replace', 'sed'] }
    );
    
    // System administration
    this.createTemplate(
      'Create System User',
      'Create a new system user with home directory',
      'sudo useradd {{options}} {{username}} && sudo passwd {{username}}',
      [
        {
          name: 'username',
          displayName: 'Username',
          description: 'Name for the new user',
          type: 'string',
          required: true,
          validation: { pattern: '^[a-z][a-z0-9_-]*$', minLength: 2, maxLength: 32 }
        },
        {
          name: 'options',
          displayName: 'User Options',
          description: 'Additional useradd options',
          type: 'enum',
          required: false,
          options: ['-m', '-m -s /bin/bash', '-m -g users', '-m -s /bin/bash -g users'],
          defaultValue: '-m -s /bin/bash'
        }
      ],
      { category: 'system', tags: ['user', 'admin'] }
    );
    
    // Development templates
    this.createTemplate(
      'NPM Project Setup',
      'Initialize new NPM project with common configuration',
      'npm init -y && npm install {{dependencies}} && npm install --save-dev {{dev-dependencies}}',
      [
        {
          name: 'dependencies',
          displayName: 'Dependencies',
          description: 'Production dependencies to install',
          type: 'string',
          required: false,
          placeholder: 'express react lodash',
          defaultValue: ''
        },
        {
          name: 'dev-dependencies',
          displayName: 'Dev Dependencies',
          description: 'Development dependencies to install',
          type: 'string',
          required: false,
          placeholder: 'typescript @types/node jest',
          defaultValue: ''
        }
      ],
      { category: 'dev', tags: ['npm', 'nodejs', 'setup'] }
    );
    
    // Network templates
    this.createTemplate(
      'Download and Extract Archive',
      'Download archive file and extract to directory',
      'curl -L "{{url}}" -o "{{filename}}" && {{extract-command}} "{{filename}}" {{extract-options}}',
      [
        {
          name: 'url',
          displayName: 'Download URL',
          description: 'URL of the archive to download',
          type: 'url',
          required: true
        },
        {
          name: 'filename',
          displayName: 'Local Filename',
          description: 'Name for the downloaded file',
          type: 'string',
          required: true,
          placeholder: 'archive.tar.gz'
        },
        {
          name: 'extract-command',
          displayName: 'Extract Command',
          description: 'Command to extract the archive',
          type: 'enum',
          required: true,
          options: ['tar -xzf', 'tar -xjf', 'unzip', 'tar -xf'],
          defaultValue: 'tar -xzf'
        },
        {
          name: 'extract-options',
          displayName: 'Extract Options',
          description: 'Additional extraction options',
          type: 'string',
          required: false,
          defaultValue: '-C .'
        }
      ],
      { category: 'network', tags: ['download', 'extract', 'curl'] }
    );
  }
  
  /**
   * Import templates from file
   */
  async importTemplates(filePath: string): Promise<number> {
    try {
      const templates = await invoke<CommandTemplate[]>('import_templates', { filePath });
      
      for (const template of templates) {
        this.templates.set(template.id, template);
      }
      
      return templates.length;
    } catch (error) {
      throw new Error(`Failed to import templates: ${error}`);
    }
  }
  
  /**
   * Export templates to file
   */
  async exportTemplates(filePath: string, templateIds?: string[]): Promise<void> {
    const templatesToExport = templateIds ?
      templateIds.map(id => this.templates.get(id)).filter(Boolean) :
      Array.from(this.templates.values());
    
    try {
      await invoke('export_templates', { templates: templatesToExport, filePath });
    } catch (error) {
      throw new Error(`Failed to export templates: ${error}`);
    }
  }
  
  /**
   * Get cached parameters for template
   */
  getCachedParameters(templateId: string): Record<string, string> {
    return this.parameterCache.get(templateId) || {};
  }
  
  /**
   * Get execution history
   */
  getExecutionHistory(templateId?: string, limit: number = 100): TemplateExecution[] {
    let history = this.executionHistory;
    
    if (templateId) {
      history = history.filter(e => e.templateId === templateId);
    }
    
    return history.slice(-limit);
  }
  
  /**
   * Get template analytics
   */
  getTemplateAnalytics(): TemplateAnalytics {
    const templates = Array.from(this.templates.values());
    
    return {
      totalTemplates: templates.length,
      totalExecutions: this.executionHistory.length,
      mostUsedTemplates: templates
        .sort((a, b) => b.usage.count - a.usage.count)
        .slice(0, 10)
        .map(t => ({ id: t.id, name: t.name, usageCount: t.usage.count })),
      categoryDistribution: this.getCategoryDistribution(),
      averageParametersPerTemplate: templates.length > 0 ?
        templates.reduce((sum, t) => sum + t.parameters.length, 0) / templates.length : 0,
      recentActivity: this.executionHistory.slice(-10).map(e => ({
        templateName: this.templates.get(e.templateId)?.name || 'Unknown',
        timestamp: e.timestamp,
        successful: e.exitCode === 0
      }))
    };
  }
  
  private getCategoryDistribution(): Record<string, number> {
    const distribution: Record<string, number> = {};
    
    for (const template of this.templates.values()) {
      distribution[template.category] = (distribution[template.category] || 0) + 1;
    }
    
    return distribution;
  }
  
  private generateTemplateId(): string {
    return `template_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  private generateCollectionId(): string {
    return `collection_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  private generateExecutionId(): string {
    return `exec_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

export interface TemplateExecutionResult {
  command: string;
  parameters: Record<string, string>;
  output?: string;
  exitCode?: number;
  executionTime?: number;
  dryRun: boolean;
}

export interface TemplateAnalytics {
  totalTemplates: number;
  totalExecutions: number;
  mostUsedTemplates: Array<{ id: string; name: string; usageCount: number }>;
  categoryDistribution: Record<string, number>;
  averageParametersPerTemplate: number;
  recentActivity: Array<{ templateName: string; timestamp: Date; successful: boolean }>;
}

export const commandTemplateService = new CommandTemplateService();
