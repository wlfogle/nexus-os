import React from 'react';

interface TopBarProps {
  notifications: string[];
  systemMetrics: any;
  onRefresh: () => void;
  isLoading: boolean;
}

export const TopBar: React.FC<TopBarProps> = ({ notifications, systemMetrics, onRefresh, isLoading }) => {
  return (
    <div className="bg-gray-800 border-b border-gray-700 p-4 flex justify-between items-center">
      <h2 className="text-xl font-semibold">Lou's Garuda AI SysAdmin Control Center</h2>
      <div className="flex items-center space-x-4">
        <span className="text-sm text-gray-400">
          {systemMetrics ? `CPU: ${systemMetrics.cpu_usage.toFixed(1)}%` : 'Loading...'}
        </span>
        <button
          onClick={onRefresh}
          disabled={isLoading}
          className="bg-blue-600 hover:bg-blue-700 px-3 py-1 rounded text-sm"
        >
          {isLoading ? 'Loading...' : 'Refresh'}
        </button>
      </div>
    </div>
  );
};
