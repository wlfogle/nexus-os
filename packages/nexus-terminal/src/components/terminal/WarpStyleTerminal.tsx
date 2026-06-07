import React, { useEffect, useCallback } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { invoke } from '@tauri-apps/api/core';
import { WarpTabBar } from './WarpTabBar';
import { TerminalWithAI } from './TerminalWithAI';
import { NewTabModal } from './NewTabModal';
import { 
  selectAllTabs,
  selectActiveTab,
  createTab,
  setCreatingTab,
  updateTabTerminalId 
} from '../../store/slices/terminalTabSlice';
import { ShellType, SHELL_CONFIGS } from '../../types/terminal';
import { terminalLogger } from '../../utils/logger';

interface WarpStyleTerminalProps {
  className?: string;
}

export const WarpStyleTerminal: React.FC<WarpStyleTerminalProps> = ({ className = '' }) => {
  const dispatch = useDispatch();
  const tabs = useSelector(selectAllTabs);
  const activeTab = useSelector(selectActiveTab);
  const isCreatingTab = useSelector((state: any) => state.terminalTabs.isCreatingTab);

  // Create default tab (fish shell, home dir) on first load
  useEffect(() => {
    if (tabs.length === 0) {
      dispatch(createTab({
        shell: ShellType.FISH,
        title: 'Terminal',
        workingDirectory: '~'
      }));
    }
  }, [tabs.length, dispatch]);

  // Create a real PTY backend for every tab that doesn't have one yet
  useEffect(() => {
    const createBackendTerminals = async () => {
      for (const tab of tabs) {
        if (tab.terminalId) continue;
        try {
          const shellConfig = SHELL_CONFIGS[tab.shell];
          const terminalId = await invoke<string>('create_terminal', {
            shell: shellConfig.executable,
            args: shellConfig.args,
            cwd: tab.workingDirectory === '~' ? null : tab.workingDirectory,
            env: tab.environmentVars
          });
          terminalLogger.info('PTY created', 'terminal_created', { terminalId, tabId: tab.id });
          dispatch(updateTabTerminalId({ tabId: tab.id, terminalId }));
        } catch (error) {
          terminalLogger.error('PTY creation failed', error as Error, 'terminal_create_failed', { tabId: tab.id });
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

  return (
    <div className={`flex flex-col h-full bg-[#0d0d0d] ${className}`}>
      {/* Tab bar */}
      <WarpTabBar />

      {/* Active terminal — fills all remaining space */}
      <div className="flex-1 min-h-0 overflow-hidden">
        {activeTab ? (
          <TerminalWithAI key={activeTab.id} tab={activeTab} />
        ) : (
          <div className="flex items-center justify-center h-full text-gray-500">
            <div className="text-center">
              <div className="text-5xl mb-3">🖥️</div>
              <p className="mb-3">No terminal open</p>
              <button
                onClick={() => dispatch(setCreatingTab(true))}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors text-sm"
              >
                New Tab
              </button>
            </div>
          </div>
        )}
      </div>

      {isCreatingTab && (
        <NewTabModal
          onCreateTab={handleCreateTab}
          onClose={handleCloseModal}
        />
      )}
    </div>
  );
};
