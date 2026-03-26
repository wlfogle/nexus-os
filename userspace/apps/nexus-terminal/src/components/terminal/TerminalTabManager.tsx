import React, { useCallback, useEffect, useState } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { WarpTabBar } from './WarpTabBar';
import { NewTabModal } from './NewTabModal';
import { TerminalWithAI } from './TerminalWithAI';
import { 
  selectAllTabs, 
  selectActiveTab, 
  setCreatingTab,
  updateTabTerminalId,
  addCommandToHistory,
  addError,
  updateTabWorkingDirectory,
  createTab,
  addRecentCommand
} from '../../store/slices/terminalTabSlice';
import { ShellType, SHELL_CONFIGS, SuggestionType } from '../../types/terminal';
import { addAISuggestion } from '../../store/slices/terminalTabSlice';

// Helper functions for error analysis
const generateErrorSuggestions = (errorMessage: string): string[] => {
  const suggestions: string[] = [];
  const lowercaseError = errorMessage.toLowerCase();
  
  if (lowercaseError.includes('permission denied')) {
    suggestions.push('Try using sudo for elevated permissions');
    suggestions.push('Check file ownership with ls -la');
    suggestions.push('Verify you have write access to the directory');
  } else if (lowercaseError.includes('command not found')) {
    suggestions.push('Install the missing command using your package manager');
    suggestions.push('Check if the command is in your PATH');
    suggestions.push('Verify the command name spelling');
  } else if (lowercaseError.includes('no such file or directory')) {
    suggestions.push('Check if the file path is correct');
    suggestions.push('Verify the file exists with ls');
    suggestions.push('Use absolute path instead of relative path');
  } else if (lowercaseError.includes('connection refused')) {
    suggestions.push('Check if the service is running');
    suggestions.push('Verify the port number and hostname');
    suggestions.push('Check firewall settings');
  } else {
    suggestions.push('Check the command syntax and parameters');
    suggestions.push('Review error message for specific details');
  }
  
  return suggestions;
};

const extractCommandFromError = (errorMessage: string): string => {
  // Extract command from various error message formats
  const patterns = [
    /command '(.+?)' not found/i,
    /(.+?): command not found/i,
    /bash: (.+?): command not found/i,
    /zsh: command not found: (.+)/i,
    /fish: Unknown command '(.+?)'/i
  ];
  
  for (const pattern of patterns) {
    const match = errorMessage.match(pattern);
    if (match && match[1]) {
      return match[1].trim();
    }
  }
  
  // Fallback: try to extract first word
  const words = errorMessage.split(/\s+/);
  return words.find(word => word.length > 2 && !word.includes(':')) || 'unknown';
};

// Analyze commands for proactive AI assistance
const analyzeCommandForAI = (dispatch: any, tabId: string, command: string) => {
  const baseCommand = command.split(' ')[0];
  
  // Detect development workflow patterns
  if (command.startsWith('git ')) {
    if (command.includes('git add') || command.includes('git commit')) {
      dispatch(addAISuggestion({
        tabId,
        suggestion: {
          id: `git-suggestion-${Date.now()}`,
          type: SuggestionType.GIT_WORKFLOW,
          title: 'Git Workflow Assistant',
          description: 'I can help generate commit messages or suggest git best practices.',
          confidence: 0.7,
          timestamp: new Date()
        }
      }));
    }
  }
  
  // Detect build/package manager commands
  if (['npm', 'yarn', 'cargo', 'make', 'cmake'].includes(baseCommand)) {
    if (command.includes('install') || command.includes('build') || command.includes('run')) {
      dispatch(addAISuggestion({
        tabId,
        suggestion: {
          id: `build-suggestion-${Date.now()}`,
          type: SuggestionType.BUILD_ASSISTANCE,
          title: 'Build Process Monitor',
          description: 'I can monitor your build process and help troubleshoot any issues.',
          confidence: 0.8,
          timestamp: new Date()
        }
      }));
    }
  }
  
  // Detect system administration commands
  if (['sudo', 'systemctl', 'service', 'docker', 'kubectl'].includes(baseCommand)) {
    dispatch(addAISuggestion({
      tabId,
      suggestion: {
        id: `sysadmin-suggestion-${Date.now()}`,
        type: SuggestionType.SYSTEM_ADMIN,
        title: 'System Administration Help',
        description: 'Need help with system administration? I can provide guidance and best practices.',
        confidence: 0.85,
        timestamp: new Date()
      }
    }));
  }
  
  // Detect potentially dangerous commands
  if (['rm', 'mv', 'cp'].includes(baseCommand) && (command.includes('-r') || command.includes('-f'))) {
    dispatch(addAISuggestion({
      tabId,
      suggestion: {
        id: `safety-warning-${Date.now()}`,
        type: SuggestionType.SAFETY_WARNING,
        title: '⚠️ Safety Check',
        description: 'This command can modify/delete files. Would you like me to verify what it will do?',
        confidence: 0.95,
        timestamp: new Date()
      }
    }));
  }
};

export const TerminalTabManager: React.FC = () => {
  const dispatch = useDispatch();
  const tabs = useSelector(selectAllTabs);
  const activeTab = useSelector(selectActiveTab);
  const isCreatingTab = useSelector((state: any) => state.terminalTabs.isCreatingTab);
  
  const [terminalOutputListeners, setTerminalOutputListeners] = useState<Map<string, () => void>>(new Map());

  // Initialize default tab if none exist
  useEffect(() => {
    if (tabs.length === 0) {
      // Create a default bash tab
      dispatch(createTab({
        shell: ShellType.BASH,
        workingDirectory: '~',
        title: 'Terminal 1'
      }));
    }
  }, [tabs.length, dispatch]);

  // Handle tab creation by calling backend
  useEffect(() => {
    const handleTabCreation = async () => {
      // Find tabs that don't have terminal IDs yet
      const tabsNeedingTerminals = tabs.filter(tab => !tab.terminalId);
      
      for (const tab of tabsNeedingTerminals) {
        try {
          const shellConfig = SHELL_CONFIGS[tab.shell];
          
          // Create terminal in backend
          const terminalId = await invoke<string>('create_terminal', {
            shell: shellConfig.executable,
            args: shellConfig.args,
            cwd: tab.workingDirectory === '~' ? null : tab.workingDirectory,
            env: tab.environmentVars
          });

          // Update tab with terminal ID
          dispatch(updateTabTerminalId({ tabId: tab.id, terminalId }));
          
          // Set up terminal output listener for this tab
          setupTerminalListener(tab.id, terminalId);

        } catch (error) {
          console.error('Failed to create terminal for tab:', tab.id, error);
          dispatch(addError({
            tabId: tab.id,
            error: {
              command: 'create_terminal',
              errorMessage: error?.toString() || 'Failed to create terminal',
              timestamp: new Date(),
              workingDirectory: tab.workingDirectory,
              suggestedFixes: [
                'Check if the shell is installed',
                'Verify directory permissions',
                'Try a different shell type'
              ]
            }
          }));
        }
      }
    };

    handleTabCreation();
  }, [tabs, dispatch]);

  const setupTerminalListener = useCallback(async (tabId: string, terminalId: string) => {
    try {
      const unlisten = await listen<{terminal_id: string, data: string, type?: string}>('terminal-output', (event) => {
        const { terminal_id, data, type } = event.payload;
        
        // Only handle events for this terminal
        if (terminal_id !== terminalId) return;

        // Analyze the output for command detection and errors
        analyzeTerminalOutput(tabId, data, type);
      });

      // Store listener for cleanup
      setTerminalOutputListeners(prev => new Map(prev.set(tabId, unlisten)));

    } catch (error) {
      console.error('Failed to setup terminal listener:', error);
    }
  }, []);

  const analyzeTerminalOutput = useCallback((tabId: string, data: string, type?: string) => {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab) return;

    // Check if this looks like a command being entered
    if (data.includes('\r') || data.includes('\n')) {
      const lines = data.split(/\r?\n/).filter(line => line.trim());
      
      for (const line of lines) {
        // Skip prompt lines and empty lines
        if (line.includes('$') || line.includes('%') || line.includes('>') || !line.trim()) {
          continue;
        }

        // This might be a command - add to history and analyze
        if (line.trim().length > 0 && !line.startsWith('  ')) {
          const command = line.trim();
          dispatch(addCommandToHistory({
            tabId,
            entry: {
              command,
              timestamp: new Date(),
              workingDirectory: tab.workingDirectory,
              output: '' // Will be filled by subsequent output
            }
          }));
          
          // Analyze command for proactive AI assistance
          analyzeCommandForAI(dispatch, tabId, command);
          
          // Update AI context with command
          dispatch(addRecentCommand({ tabId, command }));
        }
      }
    }

    // Check for error patterns
    if (type === 'stderr' || 
        data.toLowerCase().includes('error') ||
        data.toLowerCase().includes('failed') ||
        data.toLowerCase().includes('not found') ||
        data.toLowerCase().includes('permission denied') ||
        data.toLowerCase().includes('command not found') ||
        data.toLowerCase().includes('no such file')) {
      
      const errorMessage = data.trim();
      dispatch(addError({
        tabId,
        error: {
          command: tab.aiContext.recentCommands[tab.aiContext.recentCommands.length - 1] || 'unknown',
          errorMessage,
          timestamp: new Date(),
          workingDirectory: tab.workingDirectory,
          suggestedFixes: generateErrorSuggestions(errorMessage)
        }
      }));
      
      // Add proactive AI suggestion for common errors
      if (errorMessage.includes('command not found')) {
        const command = extractCommandFromError(errorMessage);
        dispatch(addAISuggestion({
          tabId,
          suggestion: {
            id: `suggestion-${Date.now()}`,
            type: SuggestionType.COMMAND_NOT_FOUND,
            title: `Install ${command}?`,
            description: `The command '${command}' was not found. Would you like me to help you install it?`,
            confidence: 0.9,
            timestamp: new Date()
          }
        }));
      }
    }

    // Check for directory changes
    const cdMatch = data.match(/cd\s+(.+)/);
    if (cdMatch) {
      const newDir = cdMatch[1].trim().replace(/^['"]|['"]$/g, ''); // Remove quotes
      dispatch(updateTabWorkingDirectory({ tabId, cwd: newDir }));
    }
  }, [tabs, dispatch]);

  // Cleanup listeners when tabs are closed
  useEffect(() => {
    const currentTabIds = new Set(tabs.map(t => t.id));
    
    // Clean up listeners for closed tabs
    terminalOutputListeners.forEach((unlisten, tabId) => {
      if (!currentTabIds.has(tabId)) {
        unlisten();
        setTerminalOutputListeners(prev => {
          const newMap = new Map(prev);
          newMap.delete(tabId);
          return newMap;
        });
      }
    });
  }, [tabs, terminalOutputListeners]);

  // Cleanup all listeners on unmount
  useEffect(() => {
    return () => {
      terminalOutputListeners.forEach(unlisten => unlisten());
    };
  }, [terminalOutputListeners]);

  const handleCloseCreatingTab = useCallback(() => {
    dispatch(setCreatingTab(false));
  }, [dispatch]);

  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Warp-Style Tab Bar */}
      <WarpTabBar />
      
      {/* Active Terminal Content */}
      <div className="flex-1 relative">
        {activeTab ? (
          <TerminalWithAI
            key={activeTab.id} // Force re-render when switching tabs
            tab={activeTab}
          />
        ) : (
          <div className="flex items-center justify-center h-full text-gray-400">
            <div className="text-center">
              <div className="text-6xl mb-4">🚀</div>
              <div className="text-xl mb-2">Welcome to NexusTerminal</div>
              <div className="text-sm">Create a new tab to get started</div>
              <button
                onClick={() => dispatch(setCreatingTab(true))}
                className="mt-4 px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Create First Tab
              </button>
            </div>
          </div>
        )}
      </div>

      {/* New Tab Modal */}
      {isCreatingTab && (
        <NewTabModal
          isOpen={isCreatingTab}
          onClose={handleCloseCreatingTab}
          onCreateTab={(config) => dispatch(createTab(config))}
        />
      )}
    </div>
  );
};
