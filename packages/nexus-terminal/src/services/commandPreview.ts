import { invoke } from '@tauri-apps/api/core';

export interface CommandPreview {
  command: string;
  willCreate: string[];
  willModify: string[];
  willDelete: string[];
  riskLevel: 'safe' | 'moderate' | 'dangerous';
  estimatedTime?: string;
  warnings: string[];
  affectedFiles: number;
  totalSize?: string;
}

export interface FileOperation {
  path: string;
  type: 'create' | 'modify' | 'delete';
  size?: number;
  isDirectory: boolean;
  permissions?: string;
}

export class CommandPreviewService {
  private dangerousPatterns = [
    /^rm\s+.*-r/,                    // rm -r, rm -rf
    /^sudo\s+rm/,                    // sudo rm
    /^dd\s+/,                        // dd command
    /^mkfs/,                         // format filesystem  
    /^fdisk/,                        // disk partitioning
    /^format/,                       // format command
    /^del\s+.*\/s/i,                // Windows del /s
    /^rmdir\s+.*\/s/i,              // Windows rmdir /s
    /^>.*\.(sh|py|js|ts|rs|go)$/,   // Overwriting code files
  ];

  private moderatePatterns = [
    /^mv\s+/,                        // move files
    /^cp\s+.*-r/,                   // recursive copy
    /^chmod\s+/,                     // change permissions
    /^chown\s+/,                     // change ownership
    /^git\s+reset\s+--hard/,        // git reset --hard
    /^git\s+clean\s+-f/,            // git clean -f
    /^npm\s+install\s+.*--save/,    // npm install with save
  ];

  async previewCommand(command: string, cwd: string): Promise<CommandPreview> {
    const preview: CommandPreview = {
      command,
      willCreate: [],
      willModify: [],
      willDelete: [],
      riskLevel: 'safe',
      warnings: [],
      affectedFiles: 0
    };

    try {
      // Get detailed preview from backend
      const backendPreview = await invoke<CommandPreview>('preview_command', { 
        command, 
        cwd 
      });
      
      Object.assign(preview, backendPreview);
    } catch (error) {
      // Fallback to frontend analysis
      this.analyzeCommandLocally(command, cwd, preview);
    }

    // Add risk level based on patterns
    this.assessRiskLevel(command, preview);
    
    return preview;
  }

  private analyzeCommandLocally(command: string, cwd: string, preview: CommandPreview): void {
    const parts = command.trim().split(/\s+/);
    const baseCommand = parts[0];

    switch (baseCommand) {
      case 'rm':
        this.analyzeRmCommand(parts, cwd, preview);
        break;
      case 'mv':
        this.analyzeMvCommand(parts, cwd, preview);
        break;
      case 'cp':
        this.analyzeCpCommand(parts, cwd, preview);
        break;
      case 'mkdir':
        this.analyzeMkdirCommand(parts, cwd, preview);
        break;
      case 'touch':
        this.analyzeTouchCommand(parts, cwd, preview);
        break;
      case 'git':
        this.analyzeGitCommand(parts, cwd, preview);
        break;
      case 'npm':
      case 'yarn':
        this.analyzePackageManagerCommand(parts, cwd, preview);
        break;
      case 'docker':
        this.analyzeDockerCommand(parts, cwd, preview);
        break;
      default:
        this.analyzeGenericCommand(command, cwd, preview);
    }
  }

  private analyzeRmCommand(parts: string[], cwd: string, preview: CommandPreview): void {
    const hasRecursive = parts.includes('-r') || parts.includes('-rf') || parts.includes('-fr');
    const hasForce = parts.includes('-f') || parts.includes('-rf') || parts.includes('-fr');
    
    // Extract file patterns (skip flags)
    const targets = parts.slice(1).filter(arg => !arg.startsWith('-'));
    
    for (const target of targets) {
      if (target.includes('*') || target.includes('?')) {
        preview.warnings.push(`Wildcard pattern "${target}" may match unexpected files`);
        preview.willDelete.push(`${cwd}/${target} (pattern match)`);
      } else {
        preview.willDelete.push(`${cwd}/${target}`);
      }
    }

    if (hasRecursive) {
      preview.warnings.push('Recursive deletion - will remove directories and all contents');
    }
    
    if (hasForce) {
      preview.warnings.push('Force flag (-f) - will not prompt for confirmation');
    }

    if (targets.some(t => t === '/' || t === '/*' || t === '~' || t === '~/*')) {
      preview.warnings.push('⚠️ EXTREMELY DANGEROUS: This could delete system or home files!');
    }

    preview.affectedFiles = targets.length * (hasRecursive ? 10 : 1); // Estimate
  }

  private analyzeMvCommand(parts: string[], cwd: string, preview: CommandPreview): void {
    if (parts.length >= 3) {
      const sources = parts.slice(1, -1);
      const destination = parts[parts.length - 1];
      
      preview.willModify.push(...sources.map(src => `${cwd}/${src}`));
      
      if (destination.includes('/')) {
        preview.willCreate.push(`${cwd}/${destination}`);
      } else {
        preview.willModify.push(`${cwd}/${destination}`);
      }
      
      preview.affectedFiles = sources.length;
      
      if (sources.some(src => src.includes('*'))) {
        preview.warnings.push('Moving multiple files with wildcard pattern');
      }
    }
  }

  private analyzeCpCommand(parts: string[], cwd: string, preview: CommandPreview): void {
    const hasRecursive = parts.includes('-r') || parts.includes('-R');
    
    if (parts.length >= 3) {
      const sources = parts.slice(1).filter(p => !p.startsWith('-')).slice(0, -1);
      const destination = parts[parts.length - 1];
      
      for (const source of sources) {
        preview.willCreate.push(`${cwd}/${destination}/${source}`);
      }
      
      preview.affectedFiles = sources.length * (hasRecursive ? 5 : 1); // Estimate
      
      if (hasRecursive) {
        preview.warnings.push('Recursive copy - will copy directories and all contents');
      }
    }
  }

  private analyzeMkdirCommand(parts: string[], cwd: string, preview: CommandPreview): void {
    const dirs = parts.slice(1).filter(p => !p.startsWith('-'));
    const hasParents = parts.includes('-p');
    
    for (const dir of dirs) {
      preview.willCreate.push(`${cwd}/${dir}`);
      if (hasParents && dir.includes('/')) {
        preview.warnings.push(`Will create parent directories for ${dir}`);
      }
    }
    
    preview.affectedFiles = dirs.length;
  }

  private analyzeTouchCommand(parts: string[], cwd: string, preview: CommandPreview): void {
    const files = parts.slice(1);
    
    for (const file of files) {
      preview.willCreate.push(`${cwd}/${file}`);
    }
    
    preview.affectedFiles = files.length;
  }

  private analyzeGitCommand(parts: string[], _cwd: string, preview: CommandPreview): void {
    if (parts.length < 2) return;
    
    const subcommand = parts[1];
    
    switch (subcommand) {
      case 'reset':
        if (parts.includes('--hard')) {
          preview.warnings.push('Hard reset will discard all uncommitted changes');
          preview.willModify.push('Working directory (uncommitted changes will be lost)');
          preview.riskLevel = 'moderate';
        }
        break;
        
      case 'clean':
        if (parts.includes('-f')) {
          preview.warnings.push('Will remove untracked files');
          preview.willDelete.push('Untracked files in repository');
        }
        if (parts.includes('-d')) {
          preview.warnings.push('Will remove untracked directories');
        }
        break;
        
      case 'checkout':
        if (parts.includes('-f')) {
          preview.warnings.push('Force checkout will discard local changes');
          preview.willModify.push('Working directory files');
        }
        break;
        
      case 'rebase':
        preview.warnings.push('Rebase will modify git history - use with caution');
        preview.willModify.push('Git commit history');
        break;
    }
  }

  private analyzePackageManagerCommand(parts: string[], cwd: string, preview: CommandPreview): void {
    const subcommand = parts[1];
    
    switch (subcommand) {
      case 'install':
        preview.willModify.push(`${cwd}/node_modules`);
        preview.willModify.push(`${cwd}/package-lock.json`);
        preview.warnings.push('Will download and install packages');
        preview.estimatedTime = 'Variable (depends on package size)';
        break;
        
      case 'uninstall':
        const packages = parts.slice(2);
        preview.willModify.push(`${cwd}/node_modules`);
        preview.warnings.push(`Will remove packages: ${packages.join(', ')}`);
        break;
        
      case 'audit':
        if (parts.includes('fix')) {
          preview.warnings.push('Will automatically fix security vulnerabilities');
          preview.willModify.push(`${cwd}/package-lock.json`);
        }
        break;
    }
  }

  private analyzeDockerCommand(parts: string[], _cwd: string, preview: CommandPreview): void {
    const subcommand = parts[1];
    
    switch (subcommand) {
      case 'run':
        const volumeMounts = parts.filter(p => p.startsWith('-v') || p.startsWith('--volume'));
        for (const mount of volumeMounts) {
          const [host] = mount.split(':');
          if (host && !host.startsWith('-')) {
            preview.warnings.push(`Will mount host directory: ${host}`);
          }
        }
        break;
        
      case 'rm':
        const containers = parts.slice(2).filter(p => !p.startsWith('-'));
        preview.warnings.push(`Will remove containers: ${containers.join(', ')}`);
        break;
        
      case 'rmi':
        const images = parts.slice(2).filter(p => !p.startsWith('-'));
        preview.warnings.push(`Will remove images: ${images.join(', ')}`);
        break;
    }
  }

  private analyzeGenericCommand(command: string, cwd: string, preview: CommandPreview): void {
    // Check for output redirection
    if (command.includes('>')) {
      const matches = command.match(/>\s*([^\s]+)/g);
      if (matches) {
        for (const match of matches) {
          const file = match.replace('>', '').trim();
          preview.willCreate.push(`${cwd}/${file}`);
          
          if (command.includes('>>')) {
            preview.warnings.push(`Will append to file: ${file}`);
          } else {
            preview.warnings.push(`Will overwrite file: ${file}`);
          }
        }
      }
    }
    
    // Check for pipe operations
    if (command.includes('|')) {
      preview.warnings.push('Pipeline command - output will be processed by multiple commands');
    }
  }

  private assessRiskLevel(command: string, preview: CommandPreview): void {
    // Check dangerous patterns
    if (this.dangerousPatterns.some(pattern => pattern.test(command))) {
      preview.riskLevel = 'dangerous';
      return;
    }
    
    // Check moderate patterns
    if (this.moderatePatterns.some(pattern => pattern.test(command))) {
      preview.riskLevel = 'moderate';
      return;
    }
    
    // Assess based on number of affected files
    if (preview.affectedFiles > 10) {
      preview.riskLevel = 'moderate';
    } else if (preview.affectedFiles > 100) {
      preview.riskLevel = 'dangerous';
    }
    
    // Check for system-critical paths
    const criticalPaths = ['/', '/usr', '/etc', '/sys', '/proc', '/boot'];
    const affectsSystemPaths = [
      ...preview.willDelete,
      ...preview.willModify
    ].some(path => 
      criticalPaths.some(critical => path.startsWith(critical))
    );
    
    if (affectsSystemPaths) {
      preview.riskLevel = 'dangerous';
      preview.warnings.push('⚠️ SYSTEM WARNING: Command affects critical system directories');
    }
  }

  /**
   * Get confirmation prompt for dangerous commands
   */
  getConfirmationPrompt(preview: CommandPreview): string | null {
    if (preview.riskLevel === 'safe') return null;
    
    let prompt = `This command will:\n`;
    
    if (preview.willDelete.length > 0) {
      prompt += `\n🗑️  DELETE:\n${preview.willDelete.map(f => `  - ${f}`).join('\n')}\n`;
    }
    
    if (preview.willModify.length > 0) {
      prompt += `\n📝 MODIFY:\n${preview.willModify.map(f => `  - ${f}`).join('\n')}\n`;
    }
    
    if (preview.willCreate.length > 0) {
      prompt += `\n📁 CREATE:\n${preview.willCreate.map(f => `  - ${f}`).join('\n')}\n`;
    }
    
    if (preview.warnings.length > 0) {
      prompt += `\n⚠️  WARNINGS:\n${preview.warnings.map(w => `  - ${w}`).join('\n')}\n`;
    }
    
    prompt += `\nRisk Level: ${preview.riskLevel.toUpperCase()}\n`;
    prompt += `Affected Files: ${preview.affectedFiles}\n`;
    
    if (preview.estimatedTime) {
      prompt += `Estimated Time: ${preview.estimatedTime}\n`;
    }
    
    prompt += `\nDo you want to proceed? (y/N)`;
    
    return prompt;
  }

  /**
   * Get safer alternative suggestions
   */
  getSaferAlternatives(command: string): string[] {
    const alternatives: string[] = [];
    
    if (command.match(/^rm\s+.*-rf/)) {
      alternatives.push('Use trash command instead: trash ' + command.split(' ').slice(-1)[0]);
      alternatives.push('Move to temp directory first: mv ' + command.split(' ').slice(-1)[0] + ' ./temp/backup_$(date +%s)');
      alternatives.push('List files first: ls -la ' + command.split(' ').slice(-1)[0]);
    }
    
    if (command.includes('git reset --hard')) {
      alternatives.push('Create backup branch first: git branch backup-$(date +%s)');
      alternatives.push('Use git stash instead: git stash push -m "backup before reset"');
    }
    
    if (command.includes('chmod 777')) {
      alternatives.push('Use more restrictive permissions: chmod 755 or chmod 644');
      alternatives.push('Set specific permissions: chmod u+rwx,g+rx,o+rx');
    }
    
    return alternatives;
  }

  /**
   * Check if command should be previewed
   */
  shouldPreview(command: string): boolean {
    const previewPatterns = [
      /^rm\s+/,
      /^mv\s+/,
      /^cp\s+.*-r/,
      /^chmod\s+/,
      /^chown\s+/,
      /^git\s+(reset|clean|rebase)/,
      /^docker\s+(rm|rmi)/,
      /^sudo\s+/,
      />/,  // Output redirection
    ];
    
    return previewPatterns.some(pattern => pattern.test(command.trim()));
  }
}

export const commandPreviewService = new CommandPreviewService();
