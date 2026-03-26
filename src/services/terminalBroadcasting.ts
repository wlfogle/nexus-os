import { invoke } from '@tauri-apps/api/core';

export interface TerminalSession {
  id: string;
  name: string;
  type: 'local' | 'remote' | 'container' | 'vm';
  status: 'active' | 'inactive' | 'error' | 'disconnected';
  connection?: {
    host?: string;
    port?: number;
    username?: string;
    keyPath?: string;
  };
  environment: Record<string, string>;
  workingDirectory: string;
  shell: string;
  tags: string[];
  createdAt: Date;
  lastActivity: Date;
}

export interface BroadcastGroup {
  id: string;
  name: string;
  description: string;
  sessionIds: string[];
  filters: BroadcastFilter[];
  settings: BroadcastSettings;
  createdAt: Date;
  lastUsed: Date;
}

export interface BroadcastFilter {
  type: 'tag' | 'name' | 'type' | 'environment' | 'regex';
  operator: 'equals' | 'contains' | 'starts_with' | 'ends_with' | 'matches';
  value: string;
  negate: boolean;
}

export interface BroadcastSettings {
  parallelExecution: boolean;
  stopOnFirstError: boolean;
  showIndividualOutput: boolean;
  aggregateOutput: boolean;
  timeout: number;
  confirmBeforeExecution: boolean;
  logCommands: boolean;
}

export interface BroadcastResult {
  groupId: string;
  command: string;
  startTime: Date;
  endTime?: Date;
  results: SessionResult[];
  overallStatus: 'success' | 'partial' | 'failed';
  summary: {
    totalSessions: number;
    successfulSessions: number;
    failedSessions: number;
    averageExecutionTime: number;
  };
}

export interface SessionResult {
  sessionId: string;
  sessionName: string;
  status: 'success' | 'failed' | 'timeout' | 'skipped';
  output: string;
  error?: string;
  exitCode: number;
  executionTime: number;
  timestamp: Date;
}

export class TerminalBroadcastingService {
  private sessions = new Map<string, TerminalSession>();
  private groups = new Map<string, BroadcastGroup>();
  private broadcastHistory: BroadcastResult[] = [];
  private activeBroadcasts = new Map<string, AbortController>();
  
  /**
   * Register a new terminal session
   */
  async registerSession(session: Omit<TerminalSession, 'id' | 'createdAt' | 'lastActivity'>): Promise<string> {
    const sessionId = this.generateSessionId();
    const fullSession: TerminalSession = {
      ...session,
      id: sessionId,
      createdAt: new Date(),
      lastActivity: new Date()
    };
    
    this.sessions.set(sessionId, fullSession);
    
    try {
      await invoke('register_broadcast_session', { session: fullSession });
    } catch (error) {
      console.warn('Failed to register session with backend:', error);
    }
    
    return sessionId;
  }
  
  /**
   * Update session status
   */
  updateSession(sessionId: string, updates: Partial<TerminalSession>): void {
    const session = this.sessions.get(sessionId);
    if (!session) return;
    
    Object.assign(session, updates);
    session.lastActivity = new Date();
    
    this.sessions.set(sessionId, session);
  }
  
  /**
   * Remove session
   */
  async removeSession(sessionId: string): Promise<void> {
    this.sessions.delete(sessionId);
    
    // Remove from all groups
    for (const group of this.groups.values()) {
      group.sessionIds = group.sessionIds.filter(id => id !== sessionId);
    }
    
    try {
      await invoke('unregister_broadcast_session', { sessionId });
    } catch (error) {
      console.warn('Failed to unregister session:', error);
    }
  }
  
  /**
   * Create broadcast group
   */
  createGroup(
    name: string,
    description: string,
    filters: BroadcastFilter[] = [],
    settings: Partial<BroadcastSettings> = {}
  ): string {
    const groupId = this.generateGroupId();
    const group: BroadcastGroup = {
      id: groupId,
      name,
      description,
      sessionIds: [],
      filters,
      settings: {
        parallelExecution: true,
        stopOnFirstError: false,
        showIndividualOutput: true,
        aggregateOutput: false,
        timeout: 30000,
        confirmBeforeExecution: true,
        logCommands: true,
        ...settings
      },
      createdAt: new Date(),
      lastUsed: new Date()
    };
    
    // Apply filters to determine initial session membership
    group.sessionIds = this.getSessionsMatchingFilters(filters);
    
    this.groups.set(groupId, group);
    return groupId;
  }
  
  /**
   * Update broadcast group
   */
  updateGroup(groupId: string, updates: Partial<BroadcastGroup>): void {
    const group = this.groups.get(groupId);
    if (!group) return;
    
    Object.assign(group, updates);
    this.groups.set(groupId, group);
  }
  
  /**
   * Delete broadcast group
   */
  deleteGroup(groupId: string): void {
    this.groups.delete(groupId);
  }
  
  /**
   * Add sessions to group
   */
  addSessionsToGroup(groupId: string, sessionIds: string[]): void {
    const group = this.groups.get(groupId);
    if (!group) return;
    
    const uniqueIds = new Set([...group.sessionIds, ...sessionIds]);
    group.sessionIds = Array.from(uniqueIds);
    
    this.groups.set(groupId, group);
  }
  
  /**
   * Remove sessions from group
   */
  removeSessionsFromGroup(groupId: string, sessionIds: string[]): void {
    const group = this.groups.get(groupId);
    if (!group) return;
    
    group.sessionIds = group.sessionIds.filter(id => !sessionIds.includes(id));
    this.groups.set(groupId, group);
  }
  
  /**
   * Execute command on group
   */
  async broadcastCommand(
    groupIdOrSessionIds: string | string[],
    command: string
  ): Promise<BroadcastResult> {
    let sessionIds: string[];
    let groupId: string;
    
    if (Array.isArray(groupIdOrSessionIds)) {
      sessionIds = groupIdOrSessionIds;
      groupId = 'adhoc_' + Date.now();
    } else {
      const group = this.groups.get(groupIdOrSessionIds);
      if (!group) {
        throw new Error('Broadcast group not found');
      }
      sessionIds = group.sessionIds;
      groupId = groupIdOrSessionIds;
      group.lastUsed = new Date();
    }
    
    const validSessions = sessionIds
      .map(id => this.sessions.get(id))
      .filter((session): session is TerminalSession => 
        session !== undefined && session.status === 'active'
      );
    
    if (validSessions.length === 0) {
      throw new Error('No active sessions available for broadcast');
    }
    
    const result: BroadcastResult = {
      groupId,
      command,
      startTime: new Date(),
      results: [],
      overallStatus: 'success',
      summary: {
        totalSessions: validSessions.length,
        successfulSessions: 0,
        failedSessions: 0,
        averageExecutionTime: 0
      }
    };
    
    // Create abort controller for this broadcast
    const abortController = new AbortController();
    this.activeBroadcasts.set(groupId, abortController);
    
    try {
      if (this.getGroupSettings(groupId).parallelExecution) {
        result.results = await this.executeParallel(validSessions, command, abortController.signal);
      } else {
        result.results = await this.executeSequential(validSessions, command, abortController.signal);
      }
    } finally {
      this.activeBroadcasts.delete(groupId);
    }
    
    result.endTime = new Date();
    
    // Calculate summary
    result.summary.successfulSessions = result.results.filter(r => r.status === 'success').length;
    result.summary.failedSessions = result.results.length - result.summary.successfulSessions;
    result.summary.averageExecutionTime = 
      result.results.reduce((sum, r) => sum + r.executionTime, 0) / result.results.length;
    
    result.overallStatus = result.summary.failedSessions === 0 ? 'success' :
                          result.summary.successfulSessions === 0 ? 'failed' : 'partial';
    
    // Store in history
    this.broadcastHistory.push(result);
    if (this.broadcastHistory.length > 1000) {
      this.broadcastHistory.shift();
    }
    
    return result;
  }
  
  private async executeParallel(
    sessions: TerminalSession[],
    command: string,
    signal: AbortSignal
  ): Promise<SessionResult[]> {
    const promises = sessions.map(session => 
      this.executeOnSession(session, command, signal)
    );
    
    return Promise.all(promises);
  }
  
  private async executeSequential(
    sessions: TerminalSession[],
    command: string,
    signal: AbortSignal
  ): Promise<SessionResult[]> {
    const results: SessionResult[] = [];
    const settings = this.getGroupSettings('');
    
    for (const session of sessions) {
      if (signal.aborted) break;
      
      const result = await this.executeOnSession(session, command, signal);
      results.push(result);
      
      if (settings.stopOnFirstError && result.status === 'failed') {
        // Mark remaining sessions as skipped
        const remainingSessions = sessions.slice(results.length);
        for (const remainingSession of remainingSessions) {
          results.push({
            sessionId: remainingSession.id,
            sessionName: remainingSession.name,
            status: 'skipped',
            output: '',
            exitCode: -1,
            executionTime: 0,
            timestamp: new Date()
          });
        }
        break;
      }
    }
    
    return results;
  }
  
  private async executeOnSession(
    session: TerminalSession,
    command: string,
    signal: AbortSignal
  ): Promise<SessionResult> {
    const startTime = Date.now();
    
    try {
      const result = await invoke<{output: string; exitCode: number}>('execute_on_session', {
        sessionId: session.id,
        command,
        timeout: this.getGroupSettings('').timeout
      });
      
      if (signal.aborted) {
        return {
          sessionId: session.id,
          sessionName: session.name,
          status: 'failed',
          output: '',
          error: 'Broadcast cancelled',
          exitCode: -1,
          executionTime: Date.now() - startTime,
          timestamp: new Date()
        };
      }
      
      return {
        sessionId: session.id,
        sessionName: session.name,
        status: result.exitCode === 0 ? 'success' : 'failed',
        output: result.output,
        exitCode: result.exitCode,
        executionTime: Date.now() - startTime,
        timestamp: new Date()
      };
    } catch (error) {
      return {
        sessionId: session.id,
        sessionName: session.name,
        status: 'failed',
        output: '',
        error: error as string,
        exitCode: -1,
        executionTime: Date.now() - startTime,
        timestamp: new Date()
      };
    }
  }
  
  /**
   * Cancel active broadcast
   */
  cancelBroadcast(groupId: string): void {
    const controller = this.activeBroadcasts.get(groupId);
    if (controller) {
      controller.abort();
      this.activeBroadcasts.delete(groupId);
    }
  }
  
  /**
   * Get sessions matching filters
   */
  private getSessionsMatchingFilters(filters: BroadcastFilter[]): string[] {
    if (filters.length === 0) return [];
    
    return Array.from(this.sessions.values())
      .filter(session => this.sessionMatchesFilters(session, filters))
      .map(session => session.id);
  }
  
  private sessionMatchesFilters(session: TerminalSession, filters: BroadcastFilter[]): boolean {
    return filters.every(filter => {
      let matches = false;
      
      switch (filter.type) {
        case 'tag':
          matches = session.tags.some(tag => this.testStringMatch(tag, filter.operator, filter.value));
          break;
        case 'name':
          matches = this.testStringMatch(session.name, filter.operator, filter.value);
          break;
        case 'type':
          matches = this.testStringMatch(session.type, filter.operator, filter.value);
          break;
        case 'environment':
          const [envKey, envValue] = filter.value.split('=');
          matches = envValue ? 
            session.environment[envKey] === envValue :
            envKey in session.environment;
          break;
        case 'regex':
          try {
            const regex = new RegExp(filter.value);
            matches = regex.test(session.name) || session.tags.some(tag => regex.test(tag));
          } catch {
            matches = false;
          }
          break;
      }
      
      return filter.negate ? !matches : matches;
    });
  }
  
  private testStringMatch(value: string, operator: BroadcastFilter['operator'], testValue: string): boolean {
    switch (operator) {
      case 'equals': return value === testValue;
      case 'contains': return value.includes(testValue);
      case 'starts_with': return value.startsWith(testValue);
      case 'ends_with': return value.endsWith(testValue);
      case 'matches': 
        try {
          return new RegExp(testValue).test(value);
        } catch {
          return false;
        }
      default: return false;
    }
  }
  
  private getGroupSettings(groupId: string): BroadcastSettings {
    const group = this.groups.get(groupId);
    return group?.settings || {
      parallelExecution: true,
      stopOnFirstError: false,
      showIndividualOutput: true,
      aggregateOutput: false,
      timeout: 30000,
      confirmBeforeExecution: true,
      logCommands: true
    };
  }
  
  /**
   * Get all sessions
   */
  getAllSessions(): TerminalSession[] {
    return Array.from(this.sessions.values());
  }
  
  /**
   * Get sessions by group
   */
  getSessionsByGroup(groupId: string): TerminalSession[] {
    const group = this.groups.get(groupId);
    if (!group) return [];
    
    return group.sessionIds
      .map(id => this.sessions.get(id))
      .filter((session): session is TerminalSession => session !== undefined);
  }
  
  /**
   * Get all groups
   */
  getAllGroups(): BroadcastGroup[] {
    return Array.from(this.groups.values());
  }
  
  /**
   * Get broadcast history
   */
  getBroadcastHistory(limit: number = 100): BroadcastResult[] {
    return this.broadcastHistory.slice(-limit);
  }
  
  /**
   * Create preset groups
   */
  createPresetGroups(): void {
    // Development group
    this.createGroup(
      'Development',
      'All development servers and environments',
      [
        { type: 'tag', operator: 'contains', value: 'dev', negate: false },
        { type: 'tag', operator: 'contains', value: 'local', negate: false }
      ],
      { parallelExecution: true, stopOnFirstError: false }
    );
    
    // Production group  
    this.createGroup(
      'Production',
      'Production servers (use with extreme caution)',
      [
        { type: 'tag', operator: 'contains', value: 'prod', negate: false }
      ],
      { 
        parallelExecution: false, 
        stopOnFirstError: true, 
        confirmBeforeExecution: true,
        timeout: 60000
      }
    );
    
    // Docker containers
    this.createGroup(
      'Docker Containers',
      'All Docker container sessions',
      [
        { type: 'type', operator: 'equals', value: 'container', negate: false }
      ],
      { parallelExecution: true, timeout: 45000 }
    );
    
    // Remote servers
    this.createGroup(
      'Remote Servers',
      'All SSH remote sessions',
      [
        { type: 'type', operator: 'equals', value: 'remote', negate: false }
      ],
      { parallelExecution: false, timeout: 60000 }
    );
  }
  
  /**
   * Test command on single session before broadcast
   */
  async testCommand(sessionId: string, command: string): Promise<SessionResult> {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error('Session not found');
    }
    
    return this.executeOnSession(session, command, new AbortController().signal);
  }
  
  /**
   * Get command suggestions for broadcast
   */
  getBroadcastSuggestions(): CommandSuggestion[] {
    return [
      {
        command: 'pwd',
        description: 'Check current directory on all sessions',
        confidence: 0.9,
        category: 'system'
      },
      {
        command: 'whoami',
        description: 'Check current user on all sessions',
        confidence: 0.9,
        category: 'system'
      },
      {
        command: 'git status',
        description: 'Check git status across all repositories',
        confidence: 0.8,
        category: 'git'
      },
      {
        command: 'df -h',
        description: 'Check disk usage on all systems',
        confidence: 0.8,
        category: 'system'
      },
      {
        command: 'systemctl status nginx',
        description: 'Check service status across servers',
        confidence: 0.7,
        category: 'system'
      },
      {
        command: 'docker ps',
        description: 'List running containers on all Docker hosts',
        confidence: 0.8,
        category: 'docker'
      }
    ];
  }
  
  /**
   * Import sessions from configuration file
   */
  async importSessions(configPath: string): Promise<number> {
    try {
      const sessions = await invoke<TerminalSession[]>('import_broadcast_sessions', { configPath });
      
      for (const session of sessions) {
        this.sessions.set(session.id, session);
      }
      
      return sessions.length;
    } catch (error) {
      throw new Error(`Failed to import sessions: ${error}`);
    }
  }
  
  /**
   * Export sessions to configuration file
   */
  async exportSessions(outputPath: string, sessionIds?: string[]): Promise<void> {
    const sessionsToExport = sessionIds ? 
      sessionIds.map(id => this.sessions.get(id)).filter(Boolean) :
      Array.from(this.sessions.values());
    
    try {
      await invoke('export_broadcast_sessions', { 
        sessions: sessionsToExport, 
        outputPath 
      });
    } catch (error) {
      throw new Error(`Failed to export sessions: ${error}`);
    }
  }
  
  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  private generateGroupId(): string {
    return `group_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  /**
   * Get active broadcast operations
   */
  getActiveBroadcasts(): string[] {
    return Array.from(this.activeBroadcasts.keys());
  }
  
  /**
   * Get broadcast statistics
   */
  getBroadcastStats(): BroadcastStats {
    const recentBroadcasts = this.broadcastHistory.slice(-100);
    
    return {
      totalBroadcasts: this.broadcastHistory.length,
      recentSuccessRate: recentBroadcasts.length > 0 ? 
        recentBroadcasts.filter(b => b.overallStatus === 'success').length / recentBroadcasts.length : 0,
      averageSessionsPerBroadcast: recentBroadcasts.length > 0 ?
        recentBroadcasts.reduce((sum, b) => sum + b.summary.totalSessions, 0) / recentBroadcasts.length : 0,
      mostUsedGroups: Array.from(this.groups.values())
        .sort((a, b) => b.lastUsed.getTime() - a.lastUsed.getTime())
        .slice(0, 5)
        .map(g => ({ id: g.id, name: g.name, lastUsed: g.lastUsed })),
      totalActiveSessions: Array.from(this.sessions.values()).filter(s => s.status === 'active').length
    };
  }
}

export interface BroadcastStats {
  totalBroadcasts: number;
  recentSuccessRate: number;
  averageSessionsPerBroadcast: number;
  mostUsedGroups: Array<{ id: string; name: string; lastUsed: Date }>;
  totalActiveSessions: number;
}

// Re-export command suggestion interface for convenience
export interface CommandSuggestion {
  command: string;
  description: string;
  confidence: number;
  category: string;
}

export const terminalBroadcastingService = new TerminalBroadcastingService();
