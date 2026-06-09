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
 <a href="/virtual-machines" className="text-sm text-blue-600 hover:text-blue-500">
 View all
 </a>
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
 </div>
 </div>
 ))
 )}
 </div>
 </div>
 );
}