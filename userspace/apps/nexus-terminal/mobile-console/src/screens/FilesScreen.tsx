import React, { useState, useEffect } from 'react';
import {
  View,
  StyleSheet,
  ScrollView,
  Alert,
  TouchableOpacity,
  Modal,
} from 'react-native';
import {
  Text,
  Card,
  Button,
  FAB,
  List,
  Searchbar,
  TextInput,
  Portal,
  Dialog,
} from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import { ConnectionManager } from '@services/ConnectionManager';

interface FileItem {
  name: string;
  path: string;
  type: 'file' | 'directory';
  size: number;
  modified: string;
  permissions: string;
  isHidden: boolean;
}

interface BreadcrumbItem {
  name: string;
  path: string;
}

export const FilesScreen: React.FC = () => {
  const [currentPath, setCurrentPath] = useState('/home');
  const [files, setFiles] = useState<FileItem[]>([]);
  const [filteredFiles, setFilteredFiles] = useState<FileItem[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [showHidden, setShowHidden] = useState(false);
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(null);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newItemName, setNewItemName] = useState('');
  const [createType, setCreateType] = useState<'file' | 'directory'>('file');

  useEffect(() => {
    loadDirectory(currentPath);
  }, [currentPath, showHidden]);

  useEffect(() => {
    filterFiles();
  }, [files, searchQuery, showHidden]);

  const loadDirectory = async (path: string) => {
    setIsLoading(true);
    try {
      const items = await ConnectionManager.browseFiles(path);
      const fileItems: FileItem[] = items.map(item => ({
        name: item.name,
        path: item.path,
        type: item.type,
        size: item.size || 0,
        modified: item.modified || '',
        permissions: item.permissions || '',
        isHidden: item.name.startsWith('.'),
      }));
      
      setFiles(fileItems);
    } catch (error) {
      Alert.alert('Error', 'Failed to load directory');
      console.error('Failed to load directory:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const filterFiles = () => {
    let filtered = files;
    
    if (!showHidden) {
      filtered = filtered.filter(file => !file.isHidden);
    }
    
    if (searchQuery) {
      filtered = filtered.filter(file => 
        file.name.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }
    
    // Sort: directories first, then files, alphabetically
    filtered.sort((a, b) => {
      if (a.type !== b.type) {
        return a.type === 'directory' ? -1 : 1;
      }
      return a.name.localeCompare(b.name);
    });
    
    setFilteredFiles(filtered);
  };

  const navigateToPath = (path: string) => {
    setCurrentPath(path);
    setSearchQuery('');
  };

  const navigateUp = () => {
    if (currentPath === '/') return;
    const parentPath = currentPath.split('/').slice(0, -1).join('/') || '/';
    navigateToPath(parentPath);
  };

  const getBreadcrumbs = (): BreadcrumbItem[] => {
    const parts = currentPath.split('/').filter(Boolean);
    const breadcrumbs: BreadcrumbItem[] = [{ name: 'root', path: '/' }];
    
    let currentBreadcrumbPath = '';
    for (const part of parts) {
      currentBreadcrumbPath += '/' + part;
      breadcrumbs.push({
        name: part,
        path: currentBreadcrumbPath,
      });
    }
    
    return breadcrumbs;
  };

  const handleFilePress = async (file: FileItem) => {
    if (file.type === 'directory') {
      navigateToPath(file.path);
    } else {
      // Try to read file content
      try {
        const content = await ConnectionManager.readFile(file.path);
        Alert.alert(
          file.name,
          content.length > 500 ? content.substring(0, 500) + '...' : content,
          [
            { text: 'Close' },
            { text: 'Edit', onPress: () => editFile(file, content) },
          ]
        );
      } catch (error) {
        Alert.alert('Error', 'Cannot read file content');
      }
    }
  };

  const handleFileLongPress = (file: FileItem) => {
    setSelectedFile(file);
    showFileActions(file);
  };

  const showFileActions = (file: FileItem) => {
    const actions = [
      { text: 'Cancel', style: 'cancel' as const },
      { text: 'Delete', style: 'destructive' as const, onPress: () => deleteFile(file) },
    ];

    if (file.type === 'file') {
      actions.splice(1, 0, { 
        text: 'Edit', 
        style: 'default' as const, 
        onPress: () => editFileAction(file) 
      });
    }

    Alert.alert(file.name, 'Choose an action:', actions);
  };

  const editFileAction = async (file: FileItem) => {
    try {
      const content = await ConnectionManager.readFile(file.path);
      editFile(file, content);
    } catch (error) {
      Alert.alert('Error', 'Cannot read file for editing');
    }
  };

  const editFile = (file: FileItem, content: string) => {
    Alert.prompt(
      `Edit ${file.name}`,
      'File content:',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Save',
          onPress: async (text) => {
            if (text !== undefined) {
              try {
                await ConnectionManager.writeFile(file.path, text);
                Alert.alert('Success', 'File saved successfully');
                loadDirectory(currentPath); // Refresh
              } catch (error) {
                Alert.alert('Error', 'Failed to save file');
              }
            }
          },
        },
      ],
      'plain-text',
      content
    );
  };

  const deleteFile = async (file: FileItem) => {
    Alert.alert(
      'Delete File',
      `Are you sure you want to delete "${file.name}"?`,
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Delete',
          style: 'destructive',
          onPress: async () => {
            try {
              await ConnectionManager.deleteFile(file.path);
              Alert.alert('Success', 'File deleted successfully');
              loadDirectory(currentPath); // Refresh
            } catch (error) {
              Alert.alert('Error', 'Failed to delete file');
            }
          },
        },
      ]
    );
  };

  const createItem = async () => {
    if (!newItemName.trim()) {
      Alert.alert('Error', 'Please enter a name');
      return;
    }

    const itemPath = `${currentPath}/${newItemName.trim()}`;
    
    try {
      if (createType === 'file') {
        await ConnectionManager.writeFile(itemPath, '');
      } else {
        // For directories, we'd need a separate API endpoint
        await ConnectionManager.executeCommand({
          command: `mkdir -p "${itemPath}"`,
        });
      }
      
      Alert.alert('Success', `${createType} created successfully`);
      setShowCreateDialog(false);
      setNewItemName('');
      loadDirectory(currentPath); // Refresh
    } catch (error) {
      Alert.alert('Error', `Failed to create ${createType}`);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  };

  const getFileIcon = (file: FileItem): string => {
    if (file.type === 'directory') return 'folder';
    
    const ext = file.name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'js': case 'ts': case 'jsx': case 'tsx': return 'code';
      case 'py': return 'code';
      case 'rs': return 'code';
      case 'go': return 'code';
      case 'txt': case 'md': case 'readme': return 'description';
      case 'jpg': case 'jpeg': case 'png': case 'gif': return 'image';
      case 'mp4': case 'avi': case 'mov': return 'movie';
      case 'mp3': case 'wav': case 'flac': return 'audiotrack';
      case 'pdf': return 'picture-as-pdf';
      case 'zip': case 'tar': case 'gz': return 'archive';
      default: return 'insert-drive-file';
    }
  };

  return (
    <View style={styles.container}>
      {/* Search bar */}
      <Searchbar
        placeholder="Search files..."
        onChangeText={setSearchQuery}
        value={searchQuery}
        style={styles.searchBar}
        inputStyle={styles.searchInput}
      />

      {/* Breadcrumb navigation */}
      <ScrollView horizontal showsHorizontalScrollIndicator={false} style={styles.breadcrumbContainer}>
        {getBreadcrumbs().map((crumb, index) => (
          <TouchableOpacity
            key={index}
            onPress={() => navigateToPath(crumb.path)}
            style={styles.breadcrumbItem}
          >
            <Text style={styles.breadcrumbText}>{crumb.name}</Text>
            {index < getBreadcrumbs().length - 1 && (
              <Icon name="chevron-right" size={16} color="#666" />
            )}
          </TouchableOpacity>
        ))}
      </ScrollView>

      {/* File list */}
      <ScrollView style={styles.fileList}>
        {currentPath !== '/' && (
          <List.Item
            title=".."
            description="Parent directory"
            left={() => <Icon name="arrow-back" size={24} color="#4CAF50" />}
            onPress={navigateUp}
            style={styles.fileItem}
          />
        )}
        
        {filteredFiles.map((file) => (
          <List.Item
            key={file.path}
            title={file.name}
            description={`${file.type === 'file' ? formatFileSize(file.size) : 'Directory'} • ${file.modified}`}
            left={() => (
              <Icon 
                name={getFileIcon(file)} 
                size={24} 
                color={file.type === 'directory' ? '#4CAF50' : '#2196F3'} 
              />
            )}
            onPress={() => handleFilePress(file)}
            onLongPress={() => handleFileLongPress(file)}
            style={[
              styles.fileItem,
              file.isHidden && styles.hiddenFile
            ]}
            titleStyle={file.isHidden ? styles.hiddenText : undefined}
          />
        ))}
        
        {filteredFiles.length === 0 && !isLoading && (
          <View style={styles.emptyState}>
            <Icon name="folder-open" size={48} color="#666" />
            <Text style={styles.emptyText}>
              {searchQuery ? 'No files match your search' : 'Directory is empty'}
            </Text>
          </View>
        )}
      </ScrollView>

      {/* Action buttons */}
      <View style={styles.actionBar}>
        <Button
          mode="outlined"
          onPress={() => setShowHidden(!showHidden)}
          icon={showHidden ? 'visibility-off' : 'visibility'}
          style={styles.actionButton}
        >
          {showHidden ? 'Hide Hidden' : 'Show Hidden'}
        </Button>
        
        <Button
          mode="outlined"
          onPress={() => loadDirectory(currentPath)}
          icon="refresh"
          loading={isLoading}
          style={styles.actionButton}
        >
          Refresh
        </Button>
      </View>

      {/* Create new file/folder FAB */}
      <FAB
        style={styles.fab}
        icon="add"
        onPress={() => setShowCreateDialog(true)}
      />

      {/* Create dialog */}
      <Portal>
        <Dialog visible={showCreateDialog} onDismiss={() => setShowCreateDialog(false)}>
          <Dialog.Title>Create New Item</Dialog.Title>
          <Dialog.Content>
            <View style={styles.createTypeRow}>
              <Button
                mode={createType === 'file' ? 'contained' : 'outlined'}
                onPress={() => setCreateType('file')}
                style={styles.typeButton}
              >
                File
              </Button>
              <Button
                mode={createType === 'directory' ? 'contained' : 'outlined'}
                onPress={() => setCreateType('directory')}
                style={styles.typeButton}
              >
                Directory
              </Button>
            </View>
            
            <TextInput
              mode="outlined"
              label={`${createType === 'file' ? 'File' : 'Directory'} name`}
              value={newItemName}
              onChangeText={setNewItemName}
              autoCapitalize="none"
              autoCorrect={false}
              style={styles.createInput}
            />
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setShowCreateDialog(false)}>Cancel</Button>
            <Button onPress={createItem}>Create</Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#121212',
  },
  searchBar: {
    margin: 8,
    backgroundColor: '#1e1e1e',
  },
  searchInput: {
    color: '#ccc',
  },
  breadcrumbContainer: {
    maxHeight: 40,
    backgroundColor: '#1a1a1a',
    paddingHorizontal: 8,
    paddingVertical: 4,
  },
  breadcrumbItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 8,
    paddingVertical: 4,
  },
  breadcrumbText: {
    color: '#4CAF50',
    fontSize: 14,
    marginRight: 4,
  },
  fileList: {
    flex: 1,
  },
  fileItem: {
    backgroundColor: '#1e1e1e',
    marginHorizontal: 8,
    marginVertical: 1,
    borderRadius: 4,
  },
  hiddenFile: {
    opacity: 0.7,
  },
  hiddenText: {
    fontStyle: 'italic',
    color: '#888',
  },
  emptyState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    fontSize: 16,
    color: '#666',
    marginTop: 16,
    textAlign: 'center',
  },
  actionBar: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    padding: 8,
    backgroundColor: '#1a1a1a',
    borderTopWidth: 1,
    borderTopColor: '#333',
  },
  actionButton: {
    flex: 1,
    marginHorizontal: 4,
  },
  fab: {
    position: 'absolute',
    margin: 16,
    right: 0,
    bottom: 80,
    backgroundColor: '#4CAF50',
  },
  createTypeRow: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  typeButton: {
    flex: 1,
    marginHorizontal: 4,
  },
  createInput: {
    marginBottom: 8,
  },
});
