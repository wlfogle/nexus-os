import React, { useState } from 'react';
import { TerminalBlock } from '../../types/terminal';

interface BlockListProps {
  blocks: TerminalBlock[];
  onRerun: (command: string) => void;
  onAskAI: (block: TerminalBlock) => void;
  onClose: () => void;
}

function formatRelativeTime(iso: string): string {
  const diffMs = Date.now() - new Date(iso).getTime();
  const sec = Math.floor(diffMs / 1000);
  if (sec < 5) return 'just now';
  if (sec < 60) return `${sec}s ago`;
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min}m ago`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr}h ago`;
  const day = Math.floor(hr / 24);
  return `${day}d ago`;
}

const actionBtnStyle: React.CSSProperties = {
  fontSize: 11,
  color: '#9ca3af',
  background: 'none',
  border: '1px solid #374151',
  borderRadius: 4,
  padding: '2px 8px',
  cursor: 'pointer',
};

// Warp-style block timeline: each completed command becomes a discrete card
// (command header, exit-code badge, timestamp, collapsible output, actions)
// instead of living solely in xterm's raw scrollback.
export const BlockList: React.FC<BlockListProps> = ({ blocks, onRerun, onAskAI, onClose }) => {
  const [collapsed, setCollapsed] = useState<Record<number, boolean>>({});

  const toggleCollapsed = (idx: number) => {
    setCollapsed(prev => ({ ...prev, [idx]: !prev[idx] }));
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      /* clipboard permission denied — non-fatal */
    }
  };

  return (
    <div
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        maxHeight: '55%',
        display: 'flex',
        flexDirection: 'column',
        background: 'rgba(10,10,10,0.97)',
        borderBottom: '1px solid #374151',
        backdropFilter: 'blur(8px)',
        zIndex: 5,
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: '4px 12px',
          borderBottom: '1px solid #374151',
          flexShrink: 0,
        }}
      >
        <span style={{ color: '#9ca3af', fontSize: 11 }}>📋 Blocks ({blocks.length})</span>
        <button
          onClick={onClose}
          style={{ color: '#6b7280', fontSize: 11, background: 'none', border: 'none', cursor: 'pointer', padding: '2px 6px' }}
        >
          × close
        </button>
      </div>

      <div className="overflow-y-auto" style={{ minHeight: 0 }}>
        {blocks.length === 0 ? (
          <div style={{ padding: '24px 16px', textAlign: 'center', color: '#6b7280', fontSize: 13 }}>
            No commands run yet in this tab.
          </div>
        ) : (
          blocks.map((block, idx) => {
            const isCollapsed = collapsed[idx] ?? false;
            const isSuccess = block.exitCode === 0;
            return (
              <div
                key={`${block.timestamp}-${idx}`}
                style={{
                  margin: '8px 12px',
                  border: `1px solid ${isSuccess ? '#1f2937' : '#7f1d1d'}`,
                  borderRadius: 8,
                  overflow: 'hidden',
                  background: '#141414',
                }}
              >
                {/* Header row */}
                <div
                  onClick={() => toggleCollapsed(idx)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    padding: '6px 10px',
                    cursor: 'pointer',
                    background: '#1a1a1a',
                    borderBottom: isCollapsed ? 'none' : '1px solid #262626',
                  }}
                >
                  <span
                    style={{
                      display: 'inline-flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      width: 16,
                      height: 16,
                      borderRadius: '50%',
                      fontSize: 10,
                      fontWeight: 700,
                      background: isSuccess ? '#065f46' : '#7f1d1d',
                      color: isSuccess ? '#6ee7b7' : '#fca5a5',
                      flexShrink: 0,
                    }}
                  >
                    {isSuccess ? '✓' : '✕'}
                  </span>
                  <span
                    style={{
                      fontFamily: 'JetBrains Mono, Monaco, monospace',
                      fontSize: 13,
                      color: '#e5e7eb',
                      flex: 1,
                      whiteSpace: 'nowrap',
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                    }}
                  >
                    {block.command}
                  </span>
                  {!isSuccess && (
                    <span style={{ fontSize: 10, color: '#fca5a5', flexShrink: 0 }}>exit {block.exitCode}</span>
                  )}
                  <span style={{ fontSize: 10, color: '#6b7280', flexShrink: 0 }}>{formatRelativeTime(block.timestamp)}</span>
                  <span style={{ fontSize: 10, color: '#6b7280', flexShrink: 0 }}>{isCollapsed ? '▸' : '▾'}</span>
                </div>

                {!isCollapsed && (
                  <>
                    <div style={{ display: 'flex', gap: 4, padding: '4px 10px', background: '#161616' }}>
                      <button
                        onClick={e => { e.stopPropagation(); copyToClipboard(block.command); }}
                        style={actionBtnStyle}
                      >
                        📋 Copy
                      </button>
                      <button
                        onClick={e => { e.stopPropagation(); onRerun(block.command); }}
                        style={actionBtnStyle}
                      >
                        ↻ Re-run
                      </button>
                      <button
                        onClick={e => { e.stopPropagation(); onAskAI(block); }}
                        style={actionBtnStyle}
                      >
                        ✨ Ask AI
                      </button>
                    </div>
                    {block.output && (
                      <pre
                        style={{
                          margin: 0,
                          padding: '8px 10px',
                          fontSize: 12,
                          lineHeight: 1.4,
                          fontFamily: 'JetBrains Mono, Monaco, monospace',
                          color: '#d1d5db',
                          whiteSpace: 'pre-wrap',
                          wordBreak: 'break-word',
                          maxHeight: 240,
                          overflowY: 'auto',
                        }}
                      >
                        {block.output.length > 4000 ? `${block.output.slice(0, 4000)}\n… [truncated]` : block.output}
                      </pre>
                    )}
                  </>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};

export default BlockList;
