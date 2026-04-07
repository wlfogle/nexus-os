import React, { useState, useEffect, useRef } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { Eye, Search, Brain, Camera, BookOpen, Zap, AlertTriangle, CheckCircle, ChevronDown } from 'lucide-react';
import { selectActiveTab, addAIMessage } from '../../store/slices/terminalTabSlice';
import { ragService } from '../../services/ragService';
import { visionService, ScreenAnalysis } from '../../services/visionService';
import { useInputRouting } from '../../hooks/useInputRouting';
import { cn } from '../../lib/utils';
import { aiLogger } from '../../utils/logger';
import './ai-scrollbars.css';

interface EnhancedAIAssistantProps {
  className?: string;
}

interface AICapability {
  id: string;
  name: string;
  description: string;
  icon: React.ElementType;
  enabled: boolean;
  status: 'ready' | 'loading' | 'error' | 'disabled';
}

interface RAGContext {
  query: string;
  results: Array<{
    content: string;
    similarity: number;
    type: string;
    path?: string;
  }>;
  contextUsed: boolean;
}

interface VisionContext {
  hasScreenshot: boolean;
  analysis?: ScreenAnalysis;
  contextUsed: boolean;
}

interface AIModel {
  name: string;
  size: string;
  type: 'vision' | 'text' | 'code';
  description: string;
  capabilities: string[];
}

const EnhancedAIAssistant: React.FC<EnhancedAIAssistantProps> = ({ className }) => {
  const dispatch = useDispatch();
  const activeTab = useSelector(selectActiveTab);
  const [message, setMessage] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const { handleInput } = useInputRouting();
  const [capabilities, setCapabilities] = useState<AICapability[]>([
    {
      id: 'rag',
      name: 'RAG Knowledge',
      description: 'Semantic search across codebase and history',
      icon: Brain,
      enabled: false,
      status: 'loading'
    },
    {
      id: 'vision',
      name: 'Computer Vision',
      description: 'See and understand your screen',
      icon: Eye,
      enabled: false,
      status: 'loading'
    },
    {
      id: 'proactive',
      name: 'Proactive Assistant',
      description: 'Automatic suggestions based on context',
      icon: Zap,
      enabled: true,
      status: 'ready'
    },
    {
      id: 'documentation',
      name: 'Smart Documentation',
      description: 'Context-aware help and documentation',
      icon: BookOpen,
      enabled: true,
      status: 'ready'
    }
  ]);
  
  const [ragContext] = useState<RAGContext>({
    query: '', 
    results: [], 
    contextUsed: false 
  });
  
  const [visionContext, setVisionContext] = useState<VisionContext>({ 
    hasScreenshot: false, 
    contextUsed: false 
  });
  
  const [showAdvancedOptions, setShowAdvancedOptions] = useState(false);
  const [useScreenContext, setUseScreenContext] = useState(false);
  const [useRAGContext, setUseRAGContext] = useState(true);
  const [availableModels, setAvailableModels] = useState<AIModel[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [showModelPicker, setShowModelPicker] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Component initialization
  useEffect(() => {
    aiLogger.debug('EnhancedAIAssistant component mounted', 'mount', { tabId: activeTab?.id });
  }, []);

  // Load available models
  useEffect(() => {
    const loadModels = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const models = await invoke('get_available_models') as string[];
        
        // Parse model names and categorize them
        const parsedModels: AIModel[] = models.map(modelName => {
          const isVision = modelName.includes('vision') || modelName.includes('llava') || modelName.includes('moondream');
          const isCode = modelName.includes('code') || modelName.includes('coder') || modelName.includes('codellama');
          const size = modelName.includes('90b') ? '90B' : 
                      modelName.includes('70b') ? '70B' :
                      modelName.includes('32b') ? '32B' :
                      modelName.includes('13b') ? '13B' :
                      modelName.includes('11b') ? '11B' :
                      modelName.includes('7b') ? '7B' :
                      modelName.includes('3b') ? '3B' :
                      modelName.includes('1b') ? '1B' : 'Unknown';
          
          const capabilities = [];
          if (isVision) capabilities.push('Vision', 'Image Analysis');
          if (isCode) capabilities.push('Code Generation', 'Debugging');
          capabilities.push('Text Generation', 'Conversation');
          
          return {
            name: modelName,
            size,
            type: isVision ? 'vision' : isCode ? 'code' : 'text',
            description: isVision ? 'Vision-capable model for image analysis' :
                        isCode ? 'Specialized for code generation and analysis' :
                        'General purpose language model',
            capabilities
          };
        });
        
        setAvailableModels(parsedModels);
        
        // Set default model
        const currentModel = await invoke('get_current_model') as string;
        setSelectedModel(currentModel);
        
      } catch (error) {
        aiLogger.error('Failed to load models', error as Error, 'load_models', { availableCount: availableModels.length });
      }
    };
    
    loadModels();
  }, []);
  
  // Initialize AI capabilities
  useEffect(() => {
    const initializeCapabilities = async () => {
      // Initialize RAG service
      try {
        await ragService.initialize();
        setCapabilities(prev => prev.map(cap => 
          cap.id === 'rag' ? { ...cap, enabled: true, status: 'ready' } : cap
        ));
        
        // Index current project if we have an active tab
        if (activeTab?.workingDirectory) {
          await ragService.indexCodebase(activeTab.workingDirectory);
        }
      } catch (error) {
        aiLogger.warn('RAG service initialization failed', error as Error, 'rag_init_failed');
        setCapabilities(prev => prev.map(cap => 
          cap.id === 'rag' ? { ...cap, status: 'error' } : cap
        ));
      }
      
      // Initialize Vision service
      try {
        await visionService.initialize();
        setCapabilities(prev => prev.map(cap => 
          cap.id === 'vision' ? { ...cap, enabled: true, status: 'ready' } : cap
        ));
      } catch (error) {
        aiLogger.warn('Vision service initialization failed', error as Error, 'vision_init_failed');
        setCapabilities(prev => prev.map(cap => 
          cap.id === 'vision' ? { ...cap, status: 'error' } : cap
        ));
      }
    };
    
    initializeCapabilities();
  }, [activeTab?.workingDirectory]);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [activeTab?.aiConversation.length]);

  // Handle proactive suggestions based on screen changes
  useEffect(() => {
    if (!capabilities.find(c => c.id === 'vision')?.enabled) return;
    
    const handleScreenChange = (analysis: ScreenAnalysis) => {
      // Auto-generate suggestions for errors
      if (analysis.detectedContext.errorMessages && 
          analysis.detectedContext.errorMessages.length > 0) {
        handleProactiveSuggestion(
          `I noticed error messages on your screen: "${analysis.detectedContext.errorMessages.join(', ')}". Would you like help resolving these?`,
          { analysis, type: 'error_detection' }
        );
      }
      
      // Suggest help for new terminal commands
      if (analysis.detectedContext.windowType === 'terminal' && 
          analysis.detectedContext.terminalCommands) {
        const recentCommand = analysis.detectedContext.terminalCommands[analysis.detectedContext.terminalCommands.length - 1];
        if (recentCommand && (recentCommand.includes('error') || recentCommand.includes('failed'))) {
          handleProactiveSuggestion(
            `I see you might be having issues with "${recentCommand}". Need assistance?`,
            { analysis, type: 'command_assistance' }
          );
        }
      }
    };
    
    // Start screen monitoring for proactive assistance
    visionService.startScreenMonitoring(handleScreenChange, 10000);
  }, [capabilities]);

  const handleProactiveSuggestion = async (suggestion: string, context: any) => {
    if (!activeTab) return;
    
    dispatch(addAIMessage({
      tabId: activeTab.id,
      message: {
        role: 'assistant',
        content: `🔍 **Proactive Suggestion**: ${suggestion}`,
        timestamp: new Date(),
        metadata: {
          type: 'proactive_suggestion',
          context,
          capabilities_used: ['vision', 'proactive']
        }
      }
    }));
  };

  const captureScreen = async () => {
    aiLogger.info('Screen capture initiated', 'capture_screen');
    try {
      const capture = await visionService.captureScreen();
      aiLogger.info('Screen captured successfully', 'capture_complete', { captureId: capture.id });
      
      const analysis = await visionService.analyzeScreen(capture.id);
      aiLogger.info('Screen analysis complete', 'analysis_complete', { summary: analysis.summary });
      
      setVisionContext({
        hasScreenshot: true,
        analysis,
        contextUsed: false
      });
      
      aiLogger.debug('Vision context updated', 'context_update', { hasScreenshot: true });
      return analysis;
    } catch (error) {
      aiLogger.error('Screen capture failed', error as Error, 'capture_failed');
      // Show error to user
      alert(`Screen capture failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      throw error;
    }
  };

  /* const performRAGSearch = async (query: string) => {
    try {
      const results = await ragService.search({
        query,
        maxResults: 5,
        threshold: 0.6
      });
      
      setRagContext({
        query,
        results: results.map(r => ({
          content: r.document.content,
          similarity: r.similarity,
          type: r.document.metadata.type,
          path: r.document.metadata.path
        })),
        contextUsed: false
      });
      
      return results;
    } catch (error) {
      aiLogger.error('RAG search failed', error as Error, 'rag_search_failed', { query });
      return [];
    }
  }; */

  const handleSendMessage = async () => {
    aiLogger.info('Send message initiated', 'send_message', { messageLength: message.length });
    
    if (!message.trim() || !activeTab) {
      aiLogger.debug('Send message aborted - empty message or no active tab', 'send_abort', { hasMessage: !!message.trim(), hasTab: !!activeTab });
      return;
    }
    
    const userMessage = message;
    aiLogger.debug('Processing message', 'process_message', { messageLength: userMessage.length });
    setMessage('');
    
    // Use central input routing - it handles both shell commands and AI queries
    setIsLoading(true);
    try {
      aiLogger.debug('Routing message to input handler', 'route_input', { message: userMessage });
      await handleInput(userMessage, () => {
        aiLogger.debug('Input handling completed', 'input_complete');
        setIsLoading(false);
      });
    } finally {
      setIsLoading(false);
    }
  };

  const getCapabilityIcon = (capability: AICapability) => {
    const Icon = capability.icon;
    const statusColors = {
      ready: 'text-green-500',
      loading: 'text-yellow-500 animate-pulse',
      error: 'text-red-500',
      disabled: 'text-gray-400'
    };
    
    return <Icon className={cn('w-4 h-4', statusColors[capability.status])} />;
  };

  const getCapabilityStatusIcon = (status: AICapability['status']) => {
    switch (status) {
      case 'ready':
        return <CheckCircle className="w-3 h-3 text-green-500" />;
      case 'loading':
        return <div className="w-3 h-3 rounded-full border-2 border-yellow-500 border-t-transparent animate-spin" />;
      case 'error':
        return <AlertTriangle className="w-3 h-3 text-red-500" />;
      default:
        return <div className="w-3 h-3 rounded-full bg-gray-400" />;
    }
  };

  if (!activeTab) {
    return (
      <div className={cn('flex items-center justify-center h-full text-gray-500', className)}>
        <div className="text-center">
          <Brain className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>No active terminal tab</p>
          <p className="text-sm">Create a tab to start using AI assistance</p>
        </div>
      </div>
    );
  }

  return (
    <div className={cn('flex flex-col h-full bg-gray-50 dark:bg-gray-900', className)}>
      {/* AI Capabilities Header */}
      <div className="flex-shrink-0 p-4 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-3">
          <h3 className="font-semibold text-gray-900 dark:text-gray-100 flex items-center gap-2">
            <Brain className="w-5 h-5 text-blue-500" />
            Enhanced AI Assistant
          </h3>
          <button
            onClick={() => setShowAdvancedOptions(!showAdvancedOptions)}
            className="text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400"
          >
            {showAdvancedOptions ? 'Hide' : 'Show'} Options
          </button>
        </div>
        
        {/* Capabilities Status & Quick Actions */}
        <div className="grid grid-cols-2 gap-2 mb-3">
          {capabilities.map(capability => (
            <div key={capability.id} className="flex items-center gap-2 text-sm">
              {getCapabilityIcon(capability)}
              <span className={cn(
                'flex-1 text-xs',
                capability.enabled ? 'text-gray-700 dark:text-gray-300' : 'text-gray-400'
              )}>
                {capability.name}
              </span>
              {getCapabilityStatusIcon(capability.status)}
            </div>
          ))}
        </div>

        {/* Main Controls - Model Picker & Screen Capture */}
        <div className="space-y-3 mb-3">
          {/* AI Model Selection - Always Visible */}
          <div className="space-y-2">
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300 flex items-center gap-2">
              <Brain className="w-4 h-4 text-blue-500" />
              AI Model:
            </label>
            <div className="relative">
              <button
                onClick={() => setShowModelPicker(!showModelPicker)}
                className="w-full flex items-center justify-between px-3 py-2 text-sm bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
              >
                <div className="flex items-center gap-2">
                  <div className={cn(
                    'w-2 h-2 rounded-full',
                    selectedModel.includes('vision') ? 'bg-purple-500' :
                    selectedModel.includes('code') ? 'bg-green-500' : 'bg-blue-500'
                  )} />
                  <span className="font-mono text-xs font-medium">
                    {selectedModel || 'llava:7b (Vision)'}
                  </span>
                  {selectedModel.includes('vision') && (
                    <Eye className="w-3 h-3 text-purple-500" />
                  )}
                </div>
                <ChevronDown className="w-4 h-4" />
              </button>
              
              {showModelPicker && (
                <div className="absolute z-10 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600 rounded-lg shadow-lg max-h-60 overflow-y-auto">
                  {availableModels.map(model => (
                    <button
                      key={model.name}
                      onClick={async () => {
                        try {
                          const { invoke } = await import('@tauri-apps/api/core');
                          await invoke('set_ai_model', { model: model.name });
                          setSelectedModel(model.name);
                          setShowModelPicker(false);
                        } catch (error) {
                          aiLogger.error('Failed to set model', error as Error, 'model_set_failed', { modelName: model.name });
                        }
                      }}
                      className={cn(
                        'w-full px-3 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors',
                        selectedModel === model.name && 'bg-blue-50 dark:bg-blue-900'
                      )}
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <div className={cn(
                          'w-2 h-2 rounded-full',
                          model.type === 'vision' ? 'bg-purple-500' :
                          model.type === 'code' ? 'bg-green-500' : 'bg-blue-500'
                        )} />
                        <span className="font-mono text-xs font-medium">{model.name}</span>
                        <span className="text-xs bg-gray-200 dark:bg-gray-600 px-1.5 py-0.5 rounded">
                          {model.size}
                        </span>
                        {model.type === 'vision' && (
                          <Eye className="w-3 h-3 text-purple-500" />
                        )}
                      </div>
                      <div className="text-xs text-gray-600 dark:text-gray-400 mb-1">
                        {model.description}
                      </div>
                      <div className="flex flex-wrap gap-1">
                        {model.capabilities.map(cap => (
                          <span 
                            key={cap}
                            className="text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 px-1 py-0.5 rounded"
                          >
                            {cap}
                          </span>
                        ))}
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Quick Actions */}
          <div className="flex gap-2">
            {capabilities.find(c => c.id === 'vision')?.enabled && (
              <button
                onClick={captureScreen}
                className="flex-1 flex items-center justify-center gap-2 px-3 py-2 text-sm bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition-colors"
              >
                <Camera className="w-4 h-4" />
                Capture Screen
              </button>
            )}
            <button
              onClick={() => setShowAdvancedOptions(!showAdvancedOptions)}
              className="px-3 py-2 text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400 border border-blue-200 dark:border-blue-600 rounded-lg transition-colors"
            >
              {showAdvancedOptions ? 'Less' : 'More'}
            </button>
          </div>
        </div>
        
        {/* Advanced Options */}
        {showAdvancedOptions && (
          <div className="space-y-3 pt-3 border-t border-gray-200 dark:border-gray-600">
            
            {/* Context Options */}
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={useRAGContext}
                onChange={(e) => setUseRAGContext(e.target.checked)}
                disabled={!capabilities.find(c => c.id === 'rag')?.enabled}
                className="rounded"
              />
              <span>Use RAG context from codebase</span>
            </label>
            
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={useScreenContext}
                onChange={(e) => setUseScreenContext(e.target.checked)}
                disabled={!capabilities.find(c => c.id === 'vision')?.enabled}
                className="rounded"
              />
              <span>Include screen context</span>
            </label>
            
            {/* Codebase Directory Selection */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Codebase Directory:</label>
              <div className="flex gap-2">
                <div className="flex-1 px-3 py-2 text-sm bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg font-mono text-xs">
                  {activeTab?.workingDirectory || '/home/user'}
                </div>
                <button
                  onClick={async () => {
                    try {
                      const { open } = await import('@tauri-apps/plugin-dialog');
                      const selected = await open({
                        directory: true,
                        multiple: false,
                        defaultPath: activeTab?.workingDirectory,
                        title: 'Select Codebase Directory for AI Analysis'
                      });
                      
                      if (selected && typeof selected === 'string') {
                        // Re-index the new directory
                        if (capabilities.find(c => c.id === 'rag')?.enabled) {
                          setCapabilities(prev => prev.map(cap => 
                            cap.id === 'rag' ? { ...cap, status: 'loading' } : cap
                          ));
                          
                          try {
                            await ragService.indexCodebase(selected);
                            setCapabilities(prev => prev.map(cap => 
                              cap.id === 'rag' ? { ...cap, status: 'ready' } : cap
                            ));
                            aiLogger.info('Codebase indexed successfully', 'index_success', { path: selected });
                          } catch (error) {
                            aiLogger.error('Failed to index codebase', error as Error, 'index_failed', { path: selected });
                            setCapabilities(prev => prev.map(cap => 
                              cap.id === 'rag' ? { ...cap, status: 'error' } : cap
                            ));
                          }
                        }
                      }
                    } catch (error) {
                      aiLogger.error('Failed to select directory', error as Error, 'directory_select_failed');
                    }
                  }}
                  className="px-3 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
                >
                  Browse
                </button>
              </div>
              <div className="text-xs text-gray-500">
                🧠 AI will analyze and search through files in this directory
              </div>
            </div>
            
            {/* Vision Actions */}
            {capabilities.find(c => c.id === 'vision')?.enabled && (
              <button
                onClick={captureScreen}
                className="flex items-center gap-2 text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400"
              >
                <Camera className="w-4 h-4" />
                Capture Screen Now
              </button>
            )}
          </div>
        )}
        
        {/* Context Indicators */}
        <div className="flex gap-2 mt-2">
          {ragContext.results.length > 0 && (
            <div className="flex items-center gap-1 text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 px-2 py-1 rounded">
              <Search className="w-3 h-3" />
              {ragContext.results.length} RAG results
            </div>
          )}
          
          {visionContext.hasScreenshot && (
            <div className="flex items-center gap-1 text-xs bg-purple-100 dark:bg-purple-900 text-purple-700 dark:text-purple-300 px-2 py-1 rounded">
              <Eye className="w-3 h-3" />
              Screen captured
            </div>
          )}
        </div>
      </div>
      
      {/* CONVERSATION DISPLAY WITH NATIVE SCROLLING */}
      <div className="flex-1 bg-gray-800 border-2 border-blue-400 flex flex-col">
        {/* Conversation Header */}
        <div className="bg-gray-700 p-3 border-b border-gray-600 flex-shrink-0">
          <div className="flex items-center justify-between">
            <div className="text-white text-sm">
              🗨️ Conversation
            </div>
          </div>
        </div>
        
        {/* Messages Display Area - With Native Scrolling */}
        <div className="p-4 flex-1 overflow-y-auto" style={{ WebkitOverflowScrolling: 'touch' }}>
          {activeTab.aiConversation.length === 0 ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center text-gray-400">
                <Brain className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>No conversation yet</p>
                <p className="text-sm">Start chatting with AI below!</p>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              {activeTab.aiConversation.map((msg, index) => (
                <div
                  key={index}
                  className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
                >
                  <div
                    className={`max-w-[80%] p-3 rounded-lg text-white ${
                      msg.role === 'user' ? 'bg-blue-600' : 'bg-gray-600'
                    }`}
                  >
                    <div className="text-xs opacity-70 mb-1">
                      {msg.role === 'user' ? '👤 You' : '🤖 AI'} • {new Date(msg.timestamp).toLocaleTimeString()}
                    </div>
                    <div className="whitespace-pre-wrap">{msg.content}</div>
                  </div>
                </div>
              ))}
              
              {isLoading && (
                <div className="flex justify-start">
                  <div className="p-3 rounded-lg bg-gray-600 text-white">
                    🤖 AI is thinking...
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </div>
          )}
        </div>
      </div>
      
      {/* Input Area */}
      <div className="flex-shrink-0 p-4 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
        <div className="flex gap-2">
          <input
            type="text"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                aiLogger.debug('Enter key pressed, sending message', 'keyboard_send', { messageLength: message.length });
                e.preventDefault();
                handleSendMessage();
              }
            }}
            placeholder="Ask me anything about your code, terminal, or what you see on screen..."
            className="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100"
            disabled={isLoading}
          />
          <button
            onClick={() => {
              aiLogger.debug('Send button clicked', 'button_send', { messageLength: message.length });
              handleSendMessage();
            }}
            disabled={isLoading || !message.trim()}
            className="px-6 py-2 bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white rounded-lg font-medium transition-colors"
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
};

export default EnhancedAIAssistant;
