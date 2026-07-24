// Ported from warpdotdev/warp (AGPL-3.0) — terminal session view.
// Renders: Warp-style block list (above) + raw xterm PTY (below) + unified input bar (bottom).
// AI conversation is handled by AgentPanel (right column in WarpStyleTerminal).
// Input bar uses useInputRouting for smart shell/AI routing (Warp's UDI pattern).
import React, { useEffect, useRef, useState, useMemo } from 'react';
import { useDispatch } from 'react-redux';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TerminalTab } from '../../types/terminal';
import {
  addError,
  addTerminalBlock,
  updateTabWorkingDirectory,
  addAIMessage,
} from '../../store/slices/terminalTabSlice';
import { useInputRouting } from '../../hooks/useInputRouting';
import { terminalLogger } from '../../utils/logger';
import BlockList from '../blocks/BlockList';
import '@xterm/xterm/css/xterm.css';

interface TerminalWithAIProps {
  tab: TerminalTab;
}

export const TerminalWithAI: React.FC<TerminalWithAIProps> = ({ tab }) => {
  const dispatch = useDispatch();
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminal = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const [isTerminalReady, setIsTerminalReady] = useState(false);

  // Active agent model name (displayed in input bar)
  const [agentModel, setAgentModel] = useState('…');
  useEffect(() => {
    invoke<string>('get_agent_model').then(setAgentModel).catch(() => setAgentModel('local'));
  }, []);

  // Centralized routing hook — Warp's classifier
  const { handleInput, isShellCommand } = useInputRouting();

  // ── Terminal theme ────────────────────────────────────────────────────────
  const terminalTheme = useMemo(() => {
    const base = {
      background: '#111114',
      foreground: '#e5e7eb',
      cursor: '#e5e7eb',
      cursorAccent: '#111114',
      selectionBackground: '#ffffff30',
      black: '#000000', red: '#ff5555', green: '#50fa7b', yellow: '#f1fa8c',
      blue: '#bd93f9', magenta: '#ff79c6', cyan: '#8be9fd', white: '#bfbfbf',
      brightBlack: '#4d4d4d', brightRed: '#ff6e6e', brightGreen: '#69ff94',
      brightYellow: '#ffffa5', brightBlue: '#d6acff', brightMagenta: '#ff92df',
      brightCyan: '#a4ffff', brightWhite: '#ffffff',
    };
    if (tab.shell === 'fish') return { ...base, blue: '#00ADD8', cyan: '#00ADD8' };
    if (tab.shell === 'zsh')  return { ...base, green: '#F15A29', brightGreen: '#F15A29' };
    return base;
  }, [tab.shell]);

  const terminalOptions = useMemo(() => ({
    theme: terminalTheme,
    fontFamily: 'JetBrains Mono, "Cascadia Code", Monaco, Menlo, "Ubuntu Mono", monospace',
    fontSize: 14,
    fontWeight: 'normal' as const,
    lineHeight: 1.2,
    letterSpacing: 0,
    cursorBlink: true,
    cursorStyle: 'block' as const,
    scrollback: 10000,
    tabStopWidth: 4,
    allowProposedApi: true,
  }), [terminalTheme]);

  // ── xterm initialization ──────────────────────────────────────────────────
  useEffect(() => {
    if (!terminalRef.current || !tab.terminalId) return;

    terminal.current = new Terminal(terminalOptions);
    fitAddon.current = new FitAddon();
    terminal.current.loadAddon(fitAddon.current);
    terminal.current.loadAddon(new WebLinksAddon());
    terminal.current.loadAddon(new SearchAddon());
    terminal.current.open(terminalRef.current);

    let resizeObserver: ResizeObserver | null = null;
    if (typeof ResizeObserver !== 'undefined') {
      resizeObserver = new ResizeObserver(() => { fitAddon.current?.fit(); });
      resizeObserver.observe(terminalRef.current);
    }
    requestAnimationFrame(() => {
      fitAddon.current?.fit();
      setTimeout(() => fitAddon.current?.fit(), 150);
    });

    // ALL direct keystrokes in the xterm area go straight to the PTY.
    // The unified input bar below handles smart routing.
    terminal.current.onData(async (data: string) => {
      if (!tab.terminalId) return;
      try {
        await invoke('write_to_terminal', { terminalId: tab.terminalId, data });
      } catch (error) {
        dispatch(addError({
          tabId: tab.id,
          error: { command: 'write_to_terminal', errorMessage: String(error), timestamp: new Date(), workingDirectory: tab.workingDirectory },
        }));
      }
    });

    setIsTerminalReady(true);

    // ── Terminal output listener ────────────────────────────────────────────
    let unlistenOutput: (() => void) | null = null;
    let unlistenCwd: (() => void) | null = null;
    const capturedTerminalId = tab.terminalId;
    const capturedTabId = tab.id;
    const cwdRef = { current: tab.workingDirectory };

    // OSC 133 parser state — used to record completed commands as TerminalBlocks
    let osc133State: 'prompt' | 'input' | 'output' = 'prompt';
    let currentCommand = '';
    let outputBuffer = '';
    const OSC_RE = /\x1b\]133;([^\x07]*)\x07/g;

    listen<{ terminal_id: string; data: string }>('terminal-output', (event) => {
      const { terminal_id, data } = event.payload;
      if (terminal_id !== capturedTerminalId) return;
      if (!terminal.current) return;

      terminal.current.write(data);

      // Fill error detection buffer (strip ANSI for text matching)
      const plain = data
        .replace(/\x1b\[[^A-Za-z]*[A-Za-z]/g, '')
        .replace(/\x1b\][^\x07]*\x07/g, '');
      termOutputBuffer.current += plain;

      // Parse OSC 133 sequences for block tracking
      let lastIndex = 0;
      OSC_RE.lastIndex = 0;
      let match;
      while ((match = OSC_RE.exec(data)) !== null) {
        const seq = match[1];
        const textBefore = data.slice(lastIndex, match.index);
        lastIndex = match.index + match[0].length;

        if (osc133State === 'input') {
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
          currentCommand = currentCommand.trim();
        } else if (seq.startsWith('D')) {
          const exitCode = parseInt(seq.split(';')[1] ?? '0', 10);
          const cmd = currentCommand.trim();
          const out = outputBuffer.trim();
          if (cmd) {
            dispatch(addTerminalBlock({
              tabId: capturedTabId,
              block: { command: cmd, output: out, exitCode, cwd: cwdRef.current, timestamp: new Date().toISOString() },
            }));
          }
          osc133State = 'prompt';
          currentCommand = '';
          outputBuffer = '';
        }
      }
      const tail = data.slice(lastIndex);
      if (osc133State === 'input') {
        currentCommand += tail.replace(/\x1b\[[^A-Za-z]*[A-Za-z]/g, '');
      } else if (osc133State === 'output') {
        outputBuffer += tail.replace(/\x1b\[[^A-Za-z]*[A-Za-z]/g, '');
      }
    }).then(u => {
      unlistenOutput = u;
      // Repaint after listener is registered
      const repaint = async () => {
        if (fitAddon.current && terminal.current) {
          fitAddon.current.fit();
          const { cols, rows } = terminal.current;
          await invoke('resize_terminal', { terminal_id: capturedTerminalId, cols: cols || 80, rows: rows || 24 }).catch(() => {});
        }
        await invoke('write_to_terminal', { terminalId: capturedTerminalId, data: '\r' }).catch(() => {});
      };
      repaint();
    }).catch(err => {
      terminalLogger.error('Failed to set up terminal output listener', err as Error, 'listener_setup_failed', { terminalId: tab.terminalId });
    });

    // OSC 7 cwd listener — keeps workingDirectory in sync with real shell cwd
    listen<{ terminal_id: string; cwd: string }>('terminal-cwd', (event) => {
      const { terminal_id, cwd } = event.payload;
      if (terminal_id !== capturedTerminalId) return;
      if (!cwd || !cwd.startsWith('/')) return;
      cwdRef.current = cwd;
      dispatch(updateTabWorkingDirectory({ tabId: capturedTabId, cwd }));
    }).then(u => { unlistenCwd = u; }).catch(() => {});

    return () => {
      resizeObserver?.disconnect();
      if (unlistenOutput) unlistenOutput();
      if (unlistenCwd) unlistenCwd();
      if (terminal.current) { terminal.current.dispose(); terminal.current = null; }
      setIsTerminalReady(false);
    };
  // Intentional dep array: tab.workingDirectory excluded (OSC 7 updates it on every fish
  // prompt, would cause xterm dispose+recreate loop). tab.shell/id/terminalId are stable
  // per-tab. isShellCommand/handleInput excluded — not used inside this effect.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab.terminalId, terminalOptions, tab.shell, tab.id, dispatch]);

  // ── Window resize ─────────────────────────────────────────────────────────
  useEffect(() => {
    const handleResize = () => {
      if (!fitAddon.current || !terminal.current || !isTerminalReady) return;
      fitAddon.current.fit();
      const { cols, rows } = terminal.current;
      if (tab.terminalId) {
        invoke('resize_terminal', { terminal_id: tab.terminalId, cols, rows }).catch(() => {});
      }
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [isTerminalReady, tab.terminalId]);

  // ── Focus unified input on terminal ready ─────────────────────────────────
  useEffect(() => {
    if (isTerminalReady) { unifiedInputRef.current?.focus(); }
  }, [isTerminalReady]);

  // ── Keyboard shortcuts ────────────────────────────────────────────────────
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // Ctrl+Shift+C — clear terminal
      if (e.ctrlKey && e.shiftKey && e.key === 'C') {
        e.preventDefault();
        terminal.current?.clear();
      }
      // Ctrl+I — toggle input mode
      if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
        e.preventDefault();
        setInputMode(m => (m === 'shell' ? 'ai' : 'shell'));
        unifiedInputRef.current?.focus();
      }
      // Escape — clear prediction
      if (e.key === 'Escape') {
        setPrediction('');
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // ── Agent question (ask_user tool) ────────────────────────────────────────
  const [agentQuestion, setAgentQuestion] = useState<{
    sessionId: string;
    question: string;
    options: string[];
    data?: { kind?: string; scan_path?: string };
  } | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<{ session_id: string; question: string; options: string[]; data?: { kind?: string; scan_path?: string } }>(
      'agent-question',
      ({ payload }) => {
        setAgentQuestion({
          sessionId: payload.session_id,
          question: payload.question,
          options: payload.options,
          data: payload.data,
        });
      }
    ).then(u => { unlisten = u; }).catch(() => {});
    return () => { if (unlisten) unlisten(); };
  }, []);

  const handleAgentAnswer = async (option: string) => {
    if (!agentQuestion) return;
    const { sessionId, data } = agentQuestion;
    setAgentQuestion(null);

    // scan_and_fix path: invoke fix engine directly, stream progress to redux
    if (data?.kind === 'scan_and_fix' && data.scan_path && option.toLowerCase().includes('fix')) {
      const fixSessionId = `fix_${Date.now()}`;
      setIsAILoading(true);
      dispatch(addAIMessage({
        tabId: tab.id,
        message: { role: 'assistant', content: '🔧 Fix engine starting…', timestamp: new Date() },
      }));

      const uFix = await listen<{
        session_id: string; stage: string; message: string;
        done: boolean; errors_found: number; errors_fixed: number;
      }>('fix-progress', ({ payload }) => {
        if (payload.session_id !== fixSessionId) return;
        const icon = payload.stage === 'done' ? '✅' : payload.stage === 'fixing' ? '🔧' : '⏳';
        dispatch(addAIMessage({
          tabId: tab.id,
          message: { role: 'assistant', content: `${icon} ${payload.message}`, timestamp: new Date() },
        }));
        if (payload.done) { setIsAILoading(false); uFix(); }
      });

      invoke('scan_and_fix', { scanPath: data.scan_path, sessionId: fixSessionId }).catch(() => {
        dispatch(addAIMessage({
          tabId: tab.id,
          message: { role: 'assistant', content: '❌ Fix engine failed to start', timestamp: new Date() },
        }));
        setIsAILoading(false);
      });
      return;
    }

    try {
      await invoke('answer_agent_question', { sessionId, answer: option });
    } catch (e) {
      console.error('[NexusTerminal] answer_agent_question failed:', e);
    }
  };

  // ── Error detection + self-healing ────────────────────────────────────────
  const [errorState, setErrorState] = useState<{ cmd: string; output: string } | null>(null);
  const [isHealing, setIsHealing] = useState(false);
  const termOutputBuffer = useRef<string>('');
  const errorCheckTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const isErrorOutput = (text: string): boolean => {
    const patterns = [
      /\berror(?:\[|:)/i, /\bfailed\b/i, /\bException\b/,
      /command not found/i, /No such file or directory/i, /Permission denied/i,
      /npm ERR!/, /FAILED.*\d+ test/i,
      /SyntaxError|TypeError|ReferenceError|ImportError|ModuleNotFoundError/,
      /\bfatal:/i, /\bAborted\b/,
    ];
    return patterns.some(p => p.test(text));
  };

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

  // Self-healing: route to AI via useInputRouting
  const handleHeal = async () => {
    if (!errorState) return;
    const errCmd = errorState.cmd;
    const errOut = errorState.output;
    setErrorState(null);
    setIsHealing(true);

    let capturedOutput = errOut;
    try {
      const result = await invoke<{ stdout: string; stderr: string; exit_code: number }>(
        'run_cmd_capture',
        { cmd: errCmd, cwd: tab.workingDirectory || null }
      );
      capturedOutput = `STDOUT:\n${result.stdout}\nSTDERR:\n${result.stderr}\nEXIT CODE: ${result.exit_code}`;
    } catch { /* use buffered output as fallback */ }

    const healPrompt = [
      `The command \`${errCmd}\` failed.`,
      ``,
      `Error output:`,
      `\`\`\``,
      capturedOutput.slice(0, 3000),
      `\`\`\``,
      ``,
      `Fix it: read the relevant file(s), apply the minimal fix, run the command again to verify, commit if passing.`,
    ].join('\n');

    setIsAILoading(true);
    dispatch(addAIMessage({
      tabId: tab.id,
      message: { role: 'user', content: `fix: ${errCmd}`, timestamp: new Date() },
    }));
    await handleInput(healPrompt, () => {
      setIsAILoading(false);
      setIsHealing(false);
    });
    setIsHealing(false);
  };

  // ── Unified input state (Warp UDI) ────────────────────────────────────────
  const [unifiedInput, setUnifiedInput] = useState('');
  const [inputMode, setInputMode] = useState<'shell' | 'ai' | 'detecting'>('detecting');
  const [isAILoading, setIsAILoading] = useState(false);
  const unifiedInputRef = useRef<HTMLInputElement>(null);
  const classifyDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);

  // ── Ghost text prediction ─────────────────────────────────────────────────
  const [prediction, setPrediction] = useState('');
  const predictDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);
  const shellHistory = useRef<string[]>([]);

  const triggerPrediction = (value: string) => {
    if (predictDebounce.current) clearTimeout(predictDebounce.current);
    if (isAILoading || inputMode === 'ai') { setPrediction(''); return; }
    predictDebounce.current = setTimeout(async () => {
      try {
        const result = await invoke<string | null>('predict_command', {
          partialInput: value,
          history: shellHistory.current.slice(0, 50),
          cwd: tab.workingDirectory || '~',
        });
        if (result && result !== value && result.startsWith(value)) {
          setPrediction(result);
        } else if (result && !value) {
          setPrediction(result);
        } else {
          setPrediction('');
        }
      } catch { setPrediction(''); }
    }, 80);
  };

  const recordShellCommand = (cmd: string) => {
    if (!cmd.trim()) return;
    shellHistory.current = [cmd, ...shellHistory.current.filter(c => c !== cmd)].slice(0, 200);
  };

  const handleUnifiedInputChange = (value: string) => {
    setUnifiedInput(value);
    setPrediction('');
    triggerPrediction(value);

    if (value.startsWith('!')) { setInputMode('shell'); return; }
    if (value.startsWith('*')) { setInputMode('ai'); return; }
    if (!value.trim()) { setInputMode('detecting'); return; }

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

  // Screenshot: capture screen + vision AI → dispatch to redux for AgentPanel
  const handleScreenshot = async () => {
    setIsAILoading(true);
    dispatch(addAIMessage({
      tabId: tab.id,
      message: { role: 'assistant', content: '📸 Capturing screen…', timestamp: new Date() },
    }));
    try {
      const result = await invoke<string>('capture_and_ask', {
        prompt: 'Describe what you see on screen in 2-3 sentences. Be specific: mention exact commands, error messages, file names, or code visible.',
      });
      dispatch(addAIMessage({
        tabId: tab.id,
        message: { role: 'assistant', content: result, timestamp: new Date() },
      }));
    } catch (e) {
      dispatch(addAIMessage({
        tabId: tab.id,
        message: { role: 'assistant', content: `❌ Vision error: ${e}`, timestamp: new Date() },
      }));
    }
    setIsAILoading(false);
  };

  // ── Submit unified input via useInputRouting ──────────────────────────────
  const handleUnifiedSubmit = async () => {
    let text = unifiedInput.trim();
    if (!text) return;

    const forceShell = text.startsWith('!');
    const forceAI = text.startsWith('*');
    if (forceShell) text = text.slice(1).trim();
    if (forceAI) text = text.slice(1).trim();
    if (!text) return;

    setUnifiedInput('');
    setPrediction('');
    setInputMode('detecting');

    const resolvedIsShell = forceShell || (!forceAI && (
      inputMode === 'shell' ||
      (inputMode === 'detecting' && isShellCommand(text))
    ));

    if (resolvedIsShell) {
      recordShellCommand(text);
      setErrorState(null);
      scheduleErrorCheck(text);
      // handleInput routes shell commands to the PTY via write_to_terminal
      handleInput(text).catch(console.error);
    } else {
      setIsAILoading(true);
      dispatch(addAIMessage({
        tabId: tab.id,
        message: { role: 'user', content: text, timestamp: new Date() },
      }));
      // handleInput routes AI queries to agent_chat_stream + dispatches final reply to redux
      handleInput(text, () => setIsAILoading(false)).catch(() => setIsAILoading(false));
    }
  };

  // ── Border color for UDI (Warp-style) ────────────────────────────────────
  const udiBorderColor =
    inputMode === 'shell' ? '#3b82f6' :
    inputMode === 'ai'    ? '#a855f7' :
                           '#2a2a2e';

  const cwdChip = tab.workingDirectory === '~'
    ? '~'
    : tab.workingDirectory.split('/').slice(-2).join('/');

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden', background: '#0d0d0d' }}>

      {/* ── Block list (completed + running commands from block events) ────── */}
      <BlockList terminalId={tab.terminalId || null} maxHeight={300} />

      {/* ── xterm PTY area ──────────────────────────────────────────────── */}
      <div style={{ flex: '1 1 0%', minHeight: 0, position: 'relative', overflow: 'hidden', background: terminalTheme.background }}>
        <div ref={terminalRef} style={{ width: '100%', height: '100%' }} />
        {!isTerminalReady && (
          <div style={{ position: 'absolute', inset: 0, background: '#0d0d0d', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <span style={{ color: '#6b7280', fontSize: 14, fontFamily: 'JetBrains Mono, monospace' }}>
              Starting {tab.shell}…
            </span>
          </div>
        )}
      </div>

      {/* ── Agent question card (ask_user tool) ───────────────────────────── */}
      {agentQuestion && (
        <div style={{
          flexShrink: 0, padding: '10px 14px',
          background: 'rgba(124,58,237,0.1)', borderTop: '1px solid rgba(124,58,237,0.35)',
          display: 'flex', flexDirection: 'column', gap: 8,
        }}>
          <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: 12 }}>
            <span style={{ fontSize: 13, color: '#c4b5fd', fontFamily: 'JetBrains Mono, monospace', lineHeight: 1.4 }}>
              🤖 {agentQuestion.question}
            </span>
            <button
              onClick={() => setAgentQuestion(null)}
              style={{ color: '#6b7280', fontSize: 11, background: 'none', border: 'none', cursor: 'pointer', flexShrink: 0 }}
            >✕</button>
          </div>
          <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
            {agentQuestion.options.map((opt, i) => (
              <button
                key={i}
                onClick={() => handleAgentAnswer(opt)}
                style={{
                  fontSize: 12, padding: '5px 14px', borderRadius: 6,
                  background: i === 0 ? '#7c3aed' : 'transparent',
                  color: i === 0 ? '#fff' : '#a78bfa',
                  border: `1px solid ${i === 0 ? '#7c3aed' : '#7c3aed'}`,
                  cursor: 'pointer', fontWeight: i === 0 ? 600 : 400,
                }}
              >{opt}</button>
            ))}
          </div>
        </div>
      )}

      {/* ── Error banner: "Fix this?" ──────────────────────────────────────── */}
      {errorState && !isHealing && (
        <div style={{
          flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          padding: '6px 12px', background: 'rgba(239,68,68,0.1)', borderTop: '1px solid rgba(239,68,68,0.25)',
        }}>
          <span style={{ fontSize: 12, color: '#fca5a5', fontFamily: 'JetBrains Mono, monospace' }}>
            ⚠️ Error in <strong>`{errorState.cmd}`</strong>
          </span>
          <div style={{ display: 'flex', gap: 8 }}>
            <button
              onClick={handleHeal}
              style={{ fontSize: 11, padding: '3px 12px', borderRadius: 4, background: '#ef4444', color: '#fff', border: 'none', cursor: 'pointer', fontWeight: 600 }}
            >🔧 Fix this</button>
            <button
              onClick={() => setErrorState(null)}
              style={{ fontSize: 11, padding: '3px 8px', borderRadius: 4, background: 'transparent', color: '#9ca3af', border: '1px solid #374151', cursor: 'pointer' }}
            >Dismiss</button>
          </div>
        </div>
      )}
      {isHealing && (
        <div style={{ flexShrink: 0, padding: '6px 12px', background: 'rgba(168,85,247,0.08)', borderTop: '1px solid rgba(168,85,247,0.25)', fontSize: 12, color: '#c084fc', fontFamily: 'JetBrains Mono, monospace' }}>
          🤖 NexusAI is analyzing and fixing the error…
        </div>
      )}

      {/* ── Warp UDI — unified input bar ──────────────────────────────────── */}
      {/* 6px margin, 8px radius — matches Warp's UDI container exactly */}
      <div style={{ flexShrink: 0, padding: '6px' }}>
        <div style={{
          display: 'flex', flexDirection: 'column', gap: 0,
          background: '#1c1c1e',
          border: `1px solid ${udiBorderColor}`,
          borderRadius: 8,
          transition: 'border-color 0.15s',
          overflow: 'hidden',
        }}>
          {/* Context row: cwd chip + mode chips + model */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '4px 12px', borderBottom: '1px solid #2a2a2e' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <span style={{ fontSize: 11, fontFamily: 'JetBrains Mono, monospace', color: '#6b7280', background: '#2a2a2e', borderRadius: 4, padding: '1px 6px' }}>
                {cwdChip}
              </span>
              {(['shell', 'detecting', 'ai'] as const).map(mode => (
                <button
                  key={mode}
                  onClick={() => setInputMode(mode)}
                  style={{
                    fontSize: 10, padding: '1px 8px', borderRadius: 4, border: 'none', cursor: 'pointer',
                    background: inputMode === mode
                      ? (mode === 'shell' ? '#1d4ed8' : mode === 'ai' ? '#7e22ce' : '#374151')
                      : 'transparent',
                    color: inputMode === mode ? '#fff' : '#6b7280',
                    fontWeight: inputMode === mode ? 600 : 400,
                    transition: 'all 0.1s',
                  }}
                >
                  {mode === 'shell' ? 'Terminal' : mode === 'ai' ? 'Agent' : 'Auto'}
                </button>
              ))}
            </div>
            <span style={{ fontSize: 10, color: '#374151', fontFamily: 'JetBrains Mono, monospace' }}>{agentModel}</span>
          </div>

          {/* Input row */}
          <div style={{ display: 'flex', alignItems: 'center', padding: '6px 12px', gap: 8 }}>
            {/* Mode dot */}
            <div style={{
              width: 6, height: 6, borderRadius: '50%', flexShrink: 0,
              background: udiBorderColor,
              boxShadow: inputMode !== 'detecting' ? `0 0 6px ${udiBorderColor}` : 'none',
              transition: 'background 0.15s, box-shadow 0.15s',
            }} />

            {/* Ghost text + input wrapper */}
            <div style={{ flex: 1, position: 'relative', display: 'flex', alignItems: 'center', overflow: 'hidden' }}>
              {prediction && inputMode !== 'ai' && (
                <div aria-hidden style={{
                  position: 'absolute', left: 0, top: 0, bottom: 0,
                  display: 'flex', alignItems: 'center',
                  fontSize: 13, fontFamily: 'JetBrains Mono, monospace',
                  pointerEvents: 'none', whiteSpace: 'pre', userSelect: 'none',
                }}>
                  <span style={{ color: 'transparent' }}>{unifiedInput}</span>
                  <span style={{ color: '#374151' }}>{prediction.slice(unifiedInput.length)}</span>
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
                placeholder={
                  prediction ? '' :
                  inputMode === 'shell' ? 'Shell command…' :
                  inputMode === 'ai' ? 'Ask NexusAI…' :
                  'Type a command or ask AI… (! shell · * agent)'
                }
                style={{
                  flex: 1, background: 'transparent', border: 'none', outline: 'none',
                  color: '#f9fafb', fontSize: 13,
                  fontFamily: 'JetBrains Mono, monospace',
                  position: 'relative',
                }}
                autoFocus
                disabled={isAILoading}
              />
            </div>

            {/* Camera / vision button */}
            <button
              onClick={handleScreenshot}
              disabled={isAILoading}
              title="Capture screen + analyze with vision AI"
              style={{
                flexShrink: 0, background: 'none', border: 'none', cursor: isAILoading ? 'default' : 'pointer',
                fontSize: 15, opacity: isAILoading ? 0.3 : 0.65, padding: '0 2px',
                transition: 'opacity 0.15s',
              }}
            >📸</button>

            {/* AI loading indicator */}
            {isAILoading && (
              <span style={{ fontSize: 10, color: '#a855f7', fontStyle: 'italic', flexShrink: 0 }}>thinking…</span>
            )}

            {/* Submit button */}
            <button
              onClick={handleUnifiedSubmit}
              disabled={isAILoading || !unifiedInput.trim()}
              style={{
                flexShrink: 0, background: '#1d4ed8', border: 'none', borderRadius: 4,
                padding: '3px 10px', color: '#fff', fontSize: 12,
                cursor: (isAILoading || !unifiedInput.trim()) ? 'default' : 'pointer',
                opacity: (isAILoading || !unifiedInput.trim()) ? 0.4 : 1,
                fontFamily: 'JetBrains Mono, monospace',
                transition: 'opacity 0.1s',
              }}
            >↵</button>
          </div>
        </div>
      </div>
    </div>
  );
};
