// @ts-nocheck - Temporarily disable type checking for Zustand imports
import { create } from 'zustand';
import { devtools, subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import type { VirtualMachine, VmStats, SystemStats } from '@/types';

interface VmStore {
  // State
  vms: VirtualMachine[];
  vmStats: Record<string, VmStats>;
  systemStats: SystemStats | null;
  selectedVm: VirtualMachine | null;
  loading: boolean;
  error: string | null;

  // Actions
  setVms: (vms: VirtualMachine[]) => void;
  addVm: (vm: VirtualMachine) => void;
  updateVm: (vmId: string, updates: Partial<VirtualMachine>) => void;
  removeVm: (vmId: string) => void;
  setVmStats: (vmId: string, stats: VmStats) => void;
  setSystemStats: (stats: SystemStats) => void;
  setSelectedVm: (vm: VirtualMachine | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Computed values
  getVmById: (vmId: string) => VirtualMachine | undefined;
  getRunningVms: () => VirtualMachine[];
  getStoppedVms: () => VirtualMachine[];
  getTotalVms: () => number;
  getTotalMemoryUsed: () => number;
  getAverageCpuUsage: () => number;
}

const createVmStore = (set: any, get: any): VmStore => ({
  // Initial state
  vms: [],
  vmStats: {},
  systemStats: null,
  selectedVm: null,
  loading: false,
  error: null,

  // Actions
  setVms: (vms: VirtualMachine[]) =>
    set((state) => {
      state.vms = vms;
      state.error = null;
    }),

  addVm: (vm: VirtualMachine) =>
    set((state) => {
      state.vms.push(vm);
    }),

  updateVm: (vmId: string, updates: Partial<VirtualMachine>) =>
    set((state) => {
      const vmIndex = state.vms.findIndex((vm) => vm.id === vmId);
      if (vmIndex !== -1) {
        Object.assign(state.vms[vmIndex], updates);
      }
    }),

  removeVm: (vmId: string) =>
    set((state) => {
      state.vms = state.vms.filter((vm) => vm.id !== vmId);
      delete state.vmStats[vmId];
      if (state.selectedVm?.id === vmId) {
        state.selectedVm = null;
      }
    }),

  setVmStats: (vmId: string, stats: VmStats) =>
    set((state) => {
      state.vmStats[vmId] = stats;
    }),

  setSystemStats: (stats: SystemStats) =>
    set((state) => {
      state.systemStats = stats;
    }),

  setSelectedVm: (vm: VirtualMachine | null) =>
    set((state) => {
      state.selectedVm = vm;
    }),

  setLoading: (loading: boolean) =>
    set((state) => {
      state.loading = loading;
    }),

  setError: (error: string | null) =>
    set((state) => {
      state.error = error;
    }),

  // Computed values
  getVmById: (vmId: string) => get().vms.find((vm) => vm.id === vmId),

  getRunningVms: () => get().vms.filter((vm) => vm.state === 'Running'),

  getStoppedVms: () => get().vms.filter((vm) => vm.state === 'Stopped'),

  getTotalVms: () => get().vms.length,

  getTotalMemoryUsed: () =>
    get()
      .vms.filter((vm) => vm.state === 'Running')
      .reduce((total, vm) => total + vm.memory, 0),

  getAverageCpuUsage: () => {
    const stats = Object.values(get().vmStats);
    if (stats.length === 0) return 0;
    const total = stats.reduce((sum, stat) => sum + stat.cpu_usage, 0);
    return total / stats.length;
  },
});

export const useVmStore = create<VmStore>()(
  devtools(
    subscribeWithSelector(
      immer(createVmStore)
    ),
    { name: 'vm-store' }
  )
);

// Selectors for performance optimization
export const selectVms = (state: VmStore) => state.vms;
export const selectRunningVms = (state: VmStore) => state.getRunningVms();
export const selectStoppedVms = (state: VmStore) => state.getStoppedVms();
export const selectVmStats = (state: VmStore) => state.vmStats;
export const selectSystemStats = (state: VmStore) => state.systemStats;
export const selectSelectedVm = (state: VmStore) => state.selectedVm;
export const selectLoading = (state: VmStore) => state.loading;
export const selectError = (state: VmStore) => state.error;
