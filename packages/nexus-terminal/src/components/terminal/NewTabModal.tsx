import React, { useState, useCallback, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { 
  XMarkIcon,
  FolderIcon,
  CommandLineIcon,
  SparklesIcon,
  ChevronRightIcon
} from '@heroicons/react/24/outline';
import { 
  setCreatingTab,
  selectAllTabs
} from '../../store/slices/terminalTabSlice';
import { 
  ShellType, 
  SHELL_CONFIGS, 
  DEFAULT_TAB_PRESETS, 
  NewTabConfig,
  TabPreset 
} from '../../types/terminal';
import { cn } from '../../lib/utils';
import { open } from '@tauri-apps/plugin-dialog';

interface NewTabModalProps {
  onCreateTab: (config: NewTabConfig) => void;
  onClose: () => void;
  isOpen?: boolean;
}

export const NewTabModal: React.FC<NewTabModalProps> = ({ onCreateTab, onClose }) => {
  const dispatch = useDispatch();
  const tabs = useSelector(selectAllTabs);
  
  const [selectedShell, setSelectedShell] = useState<ShellType>(ShellType.BASH);
  const [workingDirectory, setWorkingDirectory] = useState('~');
  const [title, setTitle] = useState('');
  const [selectedPreset, setSelectedPreset] = useState<TabPreset | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [environmentVars, setEnvironmentVars] = useState<Record<string, string>>({});

  // Reset form when modal opens
  useEffect(() => {
    setSelectedShell(ShellType.BASH);
    setWorkingDirectory('~');
    setTitle('');
    setSelectedPreset(null);
    setShowAdvanced(false);
    setEnvironmentVars({});
  }, []);

  const handlePresetClick = useCallback((preset: TabPreset) => {
    setSelectedPreset(preset);
    setSelectedShell(preset.shell);
    setWorkingDirectory(preset.workingDirectory);
    setTitle(preset.name);
    setEnvironmentVars(preset.environmentVars);
  }, []);

  const handleShellChange = useCallback((shell: ShellType) => {
    setSelectedShell(shell);
    setSelectedPreset(null); // Clear preset when manually changing shell
  }, []);

  const handleCreateTab = useCallback(() => {
    const config: NewTabConfig = {
      shell: selectedShell,
      workingDirectory,
      title: title.trim() || undefined,
      environmentVars: Object.keys(environmentVars).length > 0 ? environmentVars : undefined,
      preset: selectedPreset || undefined
    };

    onCreateTab(config);
    onClose();
  }, [onCreateTab, selectedShell, workingDirectory, title, environmentVars, selectedPreset, onClose]);

  const handleClose = useCallback(() => {
    dispatch(setCreatingTab(false));
    onClose();
  }, [dispatch, onClose]);

  const addEnvironmentVar = useCallback(() => {
    const key = `VAR_${Object.keys(environmentVars).length + 1}`;
    setEnvironmentVars(prev => ({ ...prev, [key]: '' }));
  }, [environmentVars]);

  const updateEnvironmentVar = useCallback((oldKey: string, newKey: string, value: string) => {
    setEnvironmentVars(prev => {
      const newVars = { ...prev };
      if (oldKey !== newKey) {
        delete newVars[oldKey];
      }
      newVars[newKey] = value;
      return newVars;
    });
  }, []);

  const removeEnvironmentVar = useCallback((key: string) => {
    setEnvironmentVars(prev => {
      const newVars = { ...prev };
      delete newVars[key];
      return newVars;
    });
  }, []);

  const handleBrowseDirectory = useCallback(async () => {
    try {
      // Check if we're in Tauri context
      const isTauriContext = typeof window !== 'undefined' && (window as any).__TAURI__;
      
      if (isTauriContext) {
        const selected = await open({
          directory: true,
          multiple: false,
          defaultPath: workingDirectory === '~' ? undefined : workingDirectory,
          title: 'Select Working Directory'
        });
        
        if (selected && typeof selected === 'string') {
          setWorkingDirectory(selected);
          setSelectedPreset(null); // Clear preset when manually selecting directory
        }
      } else {
        // Browser fallback - prompt for directory path
        const userPath = prompt('Enter working directory path:', workingDirectory);
        if (userPath && userPath.trim()) {
          setWorkingDirectory(userPath.trim());
          setSelectedPreset(null);
        }
      }
    } catch (error) {
      console.error('Failed to open directory dialog:', error);
    }
  }, [workingDirectory]);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-2xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <div>
            <h2 className="text-xl font-semibold text-white flex items-center">
              <CommandLineIcon className="w-6 h-6 mr-2" />
              Create New Terminal Tab
            </h2>
            <p className="text-sm text-gray-400 mt-1">
              Choose a shell and configure your new terminal session
            </p>
          </div>
          <button
            onClick={handleClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <XMarkIcon className="w-6 h-6" />
          </button>
        </div>

        <div className="p-6">
          {/* Quick Start Presets */}
          <div className="mb-6">
            <h3 className="text-lg font-medium text-white mb-3 flex items-center">
              <SparklesIcon className="w-5 h-5 mr-2" />
              Quick Start Templates
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {DEFAULT_TAB_PRESETS.map((preset) => (
                <button
                  key={preset.name}
                  onClick={() => handlePresetClick(preset)}
                  className={cn(
                    'p-4 border-2 rounded-lg text-left transition-all duration-200 hover:scale-105',
                    selectedPreset?.name === preset.name
                      ? 'border-blue-500 bg-blue-500/10'
                      : 'border-gray-600 hover:border-gray-500 hover:bg-gray-700/50'
                  )}
                >
                  <div className="flex items-center mb-2">
                    <span className="text-2xl mr-3">{preset.icon}</span>
                    <div>
                      <div className="font-medium text-white">{preset.name}</div>
                      <div className="text-sm text-gray-400">
                        {SHELL_CONFIGS[preset.shell].icon} {preset.shell}
                      </div>
                    </div>
                  </div>
                  <div className="text-xs text-gray-500">
                    {preset.description}
                  </div>
                  <div className="text-xs text-gray-600 mt-1">
                    📁 {preset.workingDirectory}
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Shell Selection */}
          <div className="mb-6">
            <h3 className="text-lg font-medium text-white mb-3">Shell Type</h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              {Object.values(ShellType).map((shell) => {
                const config = SHELL_CONFIGS[shell];
                return (
                  <button
                    key={shell}
                    onClick={() => handleShellChange(shell)}
                    className={cn(
                      'p-3 border-2 rounded-lg text-center transition-all duration-200',
                      selectedShell === shell
                        ? 'border-blue-500 bg-blue-500/10'
                        : 'border-gray-600 hover:border-gray-500 hover:bg-gray-700/50'
                    )}
                  >
                    <div className="text-2xl mb-1" style={{ color: config.color }}>
                      {config.icon}
                    </div>
                    <div className="text-sm font-medium text-white capitalize">
                      {shell}
                    </div>
                    <div className="text-xs text-gray-400 mt-1">
                      {config.features.length} features
                    </div>
                  </button>
                );
              })}
            </div>
            
            {/* Shell Description */}
            <div className="mt-3 p-3 bg-gray-700/50 rounded-lg">
              <div className="text-sm text-gray-300">
                {SHELL_CONFIGS[selectedShell].description}
              </div>
              <div className="flex flex-wrap gap-2 mt-2">
                {SHELL_CONFIGS[selectedShell].features.map((feature) => (
                  <span
                    key={feature}
                    className="px-2 py-1 bg-gray-600 text-xs text-gray-200 rounded capitalize"
                  >
                    {feature.replace('-', ' ')}
                  </span>
                ))}
              </div>
            </div>
          </div>

          {/* Basic Configuration */}
          <div className="space-y-4 mb-6">
            <div>
              <label className="block text-sm font-medium text-white mb-2">
                Working Directory
              </label>
              <div className="flex">
                <input
                  type="text"
                  value={workingDirectory}
                  onChange={(e) => setWorkingDirectory(e.target.value)}
                  className="flex-1 px-3 py-2 bg-gray-700 border border-gray-600 rounded-l-lg text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                  placeholder="~/projects/my-app"
                />
                <button
                  onClick={handleBrowseDirectory}
                  className="px-3 py-2 bg-gray-600 border border-l-0 border-gray-600 rounded-r-lg text-white hover:bg-gray-500 transition-colors"
                  title="Browse Directory"
                >
                  <FolderIcon className="w-5 h-5" />
                </button>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-white mb-2">
                Tab Title (Optional)
              </label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                placeholder="My Development Session"
              />
            </div>
          </div>

          {/* Advanced Configuration */}
          <div className="mb-6">
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="flex items-center text-white hover:text-blue-400 transition-colors"
            >
              <ChevronRightIcon 
                className={cn('w-4 h-4 mr-1 transition-transform', showAdvanced && 'rotate-90')} 
              />
              Advanced Configuration
            </button>
            
            {showAdvanced && (
              <div className="mt-4 space-y-4 p-4 bg-gray-700/30 rounded-lg">
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <label className="text-sm font-medium text-white">
                      Environment Variables
                    </label>
                    <button
                      onClick={addEnvironmentVar}
                      className="px-2 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 transition-colors"
                    >
                      Add Variable
                    </button>
                  </div>
                  
                  <div className="space-y-2">
                    {Object.entries(environmentVars).map(([key, value]) => (
                      <div key={key} className="flex gap-2">
                        <input
                          type="text"
                          value={key}
                          onChange={(e) => updateEnvironmentVar(key, e.target.value, value)}
                          className="flex-1 px-2 py-1 bg-gray-600 border border-gray-500 rounded text-white text-sm"
                          placeholder="VARIABLE_NAME"
                        />
                        <input
                          type="text"
                          value={value}
                          onChange={(e) => updateEnvironmentVar(key, key, e.target.value)}
                          className="flex-1 px-2 py-1 bg-gray-600 border border-gray-500 rounded text-white text-sm"
                          placeholder="value"
                        />
                        <button
                          onClick={() => removeEnvironmentVar(key)}
                          className="px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 transition-colors"
                        >
                          Remove
                        </button>
                      </div>
                    ))}
                    
                    {Object.keys(environmentVars).length === 0 && (
                      <div className="text-sm text-gray-400 italic">
                        No environment variables configured
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Current Stats */}
          <div className="mb-6 p-3 bg-gray-700/30 rounded-lg">
            <div className="text-sm text-gray-400 mb-1">Current Session</div>
            <div className="text-xs text-gray-500">
              {tabs.length} tabs open • Active shells: {[...new Set(tabs.map(t => t.shell))].join(', ')}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t border-gray-700 bg-gray-800/50">
          <div className="text-sm text-gray-400">
            {selectedPreset ? (
              <>Using preset: <span className="text-blue-400">{selectedPreset.name}</span></>
            ) : (
              'Custom configuration'
            )}
          </div>
          
          <div className="flex space-x-3">
            <button
              onClick={handleClose}
              className="px-4 py-2 text-gray-300 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleCreateTab}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center"
            >
              <CommandLineIcon className="w-4 h-4 mr-2" />
              Create Tab
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
