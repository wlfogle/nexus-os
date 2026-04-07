import React, { useState, useEffect } from 'react';
import { View, StyleSheet, Animated } from 'react-native';
import { Card, Text, ProgressBar, Chip } from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import NetInfo from '@react-native-community/netinfo';
import { ConnectionManager } from '@services/EnhancedConnectionManager';

interface NetworkState {
  type: string;
  isConnected: boolean;
  strength?: number;
  ipAddress?: string;
  ssid?: string;
  frequency?: number;
}

interface ConnectionMetrics {
  latency: number;
  downloadSpeed: number;
  uploadSpeed: number;
  packetLoss: number;
  quality: 'excellent' | 'good' | 'fair' | 'poor';
}

export const NetworkIndicator: React.FC = () => {
  const [networkState, setNetworkState] = useState<NetworkState | null>(null);
  const [connectionMetrics, setConnectionMetrics] = useState<ConnectionMetrics | null>(null);
  const [isTestingSpeed, setIsTestingSpeed] = useState(false);
  const pulseAnim = new Animated.Value(1);

  useEffect(() => {
    const unsubscribe = NetInfo.addEventListener(state => {
      setNetworkState({
        type: state.type,
        isConnected: state.isConnected || false,
        strength: state.details && 'strength' in state.details ? state.details.strength as number : undefined,
        ipAddress: state.details && 'ipAddress' in state.details ? state.details.ipAddress as string : undefined,
        ssid: state.details && 'ssid' in state.details ? state.details.ssid as string : undefined,
        frequency: state.details && 'frequency' in state.details ? state.details.frequency as number : undefined,
      });
    });

    startPulseAnimation();
    measureConnectionQuality();

    return () => {
      unsubscribe();
    };
  }, []);

  const startPulseAnimation = () => {
    Animated.loop(
      Animated.sequence([
        Animated.timing(pulseAnim, {
          toValue: 0.7,
          duration: 1000,
          useNativeDriver: true,
        }),
        Animated.timing(pulseAnim, {
          toValue: 1,
          duration: 1000,
          useNativeDriver: true,
        }),
      ])
    ).start();
  };

  const measureConnectionQuality = async () => {
    if (!ConnectionManager.isConnected()) return;

    try {
      const startTime = Date.now();
      
      // Measure latency with ping
      await ConnectionManager.request('/api/health');
      const latency = Date.now() - startTime;

      // Get performance metrics from ConnectionManager
      const perfMetrics = ConnectionManager.getPerformanceMetrics();
      
      // Calculate quality based on latency and error rate
      let quality: ConnectionMetrics['quality'] = 'excellent';
      if (latency > 1000 || perfMetrics.errorCount > 5) {
        quality = 'poor';
      } else if (latency > 500 || perfMetrics.errorCount > 2) {
        quality = 'fair';
      } else if (latency > 200 || perfMetrics.errorCount > 0) {
        quality = 'good';
      }

      setConnectionMetrics({
        latency,
        downloadSpeed: 0, // Would need actual speed test
        uploadSpeed: 0,
        packetLoss: (perfMetrics.errorCount / perfMetrics.requestCount) * 100,
        quality,
      });
    } catch (error) {
      console.error('Failed to measure connection quality:', error);
    }
  };

  const speedTest = async () => {
    setIsTestingSpeed(true);
    try {
      // Simulate speed test with actual data transfer
      const testData = new Array(1024 * 100).fill('x').join(''); // 100KB
      const startTime = Date.now();
      
      await ConnectionManager.request('/api/speedtest', {
        method: 'POST',
        data: { testData },
      });
      
      const duration = (Date.now() - startTime) / 1000;
      const downloadSpeed = (100 / duration); // KB/s
      
      setConnectionMetrics(prev => prev ? {
        ...prev,
        downloadSpeed,
        uploadSpeed: downloadSpeed * 0.8, // Estimate upload speed
      } : null);
      
    } catch (error) {
      console.error('Speed test failed:', error);
    } finally {
      setIsTestingSpeed(false);
    }
  };

  const getStatusColor = () => {
    if (!networkState?.isConnected) return '#F44336';
    if (!ConnectionManager.isConnected()) return '#FF9800';
    
    switch (connectionMetrics?.quality) {
      case 'excellent': return '#4CAF50';
      case 'good': return '#8BC34A';
      case 'fair': return '#FF9800';
      case 'poor': return '#F44336';
      default: return '#2196F3';
    }
  };

  const getStatusText = () => {
    if (!networkState?.isConnected) return 'No Network';
    if (!ConnectionManager.isConnected()) return 'Not Connected';
    return connectionMetrics?.quality || 'Unknown';
  };

  const getSignalStrength = () => {
    if (!networkState?.strength) return 0;
    return networkState.strength / 100;
  };

  return (
    <Card style={styles.card}>
      <Card.Content>
        <View style={styles.header}>
          <View style={styles.statusContainer}>
            <Animated.View 
              style={[
                styles.statusIndicator, 
                { 
                  backgroundColor: getStatusColor(),
                  opacity: pulseAnim,
                }
              ]} 
            />
            <Text style={[styles.statusText, { color: getStatusColor() }]}>
              {getStatusText()}
            </Text>
          </View>
          
          <Chip 
            icon="wifi" 
            style={[styles.networkChip, { backgroundColor: getStatusColor() + '20' }]}
          >
            {networkState?.type.toUpperCase() || 'Unknown'}
          </Chip>
        </View>

        {/* Network Details */}
        {networkState?.isConnected && (
          <View style={styles.networkDetails}>
            {networkState.ipAddress && (
              <View style={styles.detailRow}>
                <Icon name="router" size={16} color="#ccc" />
                <Text style={styles.detailText}>IP: {networkState.ipAddress}</Text>
              </View>
            )}
            
            {networkState.ssid && (
              <View style={styles.detailRow}>
                <Icon name="wifi" size={16} color="#ccc" />
                <Text style={styles.detailText}>SSID: {networkState.ssid}</Text>
              </View>
            )}
            
            {networkState.strength !== undefined && (
              <View style={styles.detailRow}>
                <Icon name="signal-wifi-4-bar" size={16} color="#ccc" />
                <Text style={styles.detailText}>Signal: {networkState.strength}%</Text>
                <ProgressBar 
                  progress={getSignalStrength()} 
                  color={getStatusColor()}
                  style={styles.signalBar}
                />
              </View>
            )}
          </View>
        )}

        {/* Connection Metrics */}
        {connectionMetrics && ConnectionManager.isConnected() && (
          <View style={styles.metricsContainer}>
            <View style={styles.metricRow}>
              <View style={styles.metric}>
                <Text style={styles.metricLabel}>Latency</Text>
                <Text style={styles.metricValue}>{connectionMetrics.latency}ms</Text>
              </View>
              
              <View style={styles.metric}>
                <Text style={styles.metricLabel}>Packet Loss</Text>
                <Text style={styles.metricValue}>{connectionMetrics.packetLoss.toFixed(1)}%</Text>
              </View>
            </View>
            
            {connectionMetrics.downloadSpeed > 0 && (
              <View style={styles.metricRow}>
                <View style={styles.metric}>
                  <Text style={styles.metricLabel}>Download</Text>
                  <Text style={styles.metricValue}>{connectionMetrics.downloadSpeed.toFixed(1)} KB/s</Text>
                </View>
                
                <View style={styles.metric}>
                  <Text style={styles.metricLabel}>Upload</Text>
                  <Text style={styles.metricValue}>{connectionMetrics.uploadSpeed.toFixed(1)} KB/s</Text>
                </View>
              </View>
            )}
          </View>
        )}
      </Card.Content>
    </Card>
  );
};

const styles = StyleSheet.create({
  card: {
    margin: 8,
    backgroundColor: '#1e1e1e',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  statusContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusIndicator: {
    width: 12,
    height: 12,
    borderRadius: 6,
    marginRight: 8,
  },
  statusText: {
    fontSize: 16,
    fontWeight: 'bold',
  },
  networkChip: {
    // Additional styling handled by backgroundColor prop
  },
  networkDetails: {
    marginBottom: 12,
  },
  detailRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 4,
  },
  detailText: {
    fontSize: 12,
    color: '#ccc',
    marginLeft: 8,
    flex: 1,
  },
  signalBar: {
    width: 60,
    height: 4,
    marginLeft: 8,
  },
  metricsContainer: {
    borderTopWidth: 1,
    borderTopColor: '#333',
    paddingTop: 12,
  },
  metricRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  metric: {
    flex: 1,
    alignItems: 'center',
  },
  metricLabel: {
    fontSize: 10,
    color: '#888',
    marginBottom: 2,
  },
  metricValue: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#4CAF50',
  },
});
