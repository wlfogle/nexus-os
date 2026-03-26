import React, { useState, useEffect } from 'react';
import { View, StyleSheet, ScrollView, RefreshControl, Alert } from 'react-native';
import { Card, Text, List, ProgressBar, Chip, Button } from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import { ConnectionManager } from '@services/ConnectionManager';

interface ProcessInfo {
  pid: number;
  name: string;
  cpu: number;
  memory: number;
  status: string;
}

export const SystemScreen: React.FC = () => {
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [sortBy, setSortBy] = useState<'cpu' | 'memory' | 'name'>('cpu');
  const [showAllProcesses, setShowAllProcesses] = useState(false);

  useEffect(() => {
    loadProcesses();
  }, []);

  const loadProcesses = async () => {
    try {
      const data = await ConnectionManager.getProcessList();
      setProcesses(data);
    } catch (error) {
      console.error('Failed to load processes:', error);
      Alert.alert('Error', 'Failed to load process information');
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await loadProcesses();
    setRefreshing(false);
  };

  const killProcess = async (pid: number, name: string) => {
    Alert.alert(
      'Kill Process',
      `Are you sure you want to kill "${name}" (PID: ${pid})?`,
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Kill',
          style: 'destructive',
          onPress: async () => {
            try {
              await ConnectionManager.executeCommand({
                command: `kill ${pid}`,
              });
              Alert.alert('Success', 'Process killed successfully');
              loadProcesses(); // Refresh
            } catch (error) {
              Alert.alert('Error', 'Failed to kill process');
            }
          },
        },
      ]
    );
  };

  const getSortedProcesses = () => {
    const sorted = [...processes];
    
    switch (sortBy) {
      case 'cpu':
        sorted.sort((a, b) => b.cpu - a.cpu);
        break;
      case 'memory':
        sorted.sort((a, b) => b.memory - a.memory);
        break;
      case 'name':
        sorted.sort((a, b) => a.name.localeCompare(b.name));
        break;
    }
    
    return showAllProcesses ? sorted : sorted.slice(0, 20);
  };

  const getStatusColor = (status: string): string => {
    switch (status.toLowerCase()) {
      case 'running': return '#4CAF50';
      case 'sleeping': return '#2196F3';
      case 'zombie': return '#F44336';
      case 'stopped': return '#FF9800';
      default: return '#666';
    }
  };

  const formatMemory = (bytes: number): string => {
    const mb = bytes / (1024 * 1024);
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(1)} GB`;
    }
    return `${mb.toFixed(1)} MB`;
  };

  return (
    <View style={styles.container}>
      <ScrollView 
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        {/* Sort controls */}
        <Card style={styles.card}>
          <Card.Title title="Process List" />
          <Card.Content>
            <View style={styles.controlsRow}>
              <View style={styles.sortButtons}>
                {(['cpu', 'memory', 'name'] as const).map((sort) => (
                  <Button
                    key={sort}
                    mode={sortBy === sort ? 'contained' : 'outlined'}
                    onPress={() => setSortBy(sort)}
                    style={styles.sortButton}
                    compact
                  >
                    {sort.toUpperCase()}
                  </Button>
                ))}
              </View>
              
              <Button
                mode={showAllProcesses ? 'contained' : 'outlined'}
                onPress={() => setShowAllProcesses(!showAllProcesses)}
                compact
              >
                {showAllProcesses ? 'Top 20' : 'Show All'}
              </Button>
            </View>
          </Card.Content>
        </Card>

        {/* Process list */}
        <Card style={styles.card}>
          <Card.Content>
            {getSortedProcesses().map((process) => (
              <List.Item
                key={process.pid}
                title={process.name}
                description={`PID: ${process.pid} • ${formatMemory(process.memory)}`}
                left={() => (
                  <View style={styles.processIcon}>
                    <Icon name="memory" size={20} color={getStatusColor(process.status)} />
                  </View>
                )}
                right={() => (
                  <View style={styles.processStats}>
                    <View style={styles.statRow}>
                      <Text style={styles.statLabel}>CPU:</Text>
                      <Text style={styles.statValue}>{process.cpu.toFixed(1)}%</Text>
                    </View>
                    <ProgressBar
                      progress={process.cpu / 100}
                      color={process.cpu > 50 ? '#F44336' : '#4CAF50'}
                      style={styles.progressBar}
                    />
                    <Chip
                      style={[styles.statusChip, { backgroundColor: getStatusColor(process.status) }]}
                      textStyle={styles.statusText}
                    >
                      {process.status}
                    </Chip>
                  </View>
                )}
                onPress={() => {
                  Alert.alert(
                    process.name,
                    `PID: ${process.pid}\nCPU: ${process.cpu.toFixed(1)}%\nMemory: ${formatMemory(process.memory)}\nStatus: ${process.status}`,
                    [
                      { text: 'Close' },
                      { text: 'Kill Process', style: 'destructive', onPress: () => killProcess(process.pid, process.name) },
                    ]
                  );
                }}
                style={styles.processItem}
              />
            ))}
            
            {processes.length === 0 && (
              <View style={styles.emptyState}>
                <Icon name="developer-board" size={48} color="#666" />
                <Text style={styles.emptyText}>No process information available</Text>
              </View>
            )}
          </Card.Content>
        </Card>

        {/* System information */}
        <Card style={styles.card}>
          <Card.Title 
            title="System Information" 
            left={() => <Icon name="info" size={24} color="#2196F3" />}
          />
          <Card.Content>
            <Button
              mode="outlined"
              onPress={async () => {
                try {
                  const result = await ConnectionManager.executeCommand({
                    command: 'uname -a && cat /proc/meminfo | head -5 && cat /proc/cpuinfo | grep "model name" | head -1',
                  });
                  Alert.alert('System Info', result.output);
                } catch (error) {
                  Alert.alert('Error', 'Failed to get system information');
                }
              }}
              style={styles.infoButton}
            >
              View Detailed System Info
            </Button>
            
            <Button
              mode="outlined"
              onPress={async () => {
                try {
                  const result = await ConnectionManager.executeCommand({
                    command: 'df -h',
                  });
                  Alert.alert('Disk Usage', result.output);
                } catch (error) {
                  Alert.alert('Error', 'Failed to get disk usage');
                }
              }}
              style={styles.infoButton}
            >
              Check Disk Usage
            </Button>
            
            <Button
              mode="outlined"
              onPress={async () => {
                try {
                  const result = await ConnectionManager.executeCommand({
                    command: 'ip addr show',
                  });
                  Alert.alert('Network Interfaces', result.output);
                } catch (error) {
                  Alert.alert('Error', 'Failed to get network information');
                }
              }}
              style={styles.infoButton}
            >
              Network Interfaces
            </Button>
          </Card.Content>
        </Card>
      </ScrollView>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#121212',
  },
  card: {
    margin: 8,
    backgroundColor: '#1e1e1e',
  },
  controlsRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  sortButtons: {
    flexDirection: 'row',
  },
  sortButton: {
    marginRight: 8,
  },
  processItem: {
    backgroundColor: '#2a2a2a',
    marginBottom: 4,
    borderRadius: 4,
  },
  processIcon: {
    justifyContent: 'center',
    alignItems: 'center',
    width: 40,
  },
  processStats: {
    width: 100,
    alignItems: 'flex-end',
  },
  statRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 2,
  },
  statLabel: {
    fontSize: 10,
    color: '#888',
    marginRight: 4,
  },
  statValue: {
    fontSize: 10,
    color: '#ccc',
    fontWeight: 'bold',
  },
  progressBar: {
    width: 80,
    height: 4,
    marginBottom: 4,
  },
  statusChip: {
    height: 20,
    paddingHorizontal: 4,
  },
  statusText: {
    fontSize: 8,
    color: '#fff',
  },
  emptyState: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyText: {
    fontSize: 16,
    color: '#666',
    marginTop: 16,
    textAlign: 'center',
  },
  infoButton: {
    marginBottom: 8,
  },
});
