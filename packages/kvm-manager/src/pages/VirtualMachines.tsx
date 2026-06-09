import { useState } from 'react';
import toast from 'react-hot-toast';

import { VirtualMachine } from '../types';
import VmCard from '../components/VmCard';
import CreateVmModal from '../components/CreateVmModal';
import LoadingSpinner from '../components/LoadingSpinner';

export default function VirtualMachines() {
 const [vms, setVms] = useState<VirtualMachine[]>([]);
 const [loading, setLoading] = useState(true);
 const [showCreateModal, setShowCreateModal] = useState(false);

 useEffect(() => {
 loadVms();
 const interval = setInterval(loadVms, 3000); // Refresh every 3 seconds
 return () => clearInterval(interval);
 }, []);

 const loadVms = async () => {
 try {
 const vmData = await invoke<VirtualMachine[]>('get_vms');
 setVms(vmData);
 } catch (error) {
 console.error('Failed to load VMs:', error);
 toast.error('Failed to load virtual machines');
 } finally {
 setLoading(false);
 }
 };

 const handleStartVm = async (vmId: string) => {
 try {
 await invoke('start_vm', { vmId });
 toast.success('Virtual machine started successfully');
 loadVms(); // Refresh the list
 } catch (error) {
 console.error('Failed to start VM:', error);
 toast.error('Failed to start virtual machine');
 }
 };

 const handleStopVm = async (vmId: string) => {
 try {
 await invoke('stop_vm', { vmId });
 toast.success('Virtual machine stopped successfully');
 loadVms(); // Refresh the list
 } catch (error) {
 console.error('Failed to stop VM:', error);
 toast.error('Failed to stop virtual machine');
 }
 };

 const handleDeleteVm = async (vmId: string) => {
 if (!confirm('Are you sure you want to delete this virtual machine? This action cannot be undone.')) {
 return;
 }

 try {
 await invoke('delete_vm', { vmId });
 toast.success('Virtual machine deleted successfully');
 loadVms(); // Refresh the list
 } catch (error) {
 console.error('Failed to delete VM:', error);
 toast.error('Failed to delete virtual machine');
 }
 };

 return (
 <div className="p-6">
 {/* Header */}
 <div className="flex items-center justify-between mb-8">
 <div>
 <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
 Virtual Machines
 </h1>
 <p className="mt-2 text-gray-600 dark:text-gray-400">
 Manage your virtual machines
 </p>
 </div>
 <button
 onClick={() => setShowCreateModal(true)}
 className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
 >
 <PlusIcon className="w-5 h-5 mr-2" />
 Create VM
 </button>
 </div>

 {/* VM Grid */}
 {loading ? (
 <LoadingSpinner />
 ) : vms.length === 0 ? (
 <div className="text-center py-12">
 <ComputerDesktopIcon className="mx-auto h-12 w-12 text-gray-400" />
 <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
 No virtual machines
 </h3>
 <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
 Get started by creating a new virtual machine.
 </p>
 <div className="mt-6">
 <button
 onClick={() => setShowCreateModal(true)}
 className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
 >
 <PlusIcon className="w-5 h-5 mr-2" />
 Create your first VM
 </button>
 </div>
 </div>
 ) : (
 <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
 {vms.map((vm) => (
 <VmCard
 key={vm.id}
 vm={vm}
 onStart={() => handleStartVm(vm.id)}
 onStop={() => handleStopVm(vm.id)}
 onDelete={() => handleDeleteVm(vm.id)}
 />
 ))}
 </div>
 )}

 {/* Create VM Modal */}
 {showCreateModal && (
 <CreateVmModal
 onClose={() => setShowCreateModal(false)}
 onSuccess={() => {
 setShowCreateModal(false);
 loadVms();
 }}
 />
 )}
 </div>
 );
}