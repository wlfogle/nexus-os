import React, { useState, useEffect } from 'react';
import { useSelector } from 'react-redux';
import { ChevronDown, Eye, Brain, Folder, Settings } from 'lucide-react';
import { selectActiveTab } from '../../store/slices/terminalTabSlice';
import { cn } from '../../lib/utils';

interface AIModel {
  name: string;
  size: string;
  type: 'vision' | 'text' | 'code';
  description: string;
  capabilities: string[];
}

interface AIControlBarProps {
  className?: string;
}

const AIControlBar: React.FC<AIControlBarProps> = ({ className }) => {
  const activeTab = useSelector(selectActiveTab);
  const [availableModels, setAvailableModels] = useState<AIModel[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [showModelPicker, setShowModelPicker] = useState(false);
  const [currentDirectory, setCurrentDirectory] = useState<string>('');
  const [isExpanded, setIsExpanded] = useState(false);

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
        console.error('Failed to load models:', error);
      }
    };
    
    loadModels();
  }, []);

  // Update current directory from active tab
  useEffect(() => {
    if (activeTab?.workingDirectory) {
      setCurrentDirectory(activeTab.workingDirectory);
    }
  }, [activeTab?.workingDirectory]);

  const handleDirectoryPicker = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: activeTab?.workingDirectory,
        title: 'Select Codebase Directory for AI Analysis'
      });
      
      if (selected && typeof selected === 'string') {
        setCurrentDirectory(selected);
        // You could dispatch an action here to update the active tab's working directory
        // or trigger a re-indexing of the codebase
        console.log('Selected directory:', selected);
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  };

  return (
    <div className={cn('bg-gray-800 border-b border-gray-700 px-4 py-2', className)}>
      <div className="flex items-center justify-between">
        {/* Left side - AI Model Info */}
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <Brain className="w-4 h-4 text-blue-400" />
            <span className="text-sm font-medium text-gray-200">AI Model:</span>
            
            {/* Model Picker */}
            <div className="relative">
              <button
                onClick={() => setShowModelPicker(!showModelPicker)}
                className="flex items-center justify-between px-3 py-1 text-sm bg-gray-700 border border-gray-600 rounded-lg hover:bg-gray-600 transition-colors min-w-48"
              >
                <div className="flex items-center gap-2">
                  <div className={cn(
                    'w-2 h-2 rounded-full',
                    selectedModel.includes('vision') ? 'bg-purple-500' :
                    selectedModel.includes('code') ? 'bg-green-500' : 'bg-blue-500'
                  )} />
                  <span className="font-mono text-xs text-white truncate">
                    {selectedModel || 'Select model...'}
                  </span>
                  {selectedModel && (
                    <span className="text-xs bg-gray-600 px-1.5 py-0.5 rounded">
                      {availableModels.find(m => m.name === selectedModel)?.size || ''}
                    </span>
                  )}
                </div>
                <ChevronDown className="w-4 h-4 ml-2 flex-shrink-0" />
              </button>
              
              {showModelPicker && (
                <div className="absolute z-20 w-80 mt-1 bg-gray-800 border border-gray-600 rounded-lg shadow-xl max-h-60 overflow-y-auto">
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
                          console.error('Failed to set model:', error);
                        }
                      }}
                      className={cn(
                        'w-full px-3 py-2 text-left text-sm hover:bg-gray-700 transition-colors',
                        selectedModel === model.name && 'bg-blue-50 dark:bg-blue-900'
                      )}
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <div className={cn(
                          'w-2 h-2 rounded-full',
                          model.type === 'vision' ? 'bg-purple-500' :
                          model.type === 'code' ? 'bg-green-500' : 'bg-blue-500'
                        )} />
                        <span className="font-mono text-xs font-medium text-white">{model.name}</span>
                        <span className="text-xs bg-gray-600 px-1.5 py-0.5 rounded">
                          {model.size}
                        </span>
                        {model.type === 'vision' && (
                          <Eye className="w-3 h-3 text-purple-400" />
                        )}
                      </div>
                      <div className="text-xs text-gray-400 mb-1">
                        {model.description}
                      </div>
                      <div className="flex flex-wrap gap-1">
                        {model.capabilities.map(cap => (
                          <span 
                            key={cap}
                            className="text-xs bg-gray-700 text-gray-300 px-1 py-0.5 rounded"
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
        </div>

        {/* Center - Directory Info */}
        <div className="flex items-center space-x-2">
          <Folder className="w-4 h-4 text-amber-400" />
          <span className="text-sm text-gray-300">Directory:</span>
          <button
            onClick={handleDirectoryPicker}
            className="flex items-center gap-2 px-3 py-1 text-sm bg-gray-700 border border-gray-600 rounded-lg hover:bg-gray-600 transition-colors max-w-64"
          >
            <span className="font-mono text-xs text-white truncate">
              {currentDirectory || '/home/user'}
            </span>
            <ChevronDown className="w-3 h-3 flex-shrink-0" />
          </button>
        </div>

        {/* Right side - Toggle Settings */}
        <div className="flex items-center space-x-2">
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="flex items-center gap-2 px-2 py-1 text-sm text-gray-400 hover:text-gray-200 transition-colors"
          >
            <Settings className="w-4 h-4" />
            <ChevronDown className={cn('w-3 h-3 transition-transform', isExpanded && 'rotate-180')} />
          </button>
        </div>
      </div>

      {/* Expanded Settings */}
      {isExpanded && (
        <div className="mt-3 pt-3 border-t border-gray-700">
          <div className="grid grid-cols-3 gap-4 text-xs">
            <div>
              <label className="block text-gray-400 mb-1">AI Features:</label>
              <div className="space-y-1">
                <label className="flex items-center gap-2">
                  <input type="checkbox" defaultChecked className="rounded" />
                  <span>RAG Context</span>
                </label>
                <label className="flex items-center gap-2">
                  <input type="checkbox" className="rounded" />
                  <span>Screen Context</span>
                </label>
              </div>
            </div>
            <div>
              <label className="block text-gray-400 mb-1">Command Routing:</label>
              <div className="space-y-1">
                <label className="flex items-center gap-2">
                  <input type="checkbox" defaultChecked className="rounded" />
                  <span>Smart Detection</span>
                </label>
                <label className="flex items-center gap-2">
                  <input type="checkbox" className="rounded" />
                  <span>Always Ask AI</span>
                </label>
              </div>
            </div>
            <div>
              <label className="block text-gray-400 mb-1">Response Speed:</label>
              <select className="w-full px-2 py-1 bg-gray-700 border border-gray-600 rounded text-xs">
                <option value="fast">Fast (Minimal Context)</option>
                <option value="balanced">Balanced</option>
                <option value="comprehensive">Comprehensive</option>
              </select>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AIControlBar;
