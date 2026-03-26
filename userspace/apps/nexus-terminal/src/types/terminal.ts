// Terminal Types for Warp-Style Tabs
export enum ShellType {
  BASH = 'bash',
  FISH = 'fish',
  ZSH = 'zsh',
  POWERSHELL = 'powershell',
  CUSTOM = 'custom'
}

export interface TerminalTab {
  id: string;
  title: string;
  shell: ShellType;
  workingDirectory: string;
  environmentVars: Record<string, string>;
  
  // Terminal Process
  terminalId: string; // Backend terminal process ID
  terminalHistory: CommandHistoryEntry[];
  
  // AI Context
  aiConversation: AIMessage[];
  aiContext: TabAIContext;
  
  // UI State
  isActive: boolean;
  isPinned: boolean;
  lastActivity: Date;
  customIcon?: string;
  order: number; // For drag-and-drop ordering
}

export interface CommandHistoryEntry {
  command: string;
  timestamp: Date;
  workingDirectory: string;
  exitCode?: number;
  output?: string;
  errors?: string[];
}

export interface AIMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  context?: Record<string, any>;
  metadata?: Record<string, any>;
}

export interface TabAIContext {
  projectType?: ProjectType;
  recentCommands: string[];
  workingFiles: string[];
  errors: ErrorContext[];
  suggestions: AISuggestion[];
  learningContext: Record<string, any>;
}

export interface ErrorContext {
  command: string;
  errorMessage: string;
  timestamp: Date;
  workingDirectory: string;
  suggestedFixes?: string[];
}

export interface AISuggestion {
  id: string;
  type: SuggestionType;
  title: string;
  description: string;
  command?: string;
  confidence: number;
  timestamp: Date;
}

export enum SuggestionType {
  COMMAND_IMPROVEMENT = 'command-improvement',
  ERROR_FIX = 'error-fix',
  WORKFLOW_OPTIMIZATION = 'workflow-optimization',
  SECURITY_WARNING = 'security-warning',
  PERFORMANCE_TIP = 'performance-tip',
  GIT_WORKFLOW = 'git-workflow',
  BUILD_ASSISTANCE = 'build-assistance',
  SYSTEM_ADMIN = 'system-admin',
  SAFETY_WARNING = 'safety-warning',
  COMMAND_NOT_FOUND = 'command-not-found'
}

export enum ProjectType {
  JAVASCRIPT = 'javascript',
  TYPESCRIPT = 'typescript',
  REACT = 'react',
  NODEJS = 'nodejs',
  RUST = 'rust',
  PYTHON = 'python',
  GO = 'go',
  UNKNOWN = 'unknown'
}

export interface NewTabConfig {
  shell: ShellType;
  workingDirectory: string;
  title?: string;
  environmentVars?: Record<string, string>;
  preset?: TabPreset;
}

export interface TabPreset {
  name: string;
  description: string;
  shell: ShellType;
  workingDirectory: string;
  environmentVars: Record<string, string>;
  icon: string;
  commands?: string[]; // Commands to run on tab creation
}

export interface DragState {
  draggedTabId: string;
  dropZone: 'before' | 'after' | null;
  targetTabId?: string;
}

export interface ShellConfig {
  type: ShellType;
  executable: string;
  args: string[];
  icon: string;
  color: string;
  description: string;
  features: ShellFeature[];
}

export enum ShellFeature {
  AUTOCOMPLETE = 'autocomplete',
  SYNTAX_HIGHLIGHTING = 'syntax-highlighting',
  PLUGINS = 'plugins',
  THEMES = 'themes',
  VI_MODE = 'vi-mode',
  SCRIPTING = 'scripting'
}

// Built-in shell configurations
export const SHELL_CONFIGS: Record<ShellType, ShellConfig> = {
  [ShellType.BASH]: {
    type: ShellType.BASH,
    executable: 'bash',
    args: ['--login'],
    icon: '🐚',
    color: '#4EAB5C',
    description: 'Bourne Again Shell - Traditional, reliable, widely compatible',
    features: [ShellFeature.SCRIPTING, ShellFeature.VI_MODE]
  },
  [ShellType.FISH]: {
    type: ShellType.FISH,
    executable: 'fish',
    args: [],
    icon: '🐟',
    color: '#00ADD8',
    description: 'Friendly Interactive Shell - Modern, user-friendly, great autocomplete',
    features: [ShellFeature.AUTOCOMPLETE, ShellFeature.SYNTAX_HIGHLIGHTING, ShellFeature.THEMES]
  },
  [ShellType.ZSH]: {
    type: ShellType.ZSH,
    executable: 'zsh',
    args: [],
    icon: '⚡',
    color: '#F15A29',
    description: 'Z Shell - Powerful, customizable, extensive plugin ecosystem',
    features: [ShellFeature.PLUGINS, ShellFeature.THEMES, ShellFeature.AUTOCOMPLETE, ShellFeature.VI_MODE]
  },
  [ShellType.POWERSHELL]: {
    type: ShellType.POWERSHELL,
    executable: 'pwsh',
    args: ['-NoLogo'],
    icon: '💙',
    color: '#012456',
    description: 'PowerShell - Cross-platform, object-based shell and scripting language',
    features: [ShellFeature.SCRIPTING, ShellFeature.AUTOCOMPLETE]
  },
  [ShellType.CUSTOM]: {
    type: ShellType.CUSTOM,
    executable: 'sh',
    args: [],
    icon: '💻',
    color: '#6C7B7F',
    description: 'Custom Shell - User-defined shell configuration',
    features: []
  }
};

// Default tab presets
export const DEFAULT_TAB_PRESETS: TabPreset[] = [
  {
    name: 'Frontend Development',
    description: 'React/Vue/Angular development environment',
    shell: ShellType.BASH,
    workingDirectory: '~/projects/frontend',
    environmentVars: { 
      NODE_ENV: 'development',
      BROWSER: 'none' 
    },
    icon: '⚛️',
    commands: ['npm run dev']
  },
  {
    name: 'Backend API',
    description: 'Rust/Node.js API development',
    shell: ShellType.FISH,
    workingDirectory: '~/projects/api',
    environmentVars: { 
      RUST_LOG: 'debug',
      DATABASE_URL: 'postgres://localhost/dev'
    },
    icon: '🦀',
    commands: ['cargo run']
  },
  {
    name: 'System Administration',
    description: 'System management and monitoring',
    shell: ShellType.ZSH,
    workingDirectory: './logs',
    environmentVars: {},
    icon: '⚙️'
  },
  {
    name: 'Data Science',
    description: 'Python data analysis and machine learning',
    shell: ShellType.BASH,
    workingDirectory: '~/notebooks',
    environmentVars: {
      PYTHONPATH: '~/projects/ml-utils',
      JUPYTER_CONFIG_DIR: '~/.jupyter'
    },
    icon: '🐍',
    commands: ['python -m jupyter notebook']
  },
  {
    name: 'Docker Development',
    description: 'Container management and development',
    shell: ShellType.ZSH,
    workingDirectory: '~/docker',
    environmentVars: {
      COMPOSE_PROJECT_NAME: 'dev',
      DOCKER_BUILDKIT: '1'
    },
    icon: '🐳',
    commands: ['docker-compose up -d']
  }
];
