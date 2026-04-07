import React, { useState, useEffect } from 'react';
import {
  Grid,
  Card,
  CardContent,
  Typography,
  Chip,
  IconButton,
  Button,
  Box,
  LinearProgress,
  Tooltip,
  Fab,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  useMediaQuery,
  Avatar,
} from '@mui/material';
import {
  PlayArrow,
  Stop,
  Pause,
  Settings,
  Memory,
  Storage,
  Add,
  Refresh,
  Computer,
  Timeline,
  Visibility,
  CloudUpload,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { useCustomTheme } from '../contexts/ThemeContext';
import { useAsyncOperation } from '../contexts/NotificationContext';
import { format } from 'date-fns';
import VMDetailsModal from './VMDetailsModal';
import ImportVmDialog from './ImportVmDialog';

interface VM {
  id: string;
  name: string;
  state: string;
  memory: number;
  vcpus: number;
  disk_size: number;
  os_type: string;
  vnc_port?: number;
  created_at: string;
  last_started?: string;
}

interface VMStats {
  cpu_usage: number;
  memory_usage: number;
  memory_total: number;
  disk_read: number;
  disk_write: number;
  network_rx: number;
  network_tx: number;
  uptime: number;
  timestamp: string;
}

interface SystemStats {
  timestamp: string;
  cpu_usage: number;
  memory_used: number;
  memory_total: number;
  memory_percentage: number;
  running_vms: number;
}

interface ProxmoxVMInfo {
  path: string;
  size_gb: number;
  format: string;
  last_modified: string;
  is_running: boolean;
  estimated_memory_usage: number;
}

const VirtualMachinesDashboard: React.FC = () => {
  const [vms, setVms] = useState<VM[]>([]);
  const [systemStats, setSystemStats] = useState<SystemStats | null>(null);
  const [proxmoxInfo, setProxmoxInfo] = useState<ProxmoxVMInfo | null>(null);
  const [selectedVm, setSelectedVm] = useState<VM | null>(null);
  const [vmStats, setVmStats] = useState<Record<string, VMStats>>({});
  const [detailsOpen, setDetailsOpen] = useState(false);
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const { theme, isDarkMode } = useCustomTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const { executeAsync } = useAsyncOperation();

  const PROXMOX_VM_PATH = '/run/media/garuda/Data/proxmox-ve.qcow2';

  // Load data on component mount and set up intervals
  useEffect(() => {
    loadInitialData();
    
    // Set up real-time updates
    const interval = setInterval(() => {
      loadVms();
      loadSystemStats();
      loadVmStats();
    }, 5000); // Update every 5 seconds

    return () => clearInterval(interval);
  }, []);

  const loadInitialData = async () => {
    await Promise.all([
      loadVms(),
      loadSystemStats(),
      loadProxmoxInfo(),
    ]);
  };

  const loadVms = async () => {
    try {
      const vmList = await invoke('get_vms') as VM[];
      setVms(vmList);
    } catch (error) {
      console.error('Failed to load VMs:', error);
    }
  };

  const loadSystemStats = async () => {
    try {
      const stats = await invoke('get_system_statistics') as SystemStats;
      setSystemStats(stats);
    } catch (error) {
      console.error('Failed to load system stats:', error);
    }
  };

  const loadProxmoxInfo = async () => {
    try {
      const info = await invoke('get_proxmox_info', { vmPath: PROXMOX_VM_PATH }) as ProxmoxVMInfo;
      setProxmoxInfo(info);
    } catch (error) {
      console.error('Failed to load Proxmox info:', error);
    }
  };

  const loadVmStats = async () => {
    const runningVms = vms.filter(vm => vm.state === 'running');
    const statsPromises = runningVms.map(async (vm) => {
      try {
        const stats = await invoke('get_vm_stats', { vmId: vm.id }) as VMStats;
        return { vmId: vm.id, stats };
      } catch (error) {
        return null;
      }
    });

    const results = await Promise.all(statsPromises);
    const newStats: Record<string, VMStats> = {};
    
    results.forEach((result) => {
      if (result) {
        newStats[result.vmId] = result.stats;
      }
    });
    
    setVmStats(newStats);
  };

  const handleVmAction = async (vmId: string, action: string, vmName: string) => {
    await executeAsync(
      () => invoke(action === 'start' ? 'start_vm' : action === 'stop' ? 'stop_vm' : 'delete_vm', { vmId }),
      {
        loadingMessage: `${action === 'start' ? 'Starting' : action === 'stop' ? 'Stopping' : 'Deleting'} ${vmName}...`,
        successMessage: `Successfully ${action === 'start' ? 'started' : action === 'stop' ? 'stopped' : 'deleted'} ${vmName}`,
        onSuccess: loadVms,
      }
    );
  };

  const handleCreateProxmoxVM = async (name: string, memoryGb: number, vcpus: number) => {
    await executeAsync(
      () => invoke('create_proxmox_vm', { 
        name, 
        proxmoxPath: PROXMOX_VM_PATH, 
        memoryGb, 
        vcpus 
      }),
      {
        loadingMessage: `Creating Proxmox VM: ${name}...`,
        successMessage: `Successfully created Proxmox VM: ${name}`,
        onSuccess: () => {
          loadVms();
          setCreateDialogOpen(false);
        },
      }
    );
  };

  const getStatusColor = (status: string): "default" | "primary" | "secondary" | "error" | "info" | "success" | "warning" => {
    switch (status.toLowerCase()) {
      case 'running': return 'success';
      case 'paused': return 'warning';
      case 'stopped': case 'shutoff': return 'error';
      case 'starting': return 'info';
      default: return 'default';
    }
  };

  const getOSIcon = (_osType: string) => {
    return <Computer />;
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const SystemOverview: React.FC = () => (
    <Card sx={{ mb: 3, background: isDarkMode ? 'linear-gradient(135deg, #1e2139 0%, #252945 100%)' : 'linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%)' }}>
      <CardContent>
        <Typography variant="h5" sx={{ mb: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
          <Computer color="primary" />
          System Overview
        </Typography>
        
        {systemStats && (
          <Grid container spacing={2}>
            <Grid item xs={12} sm={6} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h4" color="primary">{systemStats.running_vms}</Typography>
                <Typography variant="body2" color="text.secondary">Running VMs</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h4" color="success.main">{systemStats.cpu_usage.toFixed(1)}%</Typography>
                <Typography variant="body2" color="text.secondary">CPU Usage</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h4" color="warning.main">{systemStats.memory_percentage.toFixed(1)}%</Typography>
                <Typography variant="body2" color="text.secondary">Memory Usage</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h4" color="info.main">{formatBytes(systemStats.memory_used * 1024 * 1024)}</Typography>
                <Typography variant="body2" color="text.secondary">Used Memory</Typography>
              </Box>
            </Grid>
          </Grid>
        )}
      </CardContent>
    </Card>
  );

  const ProxmoxVMCard: React.FC = () => (
    proxmoxInfo && (
      <Card sx={{ mb: 3, border: '2px solid', borderColor: 'primary.main' }}>
        <CardContent>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
              <Avatar sx={{ bgcolor: 'primary.main', width: 56, height: 56 }}>
                <Computer />
              </Avatar>
              <Box>
                <Typography variant="h6">Proxmox VE</Typography>
                <Typography variant="body2" color="text.secondary">
                  {proxmoxInfo.size_gb.toFixed(1)} GB • {proxmoxInfo.format.toUpperCase()}
                </Typography>
              </Box>
            </Box>
            <Chip 
              label={proxmoxInfo.is_running ? 'RUNNING' : 'STOPPED'} 
              color={proxmoxInfo.is_running ? 'success' : 'error'}
              size="medium"
              sx={{ fontWeight: 600 }}
            />
          </Box>

          <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
            <Box sx={{ flex: 1, textAlign: 'center', p: 1, bgcolor: 'background.default', borderRadius: 2 }}>
              <Typography variant="h6" color="primary">{formatBytes(proxmoxInfo.estimated_memory_usage)}</Typography>
              <Typography variant="caption">Est. Memory</Typography>
            </Box>
            <Box sx={{ flex: 1, textAlign: 'center', p: 1, bgcolor: 'background.default', borderRadius: 2 }}>
              <Typography variant="h6" color="secondary">{format(new Date(proxmoxInfo.last_modified), 'MMM dd')}</Typography>
              <Typography variant="caption">Last Modified</Typography>
            </Box>
          </Box>

          <Box sx={{ display: 'flex', gap: 1 }}>
            {!proxmoxInfo.is_running && vms.find(vm => vm.name === 'proxmox-ve') && (
              <Button
                variant="contained"
                startIcon={<PlayArrow />}
                onClick={() => {
                  const proxmoxVm = vms.find(vm => vm.name === 'proxmox-ve');
                  if (proxmoxVm) {
                    handleVmAction(proxmoxVm.id, 'start', proxmoxVm.name);
                  }
                }}
                sx={{ flex: 1 }}
              >
                Start Proxmox
              </Button>
            )}
            <Button
              variant="outlined"
              startIcon={<Settings />}
              onClick={() => {
                // Open VM configuration - could integrate with VM details or settings modal
                if (proxmoxInfo) {
                  // For now, just show a success message - this could be expanded
                  // to open a VM configuration dialog
                  console.log('Opening Proxmox configuration for:', proxmoxInfo.path);
                }
              }}
              sx={{ flex: 1 }}
            >
              Configure
            </Button>
          </Box>
        </CardContent>
      </Card>
    )
  );

  const VMCard: React.FC<{ vm: VM }> = ({ vm }) => {
    const stats = vmStats[vm.id];
    const isRunning = vm.state === 'running';

    return (
      <Card 
        sx={{ 
          height: '100%',
          cursor: 'pointer',
          transition: 'all 0.2s ease-in-out',
          border: '1px solid',
          borderColor: 'divider',
          '&:hover': {
            transform: 'translateY(-4px)',
            boxShadow: theme.shadows[8],
            borderColor: 'primary.main',
          }
        }}
        onClick={() => {
          setSelectedVm(vm);
          setDetailsOpen(true);
        }}
      >
        <CardContent sx={{ p: 3 }}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 2 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
              <Avatar sx={{ bgcolor: getStatusColor(vm.state) + '.main' }}>
                {getOSIcon(vm.os_type)}
              </Avatar>
              <Box>
                <Typography variant="h6" sx={{ fontWeight: 600 }}>
                  {vm.name}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {vm.os_type} • {vm.vcpus} vCPUs
                </Typography>
              </Box>
            </Box>
            <Chip 
              label={vm.state.toUpperCase()} 
              color={getStatusColor(vm.state)}
              size="small"
              sx={{ fontWeight: 500 }}
            />
          </Box>

          {/* VM Stats */}
          {isRunning && stats && (
            <Box sx={{ mb: 2 }}>
              <Box sx={{ mb: 1 }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
                  <Typography variant="caption">CPU</Typography>
                  <Typography variant="caption">{stats.cpu_usage.toFixed(1)}%</Typography>
                </Box>
                <LinearProgress 
                  variant="determinate" 
                  value={Math.min(stats.cpu_usage, 100)} 
                  sx={{ height: 6 }}
                  color={stats.cpu_usage > 80 ? 'error' : stats.cpu_usage > 60 ? 'warning' : 'success'}
                />
              </Box>
              <Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
                  <Typography variant="caption">Memory</Typography>
                  <Typography variant="caption">
                    {formatBytes(stats.memory_usage * 1024 * 1024)} / {formatBytes(stats.memory_total * 1024 * 1024)}
                  </Typography>
                </Box>
                <LinearProgress 
                  variant="determinate" 
                  value={(stats.memory_usage / stats.memory_total) * 100} 
                  sx={{ height: 6 }}
                  color={(stats.memory_usage / stats.memory_total) > 0.8 ? 'error' : 'info'}
                />
              </Box>
            </Box>
          )}

          {/* Resource Info */}
          <Box sx={{ display: 'flex', gap: 2, mb: 2, flexWrap: 'wrap' }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
              <Memory fontSize="small" color="primary" />
              <Typography variant="body2">{formatBytes(vm.memory * 1024 * 1024)}</Typography>
            </Box>
            {vm.disk_size > 0 && (
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                <Storage fontSize="small" color="primary" />
                <Typography variant="body2">{formatBytes(vm.disk_size * 1024 * 1024 * 1024)}</Typography>
              </Box>
            )}
            {isRunning && stats && (
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                <Timeline fontSize="small" color="success" />
                <Typography variant="body2">{formatUptime(stats.uptime)}</Typography>
              </Box>
            )}
          </Box>

          {/* Action Buttons */}
          <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end' }}>
            {vm.state === 'stopped' || vm.state === 'shutoff' ? (
              <Tooltip title="Start VM">
                <IconButton 
                  size="small" 
                  color="success"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleVmAction(vm.id, 'start', vm.name);
                  }}
                >
                  <PlayArrow />
                </IconButton>
              </Tooltip>
            ) : (
              <>
                <Tooltip title="Pause VM">
                  <IconButton 
                    size="small" 
                    color="warning"
                    onClick={(e) => {
                      e.stopPropagation();
                      // handleVmAction(vm.id, 'pause', vm.name);
                    }}
                  >
                    <Pause />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Stop VM">
                  <IconButton 
                    size="small" 
                    color="error"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleVmAction(vm.id, 'stop', vm.name);
                    }}
                  >
                    <Stop />
                  </IconButton>
                </Tooltip>
              </>
            )}
            {vm.vnc_port && (
              <Tooltip title="VNC Console">
                <IconButton size="small" color="info">
                  <Visibility />
                </IconButton>
              </Tooltip>
            )}
          </Box>
        </CardContent>
      </Card>
    );
  };

  const CreateVMDialog: React.FC = () => {
    const [vmName] = useState('');
    const [memory] = useState(4);
    const [vcpus] = useState(2);

    return (
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create New Proxmox VM</DialogTitle>
        <DialogContent>
          <Box sx={{ pt: 2 }}>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              Create a new virtual machine using the Proxmox VE image at:
              <br />
              <code>{PROXMOX_VM_PATH}</code>
            </Typography>
            {/* Add form fields here */}
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button 
            variant="contained" 
            onClick={() => handleCreateProxmoxVM(vmName || 'proxmox-vm', memory, vcpus)}
            disabled={!vmName.trim()}
          >
            Create VM
          </Button>
        </DialogActions>
      </Dialog>
    );
  };

  return (
    <Box sx={{ p: isMobile ? 2 : 3 }}>
      {/* Header */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4" component="h1" sx={{ fontWeight: 600 }}>
          Virtual Machines
        </Typography>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Tooltip title="Refresh">
            <IconButton onClick={loadInitialData} color="primary">
              <Refresh />
            </IconButton>
          </Tooltip>
          {!isMobile && (
            <>
              <Button
                variant="contained"
                startIcon={<Add />}
                onClick={() => setCreateDialogOpen(true)}
              >
                Create Proxmox VM
              </Button>
              <Button
                variant="contained"
                color="secondary"
                startIcon={<CloudUpload />}
                onClick={() => setImportDialogOpen(true)}
              >
                Import/Create VM
              </Button>
            </>
          )}
        </Box>
      </Box>

      {/* System Overview */}
      <SystemOverview />

      {/* Proxmox VM Card */}
      <ProxmoxVMCard />

      {/* VM Grid */}
      <Grid container spacing={isMobile ? 2 : 3}>
        {vms.map((vm) => (
          <Grid item xs={12} sm={6} lg={4} xl={3} key={vm.id}>
            <VMCard vm={vm} />
          </Grid>
        ))}
      </Grid>

      {/* Empty State */}
      {vms.length === 0 && (
        <Box sx={{ textAlign: 'center', py: 8 }}>
          <Computer sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
          <Typography variant="h6" color="text.secondary" sx={{ mb: 1 }}>
            No virtual machines found
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
            Create your first VM to get started
          </Typography>
          <Button
            variant="contained"
            startIcon={<Add />}
            onClick={() => setImportDialogOpen(true)}
          >
            Create/Import VM
          </Button>
        </Box>
      )}

      {/* Mobile FAB */}
      {isMobile && (
        <Fab
          color="primary"
          aria-label="add"
          sx={{ position: 'fixed', bottom: 16, right: 16 }}
          onClick={() => setImportDialogOpen(true)}
        >
          <Add />
        </Fab>
      )}

      {/* Dialogs */}
      <CreateVMDialog />
      <ImportVmDialog
        open={importDialogOpen}
        onClose={() => setImportDialogOpen(false)}
        onVmCreated={loadVms}
      />
      <VMDetailsModal 
        vm={selectedVm} 
        open={detailsOpen} 
        onClose={() => {
          setDetailsOpen(false);
          setSelectedVm(null);
        }}
        onVmUpdate={loadVms}
      />
    </Box>
  );
};

export default VirtualMachinesDashboard;
