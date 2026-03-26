import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_usage: number;
  memory_percent: number;
  status: string;
  command: string;
}

interface NetworkInterface {
  name: string;
  bytes_received: number;
  bytes_transmitted: number;
  packets_received: number;
  packets_transmitted: number;
  errors_received: number;
  errors_transmitted: number;
}

interface ThermalZone {
  name: string;
  temperature: number;
  critical_temp: number;
  sensor_type: string;
}

interface SystemMonitorProps {
  metrics: any;
}

export const SystemMonitor: React.FC<SystemMonitorProps> = ({ metrics }) => {
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [networkInterfaces, setNetworkInterfaces] = useState<NetworkInterface[]>([]);
  const [thermalZones, setThermalZones] = useState<ThermalZone[]>([]);
  const [historicalData, setHistoricalData] = useState<any[]>([]);
  const [refreshRate, setRefreshRate] = useState(5); // seconds
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    fetchDetailedSystemData();
    const interval = setInterval(fetchDetailedSystemData, refreshRate * 1000);
    return () => clearInterval(interval);
  }, [refreshRate]);

  const fetchDetailedSystemData = async () => {
    try {
      setIsLoading(true);
      
      // Fetch processes
      const processData: ProcessInfo[] = await invoke('get_process_list');
      setProcesses(processData);
      
      // Fetch network interfaces
      const networkData: NetworkInterface[] = await invoke('get_network_interfaces');
      setNetworkInterfaces(networkData);
      
      // Fetch thermal zones
      const thermalData: ThermalZone[] = await invoke('get_thermal_zones');
      setThermalZones(thermalData);
      
      // Fetch historical metrics
      const histData: any[] = await invoke('get_historical_metrics', { limit: 60 });
      setHistoricalData(histData);
      
    } catch (error) {
      console.error('Failed to fetch detailed system data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatUptime = (seconds: number): string => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${days}d ${hours}h ${minutes}m`;
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">🖥️ System Monitor</h1>
        <div className="flex items-center space-x-4">
          <select 
            value={refreshRate} 
            onChange={(e) => setRefreshRate(Number(e.target.value))}
            className="bg-gray-700 text-white px-3 py-1 rounded border border-gray-600"
          >
            <option value={1}>1 second</option>
            <option value={2}>2 seconds</option>
            <option value={5}>5 seconds</option>
            <option value={10}>10 seconds</option>
          </select>
          <button 
            onClick={fetchDetailedSystemData}
            className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded transition-colors"
            disabled={isLoading}
          >
            {isLoading ? '🔄 Refreshing...' : '🔄 Refresh'}
          </button>
        </div>
      </div>

      {/* System Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
          <h3 className="text-lg font-semibold mb-2 text-blue-400">💻 CPU Status</h3>
          <div className="space-y-2">
            <p className="text-2xl font-mono">{metrics?.cpu_usage?.toFixed(1) || '0'}%</p>
            <p className="text-sm text-gray-400">Usage</p>
            <p className="text-sm">🌡️ {metrics?.cpu_temp?.toFixed(1) || '0'}°C</p>
            <p className="text-sm">⚡ {metrics?.cpu_freq || '0'} MHz</p>
          </div>
        </div>

        <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
          <h3 className="text-lg font-semibold mb-2 text-green-400">🧠 Memory Status</h3>
          <div className="space-y-2">
            <p className="text-2xl font-mono">{metrics?.memory_usage?.toFixed(1) || '0'}%</p>
            <p className="text-sm text-gray-400">Usage</p>
            <p className="text-sm">{formatBytes(metrics?.memory_total || 0)} Total</p>
            <p className="text-sm">{formatBytes(metrics?.memory_available || 0)} Available</p>
          </div>
        </div>

        <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
          <h3 className="text-lg font-semibold mb-2 text-purple-400">💾 Storage</h3>
          <div className="space-y-2">
            <p className="text-2xl font-mono">{Object.values(metrics?.disk_usage || {}).map((disk: any) => disk.usage_percent)[0]?.toFixed(1) || '0'}%</p>
            <p className="text-sm text-gray-400">Usage</p>
            <p className="text-sm">📊 {Object.keys(metrics?.disk_usage || {}).length} Disks</p>
          </div>
        </div>

        <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
          <h3 className="text-lg font-semibold mb-2 text-yellow-400">⏱️ System Info</h3>
          <div className="space-y-2">
            <p className="text-sm">{formatUptime(metrics?.uptime || 0)}</p>
            <p className="text-sm text-gray-400">Uptime</p>
            <p className="text-sm">🔢 {processes.length} Processes</p>
            <p className="text-sm">🌐 {networkInterfaces.length} Networks</p>
          </div>
        </div>
      </div>

      {/* Process Monitor */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-blue-400">⚙️ Process Monitor</h3>
          <p className="text-sm text-gray-400 mt-1">Top CPU and memory consuming processes</p>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="bg-gray-700">
              <tr>
                <th className="px-4 py-3 text-left">PID</th>
                <th className="px-4 py-3 text-left">Process Name</th>
                <th className="px-4 py-3 text-left">CPU %</th>
                <th className="px-4 py-3 text-left">Memory %</th>
                <th className="px-4 py-3 text-left">Memory</th>
                <th className="px-4 py-3 text-left">Status</th>
              </tr>
            </thead>
            <tbody>
              {processes.slice(0, 15).map((process, index) => (
                <tr key={process.pid} className={index % 2 === 0 ? 'bg-gray-800' : 'bg-gray-750'}>
                  <td className="px-4 py-2 font-mono">{process.pid}</td>
                  <td className="px-4 py-2 font-medium">{process.name}</td>
                  <td className="px-4 py-2 font-mono">
                    <span className={process.cpu_usage > 50 ? 'text-red-400' : process.cpu_usage > 20 ? 'text-yellow-400' : 'text-green-400'}>
                      {process.cpu_usage.toFixed(1)}%
                    </span>
                  </td>
                  <td className="px-4 py-2 font-mono">
                    <span className={process.memory_percent > 10 ? 'text-red-400' : process.memory_percent > 5 ? 'text-yellow-400' : 'text-green-400'}>
                      {process.memory_percent.toFixed(1)}%
                    </span>
                  </td>
                  <td className="px-4 py-2 font-mono text-gray-400">{formatBytes(process.memory_usage)}</td>
                  <td className="px-4 py-2">
                    <span className={`px-2 py-1 rounded text-xs ${
                      process.status === 'Running' ? 'bg-green-600' :
                      process.status === 'Sleeping' ? 'bg-blue-600' :
                      process.status === 'Stopped' ? 'bg-red-600' : 'bg-gray-600'
                    }`}>
                      {process.status}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Network Monitor */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-green-400">🌐 Network Monitor</h3>
          <p className="text-sm text-gray-400 mt-1">Network interface statistics and activity</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {networkInterfaces.map((iface) => (
              <div key={iface.name} className="bg-gray-700 p-4 rounded">
                <h4 className="font-semibold mb-3">{iface.name}</h4>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div>
                    <p className="text-gray-400">📥 Received</p>
                    <p className="font-mono">{formatBytes(iface.bytes_received)}</p>
                    <p className="text-xs text-gray-500">{iface.packets_received.toLocaleString()} packets</p>
                  </div>
                  <div>
                    <p className="text-gray-400">📤 Transmitted</p>
                    <p className="font-mono">{formatBytes(iface.bytes_transmitted)}</p>
                    <p className="text-xs text-gray-500">{iface.packets_transmitted.toLocaleString()} packets</p>
                  </div>
                  {(iface.errors_received > 0 || iface.errors_transmitted > 0) && (
                    <div className="col-span-2 mt-2 text-red-400">
                      <p className="text-xs">⚠️ Errors: RX {iface.errors_received}, TX {iface.errors_transmitted}</p>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Thermal Monitor */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-red-400">🌡️ Thermal Monitor</h3>
          <p className="text-sm text-gray-400 mt-1">Temperature sensors and thermal management</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {thermalZones.map((zone) => (
              <div key={zone.name} className="bg-gray-700 p-4 rounded">
                <h4 className="font-semibold mb-2">{zone.name}</h4>
                <div className="space-y-2">
                  <div className="flex justify-between items-center">
                    <span className="text-2xl font-mono">
                      <span className={zone.temperature > 80 ? 'text-red-400' : zone.temperature > 65 ? 'text-yellow-400' : 'text-green-400'}>
                        {zone.temperature.toFixed(1)}°C
                      </span>
                    </span>
                    <span className="text-xs bg-gray-600 px-2 py-1 rounded">{zone.sensor_type}</span>
                  </div>
                  <div className="w-full bg-gray-600 rounded-full h-2">
                    <div 
                      className={`h-2 rounded-full transition-all duration-300 ${
                        zone.temperature > 80 ? 'bg-red-500' : 
                        zone.temperature > 65 ? 'bg-yellow-500' : 'bg-green-500'
                      }`}
                      style={{ width: `${Math.min((zone.temperature / zone.critical_temp) * 100, 100)}%` }}
                    ></div>
                  </div>
                  <p className="text-xs text-gray-400">Critical: {zone.critical_temp.toFixed(1)}°C</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* System Resources Chart */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-purple-400">📊 Resource History</h3>
          <p className="text-sm text-gray-400 mt-1">Historical system resource usage (last hour)</p>
        </div>
        <div className="p-6">
          {historicalData.length > 0 ? (
            <div className="space-y-4">
              {/* Simple text-based chart representation */}
              <div className="space-y-2">
                <h4 className="font-semibold text-blue-400">CPU Usage Trend</h4>
                <div className="flex items-end space-x-1 h-20">
                  {historicalData.slice(-30).map((data, index) => (
                    <div key={index} className="flex-1">
                      <div 
                        className="bg-blue-500 w-full rounded-t transition-all duration-200"
                        style={{ height: `${(data.cpu_usage || 0) * 0.8}px` }}
                        title={`${data.cpu_usage?.toFixed(1)}% at ${new Date(data.timestamp * 1000).toLocaleTimeString()}`}
                      ></div>
                    </div>
                  ))}
                </div>
                <div className="text-xs text-gray-400 text-center">Last 30 samples</div>
              </div>
              
              <div className="space-y-2">
                <h4 className="font-semibold text-green-400">Memory Usage Trend</h4>
                <div className="flex items-end space-x-1 h-20">
                  {historicalData.slice(-30).map((data, index) => (
                    <div key={index} className="flex-1">
                      <div 
                        className="bg-green-500 w-full rounded-t transition-all duration-200"
                        style={{ height: `${(data.memory_usage || 0) * 0.8}px` }}
                        title={`${data.memory_usage?.toFixed(1)}% at ${new Date(data.timestamp * 1000).toLocaleTimeString()}`}
                      ></div>
                    </div>
                  ))}
                </div>
                <div className="text-xs text-gray-400 text-center">Last 30 samples</div>
              </div>
            </div>
          ) : (
            <div className="text-center text-gray-400 py-8">
              <p>No historical data available yet</p>
              <p className="text-sm mt-2">Data will appear after a few monitoring cycles</p>
            </div>
          )}
        </div>
      </div>

      {/* System Load and Performance */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-gray-800 rounded-lg border border-gray-700">
          <div className="p-6 border-b border-gray-700">
            <h3 className="text-lg font-semibold text-orange-400">⚡ System Load</h3>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <span>1 minute:</span>
                <span className="font-mono text-lg">{metrics?.system_load?.[0]?.toFixed(2) || '0.00'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span>5 minutes:</span>
                <span className="font-mono text-lg">{metrics?.system_load?.[1]?.toFixed(2) || '0.00'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span>15 minutes:</span>
                <span className="font-mono text-lg">{metrics?.system_load?.[2]?.toFixed(2) || '0.00'}</span>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg border border-gray-700">
          <div className="p-6 border-b border-gray-700">
            <h3 className="text-lg font-semibold text-indigo-400">🎮 GPU Status</h3>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <span>Usage:</span>
                <span className="font-mono text-lg">{metrics?.gpu_usage?.toFixed(1) || '0'}%</span>
              </div>
              <div className="flex justify-between items-center">
                <span>Temperature:</span>
                <span className="font-mono text-lg">{metrics?.gpu_temp?.toFixed(1) || 'N/A'}°C</span>
              </div>
              <div className="flex justify-between items-center">
                <span>Memory:</span>
                <span className="font-mono text-lg">{formatBytes(metrics?.gpu_memory || 0)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Performance Summary */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-cyan-400">📈 Performance Summary</h3>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="text-center">
              <p className="text-2xl font-bold text-blue-400">{metrics?.cpu_usage?.toFixed(0) || '0'}%</p>
              <p className="text-sm text-gray-400">Average CPU</p>
              <div className="mt-2 w-full bg-gray-700 rounded-full h-2">
                <div 
                  className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${metrics?.cpu_usage || 0}%` }}
                ></div>
              </div>
            </div>
            
            <div className="text-center">
              <p className="text-2xl font-bold text-green-400">{metrics?.memory_usage?.toFixed(0) || '0'}%</p>
              <p className="text-sm text-gray-400">Memory Usage</p>
              <div className="mt-2 w-full bg-gray-700 rounded-full h-2">
                <div 
                  className="bg-green-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${metrics?.memory_usage || 0}%` }}
                ></div>
              </div>
            </div>
            
            <div className="text-center">
              <p className="text-2xl font-bold text-red-400">{metrics?.cpu_temp?.toFixed(0) || '0'}°C</p>
              <p className="text-sm text-gray-400">CPU Temperature</p>
              <div className="mt-2 w-full bg-gray-700 rounded-full h-2">
                <div 
                  className={`h-2 rounded-full transition-all duration-300 ${
                    (metrics?.cpu_temp || 0) > 80 ? 'bg-red-500' :
                    (metrics?.cpu_temp || 0) > 65 ? 'bg-yellow-500' : 'bg-green-500'
                  }`}
                  style={{ width: `${Math.min((metrics?.cpu_temp || 0) / 100 * 100, 100)}%` }}
                ></div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
