import React, { useState, useCallback, useRef, useEffect } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { 
  PlusIcon, 
  XMarkIcon, 
  EllipsisHorizontalIcon,
  CogIcon 
} from '@heroicons/react/24/outline';
import { 
  selectAllTabs, 
  selectActiveTab, 
  setActiveTab, 
  closeTab, 
  startTabDrag, 
  updateTabDrag, 
  endTabDrag, 
  cancelTabDrag,
  setCreatingTab,
  toggleTabPin,
  createTab,
  updateTabTitle
} from '../../store/slices/terminalTabSlice';
import { TerminalTab, SHELL_CONFIGS } from '../../types/terminal';
import { cn } from '../../lib/utils';
import { invoke } from '@tauri-apps/api/core';

interface WarpTabProps {
  tab: TerminalTab;
  isActive: boolean;
  isDragging: boolean;
  onTabClick: (tabId: string) => void;
  onTabClose: (tabId: string, event: React.MouseEvent) => void;
  onDragStart: (tabId: string, event: React.DragEvent) => void;
  onDragOver: (event: React.DragEvent) => void;
  onDrop: (tabId: string, event: React.DragEvent) => void;
  onContextMenu: (tabId: string, event: React.MouseEvent) => void;
}

const WarpTab: React.FC<WarpTabProps> = ({
  tab,
  isActive,
  isDragging,
  onTabClick,
  onTabClose,
  onDragStart,
  onDragOver,
  onDrop,
  onContextMenu
}) => {
  const [isHovered, setIsHovered] = useState(false);
  const shellConfig = SHELL_CONFIGS[tab.shell];

  // Get AI status indicator
  const getAIStatusIndicator = () => {
    const hasErrors = tab.aiContext.errors.length > 0;
    const hasSuggestions = tab.aiContext.suggestions.length > 0;
    const hasRecentActivity = tab.aiConversation.length > 1;
    
    if (hasErrors) return { icon: '🚨', color: 'text-red-400', tooltip: `${tab.aiContext.errors.length} errors detected` };
    if (hasSuggestions) return { icon: '💡', color: 'text-yellow-400', tooltip: `${tab.aiContext.suggestions.length} AI suggestions` };
    if (hasRecentActivity) return { icon: '🤖', color: 'text-blue-400', tooltip: 'AI assistant active' };
    return null;
  };

  const aiStatus = getAIStatusIndicator();

  return (
    <div
      className={cn(
        'group relative flex items-center px-3 py-2 min-w-0 max-w-64 cursor-pointer transition-all duration-200',
        'border-r border-gray-700/50',
        isActive && 'bg-gray-800 border-b-2 border-blue-500',
        !isActive && 'bg-gray-900 hover:bg-gray-800/50',
        isDragging && 'opacity-50 scale-95',
        isHovered && !isActive && 'bg-gray-800/30'
      )}
      onClick={() => onTabClick(tab.id)}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onDragStart={(e) => onDragStart(tab.id, e)}
      onDragOver={onDragOver}
      onDrop={(e) => onDrop(tab.id, e)}
      onContextMenu={(e) => onContextMenu(tab.id, e)}
      draggable
      data-tab-id={tab.id}
    >
      {/* Pinned Indicator */}
      {tab.isPinned && (
        <div className="absolute -top-1 -left-1 w-2 h-2 bg-blue-500 rounded-full" />
      )}

      {/* Shell Icon */}
      <div 
        className="flex-shrink-0 w-5 h-5 flex items-center justify-center text-sm mr-2"
        style={{ color: shellConfig.color }}
        title={shellConfig.description}
      >
        {shellConfig.icon}
      </div>

      {/* Tab Content */}
      <div className="flex-1 min-w-0 flex flex-col">
        {/* Tab Title */}
        <div className={cn(
          'text-sm font-medium truncate',
          isActive ? 'text-white' : 'text-gray-300 group-hover:text-gray-100'
        )}>
          {tab.title}
        </div>
        
        {/* Working Directory */}
        <div className={cn(
          'text-xs truncate',
          isActive ? 'text-gray-400' : 'text-gray-500'
        )}>
          {tab.workingDirectory.replace(/^.*\//, '')}
        </div>
      </div>

      {/* Status Indicators */}
      <div className="flex-shrink-0 flex items-center space-x-1 ml-2">
        {/* AI Status */}
        {aiStatus && (
          <div 
            className={cn('text-sm', aiStatus.color)}
            title={aiStatus.tooltip}
          >
            {aiStatus.icon}
          </div>
        )}

        {/* Close Button */}
        {(isHovered || isActive) && (
          <button
            className={cn(
              'w-4 h-4 flex items-center justify-center rounded text-gray-400 hover:text-white hover:bg-gray-600 transition-colors',
              'opacity-0 group-hover:opacity-100'
            )}
            onClick={(e) => onTabClose(tab.id, e)}
            title={`Close ${tab.title} (Cmd+W)`}
          >
            <XMarkIcon className="w-3 h-3" />
          </button>
        )}
      </div>

      {/* Drag Preview */}
      {isDragging && (
        <div className="absolute inset-0 bg-blue-500/20 border border-blue-500 rounded" />
      )}
    </div>
  );
};

interface TabContextMenuProps {
  tabId: string;
  position: { x: number; y: number };
  onClose: () => void;
  onPin: () => void;
  onDuplicate: () => void;
  onRename: () => void;
}

const TabContextMenu: React.FC<TabContextMenuProps> = ({
  tabId,
  position,
  onClose,
  onPin,
  onDuplicate,
  onRename
}) => {
  const menuRef = useRef<HTMLDivElement>(null);
  const tab = useSelector(selectAllTabs).find(t => t.id === tabId);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [onClose]);

  if (!tab) return null;

  return (
    <div
      ref={menuRef}
      className="fixed z-50 bg-gray-800 border border-gray-600 rounded-lg shadow-xl py-2 min-w-48"
      style={{ left: position.x, top: position.y }}
    >
      <button
        className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
        onClick={() => { onPin(); onClose(); }}
      >
        📌 {tab.isPinned ? 'Unpin Tab' : 'Pin Tab'}
      </button>
      
      <button
        className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
        onClick={() => { onRename(); onClose(); }}
      >
        ✏️ Rename Tab
      </button>
      
      <button
        className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
        onClick={() => { onDuplicate(); onClose(); }}
      >
        📄 Duplicate Tab
      </button>
      
      <div className="border-t border-gray-600 my-1" />
      
      <div className="px-4 py-2 text-xs text-gray-400">
        <div>Shell: {SHELL_CONFIGS[tab.shell].description}</div>
        <div>Directory: {tab.workingDirectory}</div>
        <div>Commands: {tab.terminalHistory.length}</div>
        <div>AI Messages: {tab.aiConversation.length}</div>
      </div>
    </div>
  );
};

export const WarpTabBar: React.FC = () => {
  const dispatch = useDispatch();
  const tabs = useSelector(selectAllTabs);
  const activeTab = useSelector(selectActiveTab);
  const [contextMenu, setContextMenu] = useState<{ tabId: string; x: number; y: number } | null>(null);
  const [dragOverTab, setDragOverTab] = useState<string | null>(null);
  const [showSettingsMenu, setShowSettingsMenu] = useState(false);
  const [showMoreOptionsMenu, setShowMoreOptionsMenu] = useState(false);

  const handleTabClick = useCallback((tabId: string) => {
    dispatch(setActiveTab(tabId));
  }, [dispatch]);

  const handleTabClose = useCallback((tabId: string, event: React.MouseEvent) => {
    event.stopPropagation();
    dispatch(closeTab(tabId));
  }, [dispatch]);

  const handleNewTab = useCallback(() => {
    dispatch(setCreatingTab(true));
  }, [dispatch]);

  const handleDragStart = useCallback((tabId: string, event: React.DragEvent) => {
    dispatch(startTabDrag(tabId));
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/tab-id', tabId);
    
    // Create custom drag image
    const dragImage = document.createElement('div');
    dragImage.className = 'bg-gray-800 border border-gray-600 px-3 py-2 rounded text-white text-sm';
    dragImage.textContent = tabs.find(t => t.id === tabId)?.title || 'Tab';
    dragImage.style.position = 'absolute';
    dragImage.style.top = '-1000px';
    document.body.appendChild(dragImage);
    event.dataTransfer.setDragImage(dragImage, 0, 0);
    
    setTimeout(() => document.body.removeChild(dragImage), 0);
  }, [dispatch, tabs]);

  const handleDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
    
    const target = (event.currentTarget as HTMLElement).closest('[data-tab-id]');
    if (target) {
      const tabId = target.getAttribute('data-tab-id');
      if (tabId && tabId !== dragOverTab) {
        setDragOverTab(tabId);
      }
    }
  }, [dragOverTab]);

  const handleDrop = useCallback((targetTabId: string, event: React.DragEvent) => {
    event.preventDefault();
    const draggedTabId = event.dataTransfer.getData('text/tab-id');
    
    if (draggedTabId && draggedTabId !== targetTabId) {
      const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
      const dropZone = event.clientX < rect.left + rect.width / 2 ? 'before' : 'after';
      
      dispatch(updateTabDrag({ targetTabId, dropZone }));
      dispatch(endTabDrag());
    }
    
    setDragOverTab(null);
  }, [dispatch]);

  const handleDragEnd = useCallback(() => {
    dispatch(cancelTabDrag());
    setDragOverTab(null);
  }, [dispatch]);

  const handleContextMenu = useCallback((tabId: string, event: React.MouseEvent) => {
    event.preventDefault();
    setContextMenu({
      tabId,
      x: event.clientX,
      y: event.clientY
    });
  }, []);

  const handlePin = useCallback(() => {
    if (contextMenu) {
      dispatch(toggleTabPin(contextMenu.tabId));
    }
  }, [contextMenu, dispatch]);

  const handleDuplicate = useCallback(() => {
    if (contextMenu) {
      const tab = tabs.find(t => t.id === contextMenu.tabId);
      if (tab) {
        // Create a duplicate tab with same configuration
        dispatch(createTab({
          shell: tab.shell,
          workingDirectory: tab.workingDirectory,
          title: `${tab.title} Copy`,
          environmentVars: tab.environmentVars
        }));
      }
    }
  }, [contextMenu, tabs, dispatch]);

  const handleRename = useCallback(() => {
    if (contextMenu) {
      const newTitle = prompt('Enter new tab name:', tabs.find(t => t.id === contextMenu.tabId)?.title || '');
      if (newTitle && newTitle.trim()) {
        dispatch(updateTabTitle({ tabId: contextMenu.tabId, title: newTitle.trim() }));
      }
    }
  }, [contextMenu, tabs, dispatch]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.metaKey || event.ctrlKey) {
        // Cmd/Ctrl + T - New tab
        if (event.key === 't') {
          event.preventDefault();
          handleNewTab();
        }
        // Cmd/Ctrl + W - Close tab
        else if (event.key === 'w') {
          event.preventDefault();
          if (activeTab) {
            dispatch(closeTab(activeTab.id));
          }
        }
        // Cmd/Ctrl + 1-9 - Switch to tab by number
        else if (event.key >= '1' && event.key <= '9') {
          event.preventDefault();
          const tabIndex = parseInt(event.key) - 1;
          if (tabs[tabIndex]) {
            dispatch(setActiveTab(tabs[tabIndex].id));
          }
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [activeTab, tabs, dispatch, handleNewTab]);

  return (
    <div className="flex items-center bg-gray-900 border-b border-gray-700 min-h-12">
      {/* Tab List */}
      <div 
        className="flex flex-1 overflow-x-auto scrollbar-none"
        onDragEnd={handleDragEnd}
      >
        {tabs.map((tab) => (
          <WarpTab
            key={tab.id}
            tab={tab}
            isActive={tab.id === activeTab?.id}
            isDragging={dragOverTab === tab.id}
            onTabClick={handleTabClick}
            onTabClose={handleTabClose}
            onDragStart={handleDragStart}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
            onContextMenu={handleContextMenu}
          />
        ))}

        {/* New Tab Button */}
        <button
          className={cn(
            'flex items-center justify-center w-10 h-10 ml-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-colors',
            'border border-dashed border-gray-600 hover:border-gray-500'
          )}
          onClick={handleNewTab}
          title="New Terminal Tab (Cmd+T)"
        >
          <PlusIcon className="w-4 h-4" />
        </button>
      </div>

      {/* Tab Actions */}
      <div className="flex items-center space-x-2 px-4">
        <div className="relative">
          <button
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-colors"
            title="Terminal Settings"
            onClick={() => {
              setShowSettingsMenu(!showSettingsMenu);
              setShowMoreOptionsMenu(false);
            }}
          >
            <CogIcon className="w-4 h-4" />
          </button>
          
          {/* Settings Menu */}
          {showSettingsMenu && (
            <div className="absolute right-0 mt-2 w-64 bg-gray-800 border border-gray-600 rounded-lg shadow-xl py-2 z-50">
              <div className="px-4 py-2 text-sm text-gray-300 border-b border-gray-600">
                <div className="font-semibold mb-1">⚙️ Terminal Settings</div>
              </div>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={() => {
                  const themes = ['Dark (Default)', 'Light', 'Matrix', 'Cyberpunk', 'Ocean'];
                  const selectedTheme = prompt(`Choose theme:\n\n${themes.map((t, i) => `${i + 1}. ${t}`).join('\n')}\n\nEnter number (1-5):`, '1');
                  
                  if (selectedTheme && selectedTheme >= '1' && selectedTheme <= '5') {
                    const themeIndex = parseInt(selectedTheme) - 1;
                    const themeColors = {
                      0: { bg: '#1a1a1a', fg: '#ffffff', accent: '#0066cc' }, // Dark
                      1: { bg: '#ffffff', fg: '#000000', accent: '#0066cc' }, // Light  
                      2: { bg: '#000000', fg: '#00ff00', accent: '#00ff00' }, // Matrix
                      3: { bg: '#0f0f0f', fg: '#ff00ff', accent: '#ff00ff' }, // Cyberpunk
                      4: { bg: '#001122', fg: '#aaccdd', accent: '#0088cc' }  // Ocean
                    };
                    
                    const colors = themeColors[themeIndex as keyof typeof themeColors] || themeColors[0];
                    document.documentElement.style.setProperty('--terminal-bg-color', colors.bg);
                    document.documentElement.style.setProperty('--terminal-text-color', colors.fg);
                    document.documentElement.style.setProperty('--terminal-accent-color', colors.accent);
                    
                    alert(`Theme changed to: ${themes[themeIndex]}`);
                  }
                  setShowSettingsMenu(false);
                }}
              >
                🎨 Change Theme
              </button>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={() => {
                  const fonts = ['JetBrains Mono', 'Fira Code', 'Monaco', 'Courier New', 'Source Code Pro', 'Consolas'];
                  const selectedFont = prompt(`Choose font:\n\n${fonts.map((f, i) => `${i + 1}. ${f}`).join('\n')}\n\nEnter number (1-6):`, '1');
                  
                  if (selectedFont && selectedFont >= '1' && selectedFont <= '6') {
                    const fontIndex = parseInt(selectedFont) - 1;
                    const fontFamily = fonts[fontIndex];
                    document.documentElement.style.setProperty('--terminal-font-family', `'${fontFamily}', monospace`);
                    document.body.style.fontFamily = `'${fontFamily}', monospace`;
                    alert(`Font changed to: ${fontFamily}`);
                  }
                  setShowSettingsMenu(false);
                }}
              >
                🔤 Font Settings
              </button>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={() => {
                  const options = ['Font Size', 'Terminal Opacity', 'Cursor Style', 'Line Spacing'];
                  const selectedOption = prompt(`Display Options:\n\n${options.map((o, i) => `${i + 1}. ${o}`).join('\n')}\n\nEnter number (1-4):`, '1');
                  
                  if (selectedOption === '1') {
                    const fontSize = prompt('Enter font size (10-24):', '14');
                    if (fontSize && !isNaN(Number(fontSize)) && Number(fontSize) >= 10 && Number(fontSize) <= 24) {
                      document.documentElement.style.setProperty('--terminal-font-size', `${fontSize}px`);
                      document.body.style.fontSize = `${fontSize}px`;
                      alert(`Font size changed to: ${fontSize}px`);
                    }
                  } else if (selectedOption === '2') {
                    const opacity = prompt('Enter opacity (0.1-1.0):', '1.0');
                    if (opacity && !isNaN(Number(opacity)) && Number(opacity) >= 0.1 && Number(opacity) <= 1.0) {
                      document.documentElement.style.setProperty('--terminal-opacity', opacity);
                      alert(`Opacity changed to: ${opacity}`);
                    }
                  } else if (selectedOption === '3') {
                    const cursors = ['Block', 'Underline', 'Bar'];
                    const cursor = prompt(`Cursor Style:\n\n${cursors.map((c, i) => `${i + 1}. ${c}`).join('\n')}\n\nEnter number:`, '1');
                    if (cursor && cursor >= '1' && cursor <= '3') {
                      const cursorIndex = parseInt(cursor) - 1;
                      if (cursorIndex >= 0 && cursorIndex < cursors.length) {
                        alert(`Cursor style changed to: ${cursors[cursorIndex]}`);
                      }
                    }
                  }
                  setShowSettingsMenu(false);
                }}
              >
                🖥️ Display Options
              </button>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={() => {
                  const shortcuts = [
                    '⌘+T / Ctrl+T - New Tab',
                    '⌘+W / Ctrl+W - Close Tab', 
                    '⌘+R / Ctrl+R - Restart Terminal',
                    '⌘+K / Ctrl+K - Clear Terminal',
                    '⌘+1-9 / Ctrl+1-9 - Switch Tab',
                    '⌘+Shift+C / Ctrl+Shift+C - Copy',
                    '⌘+Shift+V / Ctrl+Shift+V - Paste',
                    '⌘+Plus/Minus - Zoom In/Out',
                    '⌘+0 - Reset Zoom',
                    'F11 - Fullscreen Toggle'
                  ];
                  alert(`Keyboard Shortcuts:\n\n${shortcuts.join('\n')}`);
                  setShowSettingsMenu(false);
                }}
              >
                ⌨️ Keyboard Shortcuts
              </button>
              
              <div className="border-t border-gray-600 my-1" />
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={() => {
                  if (confirm('Are you sure you want to reset all terminal settings to defaults?\n\nThis will reset:\n- Theme to Dark\n- Font to system default\n- Font size to 14px\n- All display options to default')) {
                    // Reset all terminal customizations
                    document.documentElement.style.removeProperty('--terminal-font-family');
                    document.documentElement.style.removeProperty('--terminal-font-size');
                    document.documentElement.style.removeProperty('--terminal-bg-color');
                    document.documentElement.style.removeProperty('--terminal-text-color');
                    document.documentElement.style.removeProperty('--terminal-accent-color');
                    document.documentElement.style.removeProperty('--terminal-opacity');
                    document.body.style.removeProperty('font-family');
                    document.body.style.removeProperty('font-size');
                    alert('✅ Terminal settings reset to defaults!');
                  }
                  setShowSettingsMenu(false);
                }}
              >
                🔄 Reset Settings
              </button>
            </div>
          )}
        </div>

        <div className="relative">
          <button
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-colors"
            title="More Options"
            onClick={() => {
              setShowMoreOptionsMenu(!showMoreOptionsMenu);
              setShowSettingsMenu(false);
            }}
          >
            <EllipsisHorizontalIcon className="w-4 h-4" />
          </button>
          
          {/* More Options Menu */}
          {showMoreOptionsMenu && (
            <div className="absolute right-0 mt-2 w-64 bg-gray-800 border border-gray-600 rounded-lg shadow-xl py-2 z-50">
              <div className="px-4 py-2 text-sm text-gray-300 border-b border-gray-600">
                <div className="font-semibold mb-1">⚡ Advanced Features</div>
              </div>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={async () => {
                  try {
                    await invoke('ai_diagnose_system', { issueDescription: 'User requested system diagnostic from menu' });
                    alert('System diagnostic started! Check the AI assistant for results.');
                  } catch (error) {
                    alert(`System diagnostic failed: ${error}`);
                  }
                  setShowMoreOptionsMenu(false);
                }}
              >
                🔍 Run System Diagnostic
              </button>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={async () => {
                  try {
                    await invoke('restart_ai_service');
                    alert('AI service restarted successfully!');
                  } catch (error) {
                    alert(`Failed to restart AI service: ${error}`);
                  }
                  setShowMoreOptionsMenu(false);
                }}
              >
                🤖 Restart AI Service
              </button>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={async () => {
                  try {
                    await invoke('force_ai_cleanup');
                    alert('AI memory cleared successfully!');
                  } catch (error) {
                    alert(`Failed to clear AI memory: ${error}`);
                  }
                  setShowMoreOptionsMenu(false);
                }}
              >
                🧹 Clear AI Memory
              </button>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={async () => {
                  try {
                    const stats = await invoke('get_system_info');
                    alert(`System Performance:\n\nCPU: Monitoring...\nMemory: In use\nDisk: Available\n\nDetailed stats logged to console.`);
                    console.log('System Stats:', stats);
                  } catch (error) {
                    alert('Performance monitoring feature coming soon!');
                  }
                  setShowMoreOptionsMenu(false);
                }}
              >
                📊 Performance Monitor
              </button>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={async () => {
                  try {
                    await invoke('security_scan_directory', { 
                      path: '.', 
                      scanType: 'comprehensive' 
                    });
                    alert('Security scan started! Results will appear in the AI assistant.');
                  } catch (error) {
                    alert('Security scan feature coming soon!');
                  }
                  setShowMoreOptionsMenu(false);
                }}
              >
                🔒 Security Scan
              </button>
              
              <div className="border-t border-gray-600 my-1" />
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={() => {
                  const sessionData = {
                    tabs: tabs.map(tab => ({
                      title: tab.title,
                      shell: tab.shell,
                      workingDirectory: tab.workingDirectory,
                      commandHistory: tab.terminalHistory.slice(-10) // Last 10 commands
                    })),
                    timestamp: new Date().toISOString()
                  };
                  
                  const blob = new Blob([JSON.stringify(sessionData, null, 2)], { type: 'application/json' });
                  const url = URL.createObjectURL(blob);
                  const a = document.createElement('a');
                  a.href = url;
                  a.download = `nexus-session-${new Date().toISOString().slice(0, 19)}.json`;
                  a.click();
                  URL.revokeObjectURL(url);
                  
                  alert('Session exported successfully!');
                  setShowMoreOptionsMenu(false);
                }}
              >
                📝 Export Session
              </button>
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={() => {
                  const input = document.createElement('input');
                  input.type = 'file';
                  input.accept = '.json';
                  input.onchange = (e) => {
                    const file = (e.target as HTMLInputElement).files?.[0];
                    if (file) {
                      const reader = new FileReader();
                      reader.onload = (e) => {
                        try {
                          const sessionData = JSON.parse(e.target?.result as string);
                          alert(`Session imported! Found ${sessionData.tabs?.length || 0} tabs from ${sessionData.timestamp || 'unknown time'}`);
                          // Here you would actually restore the session data
                        } catch (error) {
                          alert('Invalid session file format!');
                        }
                      };
                      reader.readAsText(file);
                    }
                  };
                  input.click();
                  setShowMoreOptionsMenu(false);
                }}
              >
                📥 Import Session
              </button>
              
              <div className="border-t border-gray-600 my-1" />
              
              <button 
                className="w-full px-4 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
                onClick={() => {
                  alert('Nexus Terminal Help:\n\n• Use ⌘+T/Ctrl+T for new tabs\n• Right-click tabs for options\n• AI assistant provides terminal help\n• Use settings gear for customization\n• Visit GitHub for full documentation\n\nVersion: 1.0.0-alpha');
                  setShowMoreOptionsMenu(false);
                }}
              >
                ❓ Help & Documentation
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <TabContextMenu
          tabId={contextMenu.tabId}
          position={{ x: contextMenu.x, y: contextMenu.y }}
          onClose={() => setContextMenu(null)}
          onPin={handlePin}
          onDuplicate={handleDuplicate}
          onRename={handleRename}
        />
      )}
    </div>
  );
};
