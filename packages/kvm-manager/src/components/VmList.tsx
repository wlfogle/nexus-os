import { Link } from 'react-router-dom';
import { VirtualMachine } from '../types';

interface VmListProps {
  vms: VirtualMachine[];
  title: string;
  showAll?: boolean;
}

export default function VmList({ vms, title, showAll }: VmListProps) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
      <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">{title}</h3>
          {showAll && (
            <Link
              to="/virtual-machines"
              className="text-sm text-blue-600 hover:text-blue-500"
            >
              View all
            </Link>
          )}
        </div>
      </div>
      <div className="divide-y divide-gray-200 dark:divide-gray-700">
        {vms.length === 0 ? (
          <div className="px-6 py-4 text-center text-gray-500 dark:text-gray-400">
            No virtual machines found
          </div>
        ) : (
          vms.map((vm) => (
            <div key={vm.id} className="px-6 py-4 hover:bg-gray-50 dark:hover:bg-gray-700">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-medium text-gray-900 dark:text-white">
                    {vm.name}
                  </h4>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    {vm.vcpus} vCPUs • {Math.round(vm.memory / 1024)}GB RAM
                  </p>
                </div>
                <div className="flex items-center space-x-2">
                  <span className={`px-2 py-1 text-xs font-medium rounded-full ${
                    vm.state === 'Running'
                      ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300'
                      : vm.state === 'Stopped'
                      ? 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
                      : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300'
                  }`}>
                    {vm.state}
                  </span>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
