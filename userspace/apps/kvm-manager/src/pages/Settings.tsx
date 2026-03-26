import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  CogIcon,
  ComputerDesktopIcon,
  CircleStackIcon,
  GlobeAltIcon,
  ShieldCheckIcon,
  ClockIcon,
  CpuChipIcon,
  CloudIcon,
  CommandLineIcon,
  DocumentTextIcon,
  VideoCameraIcon,
  SpeakerWaveIcon,
  DevicePhoneMobileIcon,
} from '@heroicons/react/24/outline';
import toast from 'react-hot-toast';

interface SettingsData {
  // General Settings
  dark_mode: boolean;
  confirm_destructive_actions: boolean;
  auto_refresh_interval: number;
  enable_notifications: boolean;
  
  // VM Creation Defaults
  default_memory: number;
  default_vcpus: number;
  default_disk_size: number;
  default_os_type: string;
  default_storage_format: string;
  default_network: string;
  default_storage_pool: string;
  
  // Advanced VM Settings
  enable_kvm_acceleration: boolean;
  enable_cpu_passthrough: boolean;
  enable_nested_virtualization: boolean;
  default_cpu_model: string;
  default_machine_type: string;
  
  // Graphics & Display
  default_graphics_type: string;
  enable_3d_acceleration: boolean;
  default_video_model: string;
  
  // Audio Settings
  default_audio_model: string;
  enable_audio: boolean;
  
  // Security Settings
  enable_secure_boot: boolean;
  enable_tpm: boolean;
  
  // Performance Settings
  enable_io_threads: boolean;
  enable_multiqueue: boolean;
  disk_cache_mode: string;
  
  // Connection Settings
  libvirt_uri: string;
  connection_timeout: number;
}

const DEFAULT_SETTINGS: SettingsData = {
  dark_mode: false,
  confirm_destructive_actions: true,
  auto_refresh_interval: 5,
  enable_notifications: true,
  default_memory: 2048,
  default_vcpus: 2,
  default_disk_size: 20,
  default_os_type: 'linux',
  default_storage_format: 'qcow2',
  default_network: 'default',
  default_storage_pool: 'default',
  enable_kvm_acceleration: true,
  enable_cpu_passthrough: false,
  enable_nested_virtualization: false,
  default_cpu_model: 'host-model',
  default_machine_type: 'q35',
  default_graphics_type: 'spice',
  enable_3d_acceleration: false,
  default_video_model: 'qxl',
  default_audio_model: 'ich6',
  enable_audio: true,
  enable_secure_boot: false,
  enable_tpm: false,
  enable_io_threads: true,
  enable_multiqueue: true,
  disk_cache_mode: 'writeback',
  libvirt_uri: 'qemu:///system',
  connection_timeout: 30,
};

export default function Settings() {
  const [settings, setSettings] = useState<SettingsData>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [activeSection, setActiveSection] = useState('general');

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      // Load from localStorage for now, could be extended to use Tauri's storage
      const savedSettings = localStorage.getItem('kvm-manager-settings');
      if (savedSettings) {
        setSettings({ ...DEFAULT_SETTINGS, ...JSON.parse(savedSettings) });
      }
      
      // Apply dark mode setting immediately
      const isDark = savedSettings ? JSON.parse(savedSettings).dark_mode : false;
      if (isDark) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
      toast.error('Failed to load settings');
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async (newSettings: SettingsData) => {
    setSaving(true);
    try {
      localStorage.setItem('kvm-manager-settings', JSON.stringify(newSettings));
      
      // Apply dark mode immediately
      if (newSettings.dark_mode) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
      
      setSettings(newSettings);
      toast.success('Settings saved successfully');
    } catch (error) {
      console.error('Failed to save settings:', error);
      toast.error('Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  const handleToggle = (key: keyof SettingsData) => {
    const newSettings = {
      ...settings,
      [key]: !settings[key]
    };
    saveSettings(newSettings);
  };

  const handleChange = (key: keyof SettingsData, value: any) => {
    const newSettings = {
      ...settings,
      [key]: value
    };
    saveSettings(newSettings);
  };

  const Toggle: React.FC<{ enabled: boolean; onChange: () => void; disabled?: boolean }> = ({ 
    enabled, 
    onChange, 
    disabled = false 
  }) => (
    <button
      type="button"
      onClick={onChange}
      disabled={disabled}
      className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${
        enabled ? 'bg-blue-600' : 'bg-gray-200 dark:bg-gray-700'
      } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
      role="switch"
      aria-checked={enabled}
    >
      <span
        className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
          enabled ? 'translate-x-5' : 'translate-x-0'
        }`}
      />
    </button>
  );

  const SettingRow: React.FC<{
    title: string;
    description: string;
    children: React.ReactNode;
  }> = ({ title, description, children }) => (
    <div className="flex items-center justify-between py-4">
      <div className="flex-1 pr-4">
        <h3 className="text-sm font-medium text-gray-900 dark:text-white">
          {title}
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          {description}
        </p>
      </div>
      <div className="flex-shrink-0">
        {children}
      </div>
    </div>
  );

  const sections = [
    { id: 'general', name: 'General', icon: CogIcon },
    { id: 'vm-defaults', name: 'VM Defaults', icon: ComputerDesktopIcon },
    { id: 'performance', name: 'Performance', icon: CpuChipIcon },
    { id: 'graphics', name: 'Graphics', icon: VideoCameraIcon },
    { id: 'audio', name: 'Audio', icon: SpeakerWaveIcon },
    { id: 'security', name: 'Security', icon: ShieldCheckIcon },
    { id: 'connection', name: 'Connection', icon: CloudIcon },
  ];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <div className="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
        <div className="p-6">
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">
            Settings
          </h1>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Configure KVM Manager
          </p>
        </div>
        <nav className="px-3 space-y-1">
          {sections.map(section => {
            const Icon = section.icon;
            return (
              <button
                key={section.id}
                onClick={() => setActiveSection(section.id)}
                className={`group flex items-center px-3 py-2 text-sm font-medium rounded-md w-full text-left ${
                  activeSection === section.id
                    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200'
                    : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white'
                }`}
              >
                <Icon className="mr-3 h-5 w-5 flex-shrink-0" />
                {section.name}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Main content */}
      <div className="flex-1 p-6 overflow-y-auto">
        {/* General Settings */}
        {activeSection === 'general' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">General Settings</h2>
              <p className="mt-1 text-gray-600 dark:text-gray-400">Manage your application preferences</p>
            </div>
            
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
              <div className="p-6 space-y-4">
                <SettingRow
                  title="Dark Mode"
                  description="Use dark theme for the interface"
                >
                  <Toggle
                    enabled={settings.dark_mode}
                    onChange={() => handleToggle('dark_mode')}
                  />
                </SettingRow>
                
                <SettingRow
                  title="Confirm Destructive Actions"
                  description="Show confirmation dialogs for dangerous operations like deleting VMs"
                >
                  <Toggle
                    enabled={settings.confirm_destructive_actions}
                    onChange={() => handleToggle('confirm_destructive_actions')}
                  />
                </SettingRow>
                
                <SettingRow
                  title="Enable Notifications"
                  description="Show system notifications for important events"
                >
                  <Toggle
                    enabled={settings.enable_notifications}
                    onChange={() => handleToggle('enable_notifications')}
                  />
                </SettingRow>
                
                <div className="flex items-center justify-between py-4">
                  <div className="flex-1 pr-4">
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                      Auto-refresh Interval
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      How often to refresh VM status and metrics
                    </p>
                  </div>
                  <select
                    value={settings.auto_refresh_interval}
                    onChange={(e) => handleChange('auto_refresh_interval', parseInt(e.target.value))}
                    className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  >
                    <option value={1}>1 second</option>
                    <option value={3}>3 seconds</option>
                    <option value={5}>5 seconds</option>
                    <option value={10}>10 seconds</option>
                    <option value={30}>30 seconds</option>
                    <option value={60}>1 minute</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* VM Defaults */}
        {activeSection === 'vm-defaults' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">VM Defaults</h2>
              <p className="mt-1 text-gray-600 dark:text-gray-400">Default settings for new virtual machines</p>
            </div>
            
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
              <div className="p-6 space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Default Memory (MB)
                    </label>
                    <input
                      type="number"
                      min="128"
                      step="128"
                      value={settings.default_memory}
                      onChange={(e) => handleChange('default_memory', parseInt(e.target.value))}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Default vCPUs
                    </label>
                    <input
                      type="number"
                      min="1"
                      max="64"
                      value={settings.default_vcpus}
                      onChange={(e) => handleChange('default_vcpus', parseInt(e.target.value))}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Default Disk Size (GB)
                    </label>
                    <input
                      type="number"
                      min="1"
                      value={settings.default_disk_size}
                      onChange={(e) => handleChange('default_disk_size', parseInt(e.target.value))}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Default OS Type
                    </label>
                    <select
                      value={settings.default_os_type}
                      onChange={(e) => handleChange('default_os_type', e.target.value)}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    >
                      <option value="linux">Linux</option>
                      <option value="windows">Windows</option>
                      <option value="freebsd">FreeBSD</option>
                      <option value="other">Other</option>
                    </select>
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Default Storage Format
                    </label>
                    <select
                      value={settings.default_storage_format}
                      onChange={(e) => handleChange('default_storage_format', e.target.value)}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    >
                      <option value="qcow2">QCOW2 (Recommended)</option>
                      <option value="raw">RAW</option>
                      <option value="qed">QED</option>
                    </select>
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Default Machine Type
                    </label>
                    <select
                      value={settings.default_machine_type}
                      onChange={(e) => handleChange('default_machine_type', e.target.value)}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    >
                      <option value="q35">Q35 (Recommended)</option>
                      <option value="pc">PC (i440FX)</option>
                      <option value="virt">Virtual Machine</option>
                    </select>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Performance Settings */}
        {activeSection === 'performance' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Performance Settings</h2>
              <p className="mt-1 text-gray-600 dark:text-gray-400">Optimize VM performance and resource usage</p>
            </div>
            
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
              <div className="p-6 space-y-4">
                <SettingRow
                  title="Enable KVM Hardware Acceleration"
                  description="Use hardware virtualization extensions for better performance"
                >
                  <Toggle
                    enabled={settings.enable_kvm_acceleration}
                    onChange={() => handleToggle('enable_kvm_acceleration')}
                  />
                </SettingRow>
                
                <SettingRow
                  title="Enable CPU Passthrough"
                  description="Pass host CPU features directly to VMs (host-passthrough mode)"
                >
                  <Toggle
                    enabled={settings.enable_cpu_passthrough}
                    onChange={() => handleToggle('enable_cpu_passthrough')}
                  />
                </SettingRow>
                
                <SettingRow
                  title="Enable Nested Virtualization"
                  description="Allow VMs to run their own hypervisors"
                >
                  <Toggle
                    enabled={settings.enable_nested_virtualization}
                    onChange={() => handleToggle('enable_nested_virtualization')}
                  />
                </SettingRow>
                
                <SettingRow
                  title="Enable IO Threads"
                  description="Use separate threads for disk I/O operations"
                >
                  <Toggle
                    enabled={settings.enable_io_threads}
                    onChange={() => handleToggle('enable_io_threads')}
                  />
                </SettingRow>
                
                <SettingRow
                  title="Enable Multiqueue"
                  description="Use multiple queues for network devices"
                >
                  <Toggle
                    enabled={settings.enable_multiqueue}
                    onChange={() => handleToggle('enable_multiqueue')}
                  />
                </SettingRow>
                
                <div className="flex items-center justify-between py-4">
                  <div className="flex-1 pr-4">
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                      Default CPU Model
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      CPU model to use for new VMs
                    </p>
                  </div>
                  <select
                    value={settings.default_cpu_model}
                    onChange={(e) => handleChange('default_cpu_model', e.target.value)}
                    className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  >
                    <option value="host-model">Host Model</option>
                    <option value="host-passthrough">Host Passthrough</option>
                    <option value="qemu64">QEMU64</option>
                    <option value="Haswell">Haswell</option>
                    <option value="Broadwell">Broadwell</option>
                    <option value="Skylake-Client">Skylake</option>
                  </select>
                </div>
                
                <div className="flex items-center justify-between py-4">
                  <div className="flex-1 pr-4">
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                      Disk Cache Mode
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      Caching strategy for disk operations
                    </p>
                  </div>
                  <select
                    value={settings.disk_cache_mode}
                    onChange={(e) => handleChange('disk_cache_mode', e.target.value)}
                    className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  >
                    <option value="writeback">Writeback</option>
                    <option value="writethrough">Writethrough</option>
                    <option value="none">None</option>
                    <option value="unsafe">Unsafe</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Graphics Settings */}
        {activeSection === 'graphics' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Graphics Settings</h2>
              <p className="mt-1 text-gray-600 dark:text-gray-400">Configure display and graphics options</p>
            </div>
            
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
              <div className="p-6 space-y-4">
                <SettingRow
                  title="Enable 3D Acceleration"
                  description="Enable hardware 3D acceleration for VMs"
                >
                  <Toggle
                    enabled={settings.enable_3d_acceleration}
                    onChange={() => handleToggle('enable_3d_acceleration')}
                  />
                </SettingRow>
                
                <div className="flex items-center justify-between py-4">
                  <div className="flex-1 pr-4">
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                      Default Graphics Type
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      Graphics protocol for new VMs
                    </p>
                  </div>
                  <select
                    value={settings.default_graphics_type}
                    onChange={(e) => handleChange('default_graphics_type', e.target.value)}
                    className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  >
                    <option value="spice">SPICE</option>
                    <option value="vnc">VNC</option>
                    <option value="none">None (Headless)</option>
                  </select>
                </div>
                
                <div className="flex items-center justify-between py-4">
                  <div className="flex-1 pr-4">
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                      Default Video Model
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      Video adapter model for VMs
                    </p>
                  </div>
                  <select
                    value={settings.default_video_model}
                    onChange={(e) => handleChange('default_video_model', e.target.value)}
                    className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  >
                    <option value="qxl">QXL</option>
                    <option value="virtio">VirtIO</option>
                    <option value="cirrus">Cirrus</option>
                    <option value="vga">VGA</option>
                    <option value="vmvga">VMware VGA</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Audio Settings */}
        {activeSection === 'audio' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Audio Settings</h2>
              <p className="mt-1 text-gray-600 dark:text-gray-400">Configure audio options for VMs</p>
            </div>
            
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
              <div className="p-6 space-y-4">
                <SettingRow
                  title="Enable Audio"
                  description="Enable audio devices for new VMs"
                >
                  <Toggle
                    enabled={settings.enable_audio}
                    onChange={() => handleToggle('enable_audio')}
                  />
                </SettingRow>
                
                <div className="flex items-center justify-between py-4">
                  <div className="flex-1 pr-4">
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                      Default Audio Model
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      Audio device model for new VMs
                    </p>
                  </div>
                  <select
                    value={settings.default_audio_model}
                    onChange={(e) => handleChange('default_audio_model', e.target.value)}
                    className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    disabled={!settings.enable_audio}
                  >
                    <option value="ich6">ICH6</option>
                    <option value="ac97">AC97</option>
                    <option value="es1370">ES1370</option>
                    <option value="sb16">SB16</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Security Settings */}
        {activeSection === 'security' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Security Settings</h2>
              <p className="mt-1 text-gray-600 dark:text-gray-400">Configure security features for VMs</p>
            </div>
            
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
              <div className="p-6 space-y-4">
                <SettingRow
                  title="Enable Secure Boot"
                  description="Enable UEFI Secure Boot for new VMs"
                >
                  <Toggle
                    enabled={settings.enable_secure_boot}
                    onChange={() => handleToggle('enable_secure_boot')}
                  />
                </SettingRow>
                
                <SettingRow
                  title="Enable TPM"
                  description="Enable Trusted Platform Module for new VMs"
                >
                  <Toggle
                    enabled={settings.enable_tpm}
                    onChange={() => handleToggle('enable_tpm')}
                  />
                </SettingRow>
              </div>
            </div>
          </div>
        )}

        {/* Connection Settings */}
        {activeSection === 'connection' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Connection Settings</h2>
              <p className="mt-1 text-gray-600 dark:text-gray-400">Configure libvirt connection and timeouts</p>
            </div>
            
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
              <div className="p-6 space-y-6">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Libvirt URI
                  </label>
                  <input
                    type="text"
                    value={settings.libvirt_uri}
                    onChange={(e) => handleChange('libvirt_uri', e.target.value)}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    placeholder="qemu:///system"
                  />
                  <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                    Connection URI for libvirt daemon
                  </p>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Connection Timeout (seconds)
                  </label>
                  <input
                    type="number"
                    min="5"
                    max="300"
                    value={settings.connection_timeout}
                    onChange={(e) => handleChange('connection_timeout', parseInt(e.target.value))}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  />
                  <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                    Timeout for libvirt operations
                  </p>
                </div>
              </div>
            </div>
          </div>
        )}
        
        {/* About Section */}
        <div className="mt-8 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
          <div className="p-6">
            <div className="flex items-center space-x-3">
              <CogIcon className="w-8 h-8 text-blue-600" />
              <div>
                <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                  KVM Manager
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Version 0.1.0 - Built with Tauri, Rust, and React
                </p>
              </div>
            </div>
            <div className="mt-4 text-sm text-gray-600 dark:text-gray-400">
              A modern, fast, and intuitive KVM virtualization manager that aims to
              provide a better user experience than traditional tools like virt-manager.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
