import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  Grid,
  Card,
  CardContent,
  LinearProgress,
  Chip,
  IconButton,
  Tabs,
  Tab,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  TextField,
  Tooltip,
  Avatar,
} from '@mui/material';
import {
  Close,
  Computer,
  Memory,
  Storage,
  PlayArrow,
  Stop,
  Pause,
  RestartAlt,
  Save,
  Edit,
  Visibility,
  Screenshot,
  Terminal,
  Speed,
  Settings,
} from '@mui/icons-material';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip as ChartTooltip, ResponsiveContainer } from 'recharts';
import { format } from 'date-fns';
// import { invoke } from '@tauri-apps/api/tauri';
const invoke = async (command: string, args?: any): Promise<any> => {
  console.log('Tauri invoke:', command, args);
  // Placeholder for Tauri invoke function
  return Promise.reject(new Error('Tauri invoke not available'));
};
import { useAsyncOperation } from '../contexts/NotificationContext';

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

interface VMDetailsModalProps {
  vm: VM | null;
  open: boolean;
  onClose: () => void;
  onVmUpdate?: () => void;
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

const TabPanel: React.FC<TabPanelProps> = ({ children, value, index }) => (
  <div hidden={value !== index} style={{ paddingTop: 16 }}>
    {value === index && children}
  </div>
);

const VMDetailsModal: React.FC<VMDetailsModalProps> = ({ vm, open, onClose, onVmUpdate }) => {
  const [currentTab, setCurrentTab] = useState(0);
  const [vmStats, setVmStats] = useState<VMStats | null>(null);
  const [statsHistory, setStatsHistory] = useState<VMStats[]>([]);
  const [editMode, setEditMode] = useState(false);
  const [editedVm, setEditedVm] = useState<Partial<VM>>({});
  const { executeAsync } = useAsyncOperation();

  useEffect(() => {
    if (vm && open) {
      setEditedVm({ ...vm });
      loadVmStats();
      
      // Load stats every 5 seconds when modal is open
      const interval = setInterval(loadVmStats, 5000);
      return () => clearInterval(interval);
    }
    return; // Explicit return for when condition is false
  }, [vm, open]);

  const loadVmStats = async () => {
    if (!vm || vm.state !== 'running') return;
    
    try {
      const stats = await invoke('get_vm_stats', { vmId: vm.id }) as VMStats;
      setVmStats(stats);
      
      // Add to history (keep last 20 points for charts)
      setStatsHistory(prev => {
        const newHistory = [...prev, stats].slice(-20);
        return newHistory;
      });
    } catch (error) {
      console.error('Failed to load VM stats:', error);
    }
  };

  const handleVmAction = async (action: string) => {
    if (!vm) return;
    
    await executeAsync(
      () => invoke(action === 'start' ? 'start_vm' : action === 'stop' ? 'stop_vm' : 'pause_vm', { vmId: vm.id }),
      {
        loadingMessage: `${action === 'start' ? 'Starting' : action === 'stop' ? 'Stopping' : 'Pausing'} ${vm.name}...`,
        successMessage: `Successfully ${action}ed ${vm.name}`,
        onSuccess: () => {
          onVmUpdate?.();
          onClose();
        },
      }
    );
  };

  const handleSaveChanges = async () => {
    if (!vm) return;
    
    await executeAsync(
      () => invoke('update_vm', { vmId: vm.id, updates: editedVm }),
      {
        loadingMessage: 'Updating VM configuration...',
        successMessage: 'VM configuration updated successfully',
        onSuccess: () => {
          setEditMode(false);
          onVmUpdate?.();
        },
      }
    );
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
    
    if (days > 0) return `${days}d ${hours}h ${minutes}m`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
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

  if (!vm) return null;

  return (
    <Dialog open={open} onClose={onClose} maxWidth="lg" fullWidth>
      <DialogTitle>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Avatar sx={{ bgcolor: getStatusColor(vm.state) + '.main' }}>
              <Computer />
            </Avatar>
            <Box>
              <Typography variant="h6">{vm.name}</Typography>
              <Chip label={vm.state.toUpperCase()} color={getStatusColor(vm.state)} size="small" />
            </Box>
          </Box>
          <Box sx={{ display: 'flex', gap: 1 }}>
            <Tooltip title={editMode ? "Save Changes" : "Edit VM"}>
              <IconButton onClick={editMode ? handleSaveChanges : () => setEditMode(true)}>
                {editMode ? <Save /> : <Edit />}
              </IconButton>
            </Tooltip>
            <IconButton onClick={onClose}>
              <Close />
            </IconButton>
          </Box>
        </Box>
      </DialogTitle>

      <DialogContent>
        <Tabs value={currentTab} onChange={(_, value) => setCurrentTab(value)} sx={{ mb: 3 }}>
          <Tab label="Overview" />
          <Tab label="Performance" />
          <Tab label="Configuration" />
          <Tab label="Console" />
        </Tabs>

        {/* Overview Tab */}
        <TabPanel value={currentTab} index={0}>
          <Grid container spacing={3}>
            {/* Current Stats */}
            {vm.state === 'running' && vmStats && (
              <Grid item xs={12}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 2 }}>Live Performance</Typography>
                    <Grid container spacing={3}>
                      <Grid item xs={12} sm={6} md={3}>
                        <Box sx={{ textAlign: 'center' }}>
                          <Typography variant="h4" color="primary">{vmStats.cpu_usage.toFixed(1)}%</Typography>
                          <Typography variant="body2" color="text.secondary">CPU Usage</Typography>
                          <LinearProgress 
                            variant="determinate" 
                            value={vmStats.cpu_usage} 
                            sx={{ mt: 1, height: 6 }}
                            color={vmStats.cpu_usage > 80 ? 'error' : vmStats.cpu_usage > 60 ? 'warning' : 'success'}
                          />
                        </Box>
                      </Grid>
                      <Grid item xs={12} sm={6} md={3}>
                        <Box sx={{ textAlign: 'center' }}>
                          <Typography variant="h4" color="info.main">
                            {((vmStats.memory_usage / vmStats.memory_total) * 100).toFixed(1)}%
                          </Typography>
                          <Typography variant="body2" color="text.secondary">Memory Usage</Typography>
                          <Typography variant="caption" color="text.secondary">
                            {formatBytes(vmStats.memory_usage * 1024 * 1024)} / {formatBytes(vmStats.memory_total * 1024 * 1024)}
                          </Typography>
                          <LinearProgress 
                            variant="determinate" 
                            value={(vmStats.memory_usage / vmStats.memory_total) * 100} 
                            sx={{ mt: 1, height: 6 }}
                            color="info"
                          />
                        </Box>
                      </Grid>
                      <Grid item xs={12} sm={6} md={3}>
                        <Box sx={{ textAlign: 'center' }}>
                          <Typography variant="h4" color="success.main">
                            {formatBytes(vmStats.disk_read + vmStats.disk_write)}
                          </Typography>
                          <Typography variant="body2" color="text.secondary">Disk I/O</Typography>
                          <Typography variant="caption" color="text.secondary">
                            ↓{formatBytes(vmStats.disk_read)} ↑{formatBytes(vmStats.disk_write)}
                          </Typography>
                        </Box>
                      </Grid>
                      <Grid item xs={12} sm={6} md={3}>
                        <Box sx={{ textAlign: 'center' }}>
                          <Typography variant="h4" color="warning.main">
                            {formatUptime(vmStats.uptime)}
                          </Typography>
                          <Typography variant="body2" color="text.secondary">Uptime</Typography>
                        </Box>
                      </Grid>
                    </Grid>
                  </CardContent>
                </Card>
              </Grid>
            )}

            {/* VM Information */}
            <Grid item xs={12} md={6}>
              <Card>
                <CardContent>
                  <Typography variant="h6" sx={{ mb: 2 }}>VM Information</Typography>
                  <List dense>
                    <ListItem>
                      <ListItemIcon><Computer /></ListItemIcon>
                      <ListItemText primary="VM ID" secondary={vm.id} />
                    </ListItem>
                    <ListItem>
                      <ListItemIcon><Memory /></ListItemIcon>
                      <ListItemText primary="Memory" secondary={formatBytes(vm.memory * 1024 * 1024)} />
                    </ListItem>
                    <ListItem>
                      <ListItemIcon><Speed /></ListItemIcon>
                      <ListItemText primary="vCPUs" secondary={vm.vcpus} />
                    </ListItem>
                    <ListItem>
                      <ListItemIcon><Storage /></ListItemIcon>
                      <ListItemText primary="Disk Size" secondary={formatBytes(vm.disk_size * 1024 * 1024 * 1024)} />
                    </ListItem>
                    <ListItem>
                      <ListItemIcon><Computer /></ListItemIcon>
                      <ListItemText primary="OS Type" secondary={vm.os_type} />
                    </ListItem>
                    {vm.vnc_port && (
                      <ListItem>
                        <ListItemIcon><Visibility /></ListItemIcon>
                        <ListItemText primary="VNC Port" secondary={vm.vnc_port} />
                      </ListItem>
                    )}
                  </List>
                </CardContent>
              </Card>
            </Grid>

            {/* Actions */}
            <Grid item xs={12} md={6}>
              <Card>
                <CardContent>
                  <Typography variant="h6" sx={{ mb: 2 }}>Actions</Typography>
                  <Grid container spacing={2}>
                    {vm.state === 'running' ? (
                      <>
                        <Grid item xs={6}>
                          <Button
                            fullWidth
                            variant="outlined"
                            color="warning"
                            startIcon={<Pause />}
                            onClick={() => handleVmAction('pause')}
                          >
                            Pause
                          </Button>
                        </Grid>
                        <Grid item xs={6}>
                          <Button
                            fullWidth
                            variant="outlined"
                            color="error"
                            startIcon={<Stop />}
                            onClick={() => handleVmAction('stop')}
                          >
                            Stop
                          </Button>
                        </Grid>
                        <Grid item xs={6}>
                          <Button
                            fullWidth
                            variant="outlined"
                            startIcon={<RestartAlt />}
                            onClick={() => handleVmAction('restart')}
                          >
                            Restart
                          </Button>
                        </Grid>
                        <Grid item xs={6}>
                          <Button
                            fullWidth
                            variant="outlined"
                            color="info"
                            startIcon={<Visibility />}
                          >
                            VNC Console
                          </Button>
                        </Grid>
                      </>
                    ) : (
                      <Grid item xs={12}>
                        <Button
                          fullWidth
                          variant="contained"
                          color="success"
                          startIcon={<PlayArrow />}
                          onClick={() => handleVmAction('start')}
                        >
                          Start VM
                        </Button>
                      </Grid>
                    )}
                    <Grid item xs={6}>
                      <Button
                        fullWidth
                        variant="outlined"
                        startIcon={<Screenshot />}
                      >
                        Screenshot
                      </Button>
                    </Grid>
                    <Grid item xs={6}>
                      <Button
                        fullWidth
                        variant="outlined"
                        startIcon={<Settings />}
                      >
                        Settings
                      </Button>
                    </Grid>
                  </Grid>
                </CardContent>
              </Card>
            </Grid>
          </Grid>
        </TabPanel>

        {/* Performance Tab */}
        <TabPanel value={currentTab} index={1}>
          {statsHistory.length > 0 && (
            <Grid container spacing={3}>
              <Grid item xs={12} md={6}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 2 }}>CPU Usage History</Typography>
                    <ResponsiveContainer width="100%" height={200}>
                      <AreaChart data={statsHistory}>
                        <CartesianGrid strokeDasharray="3 3" />
                        <XAxis 
                          dataKey="timestamp" 
                          tickFormatter={(value) => format(new Date(value), 'HH:mm:ss')}
                        />
                        <YAxis domain={[0, 100]} />
                        <ChartTooltip 
                          labelFormatter={(value) => format(new Date(value), 'HH:mm:ss')}
                          formatter={(value: any) => [`${value.toFixed(1)}%`, 'CPU Usage']}
                        />
                        <Area type="monotone" dataKey="cpu_usage" stroke="#1976d2" fill="#1976d2" fillOpacity={0.3} />
                      </AreaChart>
                    </ResponsiveContainer>
                  </CardContent>
                </Card>
              </Grid>

              <Grid item xs={12} md={6}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 2 }}>Memory Usage History</Typography>
                    <ResponsiveContainer width="100%" height={200}>
                      <AreaChart data={statsHistory}>
                        <CartesianGrid strokeDasharray="3 3" />
                        <XAxis 
                          dataKey="timestamp" 
                          tickFormatter={(value) => format(new Date(value), 'HH:mm:ss')}
                        />
                        <YAxis />
                        <ChartTooltip 
                          labelFormatter={(value) => format(new Date(value), 'HH:mm:ss')}
                          formatter={(value: any) => [formatBytes(value * 1024 * 1024), 'Memory Usage']}
                        />
                        <Area type="monotone" dataKey="memory_usage" stroke="#0288d1" fill="#0288d1" fillOpacity={0.3} />
                      </AreaChart>
                    </ResponsiveContainer>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          )}
        </TabPanel>

        {/* Configuration Tab */}
        <TabPanel value={currentTab} index={2}>
          <Card>
            <CardContent>
              <Typography variant="h6" sx={{ mb: 2 }}>VM Configuration</Typography>
              {editMode ? (
                <Grid container spacing={3}>
                  <Grid item xs={12} sm={6}>
                    <TextField
                      fullWidth
                      label="VM Name"
                      value={editedVm.name || ''}
                      onChange={(e) => setEditedVm({ ...editedVm, name: e.target.value })}
                    />
                  </Grid>
                  <Grid item xs={12} sm={6}>
                    <TextField
                      fullWidth
                      label="vCPUs"
                      type="number"
                      value={editedVm.vcpus || 0}
                      onChange={(e) => setEditedVm({ ...editedVm, vcpus: parseInt(e.target.value) })}
                    />
                  </Grid>
                  <Grid item xs={12} sm={6}>
                    <TextField
                      fullWidth
                      label="Memory (MB)"
                      type="number"
                      value={editedVm.memory || 0}
                      onChange={(e) => setEditedVm({ ...editedVm, memory: parseInt(e.target.value) })}
                    />
                  </Grid>
                  <Grid item xs={12} sm={6}>
                    <TextField
                      fullWidth
                      label="OS Type"
                      value={editedVm.os_type || ''}
                      onChange={(e) => setEditedVm({ ...editedVm, os_type: e.target.value })}
                    />
                  </Grid>
                </Grid>
              ) : (
                <Typography variant="body2" color="text.secondary">
                  Click the edit button to modify VM configuration
                </Typography>
              )}
            </CardContent>
          </Card>
        </TabPanel>

        {/* Console Tab */}
        <TabPanel value={currentTab} index={3}>
          <Card>
            <CardContent>
              <Typography variant="h6" sx={{ mb: 2 }}>Console Access</Typography>
              <Box sx={{ textAlign: 'center', py: 4 }}>
                <Terminal sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
                <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
                  Console access will be available here
                </Typography>
                <Button variant="contained" startIcon={<Terminal />}>
                  Open Console
                </Button>
              </Box>
            </CardContent>
          </Card>
        </TabPanel>
      </DialogContent>

      <DialogActions>
        <Button onClick={onClose}>Close</Button>
      </DialogActions>
    </Dialog>
  );
};

export default VMDetailsModal;
