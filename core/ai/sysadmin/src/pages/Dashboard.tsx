import React from 'react';

interface DashboardProps {
  metrics: any;
  insights: any[];
  hardwareStatus: any;
  onSystemUpdate: () => void;
  onSystemClean: () => void;
  onBackup: (destination: string) => void;
}

export const Dashboard: React.FC<DashboardProps> = ({ 
  metrics, 
  insights, 
  hardwareStatus, 
  onSystemUpdate, 
  onSystemClean, 
  onBackup 
}) => {
  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold mb-6">System Dashboard</h1>
      
      {/* System Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-gray-800 p-6 rounded-lg">
          <h3 className="text-lg font-semibold mb-2">CPU Usage</h3>
          <p className="text-2xl text-blue-400">{metrics?.cpu_usage?.toFixed(1) || '0'}%</p>
        </div>
        <div className="bg-gray-800 p-6 rounded-lg">
          <h3 className="text-lg font-semibold mb-2">Memory Usage</h3>
          <p className="text-2xl text-green-400">{metrics?.memory_usage?.toFixed(1) || '0'}%</p>
        </div>
        <div className="bg-gray-800 p-6 rounded-lg">
          <h3 className="text-lg font-semibold mb-2">Temperature</h3>
          <p className="text-2xl text-red-400">{metrics?.temperature?.toFixed(1) || '0'}°C</p>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="bg-gray-800 p-6 rounded-lg">
        <h3 className="text-lg font-semibold mb-4">Quick Actions</h3>
        <div className="flex space-x-4">
          <button 
            onClick={onSystemUpdate}
            className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded"
          >
            Update System
          </button>
          <button 
            onClick={onSystemClean}
            className="bg-green-600 hover:bg-green-700 px-4 py-2 rounded"
          >
            Clean System
          </button>
          <button 
            onClick={() => onBackup('/tmp/backup')}
            className="bg-purple-600 hover:bg-purple-700 px-4 py-2 rounded"
          >
            Create Backup
          </button>
        </div>
      </div>

      {/* AI Insights */}
      <div className="bg-gray-800 p-6 rounded-lg">
        <h3 className="text-lg font-semibold mb-4">AI Recommendations</h3>
        {insights?.length > 0 ? (
          <div className="space-y-2">
            {insights.slice(0, 3).map((insight, index) => (
              <div key={index} className="bg-gray-700 p-3 rounded">
                <p className="text-sm">{insight.recommendation}</p>
                <span className="text-xs text-gray-400">Confidence: {(insight.confidence * 100).toFixed(0)}%</span>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-gray-400">No recommendations available</p>
        )}
      </div>
    </div>
  );
};
