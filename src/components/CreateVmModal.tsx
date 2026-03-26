import { useState, useEffect } from 'react';
import { XMarkIcon } from '@heroicons/react/24/outline';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { invoke } from '@tauri-apps/api/core';
import toast from 'react-hot-toast';

import { VmConfig, StoragePool, Network } from '../types';

const vmConfigSchema = z.object({
  name: z.string().min(1, 'VM name is required').max(50, 'Name too long'),
  memory: z.number().min(128, 'Memory must be at least 128 MB').max(32768, 'Memory too high'),
  vcpus: z.number().min(1, 'Must have at least 1 vCPU').max(64, 'Too many vCPUs'),
  disk_size: z.number().min(1, 'Disk size must be at least 1 GB').max(1000, 'Disk size too large'),
  os_type: z.string().min(1, 'OS type is required'),
  os_variant: z.string().optional(),
  description: z.string().optional(),
});

interface CreateVmModalProps {
  onClose: () => void;
  onSuccess: () => void;
}

export default function CreateVmModal({ onClose, onSuccess }: CreateVmModalProps) {
  const [step, setStep] = useState(1);
  const [storagePools, setStoragePools] = useState<StoragePool[]>([]);
  const [networks, setNetworks] = useState<Network[]>([]);
  const [, setLoading] = useState(false);
  const [creating, setCreating] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
    watch,
  } = useForm<VmConfig>({
    resolver: zodResolver(vmConfigSchema),
    defaultValues: {
      memory: 1024,
      vcpus: 1,
      disk_size: 20,
      os_type: 'linux',
      network_config: {
        model: 'virtio',
        network_name: 'default',
      },
      storage_config: {
        pool_name: 'default',
        format: 'qcow2',
        bus: 'virtio',
        cache: 'writethrough',
      },
      display_config: {
        graphics_type: 'vnc',
        listen: '127.0.0.1',
        autoport: true,
      },
      boot_config: {
        boot_order: ['hd', 'cdrom'],
      },
    },
  });

  useEffect(() => {
    loadResources();
  }, []);

  const loadResources = async () => {
    setLoading(true);
    try {
      const [storageData, networkData] = await Promise.all([
        invoke<StoragePool[]>('get_storage_pools'),
        invoke<Network[]>('get_networks'),
      ]);
      setStoragePools(storageData);
      setNetworks(networkData);
    } catch (error) {
      console.error('Failed to load resources:', error);
      toast.error('Failed to load storage pools and networks');
    } finally {
      setLoading(false);
    }
  };

  const onSubmit = async (data: VmConfig) => {
    setCreating(true);
    try {
      await invoke('create_vm', { config: data });
      toast.success('Virtual machine created successfully');
      onSuccess();
    } catch (error) {
      console.error('Failed to create VM:', error);
      toast.error('Failed to create virtual machine');
    } finally {
      setCreating(false);
    }
  };

  const nextStep = () => setStep(step + 1);
  const prevStep = () => setStep(step - 1);

  const osTypes = [
    { value: 'linux', label: 'Linux' },
    { value: 'windows', label: 'Windows' },
    { value: 'freebsd', label: 'FreeBSD' },
    { value: 'other', label: 'Other' },
  ];

  const osVariants = {
    linux: [
      { value: 'ubuntu22.04', label: 'Ubuntu 22.04' },
      { value: 'fedora38', label: 'Fedora 38' },
      { value: 'debian12', label: 'Debian 12' },
      { value: 'rhel9', label: 'Red Hat Enterprise Linux 9' },
      { value: 'generic', label: 'Generic Linux' },
    ],
    windows: [
      { value: 'win11', label: 'Windows 11' },
      { value: 'win10', label: 'Windows 10' },
      { value: 'win2022', label: 'Windows Server 2022' },
      { value: 'win2019', label: 'Windows Server 2019' },
    ],
    freebsd: [
      { value: 'freebsd13.2', label: 'FreeBSD 13.2' },
      { value: 'freebsd12.4', label: 'FreeBSD 12.4' },
    ],
    other: [
      { value: 'generic', label: 'Generic' },
    ],
  };

  const selectedOsType = watch('os_type');

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" onClick={onClose} />

        <div className="inline-block align-bottom bg-white dark:bg-gray-800 rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <form onSubmit={handleSubmit(onSubmit)}>
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                Create Virtual Machine - Step {step} of 3
              </h3>
              <button
                type="button"
                onClick={onClose}
                className="text-gray-400 hover:text-gray-500 dark:hover:text-gray-300"
              >
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            {/* Progress Bar */}
            <div className="px-6 py-2">
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${(step / 3) * 100}%` }}
                />
              </div>
            </div>

            <div className="px-6 py-4 space-y-4">
              {/* Step 1: Basic Configuration */}
              {step === 1 && (
                <div className="space-y-4">
                  <h4 className="font-medium text-gray-900 dark:text-white">Basic Configuration</h4>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      VM Name
                    </label>
                    <input
                      {...register('name')}
                      type="text"
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                      placeholder="Enter VM name"
                    />
                    {errors.name && (
                      <p className="mt-1 text-sm text-red-600">{errors.name.message}</p>
                    )}
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Memory (MB)
                      </label>
                      <input
                        {...register('memory', { valueAsNumber: true })}
                        type="number"
                        min="128"
                        step="128"
                        className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                      />
                      {errors.memory && (
                        <p className="mt-1 text-sm text-red-600">{errors.memory.message}</p>
                      )}
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        vCPUs
                      </label>
                      <input
                        {...register('vcpus', { valueAsNumber: true })}
                        type="number"
                        min="1"
                        max="64"
                        className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                      />
                      {errors.vcpus && (
                        <p className="mt-1 text-sm text-red-600">{errors.vcpus.message}</p>
                      )}
                    </div>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Disk Size (GB)
                    </label>
                    <input
                      {...register('disk_size', { valueAsNumber: true })}
                      type="number"
                      min="1"
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                    {errors.disk_size && (
                      <p className="mt-1 text-sm text-red-600">{errors.disk_size.message}</p>
                    )}
                  </div>
                </div>
              )}

              {/* Step 2: OS Configuration */}
              {step === 2 && (
                <div className="space-y-4">
                  <h4 className="font-medium text-gray-900 dark:text-white">Operating System</h4>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      OS Type
                    </label>
                    <select
                      {...register('os_type')}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    >
                      {osTypes.map((os) => (
                        <option key={os.value} value={os.value}>
                          {os.label}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      OS Variant
                    </label>
                    <select
                      {...register('os_variant')}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    >
                      {osVariants[selectedOsType as keyof typeof osVariants]?.map((variant) => (
                        <option key={variant.value} value={variant.value}>
                          {variant.label}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Description (Optional)
                    </label>
                    <textarea
                      {...register('description')}
                      rows={3}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                      placeholder="Enter a description for this VM"
                    />
                  </div>
                </div>
              )}

              {/* Step 3: Network & Storage */}
              {step === 3 && (
                <div className="space-y-4">
                  <h4 className="font-medium text-gray-900 dark:text-white">Network & Storage</h4>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Network
                    </label>
                    <select
                      {...register('network_config.network_name')}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    >
                      {networks.map((network) => (
                        <option key={network.name} value={network.name}>
                          {network.name} ({network.forward_mode})
                        </option>
                      ))}
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Storage Pool
                    </label>
                    <select
                      {...register('storage_config.pool_name')}
                      className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    >
                      {storagePools.map((pool) => (
                        <option key={pool.name} value={pool.name}>
                          {pool.name} ({Math.round(pool.available / (1024 ** 3))}GB available)
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Disk Format
                      </label>
                      <select
                        {...register('storage_config.format')}
                        className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                      >
                        <option value="qcow2">QCOW2 (Recommended)</option>
                        <option value="raw">RAW</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Graphics
                      </label>
                      <select
                        {...register('display_config.graphics_type')}
                        className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                      >
                        <option value="vnc">VNC</option>
                        <option value="spice">SPICE</option>
                      </select>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Footer */}
            <div className="px-6 py-4 bg-gray-50 dark:bg-gray-700 flex justify-between">
              <div>
                {step > 1 && (
                  <button
                    type="button"
                    onClick={prevStep}
                    className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-500 dark:hover:text-gray-400"
                  >
                    Previous
                  </button>
                )}
              </div>
              <div className="flex space-x-3">
                <button
                  type="button"
                  onClick={onClose}
                  className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600"
                >
                  Cancel
                </button>
                {step < 3 ? (
                  <button
                    type="button"
                    onClick={nextStep}
                    className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700"
                  >
                    Next
                  </button>
                ) : (
                  <button
                    type="submit"
                    disabled={creating}
                    className="px-4 py-2 text-sm font-medium text-white bg-green-600 border border-transparent rounded-md hover:bg-green-700 disabled:opacity-50"
                  >
                    {creating ? 'Creating...' : 'Create VM'}
                  </button>
                )}
              </div>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
