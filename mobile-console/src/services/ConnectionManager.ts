import { io, Socket } from 'socket.io-client';
import axios, { AxiosInstance } from 'axios';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { EventEmitter } from 'events';

export interface ConnectionConfig {
  host: string;
  port: number;
  secure: boolean;
  authToken?: string;
}

export interface CommandRequest {
  command: string;
  args?: string[];
  cwd?: string;
}

export interface CommandResponse {
  success: boolean;
  output: string;
  error?: string;
  exitCode: number;
  executionTime: number;
}

export interface SystemStatus {
  cpu: number;
  memory: number;
  uptime: number;
  processes: number;
  networkConnections: number;
}

class ConnectionManagerClass extends EventEmitter {
  private socket: Socket | null = null;
  private httpClient: AxiosInstance | null = null;
  private config: ConnectionConfig | null = null;
  private connectionState: 'disconnected' | 'connecting' | 'connected' | 'error' = 'disconnected';
  private reconnectTimer: NodeJS.Timeout | null = null;

  async initialize() {
    try {
      const savedConfig = await AsyncStorage.getItem('connectionConfig');
      if (savedConfig) {
        this.config = JSON.parse(savedConfig);
        await this.connect();
      }
    } catch (error) {
      console.error('Failed to initialize connection:', error);
    }
  }

  async setConnectionConfig(config: ConnectionConfig) {
    this.config = config;
    await AsyncStorage.setItem('connectionConfig', JSON.stringify(config));
    
    // Reconnect with new config
    if (this.connectionState === 'connected') {
      this.disconnect();
    }
    await this.connect();
  }

  async connect(): Promise<boolean> {
    if (!this.config) {
      throw new Error('No connection configuration set');
    }

    this.connectionState = 'connecting';
    this.emit('connectionStateChanged', this.connectionState);

    try {
      // Setup HTTP client
      const protocol = this.config.secure ? 'https' : 'http';
      const baseURL = `${protocol}://${this.config.host}:${this.config.port}`;
      
      this.httpClient = axios.create({
        baseURL,
        timeout: 10000,
        headers: this.config.authToken 
          ? { 'Authorization': `Bearer ${this.config.authToken}` }
          : {}
      });

      // Setup WebSocket connection
      const socketProtocol = this.config.secure ? 'wss' : 'ws';
      const socketURL = `${socketProtocol}://${this.config.host}:${this.config.port}`;
      
      this.socket = io(socketURL, {
        auth: this.config.authToken 
          ? { token: this.config.authToken }
          : {},
        reconnection: true,
        reconnectionAttempts: 5,
        reconnectionDelay: 1000,
      });

      this.setupSocketHandlers();

      // Test connection
      await this.testConnection();
      
      this.connectionState = 'connected';
      this.emit('connectionStateChanged', this.connectionState);
      return true;

    } catch (error) {
      console.error('Connection failed:', error);
      this.connectionState = 'error';
      this.emit('connectionStateChanged', this.connectionState);
      this.scheduleReconnect();
      return false;
    }
  }

  private setupSocketHandlers() {
    if (!this.socket) return;

    this.socket.on('connect', () => {
      console.log('WebSocket connected');
      this.connectionState = 'connected';
      this.emit('connectionStateChanged', this.connectionState);
      if (this.reconnectTimer) {
        clearTimeout(this.reconnectTimer);
        this.reconnectTimer = null;
      }
    });

    this.socket.on('disconnect', (reason) => {
      console.log('WebSocket disconnected:', reason);
      this.connectionState = 'disconnected';
      this.emit('connectionStateChanged', this.connectionState);
      this.scheduleReconnect();
    });

    this.socket.on('connect_error', (error) => {
      console.error('WebSocket connection error:', error);
      this.connectionState = 'error';
      this.emit('connectionStateChanged', this.connectionState);
      this.scheduleReconnect();
    });

    // System events
    this.socket.on('systemStatus', (status: SystemStatus) => {
      this.emit('systemStatus', status);
    });

    this.socket.on('commandOutput', (data: any) => {
      this.emit('commandOutput', data);
    });

    this.socket.on('fileSystemChange', (data: any) => {
      this.emit('fileSystemChange', data);
    });

    this.socket.on('ecosystemInsights', (data: any) => {
      this.emit('ecosystemInsights', data);
    });
  }

  private async testConnection(): Promise<void> {
    if (!this.httpClient) {
      throw new Error('HTTP client not initialized');
    }

    const response = await this.httpClient.get('/api/health');
    if (response.status !== 200) {
      throw new Error(`Health check failed: ${response.status}`);
    }
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;

    this.reconnectTimer = setTimeout(() => {
      console.log('Attempting to reconnect...');
      this.connect();
    }, 5000);
  }

  disconnect() {
    if (this.socket) {
      this.socket.disconnect();
      this.socket = null;
    }

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    this.connectionState = 'disconnected';
    this.emit('connectionStateChanged', this.connectionState);
  }

  cleanup() {
    this.disconnect();
    this.removeAllListeners();
  }

  // API Methods
  async executeCommand(request: CommandRequest): Promise<CommandResponse> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    const response = await this.httpClient.post('/api/command', request);
    return response.data;
  }

  async getSystemStatus(): Promise<SystemStatus> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    const response = await this.httpClient.get('/api/system/status');
    return response.data;
  }

  async getProcessList(): Promise<any[]> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    const response = await this.httpClient.get('/api/system/processes');
    return response.data;
  }

  async browseFiles(path: string = '/'): Promise<any[]> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    const response = await this.httpClient.get('/api/files/browse', {
      params: { path }
    });
    return response.data;
  }

  async readFile(path: string): Promise<string> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    const response = await this.httpClient.get('/api/files/read', {
      params: { path }
    });
    return response.data.content;
  }

  async writeFile(path: string, content: string): Promise<void> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    await this.httpClient.post('/api/files/write', {
      path,
      content
    });
  }

  async deleteFile(path: string): Promise<void> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    await this.httpClient.delete('/api/files/delete', {
      params: { path }
    });
  }

  async getEcosystemInsights(): Promise<any> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    const response = await this.httpClient.get('/api/ecosystem/insights');
    return response.data;
  }

  // WebSocket methods
  subscribeToSystemUpdates() {
    if (this.socket) {
      this.socket.emit('subscribeSystemUpdates');
    }
  }

  unsubscribeFromSystemUpdates() {
    if (this.socket) {
      this.socket.emit('unsubscribeSystemUpdates');
    }
  }

  startTerminalSession(sessionId: string) {
    if (this.socket) {
      this.socket.emit('startTerminal', { sessionId });
    }
  }

  sendTerminalInput(sessionId: string, input: string) {
    if (this.socket) {
      this.socket.emit('terminalInput', { sessionId, input });
    }
  }

  endTerminalSession(sessionId: string) {
    if (this.socket) {
      this.socket.emit('endTerminal', { sessionId });
    }
  }

  // Getters
  getConnectionState() {
    return this.connectionState;
  }

  getConfig() {
    return this.config;
  }

  isConnected() {
    return this.connectionState === 'connected';
  }
}

export const ConnectionManager = new ConnectionManagerClass();
