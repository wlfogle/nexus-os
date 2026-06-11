import React, { useState } from 'react';
import './App.css';

function App() {
 // State Management
 const [currentPage, setCurrentPage] = useState<string>('dashboard');
 const [isLoading, setIsLoading] = useState(true);
 const [systemMetrics, setSystemMetrics] = useState<unknown | null>(null);
 const [aiInsights, setAiInsights] = useState<unknown[]>([]);
 const [hardwareStatus, setHardwareStatus] = useState<unknown | null>(null);
 const [notifications, setNotifications] = useState<string[]>([]);
 const [darkMode, setDarkMode] = useState(true);

 // Initialize app and set up data fetching
 useEffect(() => {
 initializeApp();
 const interval = setInterval(fetchSystemData, 5000); // Fetch data every 5 seconds
 
 return () => clearInterval(interval);
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
 const metrics: unknown = await fetch('/api/system-metrics');
 setSystemMetrics(metrics);

 // Fetch AI insights
 const insights: unknown[] = await fetch('/api/ai-insights');
 setAiInsights(insights);

 // Fetch hardware status
 const hardware: unknown = await fetch('/api/hardware-status');
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
 const result: string = await fetch('/api/system-update');
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
 const result: string = await fetch('/api/system-clean');
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
 const result: string = await fetch('/api/backup', { destination });
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
 return <Dashboard metrics={systemMetrics} insights={aiInsights} hardwareStatus={hardwareStatus} />;
 case 'monitor':
 return <SystemMonitor metrics={systemMetrics} />;
 case 'hardware':
 return <HardwareControl hardwareStatus={hardwareStatus} />;
 case 'ai':
 return <AIInsights insights={aiInsights} />;
 case 'settings':
 return <Settings darkMode={darkMode} setDarkMode={setDarkMode} />;
 default:
 return <Dashboard metrics={systemMetrics} insights={aiInsights} hardwareStatus={hardwareStatus} />;
 }
 };

 if (isLoading && !systemMetrics) {
 return (
 <div className="flex items-center justify-center min-h-screen bg-gray-900">
 <span className="text-white text-xl uppercase font-bold tracking-wider text-4xl">
 Loading...
 </span>
 </div>
 );
 }

 return (
 <div className={`app ${darkMode ? 'dark' : 'light'} min-h-screen bg-gray-900`}>
 {/* Sidebar */}
 <div className="sidebar p-6 w-1/5">
 <ul className="flex flex-col space-y-4">
 {[
 { title: 'Dashboard', url: '/dashboard' },
 { title: 'Monitor', url: '/monitor' },
 { title: 'Hardware', url: '/hardware' },
 { title: 'AI', url: '/ai' },
 { title: 'Settings', url: '/settings' }
 ].map(item => (
 <li key={item.title}>
 <a className={`block p-4 rounded-lg shadow-md transition-colors duration-200 ease-linear hover:bg-gray-100 ${currentPage === item.url ? 'bg-gray-100' : ''}`} onClick={() => setCurrentPage(item.url)}>
 {item.title}
 </a>
 </li>
 ))}
 </ul>
 </div>

 {/* Main Content */}
 <main className="w-4/5 p-6">
 {renderCurrentPage()}
 </main>
 </div>
 );
}

export default App;