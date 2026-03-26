import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  ComputerDesktopIcon,
  CircleStackIcon,
  GlobeAltIcon,
  ChartBarIcon,
} from '@heroicons/react/24/outline';

import { HostInfo, VirtualMachine, DashboardMetrics } from '../types';
import MetricCard from '../components/MetricCard';
import VmList from '../components/VmList';
import ResourceChart from '../components/ResourceChart';

export default function Dashboard() {
  const [hostInfo, setHostInfo] = useState<HostInfo | null>(null);
  const [vms, setVms] = useState<VirtualMachine[]>([]);
  const [metrics, setMetrics] = useState<DashboardMetrics | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadDashboardData();
    const interval = setInterval(loadDashboardData, 5000); // Refresh every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const loadDashboardData = async () => {
    try {
      const [hostData, vmData] = await Promise.all([
        invoke<HostInfo>('get_host_info'),
        invoke<VirtualMachine[]>('get_vms'),
      ]);

      setHostInfo(hostData);
      setVms(vmData);
      setMetrics(calculateMetrics(hostData, vmData));
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      setLoading(false);
    }
  };

  const calculateCpuUsageAverage = (vms: VirtualMachine[]): number => {
    const runningVms = vms.filter(vm => vm.state === 'Running');
    if (runningVms.length === 0) return 0;
    
    // Estimate CPU usage based on allocated vCPUs vs total cores
    // This is a rough estimation since we don't have real-time stats here
    const totalAllocatedCpus = runningVms.reduce((sum, vm) => sum + vm.vcpus, 0);
    const estimatedUsage = Math.min((totalAllocatedCpus * 15), 100); // Assume 15% per vCPU on average
    return estimatedUsage;
  };

  const calculateNetworkActivity = (vms: VirtualMachine[]): number => {
    const runningVms = vms.filter(vm => vm.state === 'Running');
    if (runningVms.length === 0) return 0;
    
    // Estimate network activity based on running VMs
    // This is a placeholder estimation - real implementation would need actual network stats
    return runningVms.length * 1024 * 1024; // 1MB per running VM as baseline
  };

  const calculateMetrics = (hostInfo: HostInfo, vms: VirtualMachine[]): DashboardMetrics => {
    const runningVms = vms.filter(vm => vm.state === 'Running').length;
    const stoppedVms = vms.filter(vm => vm.state === 'Stopped').length;
    const totalMemoryUsed = vms
      .filter(vm => vm.state === 'Running')
      .reduce((sum, vm) => sum + vm.memory, 0);

    return {
      total_vms: vms.length,
      running_vms: runningVms,
      stopped_vms: stoppedVms,
      total_memory_used: totalMemoryUsed,
      total_memory_available: hostInfo.memory_total,
      cpu_usage_average: calculateCpuUsageAverage(vms),
      network_activity: calculateNetworkActivity(vms),
      storage_usage: hostInfo.storage_pools.reduce((sum, pool) => sum + pool.used, 0),
    };
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          Dashboard
        </h1>
        <p className="mt-2 text-gray-600 dark:text-gray-400">
          Overview of your virtualization environment
        </p>
      </div>

      {/* Metrics Cards */}
      {metrics && (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
          <MetricCard
            title="Total VMs"
            value={metrics.total_vms}
            icon={ComputerDesktopIcon}
            color="blue"
            subtitle={`${metrics.running_vms} running, ${metrics.stopped_vms} stopped`}
          />
          <MetricCard
            title="Memory Usage"
            value={`${Math.round((metrics.total_memory_used / metrics.total_memory_available) * 100)}%`}
            icon={ChartBarIcon}
            color="green"
            subtitle={`${Math.round(metrics.total_memory_used / 1024)}GB / ${Math.round(metrics.total_memory_available / 1024)}GB`}
          />
          <MetricCard
            title="Storage Pools"
            value={hostInfo?.storage_pools.length || 0}
            icon={CircleStackIcon}
            color="purple"
            subtitle={`${Math.round(metrics.storage_usage / (1024 ** 3))}GB used`}
          />
          <MetricCard
            title="Networks"
            value={hostInfo?.networks.length || 0}
            icon={GlobeAltIcon}
            color="yellow"
            subtitle={`${hostInfo?.networks.filter(n => n.state === 'active').length || 0} active`}
          />
        </div>
      )}

      {/* Host Information */}
      {hostInfo && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Host Information
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">Hostname</dt>
              <dd className="mt-1 text-sm text-gray-900 dark:text-white">{hostInfo.hostname}</dd>
            </div>
            <div>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">Hypervisor</dt>
              <dd className="mt-1 text-sm text-gray-900 dark:text-white">
                {hostInfo.hypervisor} {hostInfo.hypervisor_version}
              </dd>
            </div>
            <div>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">CPU</dt>
              <dd className="mt-1 text-sm text-gray-900 dark:text-white">
                {hostInfo.cpu_model} ({hostInfo.cpu_cores} cores)
              </dd>
            </div>
            <div>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">Memory</dt>
              <dd className="mt-1 text-sm text-gray-900 dark:text-white">
                {Math.round(hostInfo.memory_free / 1024)}GB free / {Math.round(hostInfo.memory_total / 1024)}GB total
              </dd>
            </div>
          </div>
        </div>
      )}

      {/* Quick Actions and VM List */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Recent VMs */}
        <div className="lg:col-span-2">
          <VmList vms={vms.slice(0, 5)} title="Recent Virtual Machines" showAll />
        </div>

        {/* Resource Usage Chart */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Resource Usage
          </h3>
          <ResourceChart data={metrics} />
        </div>
      </div>
    </div>
  );
}
