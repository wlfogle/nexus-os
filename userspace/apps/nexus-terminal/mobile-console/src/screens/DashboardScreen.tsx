import React, { useState, useEffect } from 'react';
import { View, StyleSheet, ScrollView, RefreshControl, Alert } from 'react-native';
import { Card, Text, ProgressBar, Button, FAB } from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import { useDispatch, useSelector } from 'react-redux';
import { RootState } from '@store/index';
import { ConnectionManager, SystemStatus } from '@services/ConnectionManager';

interface QuickAction {
  id: string;
  title: string;
  icon: string;
  command: string;
  color: string;
}

export const DashboardScreen: React.FC = () => {
  const dispatch = useDispatch();
  const connectionStatus = useSelector((state: RootState) => state.connection.status);
  
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  const quickActions: QuickAction[] = [
    { id: '1', title: 'Update System', icon: 'system-update', command: 'sudo apt update && sudo apt upgrade', color: '#2196F3' },
    { id: '2', title: 'Check Disk Space', icon: 'storage', command: 'df -h', color: '#FF9800' },
    { id: '3', title: 'Show Processes', icon: 'list', command: 'ps aux', color: '#4CAF50' },
    { id: '4', title: 'Network Status', icon: 'wifi', command: 'ip addr show', color: '#9C27B0' },
    { id: '5', title: 'System Log', icon: 'description', command: 'journalctl -n 50', color: '#F44336' },
    { id: '6', title: 'Memory Info', icon: 'memory', command: 'free -h', color: '#00BCD4' },
  ];

  useEffect(() => {
    if (connectionStatus === 'connected') {
      fetchSystemStatus();
      subscribeToUpdates();
    }

    return () => {
      ConnectionManager.unsubscribeFromSystemUpdates();
    };
  }, [connectionStatus]);

  const fetchSystemStatus = async () => {
    try {
      const status = await ConnectionManager.getSystemStatus();
      setSystemStatus(status);
      setLastUpdate(new Date());
    } catch (error) {
      console.error('Failed to fetch system status:', error);
      Alert.alert('Error', 'Failed to fetch system status');
    }
  };

  const subscribeToUpdates = () => {
    ConnectionManager.subscribeToSystemUpdates();
    
    const handleStatusUpdate = (status: SystemStatus) => {
      setSystemStatus(status);
      setLastUpdate(new Date());
    };

    ConnectionManager.on('systemStatus', handleStatusUpdate);
    
    return () => {
      ConnectionManager.off('systemStatus', handleStatusUpdate);
    };
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await fetchSystemStatus();
    setRefreshing(false);
  };

  const executeQuickAction = async (action: QuickAction) => {
    try {
      Alert.alert(
        'Execute Command',
        `Run: ${action.command}`,
        [
          { text: 'Cancel', style: 'cancel' },
          {
            text: 'Execute',
            onPress: async () => {
              try {
                const result = await ConnectionManager.executeCommand({
                  command: action.command,
                });
                
                Alert.alert(
                  'Command Result',
                  result.success ? result.output : result.error || 'Command failed',
                  [{ text: 'OK' }]
                );
              } catch (error) {
                Alert.alert('Error', 'Failed to execute command');
              }
            },
          },
        ]
      );
    } catch (error) {
      Alert.alert('Error', 'Failed to execute quick action');
    }
  };

  const formatUptime = (seconds: number): string => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    
    if (days > 0) return `${days}d ${hours}h ${minutes}m`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const getStatusColor = (value: number, thresholds: { warning: number; critical: number }) => {
    if (value >= thresholds.critical) return '#F44336';
    if (value >= thresholds.warning) return '#FF9800';
    return '#4CAF50';
  };

  return (
    <View style={styles.container}>
      <ScrollView 
        style={styles.scrollView}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        {/* System Status Overview */}
        <Card style={styles.card}>
          <Card.Title 
            title="System Status" 
            subtitle={lastUpdate ? `Last updated: ${lastUpdate.toLocaleTimeString()}` : 'No data'}
            left={(props) => <Icon {...props} name="computer" size={24} />}
          />
          <Card.Content>
            {systemStatus ? (
              <>
                <View style={styles.statusGrid}>
                  <View style={styles.statusItem}>
                    <Text style={styles.statusLabel}>CPU Usage</Text>
                    <ProgressBar 
                      progress={systemStatus.cpu / 100} 
                      color={getStatusColor(systemStatus.cpu, { warning: 70, critical: 90 })}
                      style={styles.progressBar}
                    />
                    <Text style={styles.statusValue}>{systemStatus.cpu.toFixed(1)}%</Text>
                  </View>

                  <View style={styles.statusItem}>
                    <Text style={styles.statusLabel}>Memory Usage</Text>
                    <ProgressBar 
                      progress={systemStatus.memory / 100} 
                      color={getStatusColor(systemStatus.memory, { warning: 80, critical: 95 })}
                      style={styles.progressBar}
                    />
                    <Text style={styles.statusValue}>{systemStatus.memory.toFixed(1)}%</Text>
                  </View>
                </View>

                <View style={styles.infoRow}>
                  <View style={styles.infoItem}>
                    <Icon name="access-time" size={20} color="#4CAF50" />
                    <Text style={styles.infoText}>
                      Uptime: {formatUptime(systemStatus.uptime)}
                    </Text>
                  </View>
                  
                  <View style={styles.infoItem}>
                    <Icon name="developer-board" size={20} color="#2196F3" />
                    <Text style={styles.infoText}>
                      Processes: {systemStatus.processes}
                    </Text>
                  </View>
                </View>

                <View style={styles.infoRow}>
                  <View style={styles.infoItem}>
                    <Icon name="network-check" size={20} color="#FF9800" />
                    <Text style={styles.infoText}>
                      Network: {systemStatus.networkConnections} connections
                    </Text>
                  </View>
                </View>
              </>
            ) : (
              <Text style={styles.noDataText}>No system data available</Text>
            )}
          </Card.Content>
        </Card>

        {/* Quick Actions */}
        <Card style={styles.card}>
          <Card.Title 
            title="Quick Actions" 
            left={(props) => <Icon {...props} name="flash-on" size={24} />}
          />
          <Card.Content>
            <View style={styles.actionsGrid}>
              {quickActions.map((action) => (
                <Button
                  key={action.id}
                  mode="contained"
                  onPress={() => executeQuickAction(action)}
                  style={[styles.actionButton, { backgroundColor: action.color }]}
                  contentStyle={styles.actionContent}
                  labelStyle={styles.actionLabel}
                >
                  <Icon name={action.icon} size={20} color="white" />
                  {'\n'}{action.title}
                </Button>
              ))}
            </View>
          </Card.Content>
        </Card>

        {/* System Alerts */}
        <Card style={styles.card}>
          <Card.Title 
            title="System Alerts" 
            left={(props) => <Icon {...props} name="warning" size={24} />}
          />
          <Card.Content>
            {systemStatus && (
              <>
                {systemStatus.cpu > 90 && (
                  <View style={[styles.alert, styles.criticalAlert]}>
                    <Icon name="error" size={20} color="#F44336" />
                    <Text style={styles.alertText}>Critical: CPU usage is very high ({systemStatus.cpu.toFixed(1)}%)</Text>
                  </View>
                )}
                
                {systemStatus.memory > 95 && (
                  <View style={[styles.alert, styles.criticalAlert]}>
                    <Icon name="error" size={20} color="#F44336" />
                    <Text style={styles.alertText}>Critical: Memory usage is very high ({systemStatus.memory.toFixed(1)}%)</Text>
                  </View>
                )}
                
                {systemStatus.cpu > 70 && systemStatus.cpu <= 90 && (
                  <View style={[styles.alert, styles.warningAlert]}>
                    <Icon name="warning" size={20} color="#FF9800" />
                    <Text style={styles.alertText}>Warning: High CPU usage ({systemStatus.cpu.toFixed(1)}%)</Text>
                  </View>
                )}
                
                {systemStatus.memory > 80 && systemStatus.memory <= 95 && (
                  <View style={[styles.alert, styles.warningAlert]}>
                    <Icon name="warning" size={20} color="#FF9800" />
                    <Text style={styles.alertText}>Warning: High memory usage ({systemStatus.memory.toFixed(1)}%)</Text>
                  </View>
                )}
                
                {systemStatus.cpu <= 70 && systemStatus.memory <= 80 && (
                  <View style={[styles.alert, styles.infoAlert]}>
                    <Icon name="check-circle" size={20} color="#4CAF50" />
                    <Text style={styles.alertText}>System running normally</Text>
                  </View>
                )}
              </>
            )}
          </Card.Content>
        </Card>
      </ScrollView>

      <FAB
        style={styles.fab}
        icon="refresh"
        onPress={handleRefresh}
        loading={refreshing}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#121212',
  },
  scrollView: {
    flex: 1,
  },
  card: {
    margin: 8,
    backgroundColor: '#1e1e1e',
  },
  statusGrid: {
    marginBottom: 16,
  },
  statusItem: {
    marginBottom: 12,
  },
  statusLabel: {
    fontSize: 14,
    marginBottom: 4,
    color: '#ccc',
  },
  progressBar: {
    height: 8,
    borderRadius: 4,
    marginBottom: 4,
  },
  statusValue: {
    fontSize: 16,
    fontWeight: 'bold',
    textAlign: 'right',
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  infoItem: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  infoText: {
    marginLeft: 8,
    fontSize: 14,
    color: '#ccc',
  },
  noDataText: {
    textAlign: 'center',
    color: '#888',
    fontStyle: 'italic',
  },
  actionsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
  },
  actionButton: {
    width: '48%',
    marginBottom: 8,
    borderRadius: 8,
  },
  actionContent: {
    height: 60,
    flexDirection: 'column',
  },
  actionLabel: {
    fontSize: 12,
    textAlign: 'center',
    color: 'white',
  },
  alert: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    borderRadius: 8,
    marginBottom: 8,
  },
  criticalAlert: {
    backgroundColor: 'rgba(244, 67, 54, 0.1)',
    borderColor: '#F44336',
    borderWidth: 1,
  },
  warningAlert: {
    backgroundColor: 'rgba(255, 152, 0, 0.1)',
    borderColor: '#FF9800',
    borderWidth: 1,
  },
  infoAlert: {
    backgroundColor: 'rgba(76, 175, 80, 0.1)',
    borderColor: '#4CAF50',
    borderWidth: 1,
  },
  alertText: {
    marginLeft: 8,
    flex: 1,
    fontSize: 14,
  },
  fab: {
    position: 'absolute',
    margin: 16,
    right: 0,
    bottom: 0,
    backgroundColor: '#4CAF50',
  },
});
