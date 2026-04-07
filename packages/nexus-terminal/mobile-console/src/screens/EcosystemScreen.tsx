import React, { useState, useEffect } from 'react';
import { View, StyleSheet, ScrollView, RefreshControl, Alert } from 'react-native';
import { Card, Text, Chip, ProgressBar, Button } from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import { LineChart } from 'react-native-chart-kit';
import { Dimensions } from 'react-native';
import { ConnectionManager } from '@services/ConnectionManager';

const screenWidth = Dimensions.get('window').width;

interface EcosystemInsights {
  contextualSuggestions: Array<{
    suggestion: string;
    category: string;
    confidence: number;
    reasoning: string;
    commands: string[];
  }>;
  identifiedPatterns: Array<{
    pattern_type: string;
    description: string;
    frequency: number;
    confidence: number;
    impact: string;
  }>;
  systemAnalysis: {
    performance_patterns: Array<{
      pattern_type: string;
      trend: string;
      severity: string;
      recommendation: string;
    }>;
    optimization_opportunities: Array<{
      area: string;
      description: string;
      potential_improvement: string;
      estimated_impact: string;
    }>;
  };
  adaptiveRecommendations: Array<{
    recommendation: string;
    category: string;
    impact: string;
    confidence: number;
    implementation_steps: string[];
  }>;
  performanceMetrics: {
    cpu_trend: number[];
    memory_trend: number[];
    timestamps: string[];
  };
}

export const EcosystemScreen: React.FC = () => {
  const [insights, setInsights] = useState<EcosystemInsights | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [expandedCards, setExpandedCards] = useState<Set<string>>(new Set());

  const categories = ['all', 'performance', 'development', 'security', 'optimization'];

  useEffect(() => {
    loadEcosystemInsights();
    
    // Subscribe to real-time insights updates
    const handleInsightsUpdate = (data: any) => {
      setInsights(data);
    };

    ConnectionManager.on('ecosystemInsights', handleInsightsUpdate);
    
    return () => {
      ConnectionManager.off('ecosystemInsights', handleInsightsUpdate);
    };
  }, []);

  const loadEcosystemInsights = async () => {
    try {
      const data = await ConnectionManager.getEcosystemInsights();
      setInsights(data);
    } catch (error) {
      console.error('Failed to load ecosystem insights:', error);
      Alert.alert('Error', 'Failed to load ecosystem insights');
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await loadEcosystemInsights();
    setRefreshing(false);
  };

  const toggleCardExpansion = (cardId: string) => {
    const newExpanded = new Set(expandedCards);
    if (newExpanded.has(cardId)) {
      newExpanded.delete(cardId);
    } else {
      newExpanded.add(cardId);
    }
    setExpandedCards(newExpanded);
  };

  const getFilteredSuggestions = () => {
    if (!insights) return [];
    
    if (selectedCategory === 'all') {
      return insights.contextualSuggestions;
    }
    
    return insights.contextualSuggestions.filter(
      suggestion => suggestion.category === selectedCategory
    );
  };

  const getConfidenceColor = (confidence: number): string => {
    if (confidence >= 0.8) return '#4CAF50';
    if (confidence >= 0.6) return '#FF9800';
    return '#F44336';
  };

  const getSeverityColor = (severity: string): string => {
    switch (severity.toLowerCase()) {
      case 'critical': return '#F44336';
      case 'high': return '#FF5722';
      case 'medium': return '#FF9800';
      case 'low': return '#4CAF50';
      default: return '#2196F3';
    }
  };

  const executeCommand = async (command: string) => {
    try {
      const result = await ConnectionManager.executeCommand({ command });
      Alert.alert(
        'Command Result',
        result.success ? result.output : result.error || 'Command failed'
      );
    } catch (error) {
      Alert.alert('Error', 'Failed to execute command');
    }
  };

  const chartConfig = {
    backgroundColor: '#1e1e1e',
    backgroundGradientFrom: '#1e1e1e',
    backgroundGradientTo: '#1e1e1e',
    decimalPlaces: 1,
    color: (opacity = 1) => `rgba(76, 175, 80, ${opacity})`,
    labelColor: (opacity = 1) => `rgba(204, 204, 204, ${opacity})`,
    style: {
      borderRadius: 16,
    },
    propsForDots: {
      r: '4',
      strokeWidth: '2',
      stroke: '#4CAF50',
    },
  };

  return (
    <View style={styles.container}>
      <ScrollView 
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        {/* Category filter */}
        <ScrollView 
          horizontal 
          showsHorizontalScrollIndicator={false}
          style={styles.categoryContainer}
        >
          {categories.map((category) => (
            <Chip
              key={category}
              selected={selectedCategory === category}
              onPress={() => setSelectedCategory(category)}
              style={styles.categoryChip}
              textStyle={styles.categoryText}
            >
              {category.charAt(0).toUpperCase() + category.slice(1)}
            </Chip>
          ))}
        </ScrollView>

        {insights && (
          <>
            {/* Performance Metrics Chart */}
            {insights.performanceMetrics && (
              <Card style={styles.card}>
                <Card.Title 
                  title="Performance Trends" 
                  left={() => <Icon name="trending-up" size={24} color="#4CAF50" />}
                />
                <Card.Content>
                  <LineChart
                    data={{
                      labels: insights.performanceMetrics.timestamps.slice(-6),
                      datasets: [
                        {
                          data: insights.performanceMetrics.cpu_trend.slice(-6),
                          color: (opacity = 1) => `rgba(255, 152, 0, ${opacity})`,
                          strokeWidth: 2,
                        },
                        {
                          data: insights.performanceMetrics.memory_trend.slice(-6),
                          color: (opacity = 1) => `rgba(33, 150, 243, ${opacity})`,
                          strokeWidth: 2,
                        },
                      ],
                      legend: ['CPU %', 'Memory %'],
                    }}
                    width={screenWidth - 48}
                    height={220}
                    chartConfig={chartConfig}
                    bezier
                    style={styles.chart}
                  />
                </Card.Content>
              </Card>
            )}

            {/* Contextual Suggestions */}
            <Card style={styles.card}>
              <Card.Title 
                title="AI Suggestions" 
                subtitle={`${getFilteredSuggestions().length} suggestions`}
                left={() => <Icon name="lightbulb" size={24} color="#FF9800" />}
              />
              <Card.Content>
                {getFilteredSuggestions().slice(0, 5).map((suggestion, index) => (
                  <View key={index} style={styles.suggestionItem}>
                    <View style={styles.suggestionHeader}>
                      <Text style={styles.suggestionText}>{suggestion.suggestion}</Text>
                      <View style={styles.confidenceBadge}>
                        <Text style={[
                          styles.confidenceText,
                          { color: getConfidenceColor(suggestion.confidence) }
                        ]}>
                          {(suggestion.confidence * 100).toFixed(0)}%
                        </Text>
                      </View>
                    </View>
                    
                    <Text style={styles.reasoningText}>{suggestion.reasoning}</Text>
                    
                    {suggestion.commands.length > 0 && (
                      <ScrollView horizontal showsHorizontalScrollIndicator={false}>
                        {suggestion.commands.map((cmd, cmdIndex) => (
                          <Button
                            key={cmdIndex}
                            mode="outlined"
                            onPress={() => executeCommand(cmd)}
                            style={styles.commandButton}
                            compact
                          >
                            {cmd}
                          </Button>
                        ))}
                      </ScrollView>
                    )}
                  </View>
                ))}
              </Card.Content>
            </Card>

            {/* System analysis and other cards would go here */}
            
          </>
        )}

        {!insights && (
          <View style={styles.emptyState}>
            <Icon name="eco" size={64} color="#666" />
            <Text style={styles.emptyText}>Loading ecosystem insights...</Text>
            <Text style={styles.emptySubtext}>
              AI is analyzing your system patterns and behavior
            </Text>
          </View>
        )}
      </ScrollView>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#121212',
  },
  categoryContainer: {
    maxHeight: 50,
    backgroundColor: '#1a1a1a',
    paddingHorizontal: 8,
    paddingVertical: 8,
  },
  categoryChip: {
    marginRight: 8,
    backgroundColor: '#333',
  },
  categoryText: {
    color: '#ccc',
    fontSize: 12,
  },
  card: {
    margin: 8,
    backgroundColor: '#1e1e1e',
  },
  chart: {
    marginVertical: 8,
    borderRadius: 16,
  },
  suggestionItem: {
    marginBottom: 16,
    padding: 12,
    backgroundColor: '#2a2a2a',
    borderRadius: 8,
  },
  suggestionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 8,
  },
  suggestionText: {
    flex: 1,
    fontSize: 14,
    fontWeight: 'bold',
    color: '#fff',
    marginRight: 8,
  },
  confidenceBadge: {
    backgroundColor: '#333',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 12,
  },
  confidenceText: {
    fontSize: 12,
    fontWeight: 'bold',
  },
  reasoningText: {
    fontSize: 12,
    color: '#ccc',
    marginBottom: 8,
    fontStyle: 'italic',
  },
  commandButton: {
    marginRight: 8,
    marginTop: 4,
  },
  emptyState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    fontSize: 18,
    color: '#666',
    marginTop: 16,
    textAlign: 'center',
  },
  emptySubtext: {
    fontSize: 14,
    color: '#555',
    marginTop: 8,
    textAlign: 'center',
  },
});
