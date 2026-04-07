import { EventEmitter } from 'events';
import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import io, { Socket } from 'socket.io-client';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { Platform } from 'react-native';
import NetInfo from '@react-native-community/netinfo';
import CryptoJS from 'crypto-js';

export interface EnhancedConnectionConfig {
  host: string;
  port: number;
  secure: boolean;
  token?: string;
  username?: string;
  password?: string;
  name?: string;
  timeout?: number;
  retryAttempts?: number;
  retryDelay?: number;
  compression?: boolean;
  keepAlive?: boolean;
  autoReconnect?: boolean;
  maxConcurrentRequests?: number;
  cacheEnabled?: boolean;
  cacheTTL?: number;
}

export interface SystemMetrics {
  cpu: {
    usage: number;
    cores: number;
    frequency: number;
    temperature?: number;
  };
  memory: {
    total: number;
    used: number;
    available: number;
    percentage: number;
  };
  disk: {
    total: number;
    used: number;
    available: number;
    percentage: number;
  };
  network: {
    bytesReceived: number;
    bytesSent: number;
    packetsReceived: number;
    packetsSent: number;
  };
  uptime: number;
  timestamp: number;
}

export interface Process {
  pid: number;
  name: string;
  cpu: number;
  memory: number;
  status: string;
  user: string;
  command: string;
  startTime: number;
}

export interface FileItem {
  name: string;
  path: string;
  type: 'file' | 'directory';
  size: number;
  modified: number;
  permissions: string;
  owner: string;
  group: string;
}

export interface TerminalSession {
  id: string;
  title: string;
  active: boolean;
  history: CommandEntry[];
  currentDirectory: string;
  environment: Record<string, string>;
}

export interface CommandEntry {
  id: string;
  command: string;
  output: string;
  exitCode: number;
  timestamp: number;
  duration: number;
}

interface CacheEntry {
  data: any;
  timestamp: number;
  ttl: number;
}

interface NetworkQueueItem {
  id: string;
  request: () => Promise<any>;
  resolve: (value: any) => void;
  reject: (reason: any) => void;
  priority: number;
  retryCount: number;
}

export class EnhancedConnectionManager extends EventEmitter {
  private static instance: EnhancedConnectionManager;
  private httpClient: AxiosInstance | null = null;
  private wsClient: Socket | null = null;
  private config: EnhancedConnectionConfig | null = null;
  private connectionStatus: 'disconnected' | 'connecting' | 'connected' | 'error' = 'disconnected';
  private reconnectTimer: NodeJS.Timeout | null = null;
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private cache = new Map<string, CacheEntry>();
  private requestQueue: NetworkQueueItem[] = [];
  private activeRequests = 0;
  private networkState: any = null;
  
  // Performance metrics
  private metrics = {
    requestCount: 0,
    cacheHits: 0,
    cacheMisses: 0,
    averageResponseTime: 0,
    errorCount: 0,
  };

  constructor() {
    super();
    this.setupNetworkListener();
  }

  static getInstance(): EnhancedConnectionManager {
    if (!EnhancedConnectionManager.instance) {
      EnhancedConnectionManager.instance = new EnhancedConnectionManager();
    }
    return EnhancedConnectionManager.instance;
  }

  private setupNetworkListener() {
    NetInfo.addEventListener(state => {
      const wasConnected = this.networkState?.isConnected;
      this.networkState = state;
      
      if (!wasConnected && state.isConnected && this.config?.autoReconnect) {
        this.reconnect();
      } else if (wasConnected && !state.isConnected) {
        this.emit('network_disconnected');
      }
    });
  }

  async connect(config: EnhancedConnectionConfig): Promise<boolean> {
    this.config = { 
      timeout: 30000,
      retryAttempts: 3,
      retryDelay: 1000,
      compression: true,
      keepAlive: true,
      autoReconnect: true,
      maxConcurrentRequests: 5,
      cacheEnabled: true,
      cacheTTL: 300000, // 5 minutes
      ...config 
    };

    this.connectionStatus = 'connecting';
    this.emit('connectionStatusChanged', 'connecting');

    try {
      // Save configuration
      await AsyncStorage.setItem('connectionConfig', JSON.stringify(this.config));

      // Initialize HTTP client with advanced configuration
      this.httpClient = axios.create({
        baseURL: `${config.secure ? 'https' : 'http'}://${config.host}:${config.port}`,
        timeout: this.config.timeout,
        headers: {
          'Content-Type': 'application/json',
          'User-Agent': `NexusTerminal-Mobile/${Platform.OS}`,
          ...(this.config.token && { 'Authorization': `Bearer ${this.config.token}` }),
        },
        maxRedirects: 5,
        validateStatus: status => status < 500,
        ...(this.config.compression && {
          decompress: true,
          headers: { 'Accept-Encoding': 'gzip, deflate, br' }
        }),
      });

      // Add request/response interceptors for enhanced functionality
      this.setupHttpInterceptors();

      // Test HTTP connection
      const testResponse = await this.httpClient.get('/api/health');
      if (testResponse.status !== 200) {
        throw new Error('Health check failed');
      }

      // Initialize WebSocket connection with advanced options
      this.wsClient = io(`${config.secure ? 'wss' : 'ws'}://${config.host}:${config.port}`, {
        auth: {
          token: this.config.token,
          username: this.config.username,
          password: this.config.password,
        },
        timeout: this.config.timeout,
        reconnection: this.config.autoReconnect,
        reconnectionAttempts: this.config.retryAttempts,
        reconnectionDelay: this.config.retryDelay,
        compression: this.config.compression,
        forceNew: true,
        upgrade: true,
        transports: ['websocket', 'polling'],
      });

      this.setupWebSocketHandlers();

      return new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          reject(new Error('WebSocket connection timeout'));
        }, this.config!.timeout);

        this.wsClient!.on('connect', () => {
          clearTimeout(timeout);
          this.connectionStatus = 'connected';
          this.emit('connectionStatusChanged', 'connected');
          this.startHeartbeat();
          this.processRequestQueue();
          resolve(true);
        });

        this.wsClient!.on('connect_error', (error) => {
          clearTimeout(timeout);
          this.connectionStatus = 'error';
          this.emit('connectionStatusChanged', 'error');
          this.emit('connectionError', error.message);
          reject(error);
        });
      });

    } catch (error: any) {
      this.connectionStatus = 'error';
      this.emit('connectionStatusChanged', 'error');
      this.emit('connectionError', error.message);
      return false;
    }
  }

  private setupHttpInterceptors() {
    if (!this.httpClient) return;

    // Request interceptor for performance tracking and caching
    this.httpClient.interceptors.request.use(
      (config) => {
        config.metadata = { startTime: Date.now() };
        this.metrics.requestCount++;
        return config;
      },
      (error) => {
        this.metrics.errorCount++;
        return Promise.reject(error);
      }
    );

    // Response interceptor for performance tracking and caching
    this.httpClient.interceptors.response.use(
      (response) => {
        const endTime = Date.now();
        const startTime = response.config.metadata?.startTime || endTime;
        const duration = endTime - startTime;
        
        this.updateAverageResponseTime(duration);
        
        // Cache GET responses if caching is enabled
        if (this.config?.cacheEnabled && response.config.method === 'get') {
          this.setCacheEntry(response.config.url!, response.data);
        }
        
        return response;
      },
      (error) => {
        this.metrics.errorCount++;
        return Promise.reject(error);
      }
    );
  }

  private setupWebSocketHandlers() {
    if (!this.wsClient) return;

    this.wsClient.on('connect', () => {
      this.connectionStatus = 'connected';
      this.emit('connectionStatusChanged', 'connected');
    });

    this.wsClient.on('disconnect', (reason) => {
      this.connectionStatus = 'disconnected';
      this.emit('connectionStatusChanged', 'disconnected');
      this.emit('disconnected', reason);
      
      if (this.config?.autoReconnect && reason === 'io server disconnect') {
        this.scheduleReconnect();
      }
    });

    this.wsClient.on('reconnect', () => {
      this.emit('reconnected');
    });

    this.wsClient.on('error', (error) => {
      this.emit('connectionError', error);
    });

    // System event handlers
    this.wsClient.on('system_metrics', (data: SystemMetrics) => {
      this.emit('systemMetrics', data);
    });

    this.wsClient.on('process_update', (data: Process[]) => {
      this.emit('processUpdate', data);
    });

    this.wsClient.on('file_change', (data: any) => {
      this.emit('fileChange', data);
    });

    this.wsClient.on('terminal_output', (data: any) => {
      this.emit('terminalOutput', data);
    });
  }

  private startHeartbeat() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
    }

    this.heartbeatInterval = setInterval(() => {
      if (this.wsClient?.connected) {
        this.wsClient.emit('ping', Date.now());
      }
    }, 30000); // 30 second heartbeat
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;

    this.reconnectTimer = setTimeout(() => {
      this.reconnect();
      this.reconnectTimer = null;
    }, this.config?.retryDelay || 5000);
  }

  async reconnect(): Promise<boolean> {
    if (!this.config) return false;
    
    this.emit('reconnecting');
    return this.connect(this.config);
  }

  // Enhanced request method with queueing and caching
  async request<T>(
    endpoint: string, 
    options: AxiosRequestConfig & { 
      priority?: number;
      useCache?: boolean;
      cacheKey?: string;
    } = {}
  ): Promise<T> {
    const { priority = 1, useCache = true, cacheKey, ...axiosConfig } = options;
    const fullUrl = endpoint;
    const finalCacheKey = cacheKey || fullUrl;

    // Check cache first
    if (useCache && this.config?.cacheEnabled && axiosConfig.method === 'GET') {
      const cached = this.getCacheEntry(finalCacheKey);
      if (cached) {
        this.metrics.cacheHits++;
        return cached;
      }
      this.metrics.cacheMisses++;
    }

    // Queue request if at max concurrent requests
    if (this.activeRequests >= (this.config?.maxConcurrentRequests || 5)) {
      return new Promise((resolve, reject) => {
        this.requestQueue.push({
          id: Date.now().toString(),
          request: () => this.executeRequest<T>(endpoint, axiosConfig),
          resolve,
          reject,
          priority,
          retryCount: 0,
        });
        
        // Sort queue by priority
        this.requestQueue.sort((a, b) => b.priority - a.priority);
      });
    }

    return this.executeRequest<T>(endpoint, axiosConfig);
  }

  private async executeRequest<T>(endpoint: string, config: AxiosRequestConfig): Promise<T> {
    if (!this.httpClient) {
      throw new Error('Not connected');
    }

    this.activeRequests++;

    try {
      const response = await this.httpClient.request<T>({
        url: endpoint,
        ...config,
      });

      return response.data;
    } finally {
      this.activeRequests--;
      this.processRequestQueue();
    }
  }

  private processRequestQueue() {
    while (this.requestQueue.length > 0 && this.activeRequests < (this.config?.maxConcurrentRequests || 5)) {
      const item = this.requestQueue.shift();
      if (item) {
        item.request()
          .then(item.resolve)
          .catch(item.reject);
      }
    }
  }

  // Cache management
  private setCacheEntry(key: string, data: any) {
    if (!this.config?.cacheEnabled) return;

    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl: this.config.cacheTTL || 300000,
    });

    // Clean up expired entries periodically
    if (this.cache.size > 100) {
      this.cleanupCache();
    }
  }

  private getCacheEntry(key: string): any | null {
    if (!this.config?.cacheEnabled) return null;

    const entry = this.cache.get(key);
    if (!entry) return null;

    if (Date.now() - entry.timestamp > entry.ttl) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  private cleanupCache() {
    const now = Date.now();
    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp > entry.ttl) {
        this.cache.delete(key);
      }
    }
  }

  clearCache() {
    this.cache.clear();
  }

  // Enhanced API methods with better error handling and caching
  async getSystemInfo(): Promise<any> {
    return this.request('/api/system/info', { 
      useCache: true,
      cacheKey: 'system_info',
    });
  }

  async getSystemMetrics(): Promise<SystemMetrics> {
    return this.request('/api/system/metrics', {
      useCache: false, // Always get fresh metrics
    });
  }

  async getProcessList(): Promise<Process[]> {
    return this.request('/api/system/processes', {
      useCache: true,
      cacheKey: 'processes',
    });
  }

  async executeCommand(command: string, sessionId?: string): Promise<any> {
    return this.request('/api/terminal/execute', {
      method: 'POST',
      data: { command, sessionId },
      priority: 2, // High priority for terminal commands
    });
  }

  async browseFiles(path: string): Promise<FileItem[]> {
    return this.request(`/api/files/browse?path=${encodeURIComponent(path)}`, {
      useCache: true,
      cacheKey: `files_${path}`,
    });
  }

  async uploadFile(file: FormData, path: string): Promise<any> {
    return this.request('/api/files/upload', {
      method: 'POST',
      data: file,
      headers: { 'Content-Type': 'multipart/form-data' },
      priority: 3, // High priority for file operations
    });
  }

  async downloadFile(path: string): Promise<Blob> {
    return this.request(`/api/files/download?path=${encodeURIComponent(path)}`, {
      responseType: 'blob',
      priority: 3,
    });
  }

  // Performance and diagnostic methods
  getPerformanceMetrics() {
    return {
      ...this.metrics,
      cacheSize: this.cache.size,
      queueSize: this.requestQueue.length,
      activeRequests: this.activeRequests,
      connectionStatus: this.connectionStatus,
    };
  }

  private updateAverageResponseTime(duration: number) {
    this.metrics.averageResponseTime = 
      (this.metrics.averageResponseTime * (this.metrics.requestCount - 1) + duration) / 
      this.metrics.requestCount;
  }

  // Enhanced WebSocket methods
  emitWithAck(event: string, data: any, timeout = 10000): Promise<any> {
    return new Promise((resolve, reject) => {
      if (!this.wsClient?.connected) {
        reject(new Error('WebSocket not connected'));
        return;
      }

      const timer = setTimeout(() => {
        reject(new Error('WebSocket acknowledgment timeout'));
      }, timeout);

      this.wsClient.emit(event, data, (response: any) => {
        clearTimeout(timer);
        resolve(response);
      });
    });
  }

  // Connection management
  isConnected(): boolean {
    return this.connectionStatus === 'connected' && 
           this.httpClient !== null && 
           this.wsClient?.connected === true;
  }

  getConnectionStatus() {
    return this.connectionStatus;
  }

  disconnect() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.wsClient) {
      this.wsClient.disconnect();
      this.wsClient = null;
    }

    this.httpClient = null;
    this.connectionStatus = 'disconnected';
    this.emit('connectionStatusChanged', 'disconnected');
    this.clearCache();
  }

  // Auto-discovery methods
  async discoverServers(): Promise<EnhancedConnectionConfig[]> {
    const discoveries: EnhancedConnectionConfig[] = [];
    const subnet = await this.getLocalSubnet();
    
    if (!subnet) return discoveries;

    const promises = [];
    for (let i = 1; i <= 254; i++) {
      const host = `${subnet}.${i}`;
      promises.push(this.testHost(host));
    }

    const results = await Promise.allSettled(promises);
    results.forEach((result, index) => {
      if (result.status === 'fulfilled' && result.value) {
        discoveries.push(result.value);
      }
    });

    return discoveries;
  }

  private async getLocalSubnet(): Promise<string | null> {
    try {
      const networkState = await NetInfo.fetch();
      if (networkState.details && 'ipAddress' in networkState.details) {
        const ip = networkState.details.ipAddress as string;
        const parts = ip.split('.');
        return `${parts[0]}.${parts[1]}.${parts[2]}`;
      }
    } catch (error) {
      console.error('Failed to get local subnet:', error);
    }
    return null;
  }

  private async testHost(host: string): Promise<EnhancedConnectionConfig | null> {
    try {
      const response = await axios.get(`http://${host}:8080/api/health`, {
        timeout: 2000,
      });
      
      if (response.status === 200) {
        return {
          host,
          port: 8080,
          secure: false,
          name: `Nexus Terminal (${host})`,
        };
      }
    } catch (error) {
      // Host not available or not running Nexus Terminal
    }
    
    return null;
  }
}

// Export singleton instance
export const ConnectionManager = EnhancedConnectionManager.getInstance();
