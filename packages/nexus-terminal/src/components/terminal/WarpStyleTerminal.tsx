// Ported from warpdotdev/warp (AGPL-3.0) — top-level Warp-exact screen layout.
// Layout: WarpTabBar (top) | TerminalWithAI + AgentPanel (split) | CommandPalette (modal).
import React, { useEffect, useCallback, useState } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { invoke } from '@tauri-apps/api/core';
import { WarpTabBar } from './WarpTabBar';
import { TerminalWithAI } from './TerminalWithAI';
import { NewTabModal } from './NewTabModal';
import AgentPanel from '../panels/AgentPanel';
import CommandPalette from '../CommandPalette';
import {
  selectAllTabs,
  selectActiveTab,
  createTab,
  setCreatingTab,
  updateTabTerminalId,
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
  const isCreatingTab = useSelector(
    (state: { terminalTabs: { isCreatingTab: boolean } }) => state.terminalTabs.isCreatingTab
  );

  // ── Command palette (Ctrl/Cmd+P) ──────────────────────────────────────────
  const [paletteOpen, setPaletteOpen] = useState(false);
  // ── Agent panel collapse toggle ───────────────────────────────────────────
  const [agentPanelOpen, setAgentPanelOpen] = useState(true);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // Ctrl/Cmd+P → command palette
      if ((e.ctrlKey || e.metaKey) && e.key === 'p' && !e.shiftKey) {
        e.preventDefault();
        setPaletteOpen(v => !v);
      }
      // Ctrl/Cmd+\ → toggle agent panel
      if ((e.ctrlKey || e.metaKey) && e.key === '\\') {
        e.preventDefault();
        setAgentPanelOpen(v => !v);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // ── Create default fish tab on first load ─────────────────────────────────
  useEffect(() => {
    if (tabs.length === 0) {
      dispatch(createTab({
        shell: ShellType.FISH,
        title: 'Terminal',
        workingDirectory: '~',
      }));
    }
  }, [tabs.length, dispatch]);

  // ── Create PTY backend for every tab that lacks one ───────────────────────
  // Guarded by a ref to prevent double-creation on re-renders (e.g. OSC 7 cwd
  // updates), which would spawn duplicate PTYs.
  const creatingTabs = React.useRef<Set<string>>(new Set());

  useEffect(() => {
    const createBackendTerminals = async () => {
      for (const tab of tabs) {
        if (tab.terminalId) continue;
        if (creatingTabs.current.has(tab.id)) continue;
        creatingTabs.current.add(tab.id);
        try {
          const shellConfig = SHELL_CONFIGS[tab.shell];
          const terminalId = await invoke<string>('create_terminal', {
            shell: shellConfig.executable,
            args: shellConfig.args,
            cwd: tab.workingDirectory === '~' ? null : tab.workingDirectory,
            env: tab.environmentVars,
          });
          terminalLogger.info('PTY created', 'terminal_created', { terminalId, tabId: tab.id });
          dispatch(updateTabTerminalId({ tabId: tab.id, terminalId }));
        } catch (error) {
          terminalLogger.error('PTY creation failed', error as Error, 'terminal_create_failed', { tabId: tab.id });
        } finally {
          creatingTabs.current.delete(tab.id);
        }
      }
    };
    createBackendTerminals();
  }, [tabs, dispatch]);

  const handleCreateTab = useCallback((config: {
    shell: ShellType;
    title?: string;
    workingDirectory: string;
    environmentVars?: Record<string, string>;
  }) => {
    dispatch(createTab(config));
    dispatch(setCreatingTab(false));
  }, [dispatch]);

  const handleCloseModal = useCallback(() => {
    dispatch(setCreatingTab(false));
  }, [dispatch]);

  return (
    <div className={`flex flex-col h-full bg-[#0d0d0d] ${className}`}>
      {/* ── Top: Warp-style tab bar ─────────────────────────────────────── */}
      <WarpTabBar />

      {/* ── Main content: terminal left | agent panel right ─────────────── */}
      <div style={{ flex: '1 1 0%', minHeight: 0, display: 'flex', flexDirection: 'row', overflow: 'hidden' }}>

        {/* Left column: xterm + block list + input bar */}
        <div style={{ flex: '1 1 0%', minWidth: 0, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          {activeTab ? (
            <TerminalWithAI key={activeTab.id} tab={activeTab} />
          ) : (
            <div className="flex items-center justify-center h-full text-gray-500">
              <div className="text-center">
                <div className="text-5xl mb-3">🖥️</div>
                <p className="mb-3 text-sm">No terminal open</p>
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

        {/* Right column: collapsible AI agent panel */}
        {agentPanelOpen && (
          <AgentPanel tab={activeTab} width={320} />
        )}
      </div>

      {/* Re-open button when agent panel is hidden */}
      {!agentPanelOpen && (
        <button
          onClick={() => setAgentPanelOpen(true)}
          title="Open Agent Panel (Ctrl+\\)"
          style={{
            position: 'fixed', right: 0, top: '50%', transform: 'translateY(-50%)',
            background: '#1c1c1e', border: '1px solid #374151', borderRight: 'none',
            borderRadius: '6px 0 0 6px', padding: '8px 4px', color: '#6b7280',
            cursor: 'pointer', zIndex: 50, fontSize: 11, writingMode: 'vertical-rl',
          }}
        >
          AI ▶
        </button>
      )}

      {/* ── New tab modal ───────────────────────────────────────────────── */}
      {isCreatingTab && (
        <NewTabModal
          onCreateTab={handleCreateTab}
          onClose={handleCloseModal}
        />
      )}

      {/* ── Command palette (Ctrl/Cmd+P) ────────────────────────────────── */}
      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} />
    </div>
  );
};
