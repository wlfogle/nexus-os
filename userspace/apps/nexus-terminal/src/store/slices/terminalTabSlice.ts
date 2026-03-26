import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { 
  TerminalTab, 
  NewTabConfig, 
  ShellType, 
  CommandHistoryEntry, 
  AIMessage,
  AISuggestion,
  ErrorContext,
  DragState,
  DEFAULT_TAB_PRESETS
} from '../../types/terminal';

interface TerminalTabState {
  tabs: TerminalTab[];
  activeTabId: string | null;
  dragState: DragState | null;
  isCreatingTab: boolean;
  tabPresets: typeof DEFAULT_TAB_PRESETS;
  nextTabOrder: number;
  
// AI Context Management
  aiContextGlobal: {
    sharedKnowledge: Record<string, unknown>;
    crossTabInsights: string[];
    projectAnalysis: Record<string, unknown>;
  };
  
  // Performance tracking
  performanceMetrics: {
    tabSwitchTimes: number[];
    memoryUsage: Record<string, number>;
    lastOptimization: number;
  };
}

const initialState: TerminalTabState = {
  tabs: [],
  activeTabId: null,
  dragState: null,
  isCreatingTab: false,
  tabPresets: DEFAULT_TAB_PRESETS,
  nextTabOrder: 0,
  
  aiContextGlobal: {
    sharedKnowledge: {},
    crossTabInsights: [],
    projectAnalysis: {}
  },
  
  performanceMetrics: {
    tabSwitchTimes: [],
    memoryUsage: {},
    lastOptimization: Date.now()
  }
};

// Utility functions
const generateTabId = () => `tab-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
const generateMessageId = () => `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

const terminalTabSlice = createSlice({
  name: 'terminalTabs',
  initialState,
  reducers: {
    // Tab Management
    createTab: (state, action: PayloadAction<NewTabConfig>) => {
      const config = action.payload;
      const tabId = generateTabId();
      
      const newTab: TerminalTab = {
        id: tabId,
        title: config.title || `${config.shell} ${config.workingDirectory.split('/').pop()}`,
        shell: config.shell,
        workingDirectory: config.workingDirectory.replace('~', '/home/lou'),
        environmentVars: config.environmentVars || {},
        
        // Terminal Process - will be set when backend creates the terminal
        terminalId: '', // Will be updated after backend call
        terminalHistory: [],
        
        // AI Context
        aiConversation: [{
          id: generateMessageId(),
          role: 'system' as const,
          content: `🤖 AI Assistant ready for ${config.shell} session in ${config.workingDirectory}`,
          timestamp: new Date()
        }],
        aiContext: {
          recentCommands: [],
          workingFiles: [],
          errors: [],
          suggestions: [],
          learningContext: {
            shell: config.shell,
            workingDirectory: config.workingDirectory,
            projectType: 'unknown',
            startTime: new Date().toISOString()
          }
        },
        
        // UI State
        isActive: true, // New tabs become active
        isPinned: false,
        lastActivity: new Date(),
        order: state.nextTabOrder,
      };

      // Deactivate other tabs
      state.tabs.forEach(tab => { tab.isActive = false; });
      
      // Add new tab
      state.tabs.push(newTab);
      state.activeTabId = tabId;
      state.nextTabOrder += 1;
      state.isCreatingTab = false;
    },

    // Update tab with backend terminal ID after creation
    updateTabTerminalId: (state, action: PayloadAction<{ tabId: string; terminalId: string }>) => {
      const { tabId, terminalId } = action.payload;
      const tab = state.tabs.find(t => t.id === tabId);
      if (tab) {
        tab.terminalId = terminalId;
      }
    },

    setActiveTab: (state, action: PayloadAction<string>) => {
      const tabId = action.payload;
      const startTime = performance.now();
      
      // Deactivate all tabs
      state.tabs.forEach(tab => { tab.isActive = false; });
      
      // Activate selected tab
      const targetTab = state.tabs.find(tab => tab.id === tabId);
      if (targetTab) {
        targetTab.isActive = true;
        targetTab.lastActivity = new Date();
        state.activeTabId = tabId;
        
        // Track performance
        const switchTime = performance.now() - startTime;
        state.performanceMetrics.tabSwitchTimes.push(switchTime);
        
        // Keep only last 10 measurements
        if (state.performanceMetrics.tabSwitchTimes.length > 10) {
          state.performanceMetrics.tabSwitchTimes.shift();
        }
      }
    },

    closeTab: (state, action: PayloadAction<string>) => {
      const tabId = action.payload;
      const tabIndex = state.tabs.findIndex(tab => tab.id === tabId);
      
      if (tabIndex !== -1) {
        const closedTab = state.tabs[tabIndex];
        
        // Save AI conversation to global knowledge if valuable
        if (closedTab.aiConversation.length > 2) {
          const key = `${closedTab.shell}-${closedTab.workingDirectory}`;
          state.aiContextGlobal.sharedKnowledge[key] = {
            lastSession: closedTab.aiConversation.slice(-5),
            insights: closedTab.aiContext.suggestions,
            timestamp: new Date().toISOString()
          };
        }
        
        // Remove tab
        state.tabs.splice(tabIndex, 1);
        
        // Clean up performance metrics
        delete state.performanceMetrics.memoryUsage[tabId];
        
        // Switch to another tab if this was active
        if (state.activeTabId === tabId) {
          if (state.tabs.length > 0) {
            // Switch to the tab that was to the right, or leftmost if this was rightmost
            const newActiveIndex = Math.min(tabIndex, state.tabs.length - 1);
            const newActiveTab = state.tabs[newActiveIndex];
            if (newActiveTab) {
              newActiveTab.isActive = true;
              state.activeTabId = newActiveTab.id;
            }
          } else {
            state.activeTabId = null;
          }
        }
      }
    },

    // Drag and Drop
    startTabDrag: (state, action: PayloadAction<string>) => {
      state.dragState = {
        draggedTabId: action.payload,
        dropZone: null
      };
    },

    updateTabDrag: (state, action: PayloadAction<{ targetTabId: string; dropZone: 'before' | 'after' }>) => {
      if (state.dragState) {
        state.dragState.targetTabId = action.payload.targetTabId;
        state.dragState.dropZone = action.payload.dropZone;
      }
    },

    endTabDrag: (state) => {
      if (state.dragState && state.dragState.targetTabId && state.dragState.dropZone) {
        const draggedTab = state.tabs.find(tab => tab.id === state.dragState!.draggedTabId);
        const targetTab = state.tabs.find(tab => tab.id === state.dragState!.targetTabId);
        
        if (draggedTab && targetTab) {
          // Remove dragged tab from current position
          const draggedIndex = state.tabs.findIndex(tab => tab.id === state.dragState!.draggedTabId);
          state.tabs.splice(draggedIndex, 1);
          
          // Find new target position
          const targetIndex = state.tabs.findIndex(tab => tab.id === state.dragState!.targetTabId);
          const insertIndex = state.dragState.dropZone === 'after' ? targetIndex + 1 : targetIndex;
          
          // Insert dragged tab at new position
          state.tabs.splice(insertIndex, 0, draggedTab);
          
          // Update order values
          state.tabs.forEach((tab, index) => {
            tab.order = index;
          });
        }
      }
      
      state.dragState = null;
    },

    cancelTabDrag: (state) => {
      state.dragState = null;
    },

    // Tab Properties
    updateTabTitle: (state, action: PayloadAction<{ tabId: string; title: string }>) => {
      const { tabId, title } = action.payload;
      const tab = state.tabs.find(t => t.id === tabId);
      if (tab) {
        tab.title = title;
      }
    },

    toggleTabPin: (state, action: PayloadAction<string>) => {
      const tab = state.tabs.find(t => t.id === action.payload);
      if (tab) {
        tab.isPinned = !tab.isPinned;
      }
    },

    updateTabWorkingDirectory: (state, action: PayloadAction<{ tabId: string; cwd: string }>) => {
      const { tabId, cwd } = action.payload;
      const tab = state.tabs.find(t => t.id === tabId);
      if (tab) {
        tab.workingDirectory = cwd;
        tab.aiContext.learningContext.workingDirectory = cwd;
        
        // Add to recent working directories for AI context
        if (!tab.aiContext.recentCommands.includes(`cd ${cwd}`)) {
          tab.aiContext.recentCommands.push(`cd ${cwd}`);
        }
      }
    },

    // Command History
    addCommandToHistory: (state, action: PayloadAction<{ tabId: string; entry: CommandHistoryEntry }>) => {
      const { tabId, entry } = action.payload;
      const tab = state.tabs.find(t => t.id === tabId);
      if (tab) {
        tab.terminalHistory.push(entry);
        tab.aiContext.recentCommands.push(entry.command);
        tab.lastActivity = new Date();
        
        // Keep only last 100 commands
        if (tab.terminalHistory.length > 100) {
          tab.terminalHistory.shift();
        }
        
        // Keep only last 20 commands in AI context
        if (tab.aiContext.recentCommands.length > 20) {
          tab.aiContext.recentCommands.shift();
        }
      }
    },

    // AI Integration
    addAIMessage: (state, action: PayloadAction<{ tabId: string; message: Omit<AIMessage, 'id'> }>) => {
      const { tabId, message } = action.payload;
      const tab = state.tabs.find(t => t.id === tabId);
      if (tab) {
        const aiMessage: AIMessage = {
          ...message,
          id: generateMessageId()
        };
        
        tab.aiConversation.push(aiMessage);
        tab.lastActivity = new Date();
        
        // Keep conversation history manageable
        if (tab.aiConversation.length > 100) {
          // Keep first (system) message and last 99
          const systemMessage = tab.aiConversation[0];
          const recentMessages = tab.aiConversation.slice(-99);
          tab.aiConversation = [systemMessage, ...recentMessages];
        }
      }
    },

    addAISuggestion: (state, action: PayloadAction<{ tabId: string; suggestion: AISuggestion }>) => {
      const { tabId, suggestion } = action.payload;
      const tab = state.tabs.find(t => t.id === tabId);
      if (tab) {
        tab.aiContext.suggestions.push(suggestion);
        
        // Keep only last 10 suggestions
        if (tab.aiContext.suggestions.length > 10) {
          tab.aiContext.suggestions.shift();
        }
      }
    },

    addError: (state, action: PayloadAction<{ tabId: string; error: ErrorContext }>) => {
      const { tabId, error } = action.payload;
      const tab = state.tabs.find(t => t.id === tabId);
      if (tab) {
        tab.aiContext.errors.push(error);
        
        // Keep only last 5 errors
        if (tab.aiContext.errors.length > 5) {
          tab.aiContext.errors.shift();
        }
      }
    },

    addRecentCommand: (state, action: PayloadAction<{ tabId: string; command: string }>) => {
      const { tabId, command } = action.payload;
      const tab = state.tabs.find(t => t.id === tabId);
      if (tab) {
        tab.aiContext.recentCommands.push(command);
        
        // Keep only last 20 commands
        if (tab.aiContext.recentCommands.length > 20) {
          tab.aiContext.recentCommands.shift();
        }
      }
    },

    clearAIConversation: (state, action: PayloadAction<string>) => {
      const tab = state.tabs.find(t => t.id === action.payload);
      if (tab) {
        // Keep system message, clear the rest
        const systemMessage = tab.aiConversation.find(msg => msg.role === 'system');
        tab.aiConversation = systemMessage ? [systemMessage] : [];
      }
    },

    // Global AI Context
    updateGlobalAIContext: (state, action: PayloadAction<{ 
      sharedKnowledge?: Record<string, unknown>;
      crossTabInsights?: string[];
      projectAnalysis?: Record<string, unknown>;
    }>) => {
      const { sharedKnowledge, crossTabInsights, projectAnalysis } = action.payload;
      
      if (sharedKnowledge) {
        state.aiContextGlobal.sharedKnowledge = { ...state.aiContextGlobal.sharedKnowledge, ...sharedKnowledge };
      }
      
      if (crossTabInsights) {
        state.aiContextGlobal.crossTabInsights = crossTabInsights;
      }
      
      if (projectAnalysis) {
        state.aiContextGlobal.projectAnalysis = { ...state.aiContextGlobal.projectAnalysis, ...projectAnalysis };
      }
    },

    // Tab Creation State
    setCreatingTab: (state, action: PayloadAction<boolean>) => {
      state.isCreatingTab = action.payload;
    },

    // Performance Monitoring
    updateTabMemoryUsage: (state, action: PayloadAction<{ tabId: string; memoryMB: number }>) => {
      const { tabId, memoryMB } = action.payload;
      state.performanceMetrics.memoryUsage[tabId] = memoryMB;
    },

    optimizeTabMemory: (state) => {
      const now = Date.now();
      
      // Clean up old performance metrics
      if (now - state.performanceMetrics.lastOptimization > 300000) { // 5 minutes
        state.performanceMetrics.tabSwitchTimes = state.performanceMetrics.tabSwitchTimes.slice(-5);
        
        // Clean up old AI conversations for inactive tabs
        state.tabs.forEach(tab => {
          if (!tab.isActive && tab.aiConversation.length > 20) {
            const systemMessage = tab.aiConversation.find(msg => msg.role === 'system');
            const recentMessages = tab.aiConversation.slice(-10);
            tab.aiConversation = systemMessage ? [systemMessage, ...recentMessages] : recentMessages;
          }
        });
        
        state.performanceMetrics.lastOptimization = now;
      }
    },

    // Keyboard Shortcuts
    switchToTabByIndex: (state, action: PayloadAction<number>) => {
      const index = action.payload;
      if (index >= 0 && index < state.tabs.length) {
        const targetTab = state.tabs[index];
        // Reuse existing setActiveTab logic
        state.tabs.forEach(tab => { tab.isActive = false; });
        targetTab.isActive = true;
        targetTab.lastActivity = new Date();
        state.activeTabId = targetTab.id;
      }
    },

    switchToNextTab: (state) => {
      if (state.tabs.length > 1 && state.activeTabId) {
        const currentIndex = state.tabs.findIndex(tab => tab.id === state.activeTabId);
        const nextIndex = (currentIndex + 1) % state.tabs.length;
        const nextTab = state.tabs[nextIndex];
        
        state.tabs.forEach(tab => { tab.isActive = false; });
        nextTab.isActive = true;
        nextTab.lastActivity = new Date();
        state.activeTabId = nextTab.id;
      }
    },

    switchToPrevTab: (state) => {
      if (state.tabs.length > 1 && state.activeTabId) {
        const currentIndex = state.tabs.findIndex(tab => tab.id === state.activeTabId);
        const prevIndex = (currentIndex - 1 + state.tabs.length) % state.tabs.length;
        const prevTab = state.tabs[prevIndex];
        
        state.tabs.forEach(tab => { tab.isActive = false; });
        prevTab.isActive = true;
        prevTab.lastActivity = new Date();
        state.activeTabId = prevTab.id;
      }
    }
  }
});

export const {
  createTab,
  updateTabTerminalId,
  setActiveTab,
  closeTab,
  startTabDrag,
  updateTabDrag,
  endTabDrag,
  cancelTabDrag,
  updateTabTitle,
  toggleTabPin,
  updateTabWorkingDirectory,
  addCommandToHistory,
  addAIMessage,
  addAISuggestion,
  addError,
  addRecentCommand,
  clearAIConversation,
  updateGlobalAIContext,
  setCreatingTab,
  updateTabMemoryUsage,
  optimizeTabMemory,
  switchToTabByIndex,
  switchToNextTab,
  switchToPrevTab
} = terminalTabSlice.actions;

// Selectors
export const selectAllTabs = (state: { terminalTabs: TerminalTabState }) => state.terminalTabs.tabs;
export const selectActiveTab = (state: { terminalTabs: TerminalTabState }) => 
  state.terminalTabs.tabs.find(tab => tab.id === state.terminalTabs.activeTabId);
export const selectTabById = (tabId: string) => (state: { terminalTabs: TerminalTabState }) =>
  state.terminalTabs.tabs.find(tab => tab.id === tabId);
export const selectTabsByShell = (shell: ShellType) => (state: { terminalTabs: TerminalTabState }) =>
  state.terminalTabs.tabs.filter(tab => tab.shell === shell);
export const selectPinnedTabs = (state: { terminalTabs: TerminalTabState }) =>
  state.terminalTabs.tabs.filter(tab => tab.isPinned);
export const selectTabsWithErrors = (state: { terminalTabs: TerminalTabState }) =>
  state.terminalTabs.tabs.filter(tab => tab.aiContext.errors.length > 0);
export const selectTabsWithSuggestions = (state: { terminalTabs: TerminalTabState }) =>
  state.terminalTabs.tabs.filter(tab => tab.aiContext.suggestions.length > 0);

export default terminalTabSlice.reducer;
