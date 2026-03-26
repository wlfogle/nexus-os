import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';
import { Dashboard } from './pages/Dashboard';
import { SystemMonitor } from './pages/SystemMonitor';
import { HardwareControl } from './pages/HardwareControl';
import { AIInsights } from './pages/AIInsights';
import { Settings } from './pages/Settings';
import { Sidebar } from './components/Sidebar';
import { TopBar } from './components/TopBar';
import { LoadingSpinner } from './components/LoadingSpinner';
import './App.css';

// Types
interface SystemMetrics {
  timestamp: string;
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  network_rx: number;
  network_tx: number;
  temperature: number;
  processes: number;
  uptime: number;
}

interface AIInsight {
  pattern: string;
  confidence: number;
  recommendation: string;
  priority: number;
  timestamp: string;
}

interface HardwareStatus {
  cpu_temps: number[];
  fan_speeds: number[];
  cpu_frequencies: number[];
  gpu_usage: number | null;
  power_consumption: number | null;
}

export type Page = 'dashboard' | 'monitor' | 'hardware' | 'ai' | 'settings';

function App() {
  // State Management
  const [currentPage, setCurrentPage] = useState<Page>('dashboard');
  const [isLoading, setIsLoading] = useState(true);
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics | null>(null);
  const [aiInsights, setAiInsights] = useState<AIInsight[]>([]);
  const [hardwareStatus, setHardwareStatus] = useState<HardwareStatus | null>(null);
  const [notifications, setNotifications] = useState<string[]>([]);
  const [darkMode, setDarkMode] = useState(true);

  // Initialize app and set up data fetching
  useEffect(() => {
    initializeApp();
    const interval = setInterval(fetchSystemData, 5000); // Fetch data every 5 seconds
    
    return () => clearInterval(interval);
  }, []);

  // Set up window event listeners
  useEffect(() => {
    const setupEventListeners = async () => {
      // Listen for system tray events
      await listen('system_tray_click', (event) => {
        console.log('System tray clicked:', event);
      });

      // Listen for AI recommendations
      await listen('ai_recommendation', (event) => {
        console.log('New AI recommendation:', event);
        addNotification('New AI recommendation available');
      });
    };

    setupEventListeners();
  }, []);

  const initializeApp = async () => {
    setIsLoading(true);
    try {
      await fetchSystemData();
      console.log('Lou\'s Garuda AI SysAdmin Control Center initialized successfully');
    } catch (error) {
      console.error('Failed to initialize app:', error);
      addNotification('Failed to initialize system monitoring');
    } finally {
      setIsLoading(false);
    }
  };

  const fetchSystemData = async () => {
    try {
      // Fetch system metrics
      const metrics: SystemMetrics = await invoke('get_system_metrics');
      setSystemMetrics(metrics);

      // Fetch AI insights
      const insights: AIInsight[] = await invoke('get_ai_recommendations');
      setAiInsights(insights);

      // Fetch hardware status
      const hardware: HardwareStatus = await invoke('get_hardware_status');
      setHardwareStatus(hardware);

    } catch (error) {
      console.error('Failed to fetch system data:', error);
    }
  };

  const addNotification = (message: string) => {
    setNotifications(prev => [...prev, message]);
    // Auto-remove notification after 5 seconds
    setTimeout(() => {
      setNotifications(prev => prev.slice(1));
    }, 5000);
  };

  const handleSystemUpdate = async () => {
    try {
      setIsLoading(true);
      const result: string = await invoke('update_system');
      addNotification('System update completed successfully');
      console.log('System update result:', result);
    } catch (error) {
      console.error('System update failed:', error);
      addNotification('System update failed');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSystemClean = async () => {
    try {
      setIsLoading(true);
      const result: string = await invoke('clean_system');
      addNotification('System cleanup completed');
      console.log('System clean result:', result);
    } catch (error) {
      console.error('System cleanup failed:', error);
      addNotification('System cleanup failed');
    } finally {
      setIsLoading(false);
    }
  };

  const handleBackup = async (destination: string) => {
    try {
      setIsLoading(true);
      const result: string = await invoke('create_backup', { destination });
      addNotification('Backup created successfully');
      console.log('Backup result:', result);
    } catch (error) {
      console.error('Backup failed:', error);
      addNotification('Backup failed');
    } finally {
      setIsLoading(false);
    }
  };

  const renderCurrentPage = () => {
    switch (currentPage) {
      case 'dashboard':
        return (
          <Dashboard
            metrics={systemMetrics}
            insights={aiInsights}
            hardwareStatus={hardwareStatus}
            onSystemUpdate={handleSystemUpdate}
            onSystemClean={handleSystemClean}
            onBackup={handleBackup}
          />
        );
      case 'monitor':
        return <SystemMonitor metrics={systemMetrics} />;
      case 'hardware':
        return <HardwareControl hardwareStatus={hardwareStatus} />;
      case 'ai':
        return <AIInsights insights={aiInsights} />;
      case 'settings':
        return <Settings darkMode={darkMode} setDarkMode={setDarkMode} />;
      default:
        return (
          <Dashboard
            metrics={systemMetrics}
            insights={aiInsights}
            hardwareStatus={hardwareStatus}
            onSystemUpdate={handleSystemUpdate}
            onSystemClean={handleSystemClean}
            onBackup={handleBackup}
          />
        );
    }
  };

  if (isLoading && !systemMetrics) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900">
        <LoadingSpinner size="large" />
        <span className="ml-4 text-white text-xl">
          Initializing AI SysAdmin Control Center...
        </span>
      </div>
    );
  }

  return (
    <div className={`app ${darkMode ? 'dark' : 'light'} min-h-screen bg-gray-900 text-white flex`}>
      <Sidebar currentPage={currentPage} onPageChange={setCurrentPage} />
      
      <div className="flex-1 flex flex-col">
        <TopBar 
          notifications={notifications}
          systemMetrics={systemMetrics}
          onRefresh={fetchSystemData}
          isLoading={isLoading}
        />
        
        <main className="flex-1 p-6 overflow-auto">
          {renderCurrentPage()}
        </main>
      </div>

      {/* Global Loading Overlay */}
      {isLoading && systemMetrics && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-gray-800 p-6 rounded-lg flex items-center">
            <LoadingSpinner size="medium" />
            <span className="ml-4 text-white">Processing...</span>
          </div>
        </div>
      )}

      {/* Notifications */}
      <div className="fixed top-4 right-4 space-y-2 z-40">
        {notifications.map((notification, index) => (
          <div
            key={index}
            className="bg-blue-600 text-white px-4 py-2 rounded-lg shadow-lg animate-fade-in"
          >
            {notification}
          </div>
        ))}
      </div>
    </div>
  );
}

export default App;
