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
    }, 5000); // Update every 5 seconds
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
      
      // Load system information
      const sysInfo = await ConnectionManager.getSystemInfo();
      setSystemInfo(sysInfo);
      
      // Load initial metrics
      await fetchSystemMetrics();
      
      // Load alerts
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
      
      // Add to history and keep last 100 entries
      setMetricsHistory(prev => {
        const updated = [...prev, metrics];
        return updated.slice(-100);
      });
      
      // Check for alerts based on metrics
      checkMetricAlerts(metrics);
      
    } catch (error) {
      console.error('Failed to fetch system metrics:', error);
    }
  };

  const checkMetricAlerts = (metrics: SystemMetrics) => {
    const newAlerts: SystemAlert[] = [];
    
    // CPU usage alert
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
    
    // Memory usage alert
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
    
    // Disk usage alert
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
      // In a real implementation, this would fetch from the backend
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
    // Navigate to terminal and execute command
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
                    <Text style={styles.infoValue}>{systemInfo.os || 'Unknown'}</Text>\n                  </View>\n                  <View style={styles.infoItem}>\n                    <Text style={styles.infoLabel}>Uptime</Text>\n                    <Text style={styles.infoValue}>\n                      {currentMetrics ? Math.floor(currentMetrics.uptime / 3600) + 'h' : 'Unknown'}\n                    </Text>\n                  </View>\n                  <View style={styles.infoItem}>\n                    <Text style={styles.infoLabel}>CPU</Text>\n                    <Text style={styles.infoValue}>{systemInfo.cpu || 'Unknown'}</Text>\n                  </View>\n                  <View style={styles.infoItem}>\n                    <Text style={styles.infoLabel}>Memory</Text>\n                    <Text style={styles.infoValue}>\n                      {currentMetrics ? \n                        `${(currentMetrics.memory.total / 1024 / 1024 / 1024).toFixed(1)}GB` : \n                        'Unknown'\n                      }\n                    </Text>\n                  </View>\n                </View>\n              )}\n            </Card.Content>\n          </Card>\n        );\n      \n      case 'alerts':\n        const activeAlerts = alerts.filter(a => !a.dismissed);\n        if (activeAlerts.length === 0) return null;\n        \n        return (\n          <Card key={widget.id} style={styles.card}>\n            <Card.Title \n              title={`System Alerts (${activeAlerts.length})`}\n              left={() => <Icon name=\"warning\" size={24} color=\"#FF9800\" />}\n              right={() => (\n                <Button mode=\"text\" onPress={clearAllAlerts}>\n                  Clear All\n                </Button>\n              )}\n            />\n            <Card.Content>\n              {activeAlerts.slice(0, 3).map(alert => (\n                <View key={alert.id} style={styles.alertItem}>\n                  <View style={styles.alertContent}>\n                    <View style={styles.alertHeader}>\n                      <Icon \n                        name={alert.level === 'critical' ? 'error' : 'warning'} \n                        size={16} \n                        color={getAlertColor(alert.level)} \n                      />\n                      <Text style={[styles.alertTitle, { color: getAlertColor(alert.level) }]}>\n                        {alert.title}\n                      </Text>\n                    </View>\n                    <Text style={styles.alertMessage}>{alert.message}</Text>\n                    <Text style={styles.alertTime}>\n                      {new Date(alert.timestamp).toLocaleTimeString()}\n                    </Text>\n                  </View>\n                  <Button\n                    mode=\"text\"\n                    compact\n                    onPress={() => dismissAlert(alert.id)}\n                  >\n                    Dismiss\n                  </Button>\n                </View>\n              ))}\n            </Card.Content>\n          </Card>\n        );\n      \n      default:\n        return null;\n    }\n  };\n\n  const getAlertColor = (level: string) => {\n    switch (level) {\n      case 'critical': return '#F44336';\n      case 'error': return '#FF5722';\n      case 'warning': return '#FF9800';\n      case 'info': return '#2196F3';\n      default: return '#4CAF50';\n    }\n  };\n\n  const sortedWidgets = widgets\n    .filter(w => w.visible)\n    .sort((a, b) => a.position - b.position);\n\n  return (\n    <View style={styles.container}>\n      <ScrollView\n        refreshControl={\n          <RefreshControl\n            refreshing={isRefreshing}\n            onRefresh={onRefresh}\n            colors={['#4CAF50']}\n            progressBackgroundColor=\"#1e1e1e\"\n          />\n        }\n        onScroll={({ nativeEvent }) => {\n          const currentScrollY = nativeEvent.contentOffset.y;\n          setFabExtended(currentScrollY <= 0);\n        }}\n        scrollEventThrottle={16}\n      >\n        {/* Connection Status Header */}\n        <Surface style={styles.statusHeader}>\n          <View style={styles.statusContent}>\n            <Text style={styles.statusTitle}>Nexus Terminal</Text>\n            <Text style={styles.statusSubtitle}>\n              {connectionStatus === 'connected' ? 'Connected' : 'Disconnected'}\n            </Text>\n            {lastUpdate && (\n              <Text style={styles.lastUpdateText}>\n                Last updated: {lastUpdate.toLocaleTimeString()}\n              </Text>\n            )}\n          </View>\n          <View style={styles.statusIndicator}>\n            <Icon \n              name={connectionStatus === 'connected' ? 'check-circle' : 'error'} \n              size={32} \n              color={connectionStatus === 'connected' ? '#4CAF50' : '#F44336'} \n            />\n          </View>\n        </Surface>\n\n        {/* Widgets */}\n        {sortedWidgets.map(widget => renderWidget(widget))}\n\n        {/* Performance Summary Card */}\n        {currentMetrics && (\n          <Card style={styles.card}>\n            <Card.Title \n              title=\"System Overview\"\n              left={() => <Icon name=\"speed\" size={24} color=\"#2196F3\" />}\n            />\n            <Card.Content>\n              <View style={styles.metricsGrid}>\n                <View style={styles.metricCard}>\n                  <Text style={styles.metricLabel}>CPU</Text>\n                  <Text style={[styles.metricValue, { color: getMetricColor(currentMetrics.cpu.usage) }]}>\n                    {currentMetrics.cpu.usage.toFixed(1)}%\n                  </Text>\n                  <Text style={styles.metricSubtext}>\n                    {currentMetrics.cpu.cores} cores @ {currentMetrics.cpu.frequency}MHz\n                  </Text>\n                </View>\n                \n                <View style={styles.metricCard}>\n                  <Text style={styles.metricLabel}>Memory</Text>\n                  <Text style={[styles.metricValue, { color: getMetricColor(currentMetrics.memory.percentage) }]}>\n                    {currentMetrics.memory.percentage.toFixed(1)}%\n                  </Text>\n                  <Text style={styles.metricSubtext}>\n                    {(currentMetrics.memory.used / 1024 / 1024 / 1024).toFixed(1)}GB / \n                    {(currentMetrics.memory.total / 1024 / 1024 / 1024).toFixed(1)}GB\n                  </Text>\n                </View>\n                \n                <View style={styles.metricCard}>\n                  <Text style={styles.metricLabel}>Disk</Text>\n                  <Text style={[styles.metricValue, { color: getMetricColor(currentMetrics.disk.percentage) }]}>\n                    {currentMetrics.disk.percentage.toFixed(1)}%\n                  </Text>\n                  <Text style={styles.metricSubtext}>\n                    {(currentMetrics.disk.available / 1024 / 1024 / 1024).toFixed(1)}GB free\n                  </Text>\n                </View>\n                \n                <View style={styles.metricCard}>\n                  <Text style={styles.metricLabel}>Network</Text>\n                  <Text style={styles.metricValue}>\n                    ↓{(currentMetrics.network.bytesReceived / 1024 / 1024).toFixed(1)}MB\n                  </Text>\n                  <Text style={styles.metricSubtext}>\n                    ↑{(currentMetrics.network.bytesSent / 1024 / 1024).toFixed(1)}MB\n                  </Text>\n                </View>\n              </View>\n            </Card.Content>\n          </Card>\n        )}\n      </ScrollView>\n\n      {/* Floating Action Button */}\n      <AnimatedFAB\n        icon=\"refresh\"\n        label=\"Refresh\"\n        extended={fabExtended}\n        onPress={onRefresh}\n        visible={connectionStatus === 'connected'}\n        animateFrom=\"right\"\n        iconMode=\"dynamic\"\n        style={[styles.fab]}\n      />\n\n      {/* Customize Dialog */}\n      <Portal>\n        <Dialog visible={showCustomizeDialog} onDismiss={() => setShowCustomizeDialog(false)}>\n          <Dialog.Title>Customize Dashboard</Dialog.Title>\n          <Dialog.Content>\n            <Text style={styles.dialogDescription}>\n              Toggle widgets to show or hide them on your dashboard.\n            </Text>\n            \n            {widgets.map(widget => (\n              <View key={widget.id} style={styles.widgetToggle}>\n                <Text style={styles.widgetName}>\n                  {widget.type.charAt(0).toUpperCase() + widget.type.slice(1)}\n                </Text>\n                <Button\n                  mode={widget.visible ? 'contained' : 'outlined'}\n                  compact\n                  onPress={() => toggleWidget(widget.id)}\n                >\n                  {widget.visible ? 'Visible' : 'Hidden'}\n                </Button>\n              </View>\n            ))}\n          </Dialog.Content>\n          <Dialog.Actions>\n            <Button onPress={() => setShowCustomizeDialog(false)}>Done</Button>\n          </Dialog.Actions>\n        </Dialog>\n      </Portal>\n    </View>\n  );\n};\n\nconst getMetricColor = (percentage: number) => {\n  if (percentage > 90) return '#F44336';\n  if (percentage > 75) return '#FF9800';\n  if (percentage > 50) return '#FF9800';\n  return '#4CAF50';\n};\n\nconst styles = StyleSheet.create({\n  container: {\n    flex: 1,\n    backgroundColor: '#121212',\n  },\n  statusHeader: {\n    margin: 8,\n    padding: 16,\n    backgroundColor: '#1e1e1e',\n    borderRadius: 8,\n    flexDirection: 'row',\n    justifyContent: 'space-between',\n    alignItems: 'center',\n  },\n  statusContent: {\n    flex: 1,\n  },\n  statusTitle: {\n    fontSize: 20,\n    fontWeight: 'bold',\n    color: '#fff',\n  },\n  statusSubtitle: {\n    fontSize: 14,\n    color: '#ccc',\n  },\n  lastUpdateText: {\n    fontSize: 12,\n    color: '#888',\n    marginTop: 4,\n  },\n  statusIndicator: {\n    // Icon styling handled inline\n  },\n  card: {\n    margin: 8,\n    backgroundColor: '#1e1e1e',\n  },\n  metricsGrid: {\n    flexDirection: 'row',\n    flexWrap: 'wrap',\n    justifyContent: 'space-between',\n  },\n  metricCard: {\n    width: '48%',\n    marginBottom: 16,\n    padding: 12,\n    backgroundColor: '#2a2a2a',\n    borderRadius: 8,\n  },\n  metricLabel: {\n    fontSize: 12,\n    color: '#ccc',\n    marginBottom: 4,\n  },\n  metricValue: {\n    fontSize: 18,\n    fontWeight: 'bold',\n    marginBottom: 2,\n  },\n  metricSubtext: {\n    fontSize: 10,\n    color: '#888',\n  },\n  alertItem: {\n    flexDirection: 'row',\n    justifyContent: 'space-between',\n    alignItems: 'center',\n    marginBottom: 8,\n    padding: 8,\n    backgroundColor: '#2a2a2a',\n    borderRadius: 4,\n  },\n  alertContent: {\n    flex: 1,\n  },\n  alertHeader: {\n    flexDirection: 'row',\n    alignItems: 'center',\n    marginBottom: 4,\n  },\n  alertTitle: {\n    fontSize: 14,\n    fontWeight: 'bold',\n    marginLeft: 8,\n  },\n  alertMessage: {\n    fontSize: 12,\n    color: '#ccc',\n    marginBottom: 2,\n  },\n  alertTime: {\n    fontSize: 10,\n    color: '#888',\n  },\n  fab: {\n    position: 'absolute',\n    right: 16,\n    bottom: 16,\n    backgroundColor: '#4CAF50',\n  },\n  dialogDescription: {\n    fontSize: 14,\n    color: '#ccc',\n    marginBottom: 16,\n  },\n  widgetToggle: {\n    flexDirection: 'row',\n    justifyContent: 'space-between',\n    alignItems: 'center',\n    marginBottom: 12,\n  },\n  widgetName: {\n    fontSize: 16,\n    color: '#fff',\n  },\n});
