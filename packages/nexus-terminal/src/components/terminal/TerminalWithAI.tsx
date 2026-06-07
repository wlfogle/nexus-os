import React, { useEffect, useRef, useState, useMemo } from 'react';
import { useDispatch } from 'react-redux';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TerminalTab } from '../../types/terminal';
import { addError, addTerminalBlock } from '../../store/slices/terminalTabSlice';
import EnhancedAIAssistant from '../ai/EnhancedAIAssistant';
import { useInputRouting } from '../../hooks/useInputRouting';
import { terminalLogger } from '../../utils/logger';
import '@xterm/xterm/css/xterm.css';
interface TerminalWithAIProps {
  tab: TerminalTab;
}

export const TerminalWithAI: React.FC<TerminalWithAIProps> = ({ tab }) => {
  terminalLogger.debug('TerminalWithAI rendering', 'component_render', { tabId: tab.id });
  const dispatch = useDispatch();
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminal = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const [aiPanelOpen, setAIPanelOpen] = useState(true); // Start in AI mode by default
  const [isTerminalReady, setIsTerminalReady] = useState(false);
  const [inputBuffer, setInputBuffer] = useState('');

  // Centralized routing: differentiates shell commands from natural language
  const { handleInput, isShellCommand } = useInputRouting();

  // Memoize terminal theme based on shell type
  const terminalTheme = useMemo(() => {
    const baseTheme = {
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
    };

    // Customize theme based on shell type
    switch (tab.shell) {
      case 'fish':
        return { ...baseTheme, blue: '#00ADD8', cyan: '#00ADD8' };
      case 'zsh':
        return { ...baseTheme, green: '#F15A29', brightGreen: '#F15A29' };
      case 'powershell':
        return { ...baseTheme, blue: '#012456', brightBlue: '#5391FE' };
      default:
        return baseTheme;
    }
  }, [tab.shell]);

  // Memoize terminal options
  const terminalOptions = useMemo(() => ({
    theme: terminalTheme,
    fontFamily: 'JetBrains Mono, Monaco, Menlo, "Ubuntu Mono", monospace',
    fontSize: 14,
    fontWeight: 'normal' as const,
    lineHeight: 1.2,
    letterSpacing: 0,
    cursorBlink: true,
    cursorStyle: 'block' as const,
    scrollback: 10000,
    tabStopWidth: 4,
    allowProposedApi: true
  }), [terminalTheme]);

  // Initialize terminal
  useEffect(() => {
    if (!terminalRef.current || !tab.terminalId) return;

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

    // ALL keyboard input goes directly to the PTY — terminal is a pure shell
    // Natural language goes to the AI panel's own dedicated input (right side)
    terminal.current.onData(async (data: string) => {
      if (!tab.terminalId) return;
      try {
        await invoke('write_to_terminal', { terminal_id: tab.terminalId, data });
      } catch (error) {
        terminalLogger.error('PTY write failed', error as Error, 'write_terminal_failed', { terminalId: tab.terminalId });
        dispatch(addError({ tabId: tab.id, error: { command: 'write_to_terminal', errorMessage: String(error), timestamp: new Date(), workingDirectory: tab.workingDirectory } }));
      }
    });

    // Welcome message with AI-first greeting
    terminal.current.writeln('🚀 Welcome to NexusTerminal - AI-First Terminal Assistant');
    terminal.current.writeln('🤖 AI Chat Mode is active by default!');
    terminal.current.writeln('💡 Type commands like "ls -la" to execute shell commands');
    terminal.current.writeln('💬 Type questions like "how do I..." for AI assistance');
    terminal.current.writeln(`⚡ Shell: ${getShellWelcomeMessage(tab.shell).replace(/^.+ Welcome to /, '')}`);
    terminal.current.writeln('');

    setIsTerminalReady(true);

    // Listen for terminal output — write to xterm AND parse OSC 133 sequences
    // to record completed commands as TerminalBlocks (same as Warp's BlockContext).
    // OSC 133 sequences injected by terminal.rs:
    //   \x1b]133;A\x07  = prompt start
    //   \x1b]133;B\x07  = command start (user about to type)
    //   \x1b]133;C\x07  = output start (command running)
    //   \x1b]133;D;N\x07 = command end, N = exit code
    let unlistenTerminalOutput: (() => void) | null = null;
    const capturedTerminalId = tab.terminalId;
    const capturedTabId = tab.id;
    const capturedCwd = tab.workingDirectory;

    // OSC 133 parser state
    let osc133State: 'prompt' | 'input' | 'output' = 'prompt';
    let currentCommand = '';
    let outputBuffer = '';

    const OSC_RE = /\x1b\]133;([^\x07]*)\x07/g;

    listen<{ terminal_id: string; data: string }>('terminal-output', (event) => {
      const { terminal_id, data } = event.payload;
      if (terminal_id !== capturedTerminalId || !terminal.current) return;

      // Write raw data to xterm (including OSC sequences — xterm ignores unknown ones)
      terminal.current.write(data);

      // Parse OSC 133 sequences from the raw data stream
      let lastIndex = 0;
      OSC_RE.lastIndex = 0;
      let match;
      while ((match = OSC_RE.exec(data)) !== null) {
        const seq = match[1]; // e.g. 'A', 'B', 'C', 'D;0'
        const textBefore = data.slice(lastIndex, match.index);
        lastIndex = match.index + match[0].length;

        if (osc133State === 'input') {
          // Accumulate raw keystrokes between B and C
          // Strip ANSI escapes for clean command text
          currentCommand += textBefore.replace(/\x1b\[[^A-Za-z]*[A-Za-z]/g, '');
        } else if (osc133State === 'output') {
          outputBuffer += textBefore.replace(/\x1b\[[^A-Za-z]*[A-Za-z]/g, '');
        }

        if (seq === 'A') {
          osc133State = 'prompt';
        } else if (seq === 'B') {
          osc133State = 'input';
          currentCommand = '';
        } else if (seq === 'C') {
          osc133State = 'output';
          outputBuffer = '';
          // Clean up command (remove trailing newline/CR)
          currentCommand = currentCommand.trim();
        } else if (seq.startsWith('D')) {
          // Command finished — record block
          const exitCode = parseInt(seq.split(';')[1] ?? '0', 10);
          const cmd = currentCommand.trim();
          const out = outputBuffer.trim();

          if (cmd) {
            dispatch(addTerminalBlock({
              tabId: capturedTabId,
              block: {
                command: cmd,
                output: out,
                exitCode,
                cwd: capturedCwd,
                timestamp: new Date().toISOString(),
              }
            }));
          }

          osc133State = 'prompt';
          currentCommand = '';
          outputBuffer = '';
        }
      }

      // Accumulate text after last OSC sequence
      const tail = data.slice(lastIndex);
      if (osc133State === 'input') {
        currentCommand += tail.replace(/\x1b\[[^A-Za-z]*[A-Za-z]/g, '');
      } else if (osc133State === 'output') {
        outputBuffer += tail.replace(/\x1b\[[^A-Za-z]*[A-Za-z]/g, '');
      }
    })
      .then((unlisten) => { unlistenTerminalOutput = unlisten; })
      .catch((err) => {
        terminalLogger.error('Failed to set up terminal output listener', err as Error, 'listener_setup_failed', { terminalId: tab.terminalId });
      });

    return () => {
      if (unlistenTerminalOutput) {
        unlistenTerminalOutput();
      }
      if (terminal.current) {
        terminal.current.dispose();
        terminal.current = null;
      }
      setIsTerminalReady(false);
    };
  }, [tab.terminalId, terminalOptions, tab.shell, tab.workingDirectory, tab.id, dispatch, isShellCommand, handleInput]);

  // Handle window resize
  useEffect(() => {
    const handleResize = () => {
      if (fitAddon.current && terminal.current && isTerminalReady) {
        fitAddon.current.fit();
        const { cols, rows } = terminal.current;
        
        if (tab.terminalId) {
          invoke('resize_terminal', { 
            terminal_id: tab.terminalId, 
            cols, 
            rows 
          }).catch(error => {
            terminalLogger.error('Failed to resize terminal', error as Error, 'resize_failed', { terminalId: tab.terminalId });
          });
        }
      }
    };

    window.addEventListener('resize', handleResize);

    return () => window.removeEventListener('resize', handleResize);
  }, [isTerminalReady, tab.terminalId]);

  // Focus terminal when tab becomes active
  useEffect(() => {
    if (terminal.current && tab.isActive && isTerminalReady) {
      terminal.current.focus();
    }
  }, [tab.isActive, isTerminalReady]);

  // Force AI panel to open on mount
  useEffect(() => {
    setAIPanelOpen(true);
  }, []);

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ctrl+Shift+A - Open AI assistant
      if (event.ctrlKey && event.shiftKey && event.key === 'A') {
        event.preventDefault();
        setAIPanelOpen(true);
      }
      // Ctrl+Shift+F - Search in terminal
      else if (event.ctrlKey && event.shiftKey && event.key === 'F') {
        event.preventDefault();
        if (terminal.current) {
          // Note: XTerm.js addons don't provide getAddon method
          // We'll implement custom search functionality if needed
          terminalLogger.info('Search functionality not yet implemented', 'search_requested');
        }
      }
      // Ctrl+Shift+C - Clear terminal
      else if (event.ctrlKey && event.shiftKey && event.key === 'C') {
        event.preventDefault();
        if (terminal.current) {
          terminal.current.clear();
        }
      }
      // Escape - Close AI panel
      else if (event.key === 'Escape' && aiPanelOpen) {
        event.preventDefault();
        setAIPanelOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);


  const getShellWelcomeMessage = (shell: string): string => {
    switch (shell) {
      case 'fish':
        return '🐟 Welcome to Fish Shell - The friendly interactive shell';
      case 'zsh':
        return '⚡ Welcome to Zsh - The powerful Z shell';
      case 'powershell':
        return '💙 Welcome to PowerShell - Object-based shell';
      case 'bash':
      default:
        return '🐚 Welcome to Bash - The Bourne Again Shell';
    }
  };

  const getQuickActions = () => {
    const actions = [
      {
        icon: '🤖',
        label: 'Ask AI',
        shortcut: 'Ctrl+Shift+A',
        onClick: () => setAIPanelOpen(true),
        highlight: tab.aiContext.errors.length > 0 || tab.aiContext.suggestions.length > 0
      }
    ];

    if (tab.aiContext.errors.length > 0) {
      actions.push({
        icon: '🚨',
        label: `Fix ${tab.aiContext.errors.length} Error${tab.aiContext.errors.length > 1 ? 's' : ''}`,
        shortcut: '',
        onClick: () => {
          // Just open the AI panel - the enhanced assistant will handle error context
          setAIPanelOpen(true);
        },
        highlight: true
      });
    }

    if (tab.aiContext.suggestions.length > 0) {
      actions.push({
        icon: '💡',
        label: `${tab.aiContext.suggestions.length} Suggestion${tab.aiContext.suggestions.length > 1 ? 's' : ''}`,
        shortcut: '',
        onClick: () => setAIPanelOpen(true),
        highlight: true
      });
    }

    return actions;
  };

  return (
    <div className="flex h-full bg-gray-900 overflow-hidden">

      {/* ── LEFT: Pure PTY Terminal ──────────────────────────────────── */}
      <div className="flex-1 relative min-w-0">
        {/* hidden — real layout continues below */}
        {/* XTerm.js Container */}
        <div 
          ref={terminalRef}
          className="absolute inset-0 p-4"
          style={{ 
            backgroundColor: terminalTheme.background,
            fontFamily: terminalOptions.fontFamily 
          }}
          onDragOver={(e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = 'copy';
          }}
          onDrop={(e) => {
            e.preventDefault();
            const files = Array.from(e.dataTransfer.files);

            if (files.length > 0 && terminal.current) {
              const paths = files.map(f => `"${(f as any).path || f.name}"`).join(' ');

              terminal.current.write(paths);
            }
          }}
        />
        
        {/* Quick Actions Overlay */}
        <div className="absolute bottom-4 right-4 flex flex-col space-y-2">
          {getQuickActions().map((action, index) => (
            <button
              key={index}
              onClick={action.onClick}
              title={`${action.label} ${action.shortcut ? `(${action.shortcut})` : ''}`}
              className={`
                flex items-center px-3 py-2 rounded-lg shadow-lg transition-all duration-200 text-sm
                ${action.highlight 
                  ? 'bg-blue-600 text-white hover:bg-blue-700' 
                  : 'bg-gray-800/90 text-gray-200 hover:bg-gray-700'
                }
                backdrop-blur-sm hover:scale-105 hover:shadow-xl
              `}
            >
              <span className="mr-2">{action.icon}</span>
              {action.label}
            </button>
          ))}
        </div>

        {/* Loading overlay */}
        {!isTerminalReady && (
          <div className="absolute inset-0 bg-gray-900 flex items-center justify-center">
            <div className="text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mb-4"></div>
              <div className="text-gray-400">Initializing {tab.shell} terminal...</div>
            </div>
          </div>
        )}
      </div>

      {/* AI-First Input Interface */}
      {!aiPanelOpen && (
        <div className="bg-gray-800 border-t border-gray-700 p-4">
          <div className="max-w-4xl mx-auto">
            <div className="flex items-center gap-4 mb-3">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
                <span className="text-sm text-gray-300 font-medium">AI-First Mode</span>
              </div>
              <button
                onClick={() => setAIPanelOpen(true)}
                className="text-xs text-blue-400 hover:text-blue-300 transition-colors"
              >
                Open Full Assistant
              </button>
            </div>
            <div className="flex gap-3">
              <input
                type="text"
                value={inputBuffer}
                onChange={(e) => setInputBuffer(e.target.value)}
              onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    if (inputBuffer.trim() && !isShellCommand(inputBuffer)) {
                      setAIPanelOpen(true);
                    }
                    handleInput(inputBuffer);
                    setInputBuffer('');
                  }
                }}
                placeholder="Type commands (ls -la) or ask AI questions (how do I...)"
                className="flex-1 px-4 py-2 bg-gray-900 text-white border border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400"
              />
              <button
              onClick={() => {
                  if (inputBuffer.trim()) {
                    if (!isShellCommand(inputBuffer)) {
                      setAIPanelOpen(true);
                    }
                    handleInput(inputBuffer);
                    setInputBuffer('');
                  }
                }}
                className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors"
              >
                Send
              </button>
            </div>
            <div className="text-xs text-gray-400 mt-2">
              💡 Smart routing: Shell commands go to terminal, questions go to AI
            </div>
          </div>
        </div>
      )}

      {/* AI Assistant Panel */}
      {aiPanelOpen && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg shadow-2xl w-full max-w-6xl mx-4 h-[90vh] flex flex-col">
            {/* Window Controls Header */}
            <div className="flex items-center justify-between p-4 border-b border-gray-700 bg-gray-800 rounded-t-lg">
              <div className="flex items-center gap-3">
                <h3 className="text-lg font-semibold text-white">🤖 NexusTerminal AI Assistant</h3>
                <div className="text-xs text-gray-400 bg-gray-700 px-2 py-1 rounded">
                  AI-First Mode Active
                </div>
              </div>
              <div className="flex items-center gap-2">
                {/* Minimize button */}
                <button
                  onClick={() => setAIPanelOpen(false)}
                  className="w-6 h-6 rounded-full bg-yellow-500 hover:bg-yellow-600 flex items-center justify-center transition-colors"
                  title="Minimize (Esc)"
                >
                  <span className="text-xs text-black font-bold">−</span>
                </button>
                {/* Maximize button */}
                <button
                  onClick={() => {/* Toggle fullscreen logic could go here */}}
                  className="w-6 h-6 rounded-full bg-green-500 hover:bg-green-600 flex items-center justify-center transition-colors"
                  title="Maximize"
                >
                  <span className="text-xs text-black font-bold">□</span>
                </button>
                {/* Close button */}
                <button
                  onClick={() => setAIPanelOpen(false)}
                  className="w-6 h-6 rounded-full bg-red-500 hover:bg-red-600 flex items-center justify-center transition-colors"
                  title="Close"
                >
                  <span className="text-xs text-white font-bold">✕</span>
                </button>
              </div>
            </div>
            {/* AI Assistant Content with enforced scrolling */}
            <div 
              className="flex-1" 
              style={{
                overflow: 'auto',
                overflowY: 'scroll',
                scrollbarWidth: 'auto',
                scrollbarColor: '#6B7280 #374151'
              }}
            >
              <EnhancedAIAssistant onSwitchToTerminal={() => setAIPanelOpen(false)} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
