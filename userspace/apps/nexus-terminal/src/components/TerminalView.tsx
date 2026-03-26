import React, { useEffect, useRef, useCallback, useMemo } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useDispatch, useSelector } from 'react-redux';
import { RootState } from '../store';
import { createTerminal, addTerminalOutput, setConnectionStatus } from '../store/slices/terminalSlice';
import '@xterm/xterm/css/xterm.css';

// Memoized terminal header component
const TerminalHeader = React.memo<{
  activeTerminalId: string | null;
  terminals: Record<string, any>;
}>(({ activeTerminalId, terminals }) => (
  <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
    <div className="flex items-center space-x-2">
      <div className="flex space-x-1">
        <div className="w-3 h-3 bg-red-500 rounded-full"></div>
        <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
        <div className="w-3 h-3 bg-green-500 rounded-full"></div>
      </div>
      <span className="text-sm text-gray-300 ml-4">
        {activeTerminalId ? terminals[activeTerminalId]?.title : 'Terminal'}
      </span>
    </div>
    <div className="flex items-center space-x-2 text-xs text-gray-500">
      <span>🤖 AI Ready</span>
      <span>•</span>
      <span>⚡ Ollama Connected</span>
    </div>
  </div>
));

TerminalHeader.displayName = 'TerminalHeader';

const TerminalView: React.FC = () => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminal = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const dispatch = useDispatch();
  const { activeTerminalId, terminals } = useSelector((state: RootState) => state.terminal);

  // Memoize terminal theme configuration
  const terminalTheme = useMemo(() => ({
    background: '#1a1a1a',
    foreground: '#ffffff',
    cursor: '#ffffff',
    cursorAccent: '#000000',
    selectionBackground: '#ffffff40',
    black: '#000000',
    red: '#ff5555',
    green: '#50fa7b',
    yellow: '#f1fa8c',
    blue: '#bd93f9',
    magenta: '#ff79c6',
    cyan: '#8be9fd',
    white: '#bfbfbf',
    brightBlack: '#4d4d4d',
    brightRed: '#ff6e6e',
    brightGreen: '#69ff94',
    brightYellow: '#ffffa5',
    brightBlue: '#d6acff',
    brightMagenta: '#ff92df',
    brightCyan: '#a4ffff',
    brightWhite: '#ffffff',
  }), []);

  // Memoize terminal options
  const terminalOptions = useMemo(() => ({
    theme: terminalTheme,
    fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
    fontSize: 14,
    fontWeight: 'normal' as const,
    lineHeight: 1.2,
    letterSpacing: 0,
    cursorBlink: true,
    cursorStyle: 'block' as const,
    scrollback: 10000,
    tabStopWidth: 4,
  }), [terminalTheme]);

  // Memoized callback for terminal data handling
  const handleTerminalData = useCallback(async (data: string, terminalId: string) => {
    try {
      await invoke('write_to_terminal', { terminalId, data });
    } catch (error) {
      console.error('Failed to write to terminal:', error);
    }
  }, []);

  // Memoized callback for window resize
  const handleResize = useCallback(() => {
    fitAddon.current?.fit();
    if (activeTerminalId && terminal.current) {
      const { cols, rows } = terminal.current;
      invoke('resize_terminal', { terminalId: activeTerminalId, cols, rows });
    }
  }, [activeTerminalId]);

  useEffect(() => {
    if (terminalRef.current) {
      // Create terminal instance
      terminal.current = new Terminal(terminalOptions);

      // Add addons
      fitAddon.current = new FitAddon();
      terminal.current.loadAddon(fitAddon.current);
      terminal.current.loadAddon(new WebLinksAddon());
      terminal.current.loadAddon(new SearchAddon());

      // Open terminal
      terminal.current.open(terminalRef.current);
      fitAddon.current.fit();

      // Create terminal session
      const initializeTerminal = async () => {
        try {
          const terminalId = await invoke<string>('create_terminal', { shell: null });
          dispatch(createTerminal({ id: terminalId }));

          // Handle terminal data
          terminal.current?.onData((data: string) => {
            handleTerminalData(data, terminalId);
          });

          // Welcome message
          terminal.current?.writeln('🔗 Welcome to NexusTerminal!');
          terminal.current?.writeln('🧠 Where AI meets the command line');
          terminal.current?.writeln('');

        } catch (error) {
          console.error('Failed to initialize terminal:', error);
          terminal.current?.writeln('❌ Failed to initialize terminal backend');
        }
      };

      initializeTerminal();

      // Listen for terminal output events from backend
      const setupEventListeners = async () => {
        const unlisten = await listen<{terminal_id: string, data: string}>('terminal-output', (event) => {
          const { terminal_id, data } = event.payload;
          
          // Write output to the correct terminal
          if (terminal_id === activeTerminalId && terminal.current) {
            terminal.current.write(data);
          }
          
          // Store output in Redux for history
          dispatch(addTerminalOutput({
            terminalId: terminal_id,
            data: data,
            type: 'stdout'
          }));
        });
        
        return unlisten;
      };
      
      let unlistenTerminalOutput: (() => void) | null = null;
      setupEventListeners().then((unlisten) => {
        unlistenTerminalOutput = unlisten;
      });

      // Handle window resize

      window.addEventListener('resize', handleResize);
      
      // Set connection status
      dispatch(setConnectionStatus('connected'));

      return () => {
        window.removeEventListener('resize', handleResize);
        if (unlistenTerminalOutput) {
          unlistenTerminalOutput();
        }
        terminal.current?.dispose();
        dispatch(setConnectionStatus('disconnected'));
      };
    }
  }, []);

  const focusTerminal = useCallback(() => {
    terminal.current?.focus();
  }, []);

  useEffect(() => {
    // Focus terminal when component mounts or becomes active
    focusTerminal();
  }, [activeTerminalId, focusTerminal]);

  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Terminal Header */}
      <TerminalHeader 
        activeTerminalId={activeTerminalId} 
        terminals={terminals} 
      />

      {/* Terminal Content */}
      <div className="flex-1 p-4">
        <div 
          ref={terminalRef} 
          className="w-full h-full rounded-lg border border-gray-700 forced-scrollbars"
          style={{ 
            backgroundColor: '#1a1a1a',
            overflowY: 'scroll',
            scrollbarWidth: 'thin',
            scrollbarColor: '#4a5568 #2d3748'
          }}
        />
      </div>
    </div>
  );
};

export default TerminalView;
