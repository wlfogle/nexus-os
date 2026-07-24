// Ported from warpdotdev/warp (AGPL-3.0) — right-hand agent conversation panel.
// Shows the AI conversation from redux (activeTab.aiConversation) and streams
// live tokens from agent-token / agent-tool-call / agent-done Tauri events.
import React, { useEffect, useRef, useState } from 'react';
import { useDispatch } from 'react-redux';
import { listen } from '@tauri-apps/api/event';
import { clearAIConversation } from '../../store/slices/terminalTabSlice';
import type { TerminalTab, AIMessage } from '../../types/terminal';

interface AgentPanelProps {
  tab: TerminalTab | undefined;
  /** Width of the panel in pixels (parent controls collapse). */
  width?: number;
}

interface StreamState {
  buffer: string;
  tools: Array<{ tool: string; args: string; result?: string }>;
  isLoading: boolean;
}

const INIT_STREAM: StreamState = { buffer: '', tools: [], isLoading: false };

const AgentPanel: React.FC<AgentPanelProps> = ({ tab, width = 340 }) => {
  const dispatch = useDispatch();
  const [stream, setStream] = useState<StreamState>(INIT_STREAM);
  const bottomRef = useRef<HTMLDivElement>(null);

  // ── Listen to agent streaming events ─────────────────────────────────────
  // We listen globally (no session ID filtering) since there is typically one
  // active agent session at a time. The session ID used by useInputRouting is
  // internal to that hook; we cannot filter by it here.
  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    const setup = async () => {
      const u1 = await listen<{ session_id: string; token: string }>('agent-token', ({ payload: p }) => {
        setStream(s => ({ ...s, isLoading: true, buffer: s.buffer + p.token }));
      });
      unlisteners.push(u1);

      const u2 = await listen<{ session_id: string; tool: string; args: string }>('agent-tool-call', ({ payload: p }) => {
        setStream(s => ({
          ...s,
          isLoading: true,
          tools: [...s.tools, { tool: p.tool, args: p.args }],
        }));
      });
      unlisteners.push(u2);

      const u3 = await listen<{ session_id: string; tool: string; result: string }>('agent-tool-result', ({ payload: p }) => {
        setStream(s => {
          const tools = [...s.tools];
          const idx = [...tools].reverse().findIndex(t => t.tool === p.tool);
          if (idx >= 0) {
            const realIdx = tools.length - 1 - idx;
            tools[realIdx] = { ...tools[realIdx], result: p.result.slice(0, 600) };
          }
          return { ...s, tools };
        });
      });
      unlisteners.push(u3);

      const u4 = await listen<{ session_id: string; answer: string }>('agent-done', () => {
        // The final message is already dispatched to redux by useInputRouting.
        // Clear streaming state so the final message from redux shows.
        setStream(INIT_STREAM);
      });
      unlisteners.push(u4);

      const u5 = await listen<{ session_id: string; error: string }>('agent-error', () => {
        setStream(INIT_STREAM);
      });
      unlisteners.push(u5);
    };

    setup().catch(console.error);
    return () => { unlisteners.forEach(u => u()); };
  }, []);

  // Auto-scroll when messages arrive
  const messages = tab?.aiConversation ?? [];
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages.length, stream.buffer]);

  const handleClear = () => {
    if (tab) dispatch(clearAIConversation(tab.id));
    setStream(INIT_STREAM);
  };

  return (
    <div
      style={{
        width,
        flexShrink: 0,
        display: 'flex',
        flexDirection: 'column',
        borderLeft: '1px solid #1f2937',
        background: '#0f0f10',
        overflow: 'hidden',
      }}
    >
      {/* Panel header */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '7px 12px',
        borderBottom: '1px solid #1f2937',
        flexShrink: 0,
      }}>
        <span style={{ fontSize: 11, color: '#6b7280', fontWeight: 600, letterSpacing: '0.06em', textTransform: 'uppercase' }}>
          🤖 NexusAI
        </span>
        <div style={{ display: 'flex', gap: 6 }}>
          {stream.isLoading && (
            <span style={{ fontSize: 10, color: '#a855f7', fontStyle: 'italic' }}>thinking…</span>
          )}
          <button
            onClick={handleClear}
            title="Clear conversation"
            style={{
              fontSize: 10, color: '#4b5563', background: 'none', border: '1px solid #374151',
              borderRadius: 4, padding: '2px 7px', cursor: 'pointer',
            }}
          >
            Clear
          </button>
        </div>
      </div>

      {/* Message list */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '8px 0' }}>
        {messages.length <= 1 && !stream.isLoading ? (
          <div style={{ padding: '20px 12px', textAlign: 'center', color: '#374151', fontSize: 12 }}>
            Ask NexusAI anything. Type a question in the input bar.
          </div>
        ) : (
          messages
            .filter((m: AIMessage) => m.role !== 'system')
            .map((m: AIMessage) => (
              <MessageBubble key={m.id} message={m} />
            ))
        )}

        {/* Live streaming block */}
        {(stream.isLoading || stream.buffer || stream.tools.length > 0) && (
          <div style={{ margin: '4px 8px' }}>
            {/* Tool calls */}
            {stream.tools.map((t, i) => (
              <div key={i} style={{
                margin: '2px 0',
                padding: '4px 8px',
                background: '#1a1a1e',
                border: '1px solid #374151',
                borderRadius: 4,
                fontSize: 11,
                fontFamily: 'var(--font-mono)',
              }}>
                <span style={{ color: '#f59e0b' }}>🔧 {t.tool}</span>
                {t.result && (
                  <div style={{ color: '#6b7280', marginTop: 2 }}>
                    {t.result.slice(0, 200)}
                  </div>
                )}
              </div>
            ))}

            {/* Streaming text */}
            {(stream.buffer || stream.isLoading) && (
              <div style={{
                background: '#1c1c1e',
                border: '1px solid #374151',
                borderRadius: 6,
                padding: '8px 10px',
                fontSize: 12,
                fontFamily: 'var(--font-mono)',
                color: '#c4b5fd',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
              }}>
                <div style={{ fontSize: 10, color: '#7c3aed', marginBottom: 3 }}>🤖 NexusAI</div>
                {stream.buffer}
                {stream.isLoading && <span style={{ opacity: 0.6 }}>█</span>}
              </div>
            )}
          </div>
        )}

        <div ref={bottomRef} />
      </div>
    </div>
  );
};

// ── Individual message bubble ─────────────────────────────────────────────────
const MessageBubble: React.FC<{ message: AIMessage }> = ({ message }) => {
  const isUser = message.role === 'user';

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: isUser ? 'flex-end' : 'flex-start',
      margin: '3px 8px',
    }}>
      {!isUser && (
        <div style={{ fontSize: 10, color: '#7c3aed', marginBottom: 2, paddingLeft: 2 }}>NexusAI</div>
      )}
      <div style={{
        maxWidth: '92%',
        background: isUser ? 'rgba(59,130,246,0.2)' : '#1c1c1e',
        border: `1px solid ${isUser ? 'rgba(59,130,246,0.4)' : '#2a2a2e'}`,
        borderRadius: isUser ? '8px 8px 2px 8px' : '8px 8px 8px 2px',
        padding: '7px 10px',
        fontSize: 12,
        fontFamily: 'var(--font-mono)',
        color: isUser ? '#bfdbfe' : '#d1d5db',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
        lineHeight: 1.5,
      }}>
        {message.content}
      </div>
      <div style={{ fontSize: 9, color: '#374151', marginTop: 2, paddingLeft: 2 }}>
        {new Date(message.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
      </div>
    </div>
  );
};

export default AgentPanel;
