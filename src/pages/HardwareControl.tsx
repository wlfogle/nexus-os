import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface HardwareProfile {
  name: string;
  power_mode: string;
  cpu_governor: string;
  gpu_power_level: string;
  fan_profile: string;
  thermal_throttle_temp: number;
  overclock_enabled: boolean;
  rgb_profile?: string;
}

interface FanStatus {
  name: string;
  rpm: number;
  pwm: number;
  auto: boolean;
}

interface HardwareControlProps {
  hardwareStatus: any;
}

export const HardwareControl: React.FC<HardwareControlProps> = ({ hardwareStatus }) => {
  const [profiles, setProfiles] = useState<HardwareProfile[]>([]);
  const [activeProfile, setActiveProfile] = useState<string>('');
  const [fanSpeeds, setFanSpeeds] = useState<FanStatus[]>([]);
  const [rgbEnabled, setRgbEnabled] = useState(true);
  const [rgbColor, setRgbColor] = useState('#ff0000');
  const [rgbBrightness, setRgbBrightness] = useState(100);
  const [cpuGovernor, setCpuGovernor] = useState('');
  const [availableGovernors, setAvailableGovernors] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadHardwareData();
  }, []);

  const loadHardwareData = async () => {
    try {
      setIsLoading(true);
      
      // Load hardware profiles
      const profileData: string[] = await invoke('get_hardware_profiles');
      setProfiles(profileData.map(name => ({ 
        name, 
        power_mode: 'Balanced', 
        cpu_governor: 'schedutil',
        gpu_power_level: 'auto',
        fan_profile: 'balanced',
        thermal_throttle_temp: 90.0,
        overclock_enabled: false
      })));
      
      // Get active profile
      const activeProfileData: string = await invoke('get_active_hardware_profile');
      setActiveProfile(activeProfileData);
      
      // Get fan statuses
      const fanData: FanStatus[] = await invoke('get_fan_status');
      setFanSpeeds(fanData);
      
      // Get RGB status
      const rgbStatus: any = await invoke('get_rgb_status');
      setRgbEnabled(rgbStatus.enabled || false);
      setRgbBrightness(rgbStatus.brightness || 100);
      if (rgbStatus.color) {
        const [r, g, b] = rgbStatus.color;
        setRgbColor(`#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`);
      }
      
      // Get available CPU governors
      const governors: string[] = await invoke('get_available_cpu_governors');
      setAvailableGovernors(governors);
      
      // Get current CPU governor
      const currentGovernor: string = await invoke('get_current_cpu_governor');
      setCpuGovernor(currentGovernor);
      
    } catch (error) {
      console.error('Failed to load hardware data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleProfileChange = async (profileName: string) => {
    try {
      await invoke('set_hardware_profile', { profileName });
      setActiveProfile(profileName);
      await loadHardwareData(); // Refresh data
    } catch (error) {
      console.error('Failed to set hardware profile:', error);
    }
  };

  const handleRgbToggle = async () => {
    try {
      await invoke('toggle_rgb');
      setRgbEnabled(!rgbEnabled);
    } catch (error) {
      console.error('Failed to toggle RGB:', error);
    }
  };

  const handleRgbColorChange = async (color: string) => {
    try {
      const r = parseInt(color.slice(1, 3), 16);
      const g = parseInt(color.slice(3, 5), 16);
      const b = parseInt(color.slice(5, 7), 16);
      
      await invoke('set_rgb_color', { r, g, b });
      setRgbColor(color);
    } catch (error) {
      console.error('Failed to set RGB color:', error);
    }
  };

  const handleRgbBrightnessChange = async (brightness: number) => {
    try {
      await invoke('set_rgb_brightness', { brightness });
      setRgbBrightness(brightness);
    } catch (error) {
      console.error('Failed to set RGB brightness:', error);
    }
  };

  const handleGovernorChange = async (governor: string) => {
    try {
      await invoke('set_cpu_governor', { governor });
      setCpuGovernor(governor);
    } catch (error) {
      console.error('Failed to set CPU governor:', error);
    }
  };

  const handleFanSpeedChange = async (fanName: string, speed: number) => {
    try {
      await invoke('set_fan_speed', { fanName, speed });
      await loadHardwareData(); // Refresh fan data
    } catch (error) {
      console.error('Failed to set fan speed:', error);
    }
  };

  const hexToRgb = (hex: string) => {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return { r, g, b };
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">🔧 Hardware Control</h1>
        <div className="flex items-center space-x-4">
          <span className={`px-3 py-1 rounded text-sm ${
            hardwareStatus?.optimization_active ? 'bg-green-600' : 'bg-gray-600'
          }`}>
            {hardwareStatus?.optimization_active ? '✅ Optimization Active' : '⏸️ Optimization Paused'}
          </span>
          <button 
            onClick={loadHardwareData}
            className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded transition-colors"
            disabled={isLoading}
          >
            {isLoading ? '🔄 Loading...' : '🔄 Refresh'}
          </button>
        </div>
      </div>

      {/* Hardware Profiles */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-blue-400">⚙️ Hardware Profiles</h3>
          <p className="text-sm text-gray-400 mt-1">Predefined hardware optimization profiles</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {['balanced', 'performance', 'power_saver', 'gaming'].map((profileName) => (
              <button
                key={profileName}
                onClick={() => handleProfileChange(profileName)}
                className={`p-4 rounded-lg border-2 transition-all duration-200 ${
                  activeProfile === profileName
                    ? 'border-blue-500 bg-blue-600 bg-opacity-20'
                    : 'border-gray-600 bg-gray-700 hover:border-gray-500'
                }`}
              >
                <div className="text-center">
                  <div className="text-2xl mb-2">
                    {profileName === 'balanced' && '⚖️'}
                    {profileName === 'performance' && '🚀'}
                    {profileName === 'power_saver' && '🔋'}
                    {profileName === 'gaming' && '🎮'}
                  </div>
                  <h4 className="font-semibold capitalize">{profileName.replace('_', ' ')}</h4>
                  <p className="text-xs text-gray-400 mt-1">
                    {profileName === 'balanced' && 'Optimal balance of performance and efficiency'}
                    {profileName === 'performance' && 'Maximum performance for demanding tasks'}
                    {profileName === 'power_saver' && 'Extended battery life and low noise'}
                    {profileName === 'gaming' && 'Optimized for gaming with RGB effects'}
                  </p>
                </div>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* CPU Control */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-blue-400">💻 CPU Control</h3>
          <p className="text-sm text-gray-400 mt-1">Intel i9-13900HX processor management</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label className="block text-sm font-medium mb-2">CPU Governor</label>
              <select 
                value={cpuGovernor}
                onChange={(e) => handleGovernorChange(e.target.value)}
                className="w-full bg-gray-700 text-white px-3 py-2 rounded border border-gray-600"
              >
                {availableGovernors.map(governor => (
                  <option key={governor} value={governor}>{governor}</option>
                ))}
              </select>
              <p className="text-xs text-gray-400 mt-1">
                Current: <span className="font-mono">{cpuGovernor}</span>
              </p>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">CPU Status</label>
              <div className="bg-gray-700 p-3 rounded">
                <div className="flex justify-between items-center mb-2">
                  <span>Usage:</span>
                  <span className="font-mono">{hardwareStatus?.cpu_temp?.toFixed(1) || '0'}%</span>
                </div>
                <div className="flex justify-between items-center mb-2">
                  <span>Temperature:</span>
                  <span className="font-mono">{hardwareStatus?.cpu_temp?.toFixed(1) || '0'}°C</span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Throttling:</span>
                  <span className={hardwareStatus?.throttling ? 'text-red-400' : 'text-green-400'}>
                    {hardwareStatus?.throttling ? '⚠️ Yes' : '✅ No'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Fan Control */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-green-400">🌪️ Fan Control</h3>
          <p className="text-sm text-gray-400 mt-1">Intelligent cooling management</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {fanSpeeds.map((fan) => (
              <div key={fan.name} className="bg-gray-700 p-4 rounded">
                <h4 className="font-semibold mb-3">{fan.name}</h4>
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <span className="text-sm">RPM:</span>
                    <span className="font-mono text-lg">{fan.rpm.toLocaleString()}</span>
                  </div>
                  
                  <div>
                    <div className="flex justify-between items-center mb-2">
                      <span className="text-sm">Speed:</span>
                      <span className="font-mono">{Math.round((fan.pwm / 255) * 100)}%</span>
                    </div>
                    
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={Math.round((fan.pwm / 255) * 100)}
                      onChange={(e) => handleFanSpeedChange(fan.name, parseInt(e.target.value))}
                      className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer"
                      disabled={fan.auto}
                    />
                  </div>
                  
                  <div className="flex justify-between items-center">
                    <span className="text-sm">Mode:</span>
                    <span className={`text-xs px-2 py-1 rounded ${
                      fan.auto ? 'bg-blue-600' : 'bg-orange-600'
                    }`}>
                      {fan.auto ? '🤖 Auto' : '✋ Manual'}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* RGB Control */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-purple-400">🌈 RGB Control</h3>
          <p className="text-sm text-gray-400 mt-1">Clevo/OriginPC keyboard RGB lighting</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="font-medium">RGB Lighting</span>
                <button
                  onClick={handleRgbToggle}
                  className={`px-4 py-2 rounded transition-colors ${
                    rgbEnabled 
                      ? 'bg-green-600 hover:bg-green-700' 
                      : 'bg-gray-600 hover:bg-gray-700'
                  }`}
                >
                  {rgbEnabled ? '🌈 On' : '⚫ Off'}
                </button>
              </div>
              
              {rgbEnabled && (
                <>
                  <div>
                    <label className="block text-sm font-medium mb-2">Color</label>
                    <div className="flex items-center space-x-3">
                      <input
                        type="color"
                        value={rgbColor}
                        onChange={(e) => handleRgbColorChange(e.target.value)}
                        className="w-12 h-10 rounded border-2 border-gray-600"
                      />
                      <span className="font-mono text-sm">{rgbColor.toUpperCase()}</span>
                    </div>
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium mb-2">Brightness</label>
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={rgbBrightness}
                      onChange={(e) => handleRgbBrightnessChange(parseInt(e.target.value))}
                      className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer"
                    />
                    <div className="flex justify-between text-xs text-gray-400 mt-1">
                      <span>0%</span>
                      <span className="font-mono">{rgbBrightness}%</span>
                      <span>100%</span>
                    </div>
                  </div>
                </>
              )}
            </div>
            
            <div className="bg-gray-700 p-4 rounded flex items-center justify-center">
              <div 
                className="w-full h-24 rounded border-2 border-gray-600 flex items-center justify-center"
                style={{ 
                  backgroundColor: rgbEnabled ? rgbColor : '#333333',
                  opacity: rgbEnabled ? rgbBrightness / 100 : 0.3
                }}
              >
                <span className="text-white font-mono text-sm mix-blend-difference">
                  {rgbEnabled ? '🌈 RGB Preview' : '⚫ RGB Off'}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Thermal Management */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-red-400">🌡️ Thermal Management</h3>
          <p className="text-sm text-gray-400 mt-1">Temperature monitoring and control</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="bg-gray-700 p-4 rounded text-center">
              <h4 className="font-semibold mb-2">CPU Temperature</h4>
              <p className={`text-3xl font-mono mb-2 ${
                (hardwareStatus?.cpu_temp || 0) > 80 ? 'text-red-400' :
                (hardwareStatus?.cpu_temp || 0) > 65 ? 'text-yellow-400' : 'text-green-400'
              }`}>
                {hardwareStatus?.cpu_temp?.toFixed(1) || '0'}°C
              </p>
              <div className="w-full bg-gray-600 rounded-full h-2 mb-2">
                <div 
                  className={`h-2 rounded-full transition-all duration-300 ${
                    (hardwareStatus?.cpu_temp || 0) > 80 ? 'bg-red-500' :
                    (hardwareStatus?.cpu_temp || 0) > 65 ? 'bg-yellow-500' : 'bg-green-500'
                  }`}
                  style={{ width: `${Math.min((hardwareStatus?.cpu_temp || 0) / 100 * 100, 100)}%` }}
                ></div>
              </div>
              <p className="text-xs text-gray-400">Max Safe: 100°C</p>
            </div>
            
            <div className="bg-gray-700 p-4 rounded text-center">
              <h4 className="font-semibold mb-2">GPU Temperature</h4>
              <p className={`text-3xl font-mono mb-2 ${
                (hardwareStatus?.gpu_temp || 0) > 85 ? 'text-red-400' :
                (hardwareStatus?.gpu_temp || 0) > 70 ? 'text-yellow-400' : 'text-green-400'
              }`}>
                {hardwareStatus?.gpu_temp?.toFixed(1) || 'N/A'}°C
              </p>
              <div className="w-full bg-gray-600 rounded-full h-2 mb-2">
                <div 
                  className={`h-2 rounded-full transition-all duration-300 ${
                    (hardwareStatus?.gpu_temp || 0) > 85 ? 'bg-red-500' :
                    (hardwareStatus?.gpu_temp || 0) > 70 ? 'bg-yellow-500' : 'bg-green-500'
                  }`}
                  style={{ width: `${Math.min((hardwareStatus?.gpu_temp || 0) / 95 * 100, 100)}%` }}
                ></div>
              </div>
              <p className="text-xs text-gray-400">Max Safe: 95°C</p>
            </div>
            
            <div className="bg-gray-700 p-4 rounded text-center">
              <h4 className="font-semibold mb-2">Thermal Status</h4>
              <div className="space-y-2">
                <div className={`text-lg font-semibold ${
                  hardwareStatus?.throttling ? 'text-red-400' : 'text-green-400'
                }`}>
                  {hardwareStatus?.throttling ? '⚠️ Throttling' : '✅ Normal'}
                </div>
                <div className={`text-sm ${
                  hardwareStatus?.active_cooling ? 'text-blue-400' : 'text-gray-400'
                }`}>
                  {hardwareStatus?.active_cooling ? '🌪️ Cooling Active' : '😴 Passive'}
                </div>
                <div className={`text-sm ${
                  hardwareStatus?.battery_mode ? 'text-yellow-400' : 'text-green-400'
                }`}>
                  {hardwareStatus?.battery_mode ? '🔋 Battery Mode' : '🔌 AC Power'}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* System Information */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-cyan-400">ℹ️ System Information</h3>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <h4 className="font-semibold text-blue-400">Processor</h4>
              <div className="text-sm space-y-1">
                <p><span className="text-gray-400">Model:</span> Intel Core i9-13900HX</p>
                <p><span className="text-gray-400">Cores:</span> 24 (8P + 16E)</p>
                <p><span className="text-gray-400">Threads:</span> 32</p>
                <p><span className="text-gray-400">Base Clock:</span> 2.2 GHz</p>
                <p><span className="text-gray-400">Max Boost:</span> 5.4 GHz</p>
                <p><span className="text-gray-400">TDP:</span> 55W (Base) / 157W (Max)</p>
              </div>
            </div>
            
            <div className="space-y-2">
              <h4 className="font-semibold text-green-400">Power Management</h4>
              <div className="text-sm space-y-1">
                <p><span className="text-gray-400">Current Mode:</span> {hardwareStatus?.current_power_mode || 'Unknown'}</p>
                <p><span className="text-gray-400">Governor:</span> {hardwareStatus?.current_cpu_governor || 'Unknown'}</p>
                <p><span className="text-gray-400">GPU Power:</span> {hardwareStatus?.current_gpu_power_level || 'Unknown'}</p>
                <p><span className="text-gray-400">Fan Profile:</span> {hardwareStatus?.current_fan_profile || 'Unknown'}</p>
                <p><span className="text-gray-400">Power Source:</span> {hardwareStatus?.battery_mode ? '🔋 Battery' : '🔌 AC Adapter'}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
