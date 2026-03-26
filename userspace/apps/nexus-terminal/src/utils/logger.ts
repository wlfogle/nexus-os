/**
 * Centralized logging utility for NexusTerminal
 * Provides structured logging with different levels and contexts
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3
}

export interface LogContext {
  component?: string;
  service?: string;
  action?: string;
  metadata?: Record<string, unknown>;
}

export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  context?: LogContext;
  error?: Error;
}

class Logger {
  private logLevel: LogLevel = LogLevel.INFO;
  private logs: LogEntry[] = [];
  private maxLogs = 1000;

  constructor() {
    // Set log level based on environment
    const envLogLevel = this.getEnvVar('LOG_LEVEL', 'INFO');
    this.logLevel = this.parseLogLevel(envLogLevel);
  }

  private getEnvVar(key: string, fallback: string): string {
    // Node.js environment
    if (typeof process !== 'undefined' && process.env && process.env[key]) {
      return process.env[key]!;
    }
    
    // Tauri environment
    if (typeof window !== 'undefined' && (window as any).__TAURI__ && (window as any).__TAURI_ENV__) {
      const envValue = (window as any).__TAURI_ENV__[key];
      if (envValue) return envValue;
    }
    
    // Browser development mode check
    if (typeof window !== 'undefined' && window.location?.hostname === 'localhost') {
      // Development mode - default to DEBUG
      if (key === 'LOG_LEVEL' && fallback === 'INFO') {
        return 'DEBUG';
      }
    }
    
    return fallback;
  }

  private parseLogLevel(level: string): LogLevel {
    switch (level.toUpperCase()) {
      case 'DEBUG': return LogLevel.DEBUG;
      case 'INFO': return LogLevel.INFO;
      case 'WARN': return LogLevel.WARN;
      case 'ERROR': return LogLevel.ERROR;
      default: return LogLevel.INFO;
    }
  }

  private shouldLog(level: LogLevel): boolean {
    return level >= this.logLevel;
  }

  private formatMessage(level: LogLevel, message: string, context?: LogContext): string {
    const timestamp = new Date().toISOString();
    const levelStr = LogLevel[level].padEnd(5);
    
    let formatted = `[${timestamp}] ${levelStr} ${message}`;
    
    if (context?.component) {
      formatted += ` [${context.component}]`;
    }
    
    if (context?.service) {
      formatted += ` [${context.service}]`;
    }
    
    if (context?.action) {
      formatted += ` (${context.action})`;
    }
    
    return formatted;
  }

  private logToStorage(entry: LogEntry): void {
    this.logs.push(entry);
    
    // Maintain log size limit
    if (this.logs.length > this.maxLogs) {
      this.logs.splice(0, this.logs.length - this.maxLogs);
    }
  }

  private createLogEntry(level: LogLevel, message: string, context?: LogContext, error?: Error): LogEntry {
    return {
      timestamp: new Date().toISOString(),
      level,
      message,
      context,
      error
    };
  }

  debug(message: string, context?: LogContext): void {
    if (!this.shouldLog(LogLevel.DEBUG)) return;
    
    const entry = this.createLogEntry(LogLevel.DEBUG, message, context);
    this.logToStorage(entry);
    
    if (typeof console !== 'undefined') {
      console.debug(this.formatMessage(LogLevel.DEBUG, message, context));
    }
  }

  info(message: string, context?: LogContext): void {
    if (!this.shouldLog(LogLevel.INFO)) return;
    
    const entry = this.createLogEntry(LogLevel.INFO, message, context);
    this.logToStorage(entry);
    
    if (typeof console !== 'undefined') {
      console.info(this.formatMessage(LogLevel.INFO, message, context));
    }
  }

  warn(message: string, context?: LogContext, error?: Error): void {
    if (!this.shouldLog(LogLevel.WARN)) return;
    
    const entry = this.createLogEntry(LogLevel.WARN, message, context, error);
    this.logToStorage(entry);
    
    if (typeof console !== 'undefined') {
      console.warn(this.formatMessage(LogLevel.WARN, message, context));
      if (error) {
        console.warn('Error details:', error);
      }
    }
  }

  error(message: string, context?: LogContext, error?: Error): void {
    if (!this.shouldLog(LogLevel.ERROR)) return;
    
    const entry = this.createLogEntry(LogLevel.ERROR, message, context, error);
    this.logToStorage(entry);
    
    if (typeof console !== 'undefined') {
      console.error(this.formatMessage(LogLevel.ERROR, message, context));
      if (error) {
        console.error('Error details:', error);
      }
    }
  }

  /**
   * Get recent log entries
   */
  getRecentLogs(count: number = 50): LogEntry[] {
    return this.logs.slice(-count);
  }

  /**
   * Get logs filtered by level
   */
  getLogsByLevel(level: LogLevel, count: number = 50): LogEntry[] {
    return this.logs
      .filter(log => log.level === level)
      .slice(-count);
  }

  /**
   * Get logs filtered by component or service
   */
  getLogsByContext(component?: string, service?: string, count: number = 50): LogEntry[] {
    return this.logs
      .filter(log => {
        if (component && log.context?.component !== component) return false;
        if (service && log.context?.service !== service) return false;
        return true;
      })
      .slice(-count);
  }

  /**
   * Clear all stored logs
   */
  clearLogs(): void {
    this.logs = [];
  }

  /**
   * Set log level at runtime
   */
  setLogLevel(level: LogLevel): void {
    this.logLevel = level;
  }

  /**
   * Log with performance timing
   */
  performance(message: string, startTime: number, context?: LogContext): void {
    const duration = Date.now() - startTime;
    this.info(`${message} (${duration}ms)`, {
      ...context,
      metadata: { ...context?.metadata, duration }
    });
  }

  /**
   * Create a performance timer
   */
  startTimer(label: string): { end: (context?: LogContext) => void } {
    const startTime = Date.now();
    return {
      end: (context?: LogContext) => {
        this.performance(label, startTime, context);
      }
    };
  }

  /**
   * Log environment information on startup
   */
  logEnvironmentInfo(): void {
    const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__;
    const isNode = typeof process !== 'undefined';
    const isDev = typeof window !== 'undefined' && window.location?.hostname === 'localhost';
    
    this.info('Logger initialized', {
      component: 'Logger',
      action: 'initialize',
      metadata: {
        logLevel: LogLevel[this.logLevel],
        environment: isTauri ? 'tauri' : isNode ? 'node' : 'browser',
        development: isDev,
        maxLogs: this.maxLogs
      }
    });
  }
}

// Singleton instance
export const logger = new Logger();

// Initialize environment logging
logger.logEnvironmentInfo();

// Convenience functions for common use cases
export const createComponentLogger = (componentName: string) => ({
  debug: (message: string, action?: string, metadata?: Record<string, unknown>) =>
    logger.debug(message, { component: componentName, action, metadata }),
  info: (message: string, action?: string, metadata?: Record<string, unknown>) =>
    logger.info(message, { component: componentName, action, metadata }),
  warn: (message: string, error?: Error, action?: string, metadata?: Record<string, unknown>) =>
    logger.warn(message, { component: componentName, action, metadata }, error),
  error: (message: string, error?: Error, action?: string, metadata?: Record<string, unknown>) =>
    logger.error(message, { component: componentName, action, metadata }, error),
});

export const createServiceLogger = (serviceName: string) => ({
  debug: (message: string, action?: string, metadata?: Record<string, unknown>) =>
    logger.debug(message, { service: serviceName, action, metadata }),
  info: (message: string, action?: string, metadata?: Record<string, unknown>) =>
    logger.info(message, { service: serviceName, action, metadata }),
  warn: (message: string, error?: Error, action?: string, metadata?: Record<string, unknown>) =>
    logger.warn(message, { service: serviceName, action, metadata }, error),
  error: (message: string, error?: Error, action?: string, metadata?: Record<string, unknown>) =>
    logger.error(message, { service: serviceName, action, metadata }, error),
});

// Pre-configured loggers for common components
export const commandRoutingLogger = createServiceLogger('CommandRouting');
export const terminalLogger = createComponentLogger('Terminal');
export const aiLogger = createServiceLogger('AI');
export const visionLogger = createServiceLogger('Vision');
export const gitLogger = createServiceLogger('Git');
export const executorLogger = createComponentLogger('CommandExecutor');
export const interfaceLogger = createComponentLogger('Interface');

// Specialized command routing logger with enhanced methods
export const routingLogger = {
  ...commandRoutingLogger,
  
  routeDecision: (input: string, isShell: boolean, confidence: number, reason: string) => {
    const type = isShell ? '🐚 Shell' : '🤖 AI';
    commandRoutingLogger.info(
      `Command routed: "${input}" → ${type}`,
      'route_decision',
      { input, isShell, confidence, reason }
    );
  },
  
  routeAnalysis: (input: string, confidence: number, reason: string, alternatives?: string[]) => {
    commandRoutingLogger.debug(
      `Routing analysis for "${input}"`,
      'route_analysis', 
      { input, confidence, reason, alternatives }
    );
  },
  
  performanceTimer: (label: string) => logger.startTimer(`Command Routing: ${label}`),
  
  shellExecution: (command: string, success: boolean, error?: Error) => {
    if (success) {
      commandRoutingLogger.info(
        `Shell command executed: ${command}`,
        'shell_execute',
        { command, success }
      );
    } else {
      commandRoutingLogger.error(
        `Shell command failed: ${command}`,
        error,
        'shell_execute_failed',
        { command, success }
      );
    }
  },
  
  aiRequest: (query: string, responseTime?: number) => {
    if (responseTime !== undefined) {
      commandRoutingLogger.info(
        `AI query processed: "${query}" (${responseTime}ms)`,
        'ai_query',
        { query, responseTime }
      );
    } else {
      commandRoutingLogger.info(
        `AI query sent: "${query}"`,
        'ai_query_sent',
        { query }
      );
    }
  }
};
