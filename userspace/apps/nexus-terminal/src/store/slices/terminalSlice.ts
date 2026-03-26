import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface TerminalSession {
  id: string;
  title: string;
  shell: string;
  cwd: string;
  createdAt: string;
  isActive: boolean;
  history: string[];
}

interface TerminalOutput {
  id: string;
  terminalId: string;
  data: string;
  timestamp: number;
  type: 'stdout' | 'stderr' | 'stdin';
}

// Circular buffer implementation for terminal output
class CircularBuffer<T> {
  private buffer: (T | undefined)[];
  private head = 0;
  private tail = 0;
  private size = 0;
  private readonly maxSize: number;

  constructor(maxSize: number) {
    this.maxSize = maxSize;
    this.buffer = new Array(maxSize);
  }

  push(item: T): void {
    this.buffer[this.tail] = item;
    this.tail = (this.tail + 1) % this.maxSize;
    
    if (this.size < this.maxSize) {
      this.size++;
    } else {
      // Buffer is full, advance head
      this.head = (this.head + 1) % this.maxSize;
    }
  }

  getAll(): T[] {
    const result: T[] = [];
    for (let i = 0; i < this.size; i++) {
      const index = (this.head + i) % this.maxSize;
      const item = this.buffer[index];
      if (item !== undefined) {
        result.push(item);
      }
    }
    return result;
  }

  getLastN(n: number): T[] {
    const all = this.getAll();
    return all.slice(Math.max(0, all.length - n));
  }

  clear(): void {
    this.buffer = new Array(this.maxSize);
    this.head = 0;
    this.tail = 0;
    this.size = 0;
  }

  getCurrentSize(): number {
    return this.size;
  }

  getMaxSize(): number {
    return this.maxSize;
  }
}

interface TerminalState {
  terminals: Record<string, TerminalSession>;
  activeTerminalId: string | null;
  outputBuffers: Record<string, CircularBuffer<TerminalOutput>>;
  isCreatingTerminal: boolean;
  isLoading: boolean;
  error: string | null;
  connectionStatus: 'connected' | 'connecting' | 'disconnected';
  memoryStats: {
    totalOutputs: number;
    bufferSizes: Record<string, number>;
    lastCleanup: number;
  };
}

// Memory management constants
const TERMINAL_OUTPUT_LIMIT = 5000; // Per terminal
const GLOBAL_OUTPUT_LIMIT = 25000; // Total across all terminals
const CLEANUP_INTERVAL = 60000; // 1 minute

const initialState: TerminalState = {
  terminals: {},
  activeTerminalId: null,
  outputBuffers: {},
  isCreatingTerminal: false,
  isLoading: false,
  error: null,
  connectionStatus: 'disconnected',
  memoryStats: {
    totalOutputs: 0,
    bufferSizes: {},
    lastCleanup: Date.now(),
  },
};

const terminalSlice = createSlice({
  name: 'terminal',
  initialState,
  reducers: {
    createTerminal: (state, action: PayloadAction<{ id: string; title?: string; shell?: string; cwd?: string }>) => {
      const { id, title, shell = 'bash', cwd = '~' } = action.payload;
      const terminalTitle = title || `Terminal ${Object.keys(state.terminals).length + 1}`;
      
      state.terminals[id] = {
        id,
        title: terminalTitle,
        shell,
        cwd,
        createdAt: new Date().toISOString(),
        isActive: false,
        history: [],
      };
      
      // Deactivate other terminals if this is the first or user wants it active
      if (!state.activeTerminalId || action.payload.title) {
        Object.keys(state.terminals).forEach(termId => {
          if (termId !== id) {
            state.terminals[termId].isActive = false;
          }
        });
        state.terminals[id].isActive = true;
        state.activeTerminalId = id;
      }
      
      state.isCreatingTerminal = false;
      state.error = null;
    },
    setActiveTerminal: (state, action: PayloadAction<string>) => {
      // Deactivate all terminals
      Object.values(state.terminals).forEach(terminal => {
        terminal.isActive = false;
      });
      // Activate the selected terminal
      if (state.terminals[action.payload]) {
        state.terminals[action.payload].isActive = true;
        state.activeTerminalId = action.payload;
      }
    },
    addOutput: (state, action: PayloadAction<{ terminalId: string; output: string }>) => {
      const { terminalId, output } = action.payload;
      if (state.terminals[terminalId]) {
        state.terminals[terminalId].history.push(output);
      }
    },
    closeTerminal: (state, action: PayloadAction<string>) => {
      const terminalId = action.payload;
      delete state.terminals[terminalId];
      
      // Clean up terminal's output buffer
      if (state.outputBuffers[terminalId]) {
        delete state.outputBuffers[terminalId];
        delete state.memoryStats.bufferSizes[terminalId];
      }
      
      if (state.activeTerminalId === terminalId) {
        const remainingIds = Object.keys(state.terminals);
        state.activeTerminalId = remainingIds.length > 0 ? remainingIds[0] : null;
        if (state.activeTerminalId) {
          state.terminals[state.activeTerminalId].isActive = true;
        }
      }
    },
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload;
    },
    
    addTerminalOutput: (state, action: PayloadAction<Omit<TerminalOutput, 'id' | 'timestamp'>>) => {
      const output: TerminalOutput = {
        ...action.payload,
        id: `output-${Date.now()}-${Math.random()}`,
        timestamp: Date.now(),
      };
      
      const { terminalId } = output;
      
      // Initialize circular buffer for terminal if it doesn't exist
      if (!state.outputBuffers[terminalId]) {
        state.outputBuffers[terminalId] = new CircularBuffer(TERMINAL_OUTPUT_LIMIT);
      }
      
      // Add output to terminal's circular buffer
      state.outputBuffers[terminalId].push(output);
      
      // Update memory stats
      state.memoryStats.totalOutputs++;
      state.memoryStats.bufferSizes[terminalId] = state.outputBuffers[terminalId].getCurrentSize();
      
      // Check if global cleanup is needed
      const now = Date.now();
      const shouldCleanup = 
        now - state.memoryStats.lastCleanup > CLEANUP_INTERVAL ||
        state.memoryStats.totalOutputs > GLOBAL_OUTPUT_LIMIT;
        
      if (shouldCleanup) {
        // Cleanup old outputs across all terminals
        let totalOutputsAfterCleanup = 0;
        Object.entries(state.outputBuffers).forEach(([termId, buffer]) => {
          if (buffer instanceof CircularBuffer) {
            totalOutputsAfterCleanup += buffer.getCurrentSize();
            state.memoryStats.bufferSizes[termId] = buffer.getCurrentSize();
          }
        });
        
        state.memoryStats.totalOutputs = totalOutputsAfterCleanup;
        state.memoryStats.lastCleanup = now;
      }
    },
    
    updateTerminalCwd: (state, action: PayloadAction<{ id: string; cwd: string }>) => {
      const { id, cwd } = action.payload;
      if (state.terminals[id]) {
        state.terminals[id].cwd = cwd;
      }
    },
    
    updateTerminalTitle: (state, action: PayloadAction<{ id: string; title: string }>) => {
      const { id, title } = action.payload;
      if (state.terminals[id]) {
        state.terminals[id].title = title;
      }
    },
    
    setCreatingTerminal: (state, action: PayloadAction<boolean>) => {
      state.isCreatingTerminal = action.payload;
    },
    
    setConnectionStatus: (state, action: PayloadAction<'connected' | 'connecting' | 'disconnected'>) => {
      state.connectionStatus = action.payload;
    },
    
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
    
    clearOutput: (state, action: PayloadAction<string>) => {
      const terminalId = action.payload;
      if (state.outputBuffers[terminalId]) {
        state.outputBuffers[terminalId].clear();
        state.memoryStats.bufferSizes[terminalId] = 0;
      }
    },
    
    // New actions for memory management
    forceMemoryCleanup: (state) => {
      // Clean up all terminal buffers to free memory
      Object.keys(state.outputBuffers).forEach(terminalId => {
        const buffer = state.outputBuffers[terminalId];
        if (buffer instanceof CircularBuffer) {
          // Keep only last 1000 outputs for each terminal during cleanup
          const currentOutputs = buffer.getAll();
          const keepOutputs = currentOutputs.slice(-1000);
          
          buffer.clear();
          keepOutputs.forEach(output => buffer.push(output));
          
          state.memoryStats.bufferSizes[terminalId] = buffer.getCurrentSize();
        }
      });
      
      // Recalculate total
      state.memoryStats.totalOutputs = Object.values(state.memoryStats.bufferSizes)
        .reduce((sum, size) => sum + size, 0);
      state.memoryStats.lastCleanup = Date.now();
    },
  },
});

export const { 
  createTerminal, 
  setActiveTerminal, 
  addOutput, 
  addTerminalOutput,
  closeTerminal, 
  setLoading,
  updateTerminalCwd,
  updateTerminalTitle,
  setCreatingTerminal,
  setConnectionStatus,
  setError,
  clearOutput,
  forceMemoryCleanup
} = terminalSlice.actions;

// Selectors
export const selectActiveTerminal = (state: { terminal: TerminalState }) => {
  const { activeTerminalId, terminals } = state.terminal;
  return activeTerminalId ? terminals[activeTerminalId] : null;
};

export const selectTerminalOutput = (terminalId: string) => (state: { terminal: TerminalState }) => {
  const buffer = state.terminal.outputBuffers[terminalId];
  return buffer ? buffer.getAll() : [];
};

export const selectTerminalOutputLast = (terminalId: string, count: number) => (state: { terminal: TerminalState }) => {
  const buffer = state.terminal.outputBuffers[terminalId];
  return buffer ? buffer.getLastN(count) : [];
};

export const selectMemoryStats = (state: { terminal: TerminalState }) => {
  return {
    ...state.terminal.memoryStats,
    terminalCount: Object.keys(state.terminal.terminals).length,
    bufferCount: Object.keys(state.terminal.outputBuffers).length,
  };
};

export const selectAllTerminals = (state: { terminal: TerminalState }) => {
  return Object.values(state.terminal.terminals);
};

export default terminalSlice.reducer;
