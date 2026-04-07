import React, { useEffect, useCallback } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { invoke } from '@tauri-apps/api/core';
import { WarpTabBar } from './WarpTabBar';
import { TerminalWithAI } from './TerminalWithAI';
import { NewTabModal } from './NewTabModal';
import AIControlBar from '../ui/AIControlBar';
import { DebugOverlay, useDebugOverlay } from '../debug/DebugOverlay';
import { 
  selectAllTabs,
  selectActiveTab,
  createTab,
  setCreatingTab,
  updateTabTerminalId 
} from '../../store/slices/terminalTabSlice';
import { ShellType } from '../../types/terminal';
import { useMemoryMonitor } from '../../hooks/useMemoryMonitor';
import { terminalLogger } from '../../utils/logger';
// import type { RootState } from '../../store';

interface WarpStyleTerminalProps {
  className?: string;
}

export const WarpStyleTerminal: React.FC<WarpStyleTerminalProps> = ({ className = '' }) => {
  const dispatch = useDispatch();
  const tabs = useSelector(selectAllTabs);
  const activeTab = useSelector(selectActiveTab);
  const isCreatingTab = useSelector((state: any) => state.terminalTabs.isCreatingTab);
  const { isVisible: debugVisible, toggleDebug } = useDebugOverlay();
  
  const { 
    shouldShowMemoryWarning, 
    isMemoryCritical, 
    estimatedMemoryMB,
    triggerCleanup 
  } = useMemoryMonitor({
    enableAutoCleanup: true,
    cleanupThreshold: 200,
    cleanupInterval: 300000 // 5 minutes
  });

  // Initialize with a default tab if none exist
  useEffect(() => {
    if (tabs.length === 0) {
      // Safe way to get home directory that works in browser context
      const homeDirectory = '~';
      dispatch(createTab({
        shell: ShellType.BASH,
        title: 'Terminal',
        workingDirectory: homeDirectory
      }));
    }
  }, [tabs.length, dispatch]);

  // Create backend terminal process when new tabs are created
  useEffect(() => {
    const createBackendTerminals = async () => {
      // Check if we're in Tauri context
      const isTauriContext = typeof window !== 'undefined' && (window as any).__TAURI__;
      
      for (const tab of tabs) {
        if (!tab.terminalId) {
          try {
            if (isTauriContext) {
              try {
                const terminalId = await invoke<string>('create_simple_terminal', {
                  shell: tab.shell
                });
                terminalLogger.info('Created terminal successfully', 'terminal_created', { terminalId, tabId: tab.id, shell: tab.shell });
                dispatch(updateTabTerminalId({ tabId: tab.id, terminalId }));
              } catch (error) {
                terminalLogger.error('Failed to create Tauri terminal', error as Error, 'terminal_create_failed', { tabId: tab.id, shell: tab.shell });
                // Create a working fallback terminal ID
                const fallbackId = `terminal-${Date.now()}`;
                dispatch(updateTabTerminalId({ tabId: tab.id, terminalId: fallbackId }));
              }
            } else {
              // Browser fallback - create mock terminal ID
              const mockTerminalId = `browser-terminal-${tab.id}`;
              dispatch(updateTabTerminalId({ tabId: tab.id, terminalId: mockTerminalId }));
            }
          } catch (error) {
            terminalLogger.error('Failed to create terminal backend', error as Error, 'terminal_backend_failed', { tabId: tab.id });
            // Fallback for error case
            const fallbackTerminalId = `fallback-terminal-${tab.id}`;
            dispatch(updateTabTerminalId({ tabId: tab.id, terminalId: fallbackTerminalId }));
          }
        }
      }
    };

    createBackendTerminals();
  }, [tabs, dispatch]);

  const handleCreateTab = useCallback(async (config: {
    shell: ShellType;
    title?: string;
    workingDirectory: string;
    environmentVars?: Record<string, string>;
  }) => {
    try {
      dispatch(createTab(config));
      dispatch(setCreatingTab(false));
    } catch (error) {
      terminalLogger.error('Failed to create new tab', error as Error, 'tab_create_failed', { config });
      dispatch(setCreatingTab(false));
    }
  }, [dispatch]);

  const handleCloseModal = useCallback(() => {
    dispatch(setCreatingTab(false));
  }, [dispatch]);

  // Handle cleanup when memory is critical
  useEffect(() => {
    if (isMemoryCritical) {
      triggerCleanup();
    }
  }, [isMemoryCritical, triggerCleanup]);

  return (
    <div className={`flex flex-col h-full bg-gray-900 ${className}`}>
      {/* Memory Warning */}
      {shouldShowMemoryWarning && (
        <div className={`px-4 py-2 text-sm flex items-center justify-between ${
          isMemoryCritical ? 'bg-red-900/50 text-red-200' : 'bg-yellow-900/50 text-yellow-200'
        }`}>
          <span>
            {isMemoryCritical ? '🚨' : '⚠️'} Memory usage: {estimatedMemoryMB}MB
            {isMemoryCritical ? ' - Performance may be affected' : ''}
          </span>
          <button
            onClick={triggerCleanup}
            className="text-xs px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
          >
            Clean up
          </button>
        </div>
      )}

      {/* Tab Bar */}
      <WarpTabBar />

      {/* AI Control Bar */}
      <AIControlBar />

      {/* Terminal Content */}
      <div className="flex-1 relative overflow-hidden">
        {activeTab ? (
          <TerminalWithAI tab={activeTab} />
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-gray-400">
              <div className="text-6xl mb-4">🖥️</div>
              <h2 className="text-xl font-semibold mb-2">Welcome to NexusTerminal</h2>
              <p className="mb-4">Create a new terminal tab to get started</p>
              <button
                onClick={() => dispatch(setCreatingTab(true))}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
              >
                Create Terminal Tab
              </button>
            </div>
          </div>
        )}
      </div>

      {/* New Tab Modal */}
      {isCreatingTab && (
        <NewTabModal
          onCreateTab={handleCreateTab}
          onClose={handleCloseModal}
        />
      )}

      {/* Status Bar */}
      <div className="bg-gray-800 border-t border-gray-700 px-4 py-2 text-xs text-gray-400 flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <span>{tabs.length} tab{tabs.length !== 1 ? 's' : ''}</span>
          {activeTab && (
            <>
              <span>•</span>
              <span>{activeTab.shell}</span>
              <span>•</span>
              <span>{activeTab.workingDirectory}</span>
              <span>•</span>
              <span>{activeTab.terminalHistory.length} commands</span>
            </>
          )}
        </div>
        <div className="flex items-center space-x-4">
          {shouldShowMemoryWarning && (
            <>
              <span className={isMemoryCritical ? 'text-red-400' : 'text-yellow-400'}>
                Memory: {estimatedMemoryMB}MB
              </span>
              <span>•</span>
            </>
          )}
          <span>Nexus Terminal</span>
        </div>
      </div>


      {/* Debug Overlay */}
      <DebugOverlay isVisible={debugVisible} onToggle={toggleDebug} />
    </div>
  );
};
