// Frontend types that match the Rust backend types

export interface VirtualMachine {
  id: string;
  name: string;
  state: VmState;
  memory: number;        // Memory in MB
  vcpus: number;
  disk_size: number;     // Disk size in GB
  os_type: string;
  os_variant?: string;
  created_at: string;
  last_started?: string;
  description?: string;
  vnc_port?: number;
  spice_port?: number;
  snapshots: Snapshot[];
  network_interfaces: NetworkInterface[];
  storage_devices: StorageDevice[];
}

export type VmState = 
  | 'Running'
  | 'Stopped'
  | 'Paused'
  | 'Suspended'
  | 'ShuttingDown'
  | 'Creating'
  | 'Error';

export interface VmConfig {
  name: string;
  memory: number;
  vcpus: number;
  disk_size: number;
  os_type: string;
  os_variant?: string;
  description?: string;
  network_config: NetworkConfig;
  storage_config: StorageConfig;
  display_config: DisplayConfig;
  boot_config: BootConfig;
}

export interface NetworkConfig {
  bridge?: string;
  network_name?: string;
  mac_address?: string;
  model: string; // e1000, virtio, etc.
}

export interface StorageConfig {
  pool_name: string;
  format: string; // qcow2, raw, etc.
  bus: string;    // virtio, sata, ide, etc.
  cache: string;  // none, writeback, writethrough, etc.
}

export interface DisplayConfig {
  graphics_type: string; // vnc, spice
  listen: string;
  password?: string;
  autoport: boolean;
}

export interface BootConfig {
  boot_order: string[]; // cdrom, hd, network
  iso_path?: string;
  kernel?: string;
  initrd?: string;
  cmdline?: string;
}

export interface VmStats {
  cpu_usage: number;        // Percentage
  memory_usage: number;     // Used memory in MB
  memory_total: number;     // Total memory in MB
  disk_read: number;        // Bytes read per second
  disk_write: number;       // Bytes written per second
  network_rx: number;       // Bytes received per second
  network_tx: number;       // Bytes transmitted per second
  uptime: number;           // Uptime in seconds
  guest_agent_connected: boolean;
}

export interface SystemStats {
  cpu_usage: number;        // Percentage
  memory_used: number;      // Used system memory in MB
  memory_total: number;     // Total system memory in MB
  disk_usage: number;       // Used disk space percentage
  network_rx: number;       // System network bytes received per second
  network_tx: number;       // System network bytes transmitted per second
  load_average: [number, number, number]; // 1m, 5m, 15m load averages
  uptime: number;           // System uptime in seconds
}

export interface HostInfo {
  hostname: string;
  hypervisor: string;
  hypervisor_version: string;
  cpu_model: string;
  cpu_cores: number;
  memory_total: number;     // Total host memory in MB
  memory_free: number;      // Free host memory in MB
  storage_pools: StoragePool[];
  networks: Network[];
  active_vms: number;
  inactive_vms: number;
}

export interface Snapshot {
  name: string;
  description?: string;
  created_at: string;
  state: string;
  parent?: string;
}

export interface NetworkInterface {
  mac_address: string;
  network_name: string;
  interface_type: string;
  model: string;
  link_state: string;
}

export interface StorageDevice {
  device: string;       // vda, vdb, etc.
  source: string;       // file path or device
  format: string;       // qcow2, raw, etc.
  size: number;         // Size in bytes
  bus: string;          // virtio, sata, etc.
  cache: string;        // cache mode
}

export interface StoragePool {
  name: string;
  pool_type: string;   // dir, disk, netfs, etc.
  path: string;
  capacity: number;    // Total capacity in bytes
  available: number;   // Available space in bytes
  used: number;        // Used space in bytes
  state: string;       // active, inactive
  autostart: boolean;
  volumes: StorageVolume[];
}

export interface StorageVolume {
  name: string;
  format: string;
  capacity: number;
  allocation: number;
  path: string;
}

export interface Network {
  name: string;
  uuid: string;
  bridge_name?: string;
  forward_mode: string; // nat, route, bridge, etc.
  state: string;        // active, inactive
  autostart: boolean;
  ip_range?: string;
  dhcp_enabled: boolean;
  connected_vms: string[];
}

// UI-specific types
export interface DashboardMetrics {
  total_vms: number;
  running_vms: number;
  stopped_vms: number;
  total_memory_used: number;
  total_memory_available: number;
  cpu_usage_average: number;
  network_activity: number;
  storage_usage: number;
}

export interface ToastMessage {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message: string;
  duration?: number;
}
