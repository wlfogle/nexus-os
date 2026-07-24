// Ported from warpdotdev/warp (AGPL-3.0) — Warp-style block renderer.
// Each block represents one completed (or running) terminal command + its output.
import React, { useState } from 'react';
import type { LiveBlock } from '../../types/terminal';

interface BlockItemProps {
  block: LiveBlock;
}

/** Format a duration in milliseconds into a human-readable string. */
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
  const minutes = Math.floor(ms / 60_000);
  const seconds = Math.floor((ms % 60_000) / 1000);
  return `${minutes}m ${seconds}s`;
}

/** Format epoch ms as HH:MM:SS. */
function formatTime(epochMs: number): string {
  return new Date(epochMs).toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

const BlockItem: React.FC<BlockItemProps> = ({ block }) => {
  const [collapsed, setCollapsed] = useState(false);

  const isSuccess = block.exitCode === 0;
  const isRunning = block.status === 'running';
  const hasFailed = !isRunning && block.exitCode !== null && block.exitCode !== 0;

  // Build full output text from chunks (merge consecutive same-stream chunks)
  const outputText = block.chunks.map(c => c.text).join('');

  // Status indicator colors
  const statusColor = isRunning
    ? '#f59e0b'   // amber — running
    : isSuccess
      ? '#22c55e' // green — success
      : '#ef4444'; // red — failure

  const cwd = block.cwd.replace(/^\/home\/[^/]+/, '~');

  return (
    <div
      style={{
        margin: '4px 8px',
        borderRadius: 6,
        border: `1px solid ${isRunning ? '#374151' : hasFailed ? 'rgba(239,68,68,0.3)' : '#1f2937'}`,
        background: '#111114',
        overflow: 'hidden',
      }}
    >
      {/* Block header: status dot + command + meta */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: '5px 10px',
          background: '#18181b',
          cursor: 'pointer',
          userSelect: 'none',
        }}
        onClick={() => setCollapsed(c => !c)}
      >
        {/* Status dot */}
        <span
          style={{
            width: 7,
            height: 7,
            borderRadius: '50%',
            background: statusColor,
            flexShrink: 0,
            boxShadow: isRunning ? `0 0 6px ${statusColor}` : 'none',
          }}
        />

        {/* Collapse chevron */}
        <span style={{ color: '#4b5563', fontSize: 10, flexShrink: 0 }}>
          {collapsed ? '▶' : '▼'}
        </span>

        {/* Command */}
        <span
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: 13,
            color: hasFailed ? '#fca5a5' : '#e5e7eb',
            flex: 1,
            minWidth: 0,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {block.command || '(no command)'}
        </span>

        {/* Right meta: cwd + duration + time */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexShrink: 0 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: '#6b7280' }}>{cwd}</span>
          {block.durationMs !== null && (
            <span style={{ fontSize: 11, color: '#4b5563' }}>{formatDuration(block.durationMs)}</span>
          )}
          <span style={{ fontSize: 11, color: '#374151' }}>{formatTime(block.startedAt)}</span>
          {block.exitCode !== null && block.exitCode !== 0 && (
            <span
              style={{
                fontSize: 11,
                color: '#ef4444',
                background: 'rgba(239,68,68,0.15)',
                borderRadius: 3,
                padding: '0 5px',
              }}
            >
              exit {block.exitCode}
            </span>
          )}
          {isRunning && (
            <span style={{ fontSize: 11, color: '#f59e0b', fontStyle: 'italic' }}>running</span>
          )}
        </div>
      </div>

      {/* Block output (collapsed or expanded) */}
      {!collapsed && outputText && (
        <div
          style={{
            padding: '6px 10px 6px 25px',
            fontFamily: 'var(--font-mono)',
            fontSize: 12,
            lineHeight: 1.5,
            color: hasFailed ? '#fca5a5' : '#9ca3af',
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-all',
            maxHeight: 300,
            overflowY: 'auto',
            borderTop: '1px solid #1f2937',
          }}
        >
          {outputText}
        </div>
      )}
    </div>
  );
};

export default BlockItem;
