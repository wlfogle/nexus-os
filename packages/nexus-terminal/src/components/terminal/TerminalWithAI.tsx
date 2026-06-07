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

  // Keep focus on unified input, not xterm
  // xterm is display-only — input goes through the unified bar
  useEffect(() => {
    if (isTerminalReady) {
      unifiedInputRef.current?.focus();
    }
  }, [isTerminalReady]);

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

  // ── Unified input state ────────────────────────────────────────────────────
  const [unifiedInput, setUnifiedInput] = useState('');
  const [inputMode, setInputMode] = useState<'shell' | 'ai' | 'detecting'>('detecting');
  const [isAILoading, setIsAILoading] = useState(false);
  const [aiBlocks, setAiBlocks] = useState<Array<{ id: string; role: string; content: string; streaming?: string; tools?: Array<{tool: string; result?: string}> }>>([]);
  const aiBlocksEndRef = useRef<HTMLDivElement>(null);
  const unifiedInputRef = useRef<HTMLInputElement>(null);
  const classifyDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Classify input as user types — live mode indicator like Warp
  const handleUnifiedInputChange = (value: string) => {
    setUnifiedInput(value);

    // Force overrides: ! = shell, * = AI
    if (value.startsWith('!')) { setInputMode('shell'); return; }
    if (value.startsWith('*')) { setInputMode('ai'); return; }
    if (!value.trim()) { setInputMode('detecting'); return; }

    // Debounced Warp classifier
    if (classifyDebounce.current) clearTimeout(classifyDebounce.current);
    classifyDebounce.current = setTimeout(async () => {
      try {
        const result = await invoke<{ input_type: string }>('classify_input', { input: value });
        setInputMode(result.input_type === 'shell' ? 'shell' : 'ai');
      } catch {
        setInputMode(isShellCommand(value) ? 'shell' : 'ai');
      }
    }, 120);
  };

  // Ctrl+I toggles mode
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
        e.preventDefault();
        setInputMode(m => m === 'shell' ? 'ai' : 'shell');
        unifiedInputRef.current?.focus();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // Scroll AI blocks to bottom
  useEffect(() => {
    aiBlocksEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [aiBlocks]);

  // Submit unified input
  const handleUnifiedSubmit = async () => {
    let text = unifiedInput.trim();
    if (!text) return;

    // Strip force-override prefixes
    const forceShell = text.startsWith('!');
    const forceAI = text.startsWith('*');
    if (forceShell) text = text.slice(1).trim();
    if (forceAI) text = text.slice(1).trim();
    if (!text) return;

    setUnifiedInput('');
    setInputMode('detecting');

    const isShell = forceShell || (!forceAI && inputMode === 'shell');

    if (isShell) {
      // Execute in PTY
      if (tab.terminalId) {
        await invoke('write_to_terminal', { terminal_id: tab.terminalId, data: text + '\r' });
      }
    } else {
      // Send to AI agent — show in AI blocks panel
      const msgId = `msg_${Date.now()}`;
      setAiBlocks(prev => [...prev, { id: msgId, role: 'user', content: text }]);
      setIsAILoading(true);

      const sessionId = `session_${Date.now()}`;
      const streamId = `stream_${Date.now()}`;
      let streamBuffer = '';
      const localTools: Array<{ tool: string; args: string; result?: string }> = [];

      setAiBlocks(prev => [...prev, { id: streamId, role: 'assistant', content: '', streaming: '' }]);

      const u1 = await listen<{ session_id: string; token: string }>('agent-token', ({ payload }) => {
        if (payload.session_id !== sessionId) return;
        streamBuffer += payload.token;
        setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, streaming: streamBuffer + '█' } : b));
      });
      const u2 = await listen<{ session_id: string; tool: string; args: string }>('agent-tool-call', ({ payload }) => {
        if (payload.session_id !== sessionId) return;
        localTools.push({ tool: payload.tool, args: payload.args });
        setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, tools: [...localTools] } : b));
      });
      const u3 = await listen<{ session_id: string; tool: string; result: string }>('agent-tool-result', ({ payload }) => {
        if (payload.session_id !== sessionId) return;
        const idx = localTools.map(t => t.tool).lastIndexOf(payload.tool);
        if (idx >= 0) localTools[idx].result = payload.result.slice(0, 500);
        setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, tools: [...localTools] } : b));
      });
      const u4 = await listen<{ session_id: string; answer: string }>('agent-done', ({ payload }) => {
        if (payload.session_id !== sessionId) return;
        const toolSection = localTools.length > 0
          ? localTools.map(t => `🔧 ${t.tool}\n\`\`\`\n${(t.result ?? '').slice(0, 800)}\n\`\`\``).join('\n\n')
          : '';
        const final = toolSection && payload.answer.trim()
          ? `${toolSection}\n\n${payload.answer.trim()}`
          : toolSection || payload.answer.trim() || '✓ Done';
        setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, content: final, streaming: undefined, tools: undefined } : b));
        setIsAILoading(false);
        u1(); u2(); u3(); u4(); u5();
      });
      const u5 = await listen<{ session_id: string; error: string }>('agent-error', ({ payload }) => {
        if (payload.session_id !== sessionId) return;
        setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, content: `❌ ${payload.error}`, streaming: undefined } : b));
        setIsAILoading(false);
        u1(); u2(); u3(); u4(); u5();
      });

      // Build block context from recent terminal blocks
      const recentBlocks: any[] = (tab as any).recentBlocks ?? [];
      const failed = recentBlocks.filter((b: any) => b.exitCode !== 0);
      const blockCtx = [...failed, ...recentBlocks.filter((b: any) => b.exitCode === 0)]
        .slice(0, 3)
        .map((b: any) => `$ ${b.command}\n${b.output.slice(0, 1000)}\n(exit ${b.exitCode})`)
        .join('\n\n');
      const context = [
        `Shell: ${tab.shell}`, `Directory: ${tab.workingDirectory}`,
        blockCtx ? `\nRecent terminal output:\n${blockCtx}` : ''
      ].filter(Boolean).join('\n');

      const history = aiBlocks
        .filter(b => b.role === 'user' || b.role === 'assistant')
        .slice(-20)
        .map(b => ({ role: b.role, content: b.content }));

      await invoke('agent_chat_stream', {
        message: text, sessionId,
        history,
        cwd: tab.workingDirectory || null,
        context,
      }).catch(() => {
        setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, content: '❌ Failed to reach agent', streaming: undefined } : b));
        setIsAILoading(false);
      });
    }
  };

  const modeColor = inputMode === 'shell' ? 'text-green-400 border-green-500/40' :
                    inputMode === 'ai'    ? 'text-purple-400 border-purple-500/40' :
                                           'text-gray-400 border-gray-600';
  const modeLabel = inputMode === 'shell' ? '🖵  SHELL' :
                    inputMode === 'ai'    ? '🤖 NEXUSAI' : '···';

  return (
    <div className="flex flex-col h-full bg-[#0d0d0d] overflow-hidden">

      {/* ── Header bar: shell info + model ─────────────────────────── */}
      <div className="flex-shrink-0 flex items-center justify-between px-4 py-1.5 bg-[#1a1a1a] border-b border-gray-800 text-xs text-gray-400">
        <span className="font-mono">{tab.shell} — {tab.workingDirectory}</span>
        <div className="flex items-center gap-3">
          <span className="text-gray-500">llama3.1:8b</span>
          <span className="text-gray-600">Ctrl+I to toggle mode</span>
        </div>
      </div>

      {/* ── Terminal output (xterm.js) ───────────────────────────────── */}
      <div className="relative" style={{ flex: aiBlocks.length > 0 ? '0 0 55%' : '1 1 auto' }}>
        <div
          ref={terminalRef}
          className="absolute inset-0"
          style={{ backgroundColor: terminalTheme.background }}
        />
        {!isTerminalReady && (
          <div className="absolute inset-0 bg-[#0d0d0d] flex items-center justify-center">
            <div className="text-gray-500 text-sm">Starting {tab.shell}…</div>
          </div>
        )}
      </div>

      {/* ── AI blocks (appear below terminal when AI is used) ─────────── */}
      {aiBlocks.length > 0 && (
        <div className="flex-1 overflow-y-auto bg-[#111] border-t border-gray-800 px-4 py-3 space-y-3" style={{ minHeight: 0 }}>
          {aiBlocks.map(block => (
            <div key={block.id} className={`flex ${ block.role === 'user' ? 'justify-end' : 'justify-start' }`}>
              <div className={`max-w-[85%] rounded-lg px-4 py-2 text-sm font-mono whitespace-pre-wrap break-words
                ${ block.role === 'user'
                  ? 'bg-blue-900/60 text-blue-100'
                  : 'bg-[#1e1e1e] text-gray-200 border border-gray-700' }`}
              >
                {block.role !== 'user' && (
                  <div className="text-xs text-purple-400 mb-1 font-sans">🤖 NexusAI</div>
                )}
                {/* Live streaming */}
                {block.streaming !== undefined ? (
                  <span>{block.streaming}</span>
                ) : block.content}
                {/* Live tool calls */}
                {block.tools && block.tools.length > 0 && (
                  <div className="mt-2 space-y-1">
                    {block.tools.map((t, i) => (
                      <div key={i} className="text-xs border border-gray-700 rounded p-1">
                        <span className="text-yellow-400">🔧 {t.tool}</span>
                        {t.result && <div className="text-green-400 mt-0.5">{t.result.slice(0, 200)}</div>}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ))}
          {isAILoading && (
            <div className="flex justify-start">
              <div className="bg-[#1e1e1e] border border-gray-700 rounded-lg px-4 py-2 text-xs text-purple-400 animate-pulse">
                🤖 NexusAI thinking…
              </div>
            </div>
          )}
          <div ref={aiBlocksEndRef} />
        </div>
      )}

      {/* ── Unified input — single box, Warp-style ─────────────────────── */}
      <div className={`flex-shrink-0 border-t px-3 py-2 bg-[#1a1a1a] flex items-center gap-2 ${modeColor}`}>
        {/* Mode badge */}
        <button
          onClick={() => setInputMode(m => m === 'shell' ? 'ai' : 'shell')}
          className={`flex-shrink-0 text-xs font-mono px-2 py-1 rounded border ${modeColor} bg-transparent select-none cursor-pointer hover:opacity-80`}
          title="Ctrl+I to toggle. ! prefix forces shell, * prefix forces AI."
        >
          {modeLabel}
        </button>

        <input
          ref={unifiedInputRef}
          type="text"
          value={unifiedInput}
          onChange={e => handleUnifiedInputChange(e.target.value)}
          onKeyDown={e => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              handleUnifiedSubmit();
            }
          }}
          placeholder={inputMode === 'shell'
            ? 'Shell command…'
            : inputMode === 'ai'
            ? 'Ask NexusAI…'
            : 'Type a command or ask AI…  (! = shell, * = AI, Ctrl+I = toggle)'}
          className="flex-1 bg-transparent text-white text-sm font-mono outline-none placeholder-gray-600"
          autoFocus
          disabled={isAILoading}
        />

        <button
          onClick={handleUnifiedSubmit}
          disabled={!unifiedInput.trim() || isAILoading}
          className="flex-shrink-0 px-3 py-1 text-xs rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-30 text-white transition-colors"
        >
          ⏎
        </button>
      </div>
    </div>
  );
};
