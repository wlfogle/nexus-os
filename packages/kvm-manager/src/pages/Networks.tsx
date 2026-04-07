import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { GlobeAltIcon, PlusIcon } from '@heroicons/react/24/outline';

import { Network } from '../types';
import LoadingSpinner from '../components/LoadingSpinner';

export default function Networks() {
  const [networks, setNetworks] = useState<Network[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadNetworks();
    const interval = setInterval(loadNetworks, 10000);
    return () => clearInterval(interval);
  }, []);

  const loadNetworks = async () => {
    try {
      const networkData = await invoke<Network[]>('get_networks');
      setNetworks(networkData);
    } catch (error) {
      console.error('Failed to load networks:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            Network Management
          </h1>
          <p className="mt-2 text-gray-600 dark:text-gray-400">
            Manage virtual networks and bridges
          </p>
        </div>
        <button className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700">
          <PlusIcon className="w-5 h-5 mr-2" />
          Create Network
        </button>
      </div>

      {loading ? (
        <LoadingSpinner text="Loading networks..." />
      ) : (
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {networks.map((network) => (
            <div
              key={network.uuid}
              className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700"
            >
              <div className="p-6">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center space-x-2">
                    <GlobeAltIcon className="w-5 h-5 text-gray-400" />
                    <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                      {network.name}
                    </h3>
                  </div>
                  <span className={`px-2 py-1 text-xs font-medium rounded-full ${
                    network.state === 'active'
                      ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300'
                      : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
                  }`}>
                    {network.state}
                  </span>
                </div>

                <div className="space-y-3">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-gray-500 dark:text-gray-400">Forward Mode:</span>
                      <span className="ml-1 text-gray-900 dark:text-white">{network.forward_mode}</span>
                    </div>
                    <div>
                      <span className="text-gray-500 dark:text-gray-400">Autostart:</span>
                      <span className="ml-1 text-gray-900 dark:text-white">
                        {network.autostart ? 'Yes' : 'No'}
                      </span>
                    </div>
                  </div>

                  {network.bridge_name && (
                    <div className="text-sm">
                      <span className="text-gray-500 dark:text-gray-400">Bridge:</span>
                      <span className="ml-1 text-gray-900 dark:text-white">{network.bridge_name}</span>
                    </div>
                  )}

                  {network.ip_range && (
                    <div className="text-sm">
                      <span className="text-gray-500 dark:text-gray-400">IP Range:</span>
                      <span className="ml-1 text-gray-900 dark:text-white">{network.ip_range}</span>
                    </div>
                  )}

                  <div className="text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Connected VMs:</span>
                    <span className="ml-1 text-gray-900 dark:text-white">
                      {network.connected_vms.length}
                    </span>
                  </div>

                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    UUID: {network.uuid}
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
