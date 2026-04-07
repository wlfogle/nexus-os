import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  StyleSheet,
  ScrollView,
  RefreshControl,
  Alert,
  Dimensions,
} from 'react-native';
import {
  Card,
  Text,
  Button,
  FAB,
  Portal,
  Dialog,
  Chip,
  Surface,
  AnimatedFAB,
} from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import { useSelector, useDispatch } from 'react-redux';
import { RootState } from '@store/index';
import { ConnectionManager, SystemMetrics } from '@services/EnhancedConnectionManager';
import { PerformanceChart } from '@components/widgets/PerformanceChart';
import { QuickActions } from '@components/widgets/QuickActions';
import { NetworkIndicator } from '@components/widgets/NetworkIndicator';
import AsyncStorage from '@react-native-async-storage/async-storage';

interface DashboardWidget {
  id: string;
  type: 'performance' | 'quickActions' | 'network' | 'systemInfo' | 'alerts';
  position: number;
  visible: boolean;
  config?: any;
}

interface SystemAlert {
  id: string;
  level: 'info' | 'warning' | 'error' | 'critical';
  title: string;
  message: string;
  timestamp: number;
  dismissed: boolean;
}

const DEFAULT_WIDGETS: DashboardWidget[] = [
  { id: '1', type: 'network', position: 1, visible: true },
  { id: '2', type: 'performance', position: 2, visible: true },
  { id: '3', type: 'quickActions', position: 3, visible: true },
  { id: '4', type: 'systemInfo', position: 4, visible: true },
  { id: '5', type: 'alerts', position: 5, visible: true },
];

export const EnhancedDashboardScreen: React.FC = () => {
  const dispatch = useDispatch();
  const connectionStatus = useSelector((state: RootState) => state.connection.status);
  
  const [systemInfo, setSystemInfo] = useState<any>(null);
  const [metricsHistory, setMetricsHistory] = useState<SystemMetrics[]>([]);
  const [currentMetrics, setCurrentMetrics] = useState<SystemMetrics | null>(null);
  const [alerts, setAlerts] = useState<SystemAlert[]>([]);
  const [widgets, setWidgets] = useState<DashboardWidget[]>(DEFAULT_WIDGETS);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [showCustomizeDialog, setShowCustomizeDialog] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
  const [fabExtended, setFabExtended] = useState(true);

  useEffect(() => {
    if (connectionStatus === 'connected') {
      loadDashboardData();
      loadWidgetConfiguration();
      startMetricsPolling();
    }

    return () => {
      stopMetricsPolling();
    };
  }, [connectionStatus]);

  const metricsInterval = React.useRef<NodeJS.Timeout | null>(null);

  const startMetricsPolling = () => {
    if (metricsInterval.current) clearInterval(metricsInterval.current);
    
    metricsInterval.current = setInterval(() => {
      if (ConnectionManager.isConnected()) {
        fetchSystemMetrics();
      }
    }, 5000);
  };

  const stopMetricsPolling = () => {
    if (metricsInterval.current) {
      clearInterval(metricsInterval.current);
      metricsInterval.current = null;
    }
  };

  const loadDashboardData = async () => {
    try {
      setIsRefreshing(true);
      
      const sysInfo = await ConnectionManager.getSystemInfo();
      setSystemInfo(sysInfo);
      
      await fetchSystemMetrics();
      await loadSystemAlerts();
      
      setLastUpdate(new Date());
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
      Alert.alert('Error', 'Failed to load dashboard data');
    } finally {
      setIsRefreshing(false);
    }
  };

  const fetchSystemMetrics = async () => {
    try {
      const metrics = await ConnectionManager.getSystemMetrics();
      setCurrentMetrics(metrics);
      
      setMetricsHistory(prev => {
        const updated = [...prev, metrics];
        return updated.slice(-100);
      });
      
      checkMetricAlerts(metrics);
      
    } catch (error) {
      console.error('Failed to fetch system metrics:', error);
    }
  };

  const checkMetricAlerts = (metrics: SystemMetrics) => {
    const newAlerts: SystemAlert[] = [];
    
    if (metrics.cpu.usage > 90) {
      newAlerts.push({
        id: `cpu_${Date.now()}`,
        level: 'error',
        title: 'High CPU Usage',
        message: `CPU usage is at ${metrics.cpu.usage.toFixed(1)}%`,
        timestamp: Date.now(),
        dismissed: false,
      });
    }
    
    if (metrics.memory.percentage > 85) {
      newAlerts.push({
        id: `memory_${Date.now()}`,
        level: 'warning',
        title: 'High Memory Usage',
        message: `Memory usage is at ${metrics.memory.percentage.toFixed(1)}%`,
        timestamp: Date.now(),
        dismissed: false,
      });
    }
    
    if (metrics.disk.percentage > 95) {
      newAlerts.push({
        id: `disk_${Date.now()}`,
        level: 'critical',
        title: 'Disk Space Critical',
        message: `Disk usage is at ${metrics.disk.percentage.toFixed(1)}%`,
        timestamp: Date.now(),
        dismissed: false,
      });
    }
    
    if (newAlerts.length > 0) {
      setAlerts(prev => [...prev, ...newAlerts]);
    }
  };

  const loadSystemAlerts = async () => {
    try {
      const savedAlerts = await AsyncStorage.getItem('systemAlerts');
      if (savedAlerts) {
        setAlerts(JSON.parse(savedAlerts));
      }
    } catch (error) {
      console.error('Failed to load alerts:', error);
    }
  };

  const loadWidgetConfiguration = async () => {
    try {
      const saved = await AsyncStorage.getItem('dashboardWidgets');
      if (saved) {
        setWidgets(JSON.parse(saved));
      }
    } catch (error) {
      console.error('Failed to load widget configuration:', error);
    }
  };

  const saveWidgetConfiguration = async (newWidgets: DashboardWidget[]) => {
    try {
      await AsyncStorage.setItem('dashboardWidgets', JSON.stringify(newWidgets));
      setWidgets(newWidgets);
    } catch (error) {
      Alert.alert('Error', 'Failed to save widget configuration');
    }
  };

  const dismissAlert = (alertId: string) => {
    setAlerts(prev => 
      prev.map(alert => 
        alert.id === alertId ? { ...alert, dismissed: true } : alert
      )
    );
  };

  const clearAllAlerts = () => {
    setAlerts(prev => prev.map(alert => ({ ...alert, dismissed: true })));
  };

  const toggleWidget = (widgetId: string) => {
    const updated = widgets.map(w => 
      w.id === widgetId ? { ...w, visible: !w.visible } : w
    );
    saveWidgetConfiguration(updated);
  };

  const onRefresh = useCallback(() => {
    loadDashboardData();
  }, []);

  const executeQuickCommand = (command: string, title: string) => {
    Alert.alert(title, `Executing: ${command}`);
  };

  const renderWidget = (widget: DashboardWidget) => {
    if (!widget.visible) return null;

    switch (widget.type) {
      case 'network':
        return <NetworkIndicator key={widget.id} />;
      
      case 'performance':
        return (
          <PerformanceChart 
            key={widget.id}
            metrics={metricsHistory}
            height={200}
            chartType="area"
          />
        );
      
      case 'quickActions':
        return (
          <QuickActions 
            key={widget.id}
            onCommandExecute={executeQuickCommand}
          />
        );
      
      case 'systemInfo':
        return (
          <Card key={widget.id} style={styles.card}>
            <Card.Title 
              title="System Information" 
              left={() => <Icon name="computer" size={24} color="#4CAF50" />}
            />
            <Card.Content>
              {systemInfo && (
                <View style={styles.systemInfoGrid}>
                  <View style={styles.infoItem}>
                    <Text style={styles.infoLabel}>OS</Text>
                    <Text style={styles.infoValue}>{systemInfo.os || 'Unknown'}</Text>
                  </View>
                  <View style={styles.infoItem}>
                    <Text style={styles.infoLabel}>Uptime</Text>
                    <Text style={styles.infoValue}>
                      {currentMetrics ? Math.floor(currentMetrics.uptime / 3600) + 'h' : 'Unknown'}
                    </Text>
                  </View>
                  <View style={styles.infoItem}>
                    <Text style={styles.infoLabel}>CPU</Text>
                    <Text style={styles.infoValue}>{systemInfo.cpu || 'Unknown'}</Text>
                  </View>
                  <View style={styles.infoItem}>
                    <Text style={styles.infoLabel}>Memory</Text>
                    <Text style={styles.infoValue}>
                      {currentMetrics ? 
                        `${(currentMetrics.memory.total / 1024 / 1024 / 1024).toFixed(1)}GB` : 
                        'Unknown'
                      }
                    </Text>
                  </View>
                </View>
              )}
            </Card.Content>
          </Card>
        );
      
      case 'alerts':
        const activeAlerts = alerts.filter(a => !a.dismissed);
        if (activeAlerts.length === 0) return null;
        
        return (
          <Card key={widget.id} style={styles.card}>
            <Card.Title 
              title={`System Alerts (${activeAlerts.length})`}
              left={() => <Icon name="warning" size={24} color="#FF9800" />}
              right={() => (
                <Button mode="text" onPress={clearAllAlerts}>
                  Clear All
                </Button>
              )}
            />
            <Card.Content>
              {activeAlerts.slice(0, 3).map(alert => (
                <View key={alert.id} style={styles.alertItem}>
                  <View style={styles.alertContent}>
                    <View style={styles.alertHeader}>
                      <Icon 
                        name={alert.level === 'critical' ? 'error' : 'warning'} 
                        size={16} 
                        color={getAlertColor(alert.level)} 
                      />
                      <Text style={[styles.alertTitle, { color: getAlertColor(alert.level) }]}>
                        {alert.title}
                      </Text>
                    </View>
                    <Text style={styles.alertMessage}>{alert.message}</Text>
                    <Text style={styles.alertTime}>
                      {new Date(alert.timestamp).toLocaleTimeString()}
                    </Text>
                  </View>
                  <Button
                    mode="text"
                    compact
                    onPress={() => dismissAlert(alert.id)}
                  >
                    Dismiss
                  </Button>
                </View>
              ))}
            </Card.Content>
          </Card>
        );", "search_start_line_number": 208}]
