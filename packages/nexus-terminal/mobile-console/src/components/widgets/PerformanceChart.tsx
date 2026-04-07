import React, { useState, useEffect, useRef } from 'react';
import { View, StyleSheet, Dimensions } from 'react-native';
import { Card, Text, SegmentedButtons } from 'react-native-paper';
import { LineChart, AreaChart } from 'react-native-chart-kit';
import { SystemMetrics } from '@services/EnhancedConnectionManager';

interface PerformanceChartProps {
  metrics: SystemMetrics[];
  height?: number;
  showLegend?: boolean;
  chartType?: 'line' | 'area';
  metricType?: 'cpu' | 'memory' | 'disk' | 'network';
}

export const PerformanceChart: React.FC<PerformanceChartProps> = ({
  metrics,
  height = 200,
  showLegend = true,
  chartType = 'line',
  metricType = 'cpu',
}) => {
  const [selectedMetric, setSelectedMetric] = useState(metricType);
  const [timeRange, setTimeRange] = useState('1h');
  const screenWidth = Dimensions.get('window').width;

  const getChartData = () => {
    if (!metrics || metrics.length === 0) {
      return {
        labels: [''],
        datasets: [{ data: [0] }],
      };
    }

    // Filter data based on time range
    const now = Date.now();
    const timeRanges = {
      '5m': 5 * 60 * 1000,
      '15m': 15 * 60 * 1000,
      '1h': 60 * 60 * 1000,
      '6h': 6 * 60 * 60 * 1000,
      '24h': 24 * 60 * 60 * 1000,
    };

    const filteredMetrics = metrics.filter(
      m => now - m.timestamp <= timeRanges[timeRange as keyof typeof timeRanges]
    );

    const labels = filteredMetrics.map((_, index) => {
      if (index % Math.ceil(filteredMetrics.length / 6) === 0) {
        const date = new Date(filteredMetrics[index].timestamp);
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      }
      return '';
    });

    let datasets: any[] = [];

    switch (selectedMetric) {
      case 'cpu':
        datasets = [{
          data: filteredMetrics.map(m => m.cpu.usage),
          color: (opacity = 1) => `rgba(76, 175, 80, ${opacity})`,
          strokeWidth: 2,
        }];
        break;
      case 'memory':
        datasets = [{
          data: filteredMetrics.map(m => m.memory.percentage),
          color: (opacity = 1) => `rgba(33, 150, 243, ${opacity})`,
          strokeWidth: 2,
        }];
        break;
      case 'disk':
        datasets = [{
          data: filteredMetrics.map(m => m.disk.percentage),
          color: (opacity = 1) => `rgba(255, 152, 0, ${opacity})`,
          strokeWidth: 2,
        }];
        break;
      case 'network':
        datasets = [
          {
            data: filteredMetrics.map(m => m.network.bytesReceived / 1024 / 1024), // MB
            color: (opacity = 1) => `rgba(156, 39, 176, ${opacity})`,
            strokeWidth: 2,
          },
          {
            data: filteredMetrics.map(m => m.network.bytesSent / 1024 / 1024), // MB
            color: (opacity = 1) => `rgba(244, 67, 54, ${opacity})`,
            strokeWidth: 2,
          }
        ];
        break;
    }

    return { labels, datasets };
  };

  const chartConfig = {
    backgroundColor: '#1e1e1e',
    backgroundGradientFrom: '#1e1e1e',
    backgroundGradientTo: '#1e1e1e',
    decimalPlaces: 1,
    color: (opacity = 1) => `rgba(255, 255, 255, ${opacity})`,
    labelColor: (opacity = 1) => `rgba(255, 255, 255, ${opacity})`,
    style: {
      borderRadius: 16,
    },
    propsForDots: {
      r: '3',
      strokeWidth: '1',
      stroke: '#4CAF50',
    },
    propsForBackgroundLines: {
      strokeDasharray: '',
      stroke: '#333',
      strokeWidth: 1,
    },
    fillShadowGradient: '#4CAF50',
    fillShadowGradientOpacity: 0.3,
  };

  const ChartComponent = chartType === 'area' ? AreaChart : LineChart;

  const getMetricLabel = () => {
    switch (selectedMetric) {
      case 'cpu': return 'CPU Usage (%)';
      case 'memory': return 'Memory Usage (%)';
      case 'disk': return 'Disk Usage (%)';
      case 'network': return 'Network (MB/s)';
      default: return 'Metric';
    }
  };

  return (
    <Card style={styles.card}>
      <Card.Title 
        title="Performance Monitor"
        subtitle={getMetricLabel()}
      />
      <Card.Content>
        {/* Metric Selector */}
        <SegmentedButtons
          value={selectedMetric}
          onValueChange={setSelectedMetric}
          buttons={[
            { value: 'cpu', label: 'CPU', icon: 'memory' },
            { value: 'memory', label: 'RAM', icon: 'storage' },
            { value: 'disk', label: 'Disk', icon: 'harddisk' },
            { value: 'network', label: 'Net', icon: 'network' },
          ]}
          style={styles.segmentedButtons}
        />

        {/* Time Range Selector */}
        <SegmentedButtons
          value={timeRange}
          onValueChange={setTimeRange}
          buttons={[
            { value: '5m', label: '5m' },
            { value: '15m', label: '15m' },
            { value: '1h', label: '1h' },
            { value: '6h', label: '6h' },
            { value: '24h', label: '24h' },
          ]}
          style={styles.timeButtons}
        />

        {/* Chart */}
        <View style={styles.chartContainer}>
          <ChartComponent
            data={getChartData()}
            width={screenWidth - 48}
            height={height}
            chartConfig={chartConfig}
            bezier={chartType === 'line'}
            style={styles.chart}
          />
        </View>

        {/* Current Values */}
        {metrics.length > 0 && (
          <View style={styles.currentValues}>
            <Text style={styles.currentLabel}>Current: </Text>
            <Text style={styles.currentValue}>
              {selectedMetric === 'cpu' && `${metrics[metrics.length - 1].cpu.usage.toFixed(1)}%`}
              {selectedMetric === 'memory' && `${metrics[metrics.length - 1].memory.percentage.toFixed(1)}%`}
              {selectedMetric === 'disk' && `${metrics[metrics.length - 1].disk.percentage.toFixed(1)}%`}
              {selectedMetric === 'network' && 
                `↓${(metrics[metrics.length - 1].network.bytesReceived / 1024 / 1024).toFixed(1)}MB ` +
                `↑${(metrics[metrics.length - 1].network.bytesSent / 1024 / 1024).toFixed(1)}MB`
              }
            </Text>
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
  segmentedButtons: {
    marginBottom: 12,
  },
  timeButtons: {
    marginBottom: 16,
  },
  chartContainer: {
    alignItems: 'center',
    marginBottom: 12,
  },
  chart: {
    borderRadius: 16,
  },
  currentValues: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
  },
  currentLabel: {
    fontSize: 14,
    color: '#ccc',
  },
  currentValue: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#4CAF50',
  },
});
