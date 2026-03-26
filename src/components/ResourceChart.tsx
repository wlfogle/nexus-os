import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from 'recharts';
import { DashboardMetrics } from '../types';

interface ResourceChartProps {
  data: DashboardMetrics | null;
}

export default function ResourceChart({ data }: ResourceChartProps) {
  if (!data) {
    return (
      <div className="flex items-center justify-center h-48">
        <div className="text-gray-500 dark:text-gray-400">No data available</div>
      </div>
    );
  }

  const memoryData = [
    {
      name: 'Used',
      value: data.total_memory_used,
      color: '#3B82F6',
    },
    {
      name: 'Available',
      value: data.total_memory_available - data.total_memory_used,
      color: '#E5E7EB',
    },
  ];

  const vmStateData = [
    {
      name: 'Running',
      value: data.running_vms,
      color: '#10B981',
    },
    {
      name: 'Stopped',
      value: data.stopped_vms,
      color: '#6B7280',
    },
  ];

  return (
    <div className="space-y-6">
      {/* Memory Usage */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 dark:text-white mb-2">
          Memory Usage
        </h4>
        <div className="h-32">
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Pie
                data={memoryData}
                cx="50%"
                cy="50%"
                innerRadius={25}
                outerRadius={45}
                paddingAngle={2}
                dataKey="value"
              >
                {memoryData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip
                formatter={(value: number) => [`${Math.round(value / 1024)}GB`, 'Memory']}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>
        <div className="text-center text-xs text-gray-500 dark:text-gray-400">
          {Math.round(data.total_memory_used / 1024)}GB / {Math.round(data.total_memory_available / 1024)}GB
        </div>
      </div>

      {/* VM States */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 dark:text-white mb-2">
          VM States
        </h4>
        <div className="h-32">
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Pie
                data={vmStateData}
                cx="50%"
                cy="50%"
                innerRadius={25}
                outerRadius={45}
                paddingAngle={2}
                dataKey="value"
              >
                {vmStateData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}
