import React, { useEffect, useRef, useState, useMemo } from 'react';
import { useDispatch } from 'react-redux';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TerminalTab } from '../../types/terminal';
import { addError, addTerminalBlock, updateTabWorkingDirectory } from '../../store/slices/terminalTabSlice';
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

    // Open terminal — defer fit so flex layout has resolved
    terminal.current.open(terminalRef.current);

    // Use ResizeObserver to refit whenever the container actually changes size.
    // This is more reliable than setTimeout because it fires AFTER layout.
    let resizeObserver: ResizeObserver | null = null;
    if (typeof ResizeObserver !== 'undefined') {
      resizeObserver = new ResizeObserver(() => {
        fitAddon.current?.fit();
      });
      resizeObserver.observe(terminalRef.current);
    }

    // Also do an immediate + deferred fit as backup
    requestAnimationFrame(() => {
      fitAddon.current?.fit();
      setTimeout(() => fitAddon.current?.fit(), 150);
    });

    // ALL keyboard input goes directly to the PTY — terminal is a pure shell
    // Natural language goes to the AI panel's own dedicated input (right side)
    terminal.current.onData(async (data: string) => {
      if (!tab.terminalId) return;
      try {
        await invoke('write_to_terminal', { terminalId: tab.terminalId, data });
      } catch (error) {
        terminalLogger.error('PTY write failed', error as Error, 'write_terminal_failed', { terminalId: tab.terminalId });
        dispatch(addError({ tabId: tab.id, error: { command: 'write_to_terminal', errorMessage: String(error), timestamp: new Date(), workingDirectory: tab.workingDirectory } }));
      }
    });

    // No welcome clutter — shell prompt appears immediately

    setIsTerminalReady(true);

    // Listen for terminal output — write to xterm AND parse OSC 133 sequences
    // to record completed commands as TerminalBlocks (same as Warp's BlockContext).
    // OSC 133 sequences injected by terminal.rs:
    //   \x1b]133;A\x07  = prompt start
    //   \x1b]133;B\x07  = command start (user about to type)
    //   \x1b]133;C\x07  = output start (command running)
    //   \x1b]133;D;N\x07 = command end, N = exit code
    let unlistenTerminalOutput: (() => void) | null = null;
    let unlistenTerminalCwd: (() => void) | null = null;
    const capturedTerminalId = tab.terminalId;
    const capturedTabId = tab.id;
    // cwdRef tracks the live shell cwd — updated by OSC 7 events
    const cwdRef = { current: tab.workingDirectory };

    // OSC 133 parser state
    let osc133State: 'prompt' | 'input' | 'output' = 'prompt';
    let currentCommand = '';
    let outputBuffer = '';

    const OSC_RE = /\x1b\]133;([^\x07]*)\x07/g;

    listen<{ terminal_id: string; data: string }>('terminal-output', (event) => {
      const { terminal_id, data } = event.payload;
      if (terminal_id !== capturedTerminalId) return;
      if (!terminal.current) {
        console.error('[NexusTerminal] terminal-output received but terminal.current is null');
        return;
      }
      // Write raw data to xterm
      terminal.current.write(data);

      // Feed into error detection buffer (strip ANSI for pattern matching)
      const plain = data.replace(/\x1b\[[^A-Za-z]*[A-Za-z]/g, '').replace(/\x1b\][^\x07]*\x07/g, '');
      termOutputBuffer.current += plain;

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
                cwd: cwdRef.current,
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

    // Listen for OSC 7 cwd notifications from the PTY backend.
    // Fish emits \x1b]7;file://hostname/path\x07 on every prompt — this keeps
    // workingDirectory in sync with the real shell cwd so the agent always
    // knows where the user is.
    listen<{ terminal_id: string; cwd: string }>('terminal-cwd', (event) => {
      const { terminal_id, cwd } = event.payload;
      if (terminal_id !== capturedTerminalId) return;
      if (!cwd || !cwd.startsWith('/')) return;
      cwdRef.current = cwd;
      dispatch(updateTabWorkingDirectory({ tabId: capturedTabId, cwd }));
    })
      .then((unlisten) => { unlistenTerminalCwd = unlisten; })
      .catch(() => { /* non-fatal */ });

    return () => {
      resizeObserver?.disconnect();
      if (unlistenTerminalOutput) unlistenTerminalOutput();
      if (unlistenTerminalCwd) unlistenTerminalCwd();
      if (terminal.current) {
        terminal.current.dispose();
        terminal.current = null;
      }
      setIsTerminalReady(false);
    };
  // NOTE: tab.workingDirectory is intentionally NOT in this dep array.
  // The terminal-cwd listener dispatches updateTabWorkingDirectory which changes
  // tab.workingDirectory. Including it here would cause xterm to be disposed and
  // recreated on every fish prompt (OSC 7 fires each time), producing a blank terminal.
  // cwdRef inside the effect stays current via the terminal-cwd listener.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab.terminalId, terminalOptions, tab.shell, tab.id, dispatch, isShellCommand, handleInput]);

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

  // ── Error detection + self-healing ───────────────────────────────
  const [lastShellCmd, setLastShellCmd] = useState<string>('');
  const [errorState, setErrorState] = useState<{ cmd: string; output: string } | null>(null);
  const [isHealing, setIsHealing] = useState(false);
  const termOutputBuffer = useRef<string>(''); // accumulates raw terminal output
  const errorCheckTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Error patterns that indicate a failed command
  const isErrorOutput = (text: string): boolean => {
    const patterns = [
      /\berror(?:\[|:)/i,           // error: or error[ (Rust, TypeScript, etc.)
      /\bfailed\b/i,                // failed
      /\bException\b/,              // Python/Java exceptions
      /command not found/i,
      /No such file or directory/i,
      /Permission denied/i,
      /npm ERR!/,                   // npm errors
      /FAILED.*\d+ test/i,          // test failures
      /SyntaxError|TypeError|ReferenceError|ImportError|ModuleNotFoundError/,
      /\bfatal:/i,                  // git fatal errors
      /\bAborted\b/,
    ];
    return patterns.some(p => p.test(text));
  };

  // Check buffered output for errors after command settles (1.5s)
  const scheduleErrorCheck = (cmd: string) => {
    termOutputBuffer.current = '';
    if (errorCheckTimer.current) clearTimeout(errorCheckTimer.current);
    errorCheckTimer.current = setTimeout(() => {
      const output = termOutputBuffer.current;
      if (output && isErrorOutput(output)) {
        setErrorState({ cmd, output: output.slice(0, 4000) });
      }
    }, 1500);
  };

  // Self-healing: re-run command via agent with full error context
  const handleHeal = async () => {
    if (!errorState) return;
    setErrorState(null);
    setIsHealing(true);

    // Re-run command via run_cmd to get clean captured output for the agent
    let capturedOutput = errorState.output;
    try {
      const result = await invoke<{ stdout: string; stderr: string; exit_code: number }>(
        'run_cmd_capture',
        { cmd: errorState.cmd, cwd: tab.workingDirectory || null }
      );
      capturedOutput = `STDOUT:\n${result.stdout}\nSTDERR:\n${result.stderr}\nEXIT CODE: ${result.exit_code}`;
    } catch { /* use buffered output as fallback */ }

    // Build self-healing agent message
    const healPrompt = [
      `The command \`${errorState.cmd}\` failed.`,
      ``,
      `Error output:`,
      `\`\`\``,
      capturedOutput.slice(0, 3000),
      `\`\`\``,
      ``,
      `Fix it:`,
      `1. Read the relevant file(s)`,
      `2. Apply the minimal fix with edit_file`,
      `3. Run \`${errorState.cmd}\` again to verify`,
      `4. Repeat until it passes`,
      `5. Commit if it passes`,
    ].join('\n');

    const sessionId = `heal_${Date.now()}`;
    const streamId = `heal_stream_${Date.now()}`;
    let streamBuffer = '';
    const localTools: Array<{ tool: string; args: string; result?: string }> = [];
    setAiBlocks(prev => [...prev, { id: `heal_user_${Date.now()}`, role: 'user', content: `fix: ${errorState.cmd}` }]);
    setAiBlocks(prev => [...prev, { id: streamId, role: 'assistant', content: '', streaming: '' }]);

    const u1 = await listen<any>('agent-token', ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      streamBuffer += payload.token;
      setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, streaming: streamBuffer + '█' } : b));
    });
    const u2 = await listen<any>('agent-tool-call', ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      localTools.push({ tool: payload.tool, args: payload.args });
      setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, tools: [...localTools] } : b));
    });
    const u3 = await listen<any>('agent-tool-result', ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      const idx = localTools.map(t => t.tool).lastIndexOf(payload.tool);
      if (idx >= 0) localTools[idx].result = payload.result.slice(0, 500);
      setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, tools: [...localTools] } : b));
    });
    const u4 = await listen<any>('agent-done', ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      const toolSec = localTools.length > 0
        ? localTools.map(t => `🔧 ${t.tool}\n\`\`\`\n${(t.result ?? '').slice(0, 600)}\n\`\`\``).join('\n\n')
        : '';
      const final = toolSec && payload.answer.trim()
        ? `${toolSec}\n\n${payload.answer.trim()}`
        : toolSec || payload.answer.trim() || '✓ Done';
      setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, content: final, streaming: undefined, tools: undefined } : b));
      setIsHealing(false);
      u1(); u2(); u3(); u4(); u5();
    });
    const u5 = await listen<any>('agent-error', ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      setAiBlocks(prev => prev.map(b => b.id === streamId ? { ...b, content: `❌ ${payload.error}`, streaming: undefined } : b));
      setIsHealing(false);
      u1(); u2(); u3(); u4(); u5();
    });

    await invoke('agent_chat_stream', {
      message: healPrompt,
      sessionId,
      history: [],
      cwd: (!tab.workingDirectory || tab.workingDirectory === '~') ? null : tab.workingDirectory,
      context: `Shell: ${tab.shell}\nDirectory: ${tab.workingDirectory}`,
    }).catch(() => { setIsHealing(false); });
  };

  // ── Session memory — persist AI blocks to localStorage ───────────────────
  const SESSION_KEY = `nexusai_session_${tab.id}`;

  // Restore session on mount
  useEffect(() => {
    try {
      const saved = localStorage.getItem(SESSION_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed) && parsed.length > 0) {
          setAiBlocks(parsed);
        }
      }
    } catch { /* ignore */ }
  }, [tab.id]);

  // Save session whenever aiBlocks changes
  useEffect(() => {
    if (aiBlocks.length === 0) return;
    try {
      // Keep last 50 blocks, strip streaming state
      const toSave = aiBlocks.slice(-50).map(b => ({ ...b, streaming: undefined, tools: undefined }));
      localStorage.setItem(SESSION_KEY, JSON.stringify(toSave));
    } catch { /* ignore */ }
  }, [aiBlocks]);

  // ── Ghost text prediction ──────────────────────────────────────────
  const [prediction, setPrediction] = useState('');
  const predictDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);
  // Track history of shell commands for prediction
  const shellHistory = useRef<string[]>([]);

  // Trigger prediction on input change
  const triggerPrediction = (value: string) => {
    if (predictDebounce.current) clearTimeout(predictDebounce.current);
    if (isAILoading) { setPrediction(''); return; }
    // Only predict in shell mode or detecting (not AI queries)
    if (inputMode === 'ai') { setPrediction(''); return; }
    predictDebounce.current = setTimeout(async () => {
      try {
        const result = await invoke<string | null>('predict_command', {
          partialInput: value,
          history: shellHistory.current.slice(0, 50),
          cwd: tab.workingDirectory || '~',
        });
        // Only show if prediction extends the current input
        if (result && result !== value && result.startsWith(value)) {
          setPrediction(result);
        } else if (result && !value) {
          // Zero-input next-command suggestion
          setPrediction(result);
        } else {
          setPrediction('');
        }
      } catch {
        setPrediction('');
      }
    }, 80); // 80ms debounce — fast enough that badge updates before most Enter presses
  };

  // Record completed shell commands into history for prediction
  const recordShellCommand = (cmd: string) => {
    if (!cmd.trim()) return;
    shellHistory.current = [cmd, ...shellHistory.current.filter(c => c !== cmd)].slice(0, 200);
  };

  // Classify input as user types — live mode indicator like Warp
  const handleUnifiedInputChange = (value: string) => {
    setUnifiedInput(value);
    setPrediction(''); // clear stale prediction immediately
    triggerPrediction(value);

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

  // Camera button: capture screen + ask vision AI what's visible
  const handleScreenshot = async (prompt?: string) => {
    setIsAILoading(true);
    const msgId = `shot_${Date.now()}`;
    setAiBlocks(prev => [...prev, { id: msgId, role: 'assistant', content: '', streaming: '📸 Capturing screen…' }]);
    try {
      const result = await invoke<string>('capture_and_ask', {
        prompt: prompt || 'Describe what you see on screen in 2-3 sentences. Be specific: mention exact commands, error messages, file names, or code visible. Skip generic observations.',
      });
      setAiBlocks(prev => prev.map(b => b.id === msgId ? { ...b, content: result, streaming: undefined } : b));
    } catch (e) {
      setAiBlocks(prev => prev.map(b => b.id === msgId ? { ...b, content: `❌ Vision error: ${e}`, streaming: undefined } : b));
    }
    setIsAILoading(false);
  };

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

    // Routing when badge has fired: shell→shell, ai→AI.
    // Routing when still detecting (typed too fast): use synchronous isShellCommand().
    // This mirrors Warp: auto-detect uses the fast classifier, not a blind default.
    const isShell = forceShell || (!forceAI && (
      inputMode === 'shell' ||
      (inputMode === 'detecting' && isShellCommand(text))
    ));

    if (isShell) {
      recordShellCommand(text);
      setPrediction('');
      setLastShellCmd(text);
      setErrorState(null); // clear previous error
      scheduleErrorCheck(text); // start watching for errors
      // Execute in PTY
      if (!tab.terminalId) {
        console.error('[NexusTerminal] No terminalId on tab — cannot execute shell command');
        setAiBlocks(prev => [...prev, { id: `err_${Date.now()}`, role: 'assistant', content: `❌ No terminal session. Restart the app.` }]);
        return;
      }
      try {
        console.log('[NexusTerminal] write_to_terminal:', tab.terminalId, JSON.stringify(text + '\r'));
        // Warp uses bracketed paste for fish to prevent per-keystroke interpretation.
        // This ensures fish treats the whole string as a complete command.
        const isFish = tab.shell === 'fish';
        const payload = isFish
          ? `\x1b[200~${text}\x1b[201~\r`  // bracketed paste
          : `${text}\r`;
        await invoke('write_to_terminal', { terminalId: tab.terminalId, data: payload });
        console.log('[NexusTerminal] write_to_terminal succeeded');
      } catch (e) {
        console.error('[NexusTerminal] write_to_terminal FAILED:', e);
        setAiBlocks(prev => [...prev, { id: `err_${Date.now()}`, role: 'assistant', content: `❌ Shell error: ${e}` }]);
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
        // Resolve '~' to null so Rust uses the actual home directory
        cwd: (!tab.workingDirectory || tab.workingDirectory === '~') ? null : tab.workingDirectory,
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

  // Warp UDI border color — blue for shell, magenta for AI, dim for detecting
  const udiBorderColor =
    inputMode === 'shell' ? '#3b82f6' :
    inputMode === 'ai'    ? '#a855f7' :
                           '#374151';

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden', background: '#0d0d0d' }}>

      {/* ── Terminal (always full) + AI overlay ───────────────────────── */}
      <div style={{ flex: '1 1 0%', minHeight: 0, position: 'relative', overflow: 'hidden', backgroundColor: terminalTheme.background }}>
        {/* xterm always takes full space */}
        <div ref={terminalRef} style={{ width: '100%', height: '100%' }} />

        {!isTerminalReady && (
          <div style={{ position: 'absolute', inset: 0, background: '#0d0d0d', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <span style={{ color: '#6b7280', fontSize: 14 }}>Starting {tab.shell}…</span>
          </div>
        )}

        {/* AI blocks float as an overlay panel at the bottom of the terminal */}
        {aiBlocks.length > 0 && (
          <div style={{
            position: 'absolute', bottom: 0, left: 0, right: 0,
            maxHeight: '55%',
            display: 'flex', flexDirection: 'column',
            background: 'rgba(10,10,10,0.96)',
            borderTop: '1px solid #374151',
            backdropFilter: 'blur(8px)',
          }}>
            {/* overlay header with dismiss */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '4px 12px', borderBottom: '1px solid #374151', flexShrink: 0 }}>
              <span style={{ color: '#9ca3af', fontSize: 11 }}>NexusAI</span>
              <button
                onClick={() => setAiBlocks([])}
                style={{ color: '#6b7280', fontSize: 11, background: 'none', border: 'none', cursor: 'pointer', padding: '2px 6px' }}
              >
                × dismiss
              </button>
            </div>
            <div className="overflow-y-auto px-4 py-3 space-y-3" style={{ minHeight: 0 }}>
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
          </div>
        )}
      </div>

      {/* ── Error banner: "Fix this?" ─────────────────────────────── */}
      {errorState && !isHealing && (
        <div style={{
          flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          padding: '6px 12px', background: 'rgba(239,68,68,0.12)', borderTop: '1px solid rgba(239,68,68,0.3)',
        }}>
          <span style={{ fontSize: 12, color: '#fca5a5', fontFamily: 'monospace' }}>
            ⚠️ Error in <strong>`{errorState.cmd}`</strong>
          </span>
          <div style={{ display: 'flex', gap: 8 }}>
            <button
              onClick={handleHeal}
              style={{
                fontSize: 11, padding: '3px 12px', borderRadius: 4,
                background: '#ef4444', color: '#fff', border: 'none', cursor: 'pointer', fontWeight: 600,
              }}
            >
              🔧 Fix this
            </button>
            <button
              onClick={() => setErrorState(null)}
              style={{ fontSize: 11, padding: '3px 8px', borderRadius: 4, background: 'transparent', color: '#9ca3af', border: '1px solid #374151', cursor: 'pointer' }}
            >
              Dismiss
            </button>
          </div>
        </div>
      )}
      {isHealing && (
        <div style={{ flexShrink: 0, padding: '6px 12px', background: 'rgba(168,85,247,0.1)', borderTop: '1px solid rgba(168,85,247,0.3)', fontSize: 12, color: '#c084fc', fontFamily: 'monospace' }}>
          🤖 NexusAI is analyzing and fixing the error…
        </div>
      )}

      {/* ── Warp-style UDI ───────────────────────────────────────────── */}
      {/* 6px margin all sides, 8px corner radius — matches Warp's UDI container */}
      <div style={{ flexShrink: 0, padding: '6px' }}>
        <div style={{
          display: 'flex', flexDirection: 'column', gap: 0,
          background: '#1c1c1e',
          border: `1px solid ${udiBorderColor}`,
          borderRadius: 8,
          transition: 'border-color 0.15s',
          overflow: 'hidden',
        }}>
          {/* Context row: cwd chip + model name */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '4px 12px', borderBottom: '1px solid #2a2a2e' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              {/* cwd chip */}
              <span style={{ fontSize: 11, fontFamily: 'monospace', color: '#6b7280', background: '#2a2a2e', borderRadius: 4, padding: '1px 6px' }}>
                {tab.workingDirectory === '~' ? '~' : tab.workingDirectory.split('/').slice(-2).join('/')}
              </span>
              {/* mode chips: Terminal | Agent | Auto */}
              {(['shell', 'detecting', 'ai'] as const).map(mode => (
                <button key={mode}
                  onClick={() => setInputMode(mode)}
                  style={{
                    fontSize: 10, padding: '1px 8px', borderRadius: 4, border: 'none', cursor: 'pointer',
                    background: inputMode === mode ? (mode === 'shell' ? '#1d4ed8' : mode === 'ai' ? '#7e22ce' : '#374151') : 'transparent',
                    color: inputMode === mode ? '#fff' : '#6b7280',
                    fontWeight: inputMode === mode ? 600 : 400,
                  }}
                >
                  {mode === 'shell' ? 'Terminal' : mode === 'ai' ? 'Agent' : 'Auto'}
                </button>
              ))}
            </div>
            <span style={{ fontSize: 10, color: '#4b5563', fontFamily: 'monospace' }}>codestral:22b</span>
          </div>

          {/* Input row */}
          <div style={{ display: 'flex', alignItems: 'center', padding: '6px 12px', gap: 8 }}>
            {/* Autodetect indicator dot */}
            <div style={{
              width: 6, height: 6, borderRadius: '50%', flexShrink: 0,
              background: udiBorderColor,
              boxShadow: inputMode !== 'detecting' ? `0 0 6px ${udiBorderColor}` : 'none',
              transition: 'background 0.15s, box-shadow 0.15s',
            }} />
{/* Ghost text + input wrapper — Warp-style inline prediction */}
            <div style={{ flex: 1, position: 'relative', display: 'flex', alignItems: 'center', overflow: 'hidden' }}>
              {/* Ghost text layer (behind) */}
              {prediction && inputMode !== 'ai' && (
                <div aria-hidden style={{
                  position: 'absolute', left: 0, top: 0, bottom: 0,
                  display: 'flex', alignItems: 'center',
                  fontSize: 13, fontFamily: 'monospace',
                  pointerEvents: 'none', whiteSpace: 'pre', userSelect: 'none',
                  color: 'transparent',  // invisible spacer matching typed text
                }}>
                  {unifiedInput}
                  <span style={{ color: '#4b5563' }}>{prediction.slice(unifiedInput.length)}</span>
                </div>
              )}
              <input
                ref={unifiedInputRef}
                type="text"
                value={unifiedInput}
                onChange={e => handleUnifiedInputChange(e.target.value)}
                onKeyDown={e => {
                  if (e.key === 'Tab' && prediction && inputMode !== 'ai') {
                    e.preventDefault();
                    setUnifiedInput(prediction);
                    setPrediction('');
                    triggerPrediction(prediction);
                  } else if (e.key === 'Escape') {
                    setPrediction('');
                  } else if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    setPrediction('');
                    handleUnifiedSubmit();
                  }
                }}
                placeholder={prediction ? '' : (inputMode === 'shell' ? 'Shell command…' : inputMode === 'ai' ? 'Ask NexusAI…' : 'Type a command or ask AI…')}
                style={{ flex: 1, background: 'transparent', border: 'none', outline: 'none', color: '#f9fafb', fontSize: 13, fontFamily: 'monospace', position: 'relative' }}
                autoFocus
                disabled={isAILoading}
              />
            </div>
            <span style={{ fontSize: 10, color: '#4b5563' }}>{'! shell · * ai · ** deep · > webui · @file'}</span>
            {/* Camera button — capture screen + ask vision AI */}
            <button
              onClick={() => handleScreenshot()}
              disabled={isAILoading}
              title="Capture screen and analyze with llama3.2-vision:11b"
              style={{
                flexShrink: 0, background: 'none', border: 'none', cursor: 'pointer',
                fontSize: 15, opacity: isAILoading ? 0.3 : 0.7, padding: '0 2px',
                transition: 'opacity 0.15s',
              }}
              onMouseEnter={e => (e.currentTarget.style.opacity = '1')}
              onMouseLeave={e => (e.currentTarget.style.opacity = isAILoading ? '0.3' : '0.7')}
            >
              📸
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
