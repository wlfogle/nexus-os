import React, { useState } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  TextField,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Slider,
  Switch,
  FormControlLabel,
  Alert,
  Chip,
  IconButton,
  Paper,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  ListItemSecondaryAction,
} from '@mui/material';
import {
  CloudUpload,
  Computer,
  Storage,
  Memory,
  FolderOpen,
  Delete,
  Add,
  Warning,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { useAsyncOperation } from '../contexts/NotificationContext';

interface ImportVmDialogProps {
  open: boolean;
  onClose: () => void;
  onVmCreated: () => void;
}

interface QcowInfo {
  path: string;
  filename: string;
  size_gb: number;
  format: string;
  virtual_size_gb: number;
  cluster_size?: number;
  backing_file?: string;
}

interface VmProfile {
  name: string;
  description: string;
  os_type: string;
  os_variant?: string;
  memory: number;
  vcpus: number;
  created_at: string;
  storage_devices: Array<{
    device: string;
    source: string;
    format: string;
    size: number;
    bus: string;
    cache: string;
  }>;
  network_interfaces: Array<{
    mac_address: string;
    network_name: string;
    interface_type: string;
    model: string;
    link_state: string;
  }>;
}

const ImportVmDialog: React.FC<ImportVmDialogProps> = ({ open, onClose, onVmCreated }) => {
  const [activeTab, setActiveTab] = useState<'profiles' | 'qcow2' | 'import' | 'create'>('profiles');
  const [vmName, setVmName] = useState('');
  const [memory, setMemory] = useState(4);
  const [vcpus, setVcpus] = useState(2);
  const [osType, setOsType] = useState('linux');
  const [diskSize, setDiskSize] = useState(20);
  const [enableNvme, setEnableNvme] = useState(false);
  const [enableVirtio, setEnableVirtio] = useState(true);
  const [selectedQcowFiles, setSelectedQcowFiles] = useState<QcowInfo[]>([]);
  const [importXmlPath, setImportXmlPath] = useState('');
  const [loading] = useState(false);
  const [profiles, setProfiles] = useState<VmProfile[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<VmProfile | null>(null);
  const [loadingProfiles, setLoadingProfiles] = useState(false);
  const { executeAsync } = useAsyncOperation();

  // Load profiles when dialog opens or profiles tab is activated
  React.useEffect(() => {
    if (open && activeTab === 'profiles') {
      loadProfiles();
    }
  }, [open, activeTab]);

  const loadProfiles = async () => {
    setLoadingProfiles(true);
    try {
      const profilesList = await invoke('get_profiles') as VmProfile[];
      setProfiles(profilesList);
    } catch (error) {
      console.error('Failed to load profiles:', error);
      setProfiles([]);
    } finally {
      setLoadingProfiles(false);
    }
  };

  const handleCreateVmFromProfile = async () => {
    if (!selectedProfile) {
      return;
    }

    await executeAsync(
      () => invoke('create_vm_from_profile', {
        profileName: selectedProfile.name,
      }),
      {
        loadingMessage: `Creating VM from profile: ${selectedProfile.name}...`,
        successMessage: `Successfully created VM from profile: ${selectedProfile.name}`,
        onSuccess: () => {
          onVmCreated();
          handleClose();
        },
      }
    );
  };

  const handleReset = () => {
    setVmName('');
    setMemory(4);
    setVcpus(2);
    setOsType('linux');
    setDiskSize(20);
    setEnableNvme(false);
    setEnableVirtio(true);
    setSelectedQcowFiles([]);
    setImportXmlPath('');
    setActiveTab('qcow2');
  };

  const handleClose = () => {
    handleReset();
    onClose();
  };

  const handleSelectQcowFile = async () => {
    try {
      const qcowFiles = await invoke('browse_qcow2_files') as string[];
      if (qcowFiles.length > 0) {
        // Get info for each file and add to selection
        const fileInfoPromises = qcowFiles.slice(0, 10).map(async (filePath) => {
          try {
            const info = await invoke('get_qcow2_info', { path: filePath });
            return info;
          } catch (error) {
            console.error(`Failed to get info for ${filePath}:`, error);
            return null;
          }
        });
        
        const fileInfos = (await Promise.all(fileInfoPromises)).filter(Boolean);
        setSelectedQcowFiles(prev => [...prev, ...fileInfos as QcowInfo[]]);
      } else {
        console.log('No QCOW2 files found in common directories');
      }
    } catch (error) {
      console.error('Failed to browse QCOW2 files:', error);
    }
  };

  const handleSelectXmlFile = async () => {
    try {
      const xmlFiles = await invoke('browse_xml_files') as string[];
      if (xmlFiles.length > 0) {
        // For now, just set the first XML file found
        // This could be expanded to show a selection dialog
        setImportXmlPath(xmlFiles[0]);
        console.log('Selected XML file:', xmlFiles[0]);
      } else {
        console.log('No XML files found in common directories');
      }
    } catch (error) {
      console.error('Failed to browse XML files:', error);
    }
  };

  const handleRemoveQcowFile = (path: string) => {
    setSelectedQcowFiles(prev => prev.filter(file => file.path !== path));
  };

  const handleCreateVmFromQcow = async () => {
    if (!vmName.trim() || selectedQcowFiles.length === 0) {
      return;
    }

    // For now, use the first qcow2 file. TODO: Support multiple files
    const firstQcowFile = selectedQcowFiles[0];
    if (!firstQcowFile) return;
    
    await executeAsync(
      () => invoke('create_vm_from_qcow2', {
        qcowPath: firstQcowFile.path,
        vmName: vmName,
        memoryMb: memory * 1024, // Convert GB to MB
        vcpus,
        passthroughDevice: enableNvme ? "/dev/nvme0n1" : null,
      }),
      {
        loadingMessage: `Creating VM from QCOW2: ${vmName}...`,
        successMessage: `Successfully created VM: ${vmName}`,
        onSuccess: () => {
          onVmCreated();
          handleClose();
        },
      }
    );
  };

  const handleImportVm = async () => {
    if (!importXmlPath.trim()) {
      return;
    }

    await executeAsync(
      () => invoke('import_vm_from_xml', { xml_path: importXmlPath }),
      {
        loadingMessage: 'Importing VM from XML...',
        successMessage: 'Successfully imported VM',
        onSuccess: () => {
          onVmCreated();
          handleClose();
        },
      }
    );
  };

  const handleCreateNewVm = async () => {
    if (!vmName.trim()) {
      return;
    }

    await executeAsync(
      () => invoke('create_vm', {
        name: vmName,
        memoryGb: memory,
        vcpus,
        diskSizeGb: diskSize,
        osType,
        enableNvme,
        enableVirtio,
      }),
      {
        loadingMessage: `Creating new VM: ${vmName}...`,
        successMessage: `Successfully created VM: ${vmName}`,
        onSuccess: () => {
          onVmCreated();
          handleClose();
        },
      }
    );
  };

  const formatBytes = (gb: number) => {
    return `${gb.toFixed(1)} GB`;
  };

  const TabButton: React.FC<{ tab: string; label: string; icon: React.ReactNode }> = ({ tab, label, icon }) => (
    <Button
      variant={activeTab === tab ? 'contained' : 'outlined'}
      startIcon={icon}
      onClick={() => setActiveTab(tab as any)}
      sx={{ flex: 1 }}
    >
      {label}
    </Button>
  );

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="md" fullWidth>
      <DialogTitle>VM Management</DialogTitle>
      <DialogContent>
        <Box sx={{ mb: 3 }}>
          {/* Tab Buttons */}
          <Box sx={{ display: 'flex', gap: 1, mb: 3 }}>
            <TabButton tab="profiles" label="From Profile" icon={<Computer />} />
            <TabButton tab="qcow2" label="From Image" icon={<Storage />} />
            <TabButton tab="import" label="Import XML" icon={<CloudUpload />} />
            <TabButton tab="create" label="Create New" icon={<Add />} />
          </Box>

          {/* Profiles Tab */}
          {activeTab === 'profiles' && (
            <Box>
              <Typography variant="h6" sx={{ mb: 2 }}>Create VM from Profile</Typography>
              
              <Alert severity="info" sx={{ mb: 3 }}>
                Select a predefined VM profile to quickly create a VM with optimized settings
              </Alert>

              {loadingProfiles ? (
                <Box sx={{ display: 'flex', justifyContent: 'center', p: 3 }}>
                  <Typography>Loading profiles...</Typography>
                </Box>
              ) : profiles.length === 0 ? (
                <Alert severity="warning" sx={{ mb: 3 }}>
                  No profiles found. Create profiles in the 'profiles' directory.
                </Alert>
              ) : (
                <Box>
                  <Typography variant="subtitle1" sx={{ mb: 2 }}>Available Profiles</Typography>
                  <List>
                    {profiles.map((profile) => (
                      <ListItem
                        key={profile.name}
                        button
                        selected={selectedProfile?.name === profile.name}
                        onClick={() => setSelectedProfile(profile)}
                        sx={{
                          border: 1,
                          borderColor: selectedProfile?.name === profile.name ? 'primary.main' : 'divider',
                          borderRadius: 1,
                          mb: 1,
                          bgcolor: selectedProfile?.name === profile.name ? 'action.selected' : 'background.paper'
                        }}
                      >
                        <ListItemIcon>
                          <Computer color={selectedProfile?.name === profile.name ? 'primary' : 'inherit'} />
                        </ListItemIcon>
                        <ListItemText
                          primary={profile.name}
                          secondary={
                            <Box>
                              <Typography variant="body2" color="text.secondary">
                                {profile.description}
                              </Typography>
                              <Box sx={{ display: 'flex', gap: 1, mt: 1 }}>
                                <Chip 
                                  label={`${profile.memory} MB RAM`} 
                                  size="small" 
                                  icon={<Memory />} 
                                />
                                <Chip 
                                  label={`${profile.vcpus} vCPUs`} 
                                  size="small" 
                                  icon={<Computer />} 
                                />
                                <Chip 
                                  label={profile.os_type} 
                                  size="small" 
                                  color="primary"
                                />
                                <Chip 
                                  label={`${profile.storage_devices.length} disk(s)`} 
                                  size="small" 
                                  icon={<Storage />} 
                                />
                              </Box>
                            </Box>
                          }
                        />
                      </ListItem>
                    ))}
                  </List>
                </Box>
              )}

              {selectedProfile && (
                <Paper sx={{ p: 2, mt: 3, bgcolor: 'background.default' }}>
                  <Typography variant="subtitle1" sx={{ mb: 2 }}>Profile Details</Typography>
                  <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                    <Typography variant="body2"><strong>OS:</strong> {selectedProfile.os_type} {selectedProfile.os_variant && `(${selectedProfile.os_variant})`}</Typography>
                    <Typography variant="body2"><strong>Memory:</strong> {(selectedProfile.memory / 1024).toFixed(1)} GB</Typography>
                    <Typography variant="body2"><strong>vCPUs:</strong> {selectedProfile.vcpus}</Typography>
                    {selectedProfile.storage_devices.length > 0 && (
                      <Box>
                        <Typography variant="body2"><strong>Storage Devices:</strong></Typography>
                        {selectedProfile.storage_devices.map((device, index) => (
                          <Typography key={index} variant="caption" display="block" sx={{ ml: 2 }}>
                            • {device.device}: {device.source} ({device.format})
                          </Typography>
                        ))}
                      </Box>
                    )}
                    {selectedProfile.network_interfaces.length > 0 && (
                      <Box>
                        <Typography variant="body2"><strong>Network:</strong></Typography>
                        {selectedProfile.network_interfaces.map((iface, index) => (
                          <Typography key={index} variant="caption" display="block" sx={{ ml: 2 }}>
                            • {iface.model} on {iface.network_name} ({iface.mac_address})
                          </Typography>
                        ))}
                      </Box>
                    )}
                  </Box>
                </Paper>
              )}
            </Box>
          )}

          {/* QCOW2 Tab */}
          {activeTab === 'qcow2' && (
            <Box>
              <Typography variant="h6" sx={{ mb: 2 }}>Create VM from Disk Images</Typography>
              
              {/* File Selection */}
              <Paper sx={{ p: 2, mb: 3, bgcolor: 'background.default' }}>
                <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
                  <Typography variant="subtitle1">Disk Images</Typography>
                  <Button
                    variant="outlined"
                    startIcon={<FolderOpen />}
                    onClick={handleSelectQcowFile}
                    disabled={loading}
                  >
                    Select Images
                  </Button>
                </Box>

                {selectedQcowFiles.length > 0 ? (
                  <List dense>
                    {selectedQcowFiles.map((file) => (
                      <ListItem key={file.path} sx={{ px: 0 }}>
                        <ListItemIcon>
                          <Storage color="primary" />
                        </ListItemIcon>
                        <ListItemText
                          primary={file.filename}
                          secondary={
                            <Box>
                              <Typography variant="caption" component="div">
                                {formatBytes(file.size_gb)} on disk • {formatBytes(file.virtual_size_gb)} virtual • {file.format.toUpperCase()}
                              </Typography>
                              {file.backing_file && (
                                <Chip label="Linked Clone" size="small" color="info" sx={{ mt: 0.5 }} />
                              )}
                            </Box>
                          }
                        />
                        <ListItemSecondaryAction>
                          <IconButton
                            edge="end"
                            onClick={() => handleRemoveQcowFile(file.path)}
                            size="small"
                          >
                            <Delete />
                          </IconButton>
                        </ListItemSecondaryAction>
                      </ListItem>
                    ))}
                  </List>
                ) : (
                  <Alert severity="info" sx={{ mb: 2 }}>
                    Select one or more disk images (QCOW2, VMDK, VDI) to create a VM
                  </Alert>
                )}
              </Paper>

              {/* VM Configuration */}
              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                <TextField
                  label="VM Name"
                  value={vmName}
                  onChange={(e) => setVmName(e.target.value)}
                  fullWidth
                  required
                />

                <Box sx={{ display: 'flex', gap: 2 }}>
                  <FormControl fullWidth>
                    <InputLabel>OS Type</InputLabel>
                    <Select
                      value={osType}
                      onChange={(e) => setOsType(e.target.value)}
                      label="OS Type"
                    >
                      <MenuItem value="linux">Linux</MenuItem>
                      <MenuItem value="windows">Windows</MenuItem>
                      <MenuItem value="unix">Unix</MenuItem>
                      <MenuItem value="other">Other</MenuItem>
                    </Select>
                  </FormControl>
                </Box>

                <Box sx={{ display: 'flex', gap: 2 }}>
                  <Box sx={{ flex: 1 }}>
                    <Typography gutterBottom>
                      <Memory fontSize="small" sx={{ verticalAlign: 'middle', mr: 1 }} />
                      Memory: {memory} GB
                    </Typography>
                    <Slider
                      value={memory}
                      onChange={(_, value) => setMemory(value as number)}
                      min={1}
                      max={64}
                      step={1}
                      marks={[
                        { value: 2, label: '2GB' },
                        { value: 8, label: '8GB' },
                        { value: 16, label: '16GB' },
                        { value: 32, label: '32GB' },
                      ]}
                    />
                  </Box>
                  
                  <Box sx={{ flex: 1 }}>
                    <Typography gutterBottom>
                      <Computer fontSize="small" sx={{ verticalAlign: 'middle', mr: 1 }} />
                      vCPUs: {vcpus}
                    </Typography>
                    <Slider
                      value={vcpus}
                      onChange={(_, value) => setVcpus(value as number)}
                      min={1}
                      max={16}
                      step={1}
                      marks={[
                        { value: 1, label: '1' },
                        { value: 2, label: '2' },
                        { value: 4, label: '4' },
                        { value: 8, label: '8' },
                      ]}
                    />
                  </Box>
                </Box>

                <Box sx={{ display: 'flex', gap: 2 }}>
                  <FormControlLabel
                    control={
                      <Switch
                        checked={enableVirtio}
                        onChange={(e) => setEnableVirtio(e.target.checked)}
                      />
                    }
                    label="VirtIO (Recommended)"
                  />
                  <FormControlLabel
                    control={
                      <Switch
                        checked={enableNvme}
                        onChange={(e) => setEnableNvme(e.target.checked)}
                      />
                    }
                    label="NVMe Storage"
                  />
                </Box>

                {selectedQcowFiles.some(f => f.backing_file) && (
                  <Alert severity="warning" icon={<Warning />}>
                    Some selected images are linked clones. Make sure the backing files are accessible.
                  </Alert>
                )}
              </Box>
            </Box>
          )}

          {/* Import XML Tab */}
          {activeTab === 'import' && (
            <Box>
              <Typography variant="h6" sx={{ mb: 2 }}>Import VM from XML</Typography>
              
              <Alert severity="info" sx={{ mb: 3 }}>
                Import an existing VM configuration from a libvirt XML file
              </Alert>

              <Paper sx={{ p: 2, mb: 3, bgcolor: 'background.default' }}>
                <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
                  <Typography variant="subtitle1">XML Configuration</Typography>
                  <Button
                    variant="outlined"
                    startIcon={<FolderOpen />}
                    onClick={handleSelectXmlFile}
                  >
                    Select XML File
                  </Button>
                </Box>

                {importXmlPath ? (
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <CloudUpload color="primary" />
                    <Typography variant="body2">{importXmlPath.split('/').pop()}</Typography>
                    <IconButton size="small" onClick={() => setImportXmlPath('')}>
                      <Delete />
                    </IconButton>
                  </Box>
                ) : (
                  <Typography variant="body2" color="text.secondary">
                    No XML file selected
                  </Typography>
                )}
              </Paper>
            </Box>
          )}

          {/* Create New VM Tab */}
          {activeTab === 'create' && (
            <Box>
              <Typography variant="h6" sx={{ mb: 2 }}>Create New VM</Typography>
              
              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                <TextField
                  label="VM Name"
                  value={vmName}
                  onChange={(e) => setVmName(e.target.value)}
                  fullWidth
                  required
                />

                <FormControl fullWidth>
                  <InputLabel>OS Type</InputLabel>
                  <Select
                    value={osType}
                    onChange={(e) => setOsType(e.target.value)}
                    label="OS Type"
                  >
                    <MenuItem value="linux">Linux</MenuItem>
                    <MenuItem value="windows">Windows</MenuItem>
                    <MenuItem value="unix">Unix</MenuItem>
                    <MenuItem value="other">Other</MenuItem>
                  </Select>
                </FormControl>

                <Box sx={{ display: 'flex', gap: 2 }}>
                  <Box sx={{ flex: 1 }}>
                    <Typography gutterBottom>
                      <Memory fontSize="small" sx={{ verticalAlign: 'middle', mr: 1 }} />
                      Memory: {memory} GB
                    </Typography>
                    <Slider
                      value={memory}
                      onChange={(_, value) => setMemory(value as number)}
                      min={1}
                      max={64}
                      step={1}
                      marks={[
                        { value: 2, label: '2GB' },
                        { value: 8, label: '8GB' },
                        { value: 16, label: '16GB' },
                        { value: 32, label: '32GB' },
                      ]}
                    />
                  </Box>
                  
                  <Box sx={{ flex: 1 }}>
                    <Typography gutterBottom>
                      <Computer fontSize="small" sx={{ verticalAlign: 'middle', mr: 1 }} />
                      vCPUs: {vcpus}
                    </Typography>
                    <Slider
                      value={vcpus}
                      onChange={(_, value) => setVcpus(value as number)}
                      min={1}
                      max={16}
                      step={1}
                      marks={[
                        { value: 1, label: '1' },
                        { value: 2, label: '2' },
                        { value: 4, label: '4' },
                        { value: 8, label: '8' },
                      ]}
                    />
                  </Box>
                </Box>

                <Box>
                  <Typography gutterBottom>
                    <Storage fontSize="small" sx={{ verticalAlign: 'middle', mr: 1 }} />
                    Disk Size: {diskSize} GB
                  </Typography>
                  <Slider
                    value={diskSize}
                    onChange={(_, value) => setDiskSize(value as number)}
                    min={10}
                    max={500}
                    step={5}
                    marks={[
                      { value: 20, label: '20GB' },
                      { value: 50, label: '50GB' },
                      { value: 100, label: '100GB' },
                      { value: 250, label: '250GB' },
                    ]}
                  />
                </Box>

                <Box sx={{ display: 'flex', gap: 2 }}>
                  <FormControlLabel
                    control={
                      <Switch
                        checked={enableVirtio}
                        onChange={(e) => setEnableVirtio(e.target.checked)}
                      />
                    }
                    label="VirtIO (Recommended)"
                  />
                  <FormControlLabel
                    control={
                      <Switch
                        checked={enableNvme}
                        onChange={(e) => setEnableNvme(e.target.checked)}
                      />
                    }
                    label="NVMe Storage"
                  />
                </Box>
              </Box>
            </Box>
          )}
        </Box>
      </DialogContent>

      <DialogActions>
        <Button onClick={handleClose}>Cancel</Button>
        {activeTab === 'profiles' && (
          <Button
            variant="contained"
            onClick={handleCreateVmFromProfile}
            disabled={!selectedProfile}
          >
            Create VM from Profile
          </Button>
        )}
        {activeTab === 'qcow2' && (
          <Button
            variant="contained"
            onClick={handleCreateVmFromQcow}
            disabled={!vmName.trim() || selectedQcowFiles.length === 0}
          >
            Create VM
          </Button>
        )}
        {activeTab === 'import' && (
          <Button
            variant="contained"
            onClick={handleImportVm}
            disabled={!importXmlPath.trim()}
          >
            Import VM
          </Button>
        )}
        {activeTab === 'create' && (
          <Button
            variant="contained"
            onClick={handleCreateNewVm}
            disabled={!vmName.trim()}
          >
            Create VM
          </Button>
        )}
      </DialogActions>
    </Dialog>
  );
};

export default ImportVmDialog;
