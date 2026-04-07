import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { CircleStackIcon, PlusIcon } from '@heroicons/react/24/outline';

import { StoragePool } from '../types';
import LoadingSpinner from '../components/LoadingSpinner';

export default function Storage() {
  const [storagePools, setStoragePools] = useState<StoragePool[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadStoragePools();
    const interval = setInterval(loadStoragePools, 10000);
    return () => clearInterval(interval);
  }, []);

  const loadStoragePools = async () => {
    try {
      const pools = await invoke<StoragePool[]>('get_storage_pools');
      setStoragePools(pools);
    } catch (error) {
      console.error('Failed to load storage pools:', error);
    } finally {
      setLoading(false);
    }
  };

  const formatBytes = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${Math.round(bytes / Math.pow(1024, i) * 100) / 100} ${sizes[i]}`;
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            Storage Management
          </h1>
          <p className="mt-2 text-gray-600 dark:text-gray-400">
            Manage storage pools and volumes
          </p>
        </div>
        <button className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700">
          <PlusIcon className="w-5 h-5 mr-2" />
          Create Pool
        </button>
      </div>

      {loading ? (
        <LoadingSpinner text="Loading storage pools..." />
      ) : (
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {storagePools.map((pool) => (
            <div
              key={pool.name}
              className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700"
            >
              <div className="p-6">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center space-x-2">
                    <CircleStackIcon className="w-5 h-5 text-gray-400" />
                    <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                      {pool.name}
                    </h3>
                  </div>
                  <span className={`px-2 py-1 text-xs font-medium rounded-full ${
                    pool.state === 'active'
                      ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300'
                      : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
                  }`}>
                    {pool.state}
                  </span>
                </div>

                <div className="space-y-3">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-gray-500 dark:text-gray-400">Type:</span>
                      <span className="ml-1 text-gray-900 dark:text-white">{pool.pool_type}</span>
                    </div>
                    <div>
                      <span className="text-gray-500 dark:text-gray-400">Autostart:</span>
                      <span className="ml-1 text-gray-900 dark:text-white">
                        {pool.autostart ? 'Yes' : 'No'}
                      </span>
                    </div>
                  </div>

                  <div>
                    <div className="flex justify-between text-sm mb-1">
                      <span className="text-gray-500 dark:text-gray-400">Storage Usage</span>
                      <span className="text-gray-900 dark:text-white">
                        {formatBytes(pool.used)} / {formatBytes(pool.capacity)}
                      </span>
                    </div>
                    <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full"
                        style={{ width: `${(pool.used / pool.capacity) * 100}%` }}
                      />
                    </div>
                  </div>

                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    Path: {pool.path}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
