import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface AIRecommendation {
  id: string;
  category: string;
  title: string;
  description: string;
  priority: number;
  actions: string[];
  auto_apply: boolean;
  timestamp: number;
}

interface SystemInsight {
  pattern: string;
  confidence: number;
  recommendation: string;
  priority: number;
  timestamp: string;
}

interface DecisionStat {
  action: string;
  success_rate: number;
  total_attempts: number;
  last_used: number;
}

interface AIInsightsProps {
  insights: SystemInsight[];
}

export const AIInsights: React.FC<AIInsightsProps> = ({ insights }) => {
  const [recommendations, setRecommendations] = useState<AIRecommendation[]>([]);
  const [decisionStats, setDecisionStats] = useState<DecisionStat[]>([]);
  const [aiSettings, setAiSettings] = useState({
    learningEnabled: true,
    autoOptimize: false,
    confidenceThreshold: 0.75,
    sensitivityLevel: 5.0
  });
  const [performanceTrends, setPerformanceTrends] = useState<any>({});
  const [isLoading, setIsLoading] = useState(false);
  const [chatInput, setChatInput] = useState('');
  const [chatHistory, setChatHistory] = useState<Array<{type: 'user' | 'ai', message: string, timestamp: number}>>([]);

  useEffect(() => {
    loadAIData();
    const interval = setInterval(loadAIData, 30000); // Refresh every 30 seconds
    return () => clearInterval(interval);
  }, []);

  const loadAIData = async () => {
    try {
      setIsLoading(true);
      
      // Get AI recommendations
      const recsData: AIRecommendation[] = await invoke('get_ai_recommendations');
      setRecommendations(recsData);
      
      // Get decision statistics
      const statsData: any = await invoke('get_decision_statistics');
      const formattedStats: DecisionStat[] = Object.entries(statsData)
        .filter(([key]) => key.startsWith('action_'))
        .map(([key, value]) => ({
          action: key.replace('action_', ''),
          success_rate: Math.random() * 100, // Placeholder
          total_attempts: value as number,
          last_used: Date.now() - Math.random() * 86400000
        }));
      setDecisionStats(formattedStats);
      
      // Get performance trends
      const trendsData: any = await invoke('get_performance_trends');
      setPerformanceTrends(trendsData);
      
    } catch (error) {
      console.error('Failed to load AI data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleApplyRecommendation = async (recId: string) => {
    try {
      await invoke('apply_ai_recommendation', { recommendationId: recId });
      await loadAIData(); // Refresh recommendations
    } catch (error) {
      console.error('Failed to apply recommendation:', error);
    }
  };

  const handleDismissRecommendation = async (recId: string) => {
    try {
      await invoke('dismiss_ai_recommendation', { recommendationId: recId });
      setRecommendations(prev => prev.filter(r => r.id !== recId));
    } catch (error) {
      console.error('Failed to dismiss recommendation:', error);
    }
  };

  const handleChatSubmit = async () => {
    if (!chatInput.trim()) return;
    
    const userMessage = chatInput;
    setChatInput('');
    
    // Add user message to chat
    setChatHistory(prev => [...prev, {
      type: 'user',
      message: userMessage,
      timestamp: Date.now()
    }]);
    
    try {
      // Process natural language query
      const response: string = await invoke('process_natural_language', { query: userMessage });
      
      // Add AI response to chat
      setChatHistory(prev => [...prev, {
        type: 'ai',
        message: response,
        timestamp: Date.now()
      }]);
      
    } catch (error) {
      console.error('Failed to process chat:', error);
      setChatHistory(prev => [...prev, {
        type: 'ai',
        message: 'Sorry, I encountered an error processing your request.',
        timestamp: Date.now()
      }]);
    }
  };

  const getPriorityColor = (priority: number) => {
    if (priority >= 9) return 'text-red-400 bg-red-900';
    if (priority >= 7) return 'text-orange-400 bg-orange-900';
    if (priority >= 5) return 'text-yellow-400 bg-yellow-900';
    return 'text-green-400 bg-green-900';
  };

  const getPriorityLabel = (priority: number) => {
    if (priority >= 9) return 'Critical';
    if (priority >= 7) return 'High';
    if (priority >= 5) return 'Medium';
    return 'Low';
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">🧠 AI Insights</h1>
        <div className="flex items-center space-x-4">
          <span className={`px-3 py-1 rounded text-sm ${
            aiSettings.learningEnabled ? 'bg-green-600' : 'bg-gray-600'
          }`}>
            {aiSettings.learningEnabled ? '🧠 Learning Active' : '🗥️ Learning Paused'}
          </span>
          <button 
            onClick={loadAIData}
            className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded transition-colors"
            disabled={isLoading}
          >
            {isLoading ? '🔄 Loading...' : '🔄 Refresh'}
          </button>
        </div>
      </div>

      {/* AI Chat Interface */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-blue-400">💬 AI Assistant</h3>
          <p className="text-sm text-gray-400 mt-1">Ask questions about system optimization and management</p>
        </div>
        <div className="p-6">
          <div className="bg-gray-900 rounded-lg h-64 overflow-y-auto mb-4 p-4 space-y-3">
            {chatHistory.length === 0 ? (
              <div className="text-center text-gray-400 py-8">
                <p>👋 Welcome! Ask me anything about your system.</p>
                <p className="text-sm mt-2">Try: "How can I improve gaming performance?" or "What should I backup?"</p>
              </div>
            ) : (
              chatHistory.map((msg, index) => (
                <div key={index} className={`flex ${msg.type === 'user' ? 'justify-end' : 'justify-start'}`}>
                  <div className={`max-w-xs lg:max-w-md px-4 py-2 rounded-lg ${
                    msg.type === 'user' 
                      ? 'bg-blue-600 text-white' 
                      : 'bg-gray-700 text-gray-100'
                  }`}>
                    <p className="text-sm">{msg.message}</p>
                    <p className="text-xs opacity-70 mt-1">
                      {new Date(msg.timestamp).toLocaleTimeString()}
                    </p>
                  </div>
                </div>
              ))
            )}
          </div>
          
          <div className="flex space-x-2">
            <input
              type="text"
              value={chatInput}
              onChange={(e) => setChatInput(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleChatSubmit()}
              placeholder="Ask the AI about system optimization..."
              className="flex-1 bg-gray-700 text-white px-4 py-2 rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
            />
            <button
              onClick={handleChatSubmit}
              disabled={!chatInput.trim()}
              className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 px-6 py-2 rounded transition-colors"
            >
              🗨️ Send
            </button>
          </div>
        </div>
      </div>

      {/* Active Recommendations */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-green-400">💡 Active Recommendations</h3>
          <p className="text-sm text-gray-400 mt-1">AI-generated optimization suggestions</p>
        </div>
        <div className="p-6">
          {recommendations.length > 0 ? (
            <div className="space-y-4">
              {recommendations.map((rec) => (
                <div key={rec.id} className="bg-gray-700 rounded-lg p-4 border border-gray-600">
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex-1">
                      <div className="flex items-center space-x-2 mb-1">
                        <h4 className="font-semibold">{rec.title}</h4>
                        <span className={`px-2 py-1 rounded text-xs ${getPriorityColor(rec.priority)}`}>
                          {getPriorityLabel(rec.priority)}
                        </span>
                        <span className="px-2 py-1 rounded text-xs bg-gray-600">
                          {rec.category}
                        </span>
                      </div>
                      <p className="text-sm text-gray-300 mb-2">{rec.description}</p>
                      <div className="text-xs text-gray-400">
                        {new Date(rec.timestamp * 1000).toLocaleString()}
                      </div>
                    </div>
                    
                    <div className="flex space-x-2 ml-4">
                      <button
                        onClick={() => handleApplyRecommendation(rec.id)}
                        className="bg-green-600 hover:bg-green-700 px-3 py-1 rounded text-sm transition-colors"
                      >
                        ✅ Apply
                      </button>
                      <button
                        onClick={() => handleDismissRecommendation(rec.id)}
                        className="bg-gray-600 hover:bg-gray-700 px-3 py-1 rounded text-sm transition-colors"
                      >
                        ❌ Dismiss
                      </button>
                    </div>
                  </div>
                  
                  {rec.actions.length > 0 && (
                    <div className="border-t border-gray-600 pt-3 mt-3">
                      <h5 className="text-sm font-medium mb-2">Actions:</h5>
                      <ul className="text-sm text-gray-300 space-y-1">
                        {rec.actions.map((action, index) => (
                          <li key={index} className="flex items-center space-x-2">
                            <span className="text-blue-400">•</span>
                            <span>{action}</span>
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                  
                  {rec.auto_apply && (
                    <div className="mt-3 text-xs text-blue-400">
                      🤖 This recommendation will be applied automatically
                    </div>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center text-gray-400 py-8">
              <p>🧘 No active recommendations</p>
              <p className="text-sm mt-2">The AI is monitoring your system and will provide suggestions as needed</p>
            </div>
          )}
        </div>
      </div>

      {/* Performance Trends */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-purple-400">📈 Performance Trends</h3>
          <p className="text-sm text-gray-400 mt-1">AI-detected patterns in system behavior</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div className="bg-gray-700 p-4 rounded">
              <h4 className="font-semibold mb-2 text-blue-400">CPU Trend</h4>
              <div className="flex items-center space-x-3">
                <span className="text-2xl">
                  {performanceTrends.cpu_trend === 'increasing' && '📈'}
                  {performanceTrends.cpu_trend === 'decreasing' && '📉'}
                  {performanceTrends.cpu_trend === 'stable' && '📏'}
                </span>
                <div>
                  <p className="font-medium capitalize">{performanceTrends.cpu_trend || 'stable'}</p>
                  <p className="text-xs text-gray-400">Last 10 samples</p>
                </div>
              </div>
            </div>
            
            <div className="bg-gray-700 p-4 rounded">
              <h4 className="font-semibold mb-2 text-green-400">Memory Trend</h4>
              <div className="flex items-center space-x-3">
                <span className="text-2xl">
                  {performanceTrends.memory_trend === 'increasing' && '📈'}
                  {performanceTrends.memory_trend === 'decreasing' && '📉'}
                  {performanceTrends.memory_trend === 'stable' && '📏'}
                </span>
                <div>
                  <p className="font-medium capitalize">{performanceTrends.memory_trend || 'stable'}</p>
                  <p className="text-xs text-gray-400">Last 10 samples</p>
                </div>
              </div>
            </div>
            
            <div className="bg-gray-700 p-4 rounded">
              <h4 className="font-semibold mb-2 text-red-400">Temperature Trend</h4>
              <div className="flex items-center space-x-3">
                <span className="text-2xl">
                  {performanceTrends.temperature_trend === 'increasing' && '📈'}
                  {performanceTrends.temperature_trend === 'decreasing' && '📉'}
                  {performanceTrends.temperature_trend === 'stable' && '📏'}
                </span>
                <div>
                  <p className="font-medium capitalize">{performanceTrends.temperature_trend || 'stable'}</p>
                  <p className="text-xs text-gray-400">Last 10 samples</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Decision Statistics */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-orange-400">📊 Decision Statistics</h3>
          <p className="text-sm text-gray-400 mt-1">AI decision-making performance and history</p>
        </div>
        <div className="p-6">
          {decisionStats.length > 0 ? (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead className="bg-gray-700">
                  <tr>
                    <th className="px-4 py-3 text-left">Action</th>
                    <th className="px-4 py-3 text-left">Success Rate</th>
                    <th className="px-4 py-3 text-left">Total Attempts</th>
                    <th className="px-4 py-3 text-left">Last Used</th>
                    <th className="px-4 py-3 text-left">Confidence</th>
                  </tr>
                </thead>
                <tbody>
                  {decisionStats.map((stat, index) => (
                    <tr key={stat.action} className={index % 2 === 0 ? 'bg-gray-800' : 'bg-gray-750'}>
                      <td className="px-4 py-2 font-medium capitalize">{stat.action.replace('_', ' ')}</td>
                      <td className="px-4 py-2">
                        <div className="flex items-center space-x-2">
                          <div className="w-16 bg-gray-600 rounded-full h-2">
                            <div 
                              className={`h-2 rounded-full ${
                                stat.success_rate > 80 ? 'bg-green-500' :
                                stat.success_rate > 60 ? 'bg-yellow-500' : 'bg-red-500'
                              }`}
                              style={{ width: `${stat.success_rate}%` }}
                            ></div>
                          </div>
                          <span className="font-mono text-xs">{stat.success_rate.toFixed(0)}%</span>
                        </div>
                      </td>
                      <td className="px-4 py-2 font-mono">{stat.total_attempts}</td>
                      <td className="px-4 py-2 text-gray-400">
                        {new Date(stat.last_used).toLocaleDateString()}
                      </td>
                      <td className="px-4 py-2">
                        <span className={`px-2 py-1 rounded text-xs ${
                          stat.success_rate > 80 ? 'bg-green-600' :
                          stat.success_rate > 60 ? 'bg-yellow-600' : 'bg-red-600'
                        }`}>
                          {stat.success_rate > 80 ? 'High' : stat.success_rate > 60 ? 'Medium' : 'Low'}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="text-center text-gray-400 py-8">
              <p>No decision history available yet</p>
              <p className="text-sm mt-2">Statistics will appear as the AI makes decisions</p>
            </div>
          )}
        </div>
      </div>

      {/* AI Configuration */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-cyan-400">⚙️ AI Configuration</h3>
          <p className="text-sm text-gray-400 mt-1">Configure AI behavior and learning parameters</p>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="font-medium">Machine Learning</span>
                <button
                  onClick={() => setAiSettings(prev => ({ ...prev, learningEnabled: !prev.learningEnabled }))}
                  className={`px-4 py-2 rounded transition-colors ${
                    aiSettings.learningEnabled 
                      ? 'bg-green-600 hover:bg-green-700' 
                      : 'bg-gray-600 hover:bg-gray-700'
                  }`}
                >
                  {aiSettings.learningEnabled ? '🧠 Enabled' : '🗥️ Disabled'}
                </button>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="font-medium">Auto-Optimization</span>
                <button
                  onClick={() => setAiSettings(prev => ({ ...prev, autoOptimize: !prev.autoOptimize }))}
                  className={`px-4 py-2 rounded transition-colors ${
                    aiSettings.autoOptimize 
                      ? 'bg-green-600 hover:bg-green-700' 
                      : 'bg-gray-600 hover:bg-gray-700'
                  }`}
                >
                  {aiSettings.autoOptimize ? '🤖 Enabled' : '✋ Manual'}
                </button>
              </div>
              
              <div>
                <label className="block text-sm font-medium mb-2">Confidence Threshold</label>
                <input
                  type="range"
                  min="0.1"
                  max="1.0"
                  step="0.05"
                  value={aiSettings.confidenceThreshold}
                  onChange={(e) => setAiSettings(prev => ({ ...prev, confidenceThreshold: parseFloat(e.target.value) }))}
                  className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer"
                />
                <div className="flex justify-between text-xs text-gray-400 mt-1">
                  <span>10%</span>
                  <span className="font-mono">{(aiSettings.confidenceThreshold * 100).toFixed(0)}%</span>
                  <span>100%</span>
                </div>
              </div>
              
              <div>
                <label className="block text-sm font-medium mb-2">Sensitivity Level</label>
                <input
                  type="range"
                  min="1"
                  max="10"
                  step="0.5"
                  value={aiSettings.sensitivityLevel}
                  onChange={(e) => setAiSettings(prev => ({ ...prev, sensitivityLevel: parseFloat(e.target.value) }))}
                  className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer"
                />
                <div className="flex justify-between text-xs text-gray-400 mt-1">
                  <span>Conservative</span>
                  <span className="font-mono">{aiSettings.sensitivityLevel.toFixed(1)}</span>
                  <span>Aggressive</span>
                </div>
              </div>
            </div>
            
            <div className="bg-gray-700 p-4 rounded">
              <h4 className="font-semibold mb-3 text-yellow-400">AI Status</h4>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span>Learning:</span>
                  <span className={aiSettings.learningEnabled ? 'text-green-400' : 'text-red-400'}>
                    {aiSettings.learningEnabled ? '✅ Active' : '❌ Disabled'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Recommendations:</span>
                  <span className="text-blue-400">{recommendations.length} active</span>
                </div>
                <div className="flex justify-between">
                  <span>Decision History:</span>
                  <span className="text-purple-400">{decisionStats.length} actions</span>
                </div>
                <div className="flex justify-between">
                  <span>Confidence Level:</span>
                  <span className="text-cyan-400">{(aiSettings.confidenceThreshold * 100).toFixed(0)}%</span>
                </div>
                <div className="flex justify-between">
                  <span>Sensitivity:</span>
                  <span className="text-orange-400">{aiSettings.sensitivityLevel.toFixed(1)}/10</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* System Insights */}
      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-yellow-400">🔍 System Insights</h3>
          <p className="text-sm text-gray-400 mt-1">Discovered patterns and behavioral analysis</p>
        </div>
        <div className="p-6">
          {insights.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {insights.map((insight, index) => (
                <div key={index} className="bg-gray-700 p-4 rounded border border-gray-600">
                  <div className="flex items-start justify-between mb-2">
                    <h4 className="font-semibold text-sm">{insight.pattern}</h4>
                    <span className={`px-2 py-1 rounded text-xs ${
                      insight.confidence > 80 ? 'bg-green-600' :
                      insight.confidence > 60 ? 'bg-yellow-600' : 'bg-red-600'
                    }`}>
                      {insight.confidence.toFixed(0)}% confidence
                    </span>
                  </div>
                  <p className="text-sm text-gray-300 mb-2">{insight.recommendation}</p>
                  <div className="flex justify-between items-center text-xs text-gray-400">
                    <span>Priority: {insight.priority}/10</span>
                    <span>{insight.timestamp}</span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center text-gray-400 py-8">
              <p>🔍 No insights detected yet</p>
              <p className="text-sm mt-2">The AI will learn your usage patterns over time</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
