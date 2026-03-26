import React, { useState, useEffect } from 'react';
import { View, StyleSheet, ScrollView, Alert } from 'react-native';
import { 
  Card, 
  Button, 
  FAB, 
  Dialog, 
  Portal, 
  TextInput, 
  Text,
  Chip,
  Menu,
  IconButton,
} from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { ConnectionManager } from '@services/EnhancedConnectionManager';

interface QuickAction {
  id: string;
  title: string;
  command: string;
  icon: string;
  color: string;
  category: string;
  confirmBefore?: boolean;
  description?: string;
}

const DEFAULT_ACTIONS: QuickAction[] = [
  {
    id: '1',
    title: 'System Info',
    command: 'uname -a && lscpu | head -10',
    icon: 'info',
    color: '#4CAF50',
    category: 'system',
    description: 'Display basic system information',
  },
  {
    id: '2',
    title: 'Top Processes',
    command: 'ps aux --sort=-%cpu | head -10',
    icon: 'list',
    color: '#2196F3',
    category: 'monitoring',
    description: 'Show top CPU-consuming processes',
  },
  {
    id: '3',
    title: 'Disk Usage',
    command: 'df -h',
    icon: 'storage',
    color: '#FF9800',
    category: 'storage',
    description: 'Check disk space usage',
  },
  {
    id: '4',
    title: 'Network Stats',
    command: 'ss -tuln',
    icon: 'network-outline',
    color: '#9C27B0',
    category: 'network',
    description: 'Display network connections',
  },
  {
    id: '5',
    title: 'Memory Info',
    command: 'free -h',
    icon: 'memory',
    color: '#00BCD4',
    category: 'monitoring',
    description: 'Show memory usage statistics',
  },
  {
    id: '6',
    title: 'Update System',
    command: 'sudo pacman -Syu',
    icon: 'system-update',
    color: '#F44336',
    category: 'maintenance',
    confirmBefore: true,
    description: 'Update system packages',
  },
  {
    id: '7',
    title: 'Git Status',
    command: 'git status',
    icon: 'git',
    color: '#FF5722',
    category: 'development',
    description: 'Check git repository status',
  },
  {
    id: '8',
    title: 'Docker Containers',
    command: 'docker ps -a',
    icon: 'docker',
    color: '#607D8B',
    category: 'containers',
    description: 'List Docker containers',
  },
];

interface QuickActionsProps {
  onCommandExecute?: (command: string, title: string) => void;
}

export const QuickActions: React.FC<QuickActionsProps> = ({ onCommandExecute }) => {
  const [actions, setActions] = useState<QuickAction[]>(DEFAULT_ACTIONS);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [editingAction, setEditingAction] = useState<QuickAction | null>(null);
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [showCategoryMenu, setShowCategoryMenu] = useState(false);
  const [newAction, setNewAction] = useState<Partial<QuickAction>>({
    category: 'custom',
    color: '#4CAF50',
    icon: 'play-arrow',
  });

  useEffect(() => {
    loadActions();
  }, []);

  const loadActions = async () => {
    try {
      const saved = await AsyncStorage.getItem('quickActions');
      if (saved) {
        setActions(JSON.parse(saved));
      }
    } catch (error) {
      console.error('Failed to load quick actions:', error);
    }
  };

  const saveActions = async (updatedActions: QuickAction[]) => {
    try {
      await AsyncStorage.setItem('quickActions', JSON.stringify(updatedActions));
      setActions(updatedActions);
    } catch (error) {
      Alert.alert('Error', 'Failed to save quick actions');
    }
  };

  const executeAction = async (action: QuickAction) => {
    if (action.confirmBefore) {
      Alert.alert(
        'Confirm Action',
        `Execute: ${action.title}\n\nCommand: ${action.command}`,
        [
          { text: 'Cancel', style: 'cancel' },
          { text: 'Execute', onPress: () => performExecution(action) },
        ]
      );
    } else {
      performExecution(action);
    }
  };

  const performExecution = async (action: QuickAction) => {
    try {
      if (onCommandExecute) {
        onCommandExecute(action.command, action.title);
      } else {
        const result = await ConnectionManager.executeCommand(action.command);
        Alert.alert(action.title, result.output || 'Command executed successfully');
      }
    } catch (error) {
      Alert.alert('Execution Error', `Failed to execute: ${action.title}`);
    }
  };

  const addAction = () => {
    if (!newAction.title || !newAction.command) {
      Alert.alert('Validation Error', 'Title and command are required');
      return;
    }

    const action: QuickAction = {
      id: Date.now().toString(),
      title: newAction.title,
      command: newAction.command,
      icon: newAction.icon || 'play-arrow',
      color: newAction.color || '#4CAF50',
      category: newAction.category || 'custom',
      confirmBefore: newAction.confirmBefore || false,
      description: newAction.description,
    };

    const updatedActions = [...actions, action];
    saveActions(updatedActions);
    setShowAddDialog(false);
    setNewAction({ category: 'custom', color: '#4CAF50', icon: 'play-arrow' });
  };

  const editAction = (action: QuickAction) => {
    setEditingAction(action);
    setNewAction(action);
    setShowEditDialog(true);
  };

  const updateAction = () => {
    if (!editingAction || !newAction.title || !newAction.command) {
      Alert.alert('Validation Error', 'Title and command are required');
      return;
    }

    const updatedActions = actions.map(a => 
      a.id === editingAction.id ? { ...editingAction, ...newAction } : a
    );
    
    saveActions(updatedActions);
    setShowEditDialog(false);
    setEditingAction(null);
    setNewAction({ category: 'custom', color: '#4CAF50', icon: 'play-arrow' });
  };

  const deleteAction = (actionId: string) => {
    Alert.alert(
      'Delete Action',
      'Are you sure you want to delete this quick action?',
      [
        { text: 'Cancel', style: 'cancel' },
        { 
          text: 'Delete', 
          style: 'destructive',
          onPress: () => {
            const updatedActions = actions.filter(a => a.id !== actionId);
            saveActions(updatedActions);
          }
        },
      ]
    );
  };

  const getFilteredActions = () => {
    if (selectedCategory === 'all') {
      return actions;
    }
    return actions.filter(action => action.category === selectedCategory);
  };

  const getCategories = () => {
    const categories = ['all', ...new Set(actions.map(a => a.category))];
    return categories;
  };

  const resetToDefaults = () => {
    Alert.alert(
      'Reset Actions',
      'Reset to default quick actions? This will remove all custom actions.',
      [
        { text: 'Cancel', style: 'cancel' },
        { 
          text: 'Reset', 
          style: 'destructive',
          onPress: () => saveActions(DEFAULT_ACTIONS)
        },
      ]
    );
  };

  return (
    <Card style={styles.card}>
      <Card.Title 
        title="Quick Actions"
        right={() => (
          <View style={styles.headerActions}>
            <Menu
              visible={showCategoryMenu}
              onDismiss={() => setShowCategoryMenu(false)}
              anchor={
                <IconButton
                  icon="filter-list"
                  onPress={() => setShowCategoryMenu(true)}
                />
              }
            >
              {getCategories().map(category => (
                <Menu.Item
                  key={category}
                  onPress={() => {
                    setSelectedCategory(category);
                    setShowCategoryMenu(false);
                  }}
                  title={category.charAt(0).toUpperCase() + category.slice(1)}
                />
              ))}
            </Menu>
            <IconButton
              icon="add"
              onPress={() => setShowAddDialog(true)}
            />
          </View>
        )}
      />
      <Card.Content>
        {/* Category Filter */}
        <ScrollView horizontal showsHorizontalScrollIndicator={false} style={styles.categoryScroll}>
          {getCategories().map(category => (
            <Chip
              key={category}
              selected={selectedCategory === category}
              onPress={() => setSelectedCategory(category)}
              style={styles.categoryChip}
            >
              {category.charAt(0).toUpperCase() + category.slice(1)}
            </Chip>
          ))}
        </ScrollView>

        {/* Actions Grid */}
        <View style={styles.actionsGrid}>
          {getFilteredActions().map((action) => (
            <View key={action.id} style={styles.actionContainer}>
              <Button
                mode="contained"
                onPress={() => executeAction(action)}
                onLongPress={() => editAction(action)}
                style={[styles.actionButton, { backgroundColor: action.color }]}
                contentStyle={styles.actionContent}
                labelStyle={styles.actionLabel}
                icon={action.icon}
              >
                {action.title}
              </Button>
            </View>
          ))}
        </View>

        {/* Reset Button */}
        <Button
          mode="text"
          onPress={resetToDefaults}
          style={styles.resetButton}
          textColor="#F44336"
        >
          Reset to Defaults
        </Button>
      </Card.Content>

      {/* Add Action Dialog */}
      <Portal>
        <Dialog visible={showAddDialog} onDismiss={() => setShowAddDialog(false)}>
          <Dialog.Title>Add Quick Action</Dialog.Title>
          <Dialog.Content>
            <TextInput
              mode="outlined"
              label="Title"
              value={newAction.title || ''}
              onChangeText={(text) => setNewAction(prev => ({ ...prev, title: text }))}
              style={styles.input}
            />
            
            <TextInput
              mode="outlined"
              label="Command"
              value={newAction.command || ''}
              onChangeText={(text) => setNewAction(prev => ({ ...prev, command: text }))}
              multiline
              numberOfLines={3}
              style={styles.input}
            />
            
            <TextInput
              mode="outlined"
              label="Description (optional)"
              value={newAction.description || ''}
              onChangeText={(text) => setNewAction(prev => ({ ...prev, description: text }))}
              style={styles.input}
            />
            
            <TextInput
              mode="outlined"
              label="Category"
              value={newAction.category || ''}
              onChangeText={(text) => setNewAction(prev => ({ ...prev, category: text }))}
              style={styles.input}
            />
            
            <TextInput
              mode="outlined"
              label="Icon Name"
              value={newAction.icon || ''}
              onChangeText={(text) => setNewAction(prev => ({ ...prev, icon: text }))}
              style={styles.input}
            />
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setShowAddDialog(false)}>Cancel</Button>
            <Button onPress={addAction}>Add</Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>

      {/* Edit Action Dialog */}
      <Portal>
        <Dialog visible={showEditDialog} onDismiss={() => setShowEditDialog(false)}>
          <Dialog.Title>Edit Quick Action</Dialog.Title>
          <Dialog.Content>
            <TextInput
              mode="outlined"
              label="Title"
              value={newAction.title || ''}
              onChangeText={(text) => setNewAction(prev => ({ ...prev, title: text }))}
              style={styles.input}
            />
            
            <TextInput
              mode="outlined"
              label="Command"
              value={newAction.command || ''}
              onChangeText={(text) => setNewAction(prev => ({ ...prev, command: text }))}
              multiline
              numberOfLines={3}
              style={styles.input}
            />
            
            <Button
              mode="text"
              textColor="#F44336"
              onPress={() => editingAction && deleteAction(editingAction.id)}
              style={styles.deleteButton}
            >
              Delete Action
            </Button>
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setShowEditDialog(false)}>Cancel</Button>
            <Button onPress={updateAction}>Update</Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>
    </Card>
  );
};

const styles = StyleSheet.create({
  card: {
    margin: 8,
    backgroundColor: '#1e1e1e',
  },
  headerActions: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  categoryScroll: {
    marginBottom: 16,
  },
  categoryChip: {
    marginRight: 8,
  },
  actionsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  actionContainer: {
    width: '48%',
    marginBottom: 8,
  },
  actionButton: {
    borderRadius: 8,
  },
  actionContent: {
    height: 60,
    flexDirection: 'column',
  },
  actionLabel: {
    fontSize: 12,
    textAlign: 'center',
  },
  resetButton: {
    alignSelf: 'center',
  },
  input: {
    marginBottom: 12,
  },
  deleteButton: {
    marginTop: 8,
  },
});
