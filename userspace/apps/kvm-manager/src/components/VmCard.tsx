import { useState, useCallback, memo } from 'react';
import {
  PlayIcon,
  StopIcon,
  PauseIcon,
  TrashIcon,
  EllipsisVerticalIcon,
  ComputerDesktopIcon,
  CpuChipIcon,
  CircleStackIcon,
} from '@heroicons/react/24/outline';
import { Menu } from '@headlessui/react';

import { VirtualMachine, VmState } from '../types';

interface VmCardProps {
  vm: VirtualMachine;
  onStart: () => void;
  onStop: () => void;
  onDelete: () => void;
}

const VmCard = memo(function VmCard({ vm, onStart, onStop, onDelete }: VmCardProps) {
  const [loading, setLoading] = useState(false);

  const getStateColor = (state: VmState) => {
    switch (state) {
      case 'Running':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300';
      case 'Stopped':
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
      case 'Paused':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300';
      case 'Error':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300';
      default:
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300';
    }
  };

  const getStateIcon = (state: VmState) => {
    switch (state) {
      case 'Running':
        return <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />;
      case 'Stopped':
        return <div className="w-2 h-2 bg-gray-400 rounded-full" />;
      case 'Paused':
        return <div className="w-2 h-2 bg-yellow-400 rounded-full" />;
      case 'Error':
        return <div className="w-2 h-2 bg-red-400 rounded-full" />;
      default:
        return <div className="w-2 h-2 bg-blue-400 rounded-full" />;
    }
  };

  const handleAction = useCallback(async (action: () => void) => {
    setLoading(true);
    try {
      await action();
    } finally {
      setLoading(false);
    }
  }, []);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 hover:shadow-lg transition-shadow">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center space-x-2">
          <ComputerDesktopIcon className="w-5 h-5 text-gray-400" />
          <h3 className="text-sm font-medium text-gray-900 dark:text-white truncate">
            {vm.name}
          </h3>
        </div>
        <Menu as="div" className="relative">
          <Menu.Button className="p-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700">
            <EllipsisVerticalIcon className="w-4 h-4 text-gray-400" />
          </Menu.Button>
          <Menu.Items className="absolute right-0 z-10 mt-2 w-48 rounded-md bg-white dark:bg-gray-800 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none">
            <div className="py-1">
              <Menu.Item>
                {({ active }) => (
                  <button
                    onClick={() => handleAction(onDelete)}
                    className={`${
                      active ? 'bg-red-50 dark:bg-red-900 text-red-600' : 'text-gray-700 dark:text-gray-300'
                    } group flex items-center px-4 py-2 text-sm w-full`}
                  >
                    <TrashIcon className="w-4 h-4 mr-3" />
                    Delete VM
                  </button>
                )}
              </Menu.Item>
            </div>
          </Menu.Items>
        </Menu>
      </div>

      {/* Content */}
      <div className="p-4">
        {/* Status */}
        <div className="flex items-center space-x-2 mb-3">
          {getStateIcon(vm.state)}
          <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStateColor(vm.state)}`}>
            {vm.state}
          </span>
        </div>

        {/* Resources */}
        <div className="grid grid-cols-2 gap-3 mb-4">
          <div className="flex items-center space-x-2">
            <CpuChipIcon className="w-4 h-4 text-gray-400" />
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {vm.vcpus} vCPU{vm.vcpus > 1 ? 's' : ''}
            </span>
          </div>
          <div className="flex items-center space-x-2">
            <CircleStackIcon className="w-4 h-4 text-gray-400" />
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {Math.round(vm.memory / 1024)}GB RAM
            </span>
          </div>
        </div>

        {/* OS Info */}
        <div className="mb-4">
          <span className="text-xs text-gray-500 dark:text-gray-400">
            {vm.os_type} {vm.os_variant && `(${vm.os_variant})`}
          </span>
        </div>

        {/* Actions */}
        <div className="flex space-x-2">
          {vm.state === 'Stopped' ? (
            <button
              onClick={() => handleAction(onStart)}
              disabled={loading}
              className="flex-1 inline-flex items-center justify-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
            >
              <PlayIcon className="w-4 h-4 mr-1" />
              Start
            </button>
          ) : vm.state === 'Running' ? (
            <button
              onClick={() => handleAction(onStop)}
              disabled={loading}
              className="flex-1 inline-flex items-center justify-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50"
            >
              <StopIcon className="w-4 h-4 mr-1" />
              Stop
            </button>
          ) : (
            <button
              disabled
              className="flex-1 inline-flex items-center justify-center px-3 py-2 border border-gray-300 text-sm leading-4 font-medium rounded-md text-gray-400 bg-gray-50 dark:bg-gray-700 dark:border-gray-600"
            >
              <PauseIcon className="w-4 h-4 mr-1" />
              {vm.state}
            </button>
          )}
        </div>
      </div>
    </div>
  );
});

export default VmCard;
