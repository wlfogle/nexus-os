// Ported from warpdotdev/warp (AGPL-3.0) — block list consumer.
// Subscribes to block:start / block:output / block:end Tauri events and renders
// completed + in-progress blocks above the raw xterm display.
import React, { useEffect, useRef } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { listen } from '@tauri-apps/api/event';
import {
  blockStarted,
  blockOutput,
  blockEnded,
  selectLiveBlocksForTerminal,
} from '../../store/slices/terminalSlice';
import type {
  BlockStartPayload,
  BlockOutputPayload,
  BlockEndPayload,
} from '../../types/terminal';
import BlockItem from './BlockItem';

interface BlockListProps {
  /** Filter blocks by this terminal ID — only show blocks for the active terminal. */
  terminalId: string | null | undefined;
  /** Max height of the scrollable block list. */
  maxHeight?: number;
}

/**
 * Subscribes to the three block Tauri events and renders a Warp-style block list.
 * Multiple instances can coexist safely — each filters by terminalId.
 */
const BlockList: React.FC<BlockListProps> = ({ terminalId, maxHeight = 400 }) => {
  const dispatch = useDispatch();
  const blocks = useSelector(
    selectLiveBlocksForTerminal(terminalId ?? '__none__')
  );
  const bottomRef = useRef<HTMLDivElement>(null);

  // ── Subscribe to block events ─────────────────────────────────────────────
  useEffect(() => {
    if (!terminalId) return;

    const unlisteners: Array<() => void> = [];

    const setup = async () => {
      const u1 = await listen<BlockStartPayload>('block:start', ({ payload }) => {
        if (payload.terminalId !== terminalId) return;
        dispatch(blockStarted({
          blockId: payload.blockId,
          terminalId: payload.terminalId,
          command: payload.command,
          cwd: payload.cwd,
          startedAt: payload.startedAt,
        }));
      });
      unlisteners.push(u1);

      const u2 = await listen<BlockOutputPayload>('block:output', ({ payload }) => {
        if (payload.terminalId !== terminalId) return;
        dispatch(blockOutput({
          blockId: payload.blockId,
          chunk: payload.chunk,
          stream: payload.stream,
        }));
      });
      unlisteners.push(u2);

      const u3 = await listen<BlockEndPayload>('block:end', ({ payload }) => {
        if (payload.terminalId !== terminalId) return;
        dispatch(blockEnded({
          blockId: payload.blockId,
          exitCode: payload.exitCode,
          endedAt: payload.endedAt,
          durationMs: payload.durationMs,
        }));
      });
      unlisteners.push(u3);
    };

    setup().catch(console.error);

    return () => {
      unlisteners.forEach(u => u());
    };
  }, [terminalId, dispatch]);

  // Auto-scroll to bottom when a new block arrives
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [blocks.length]);

  if (blocks.length === 0) return null;

  return (
    <div
      style={{
        maxHeight,
        overflowY: 'auto',
        flexShrink: 0,
        borderBottom: '1px solid #1f2937',
        paddingTop: 4,
        paddingBottom: 4,
      }}
    >
      {blocks.map(block => (
        <BlockItem key={block.blockId} block={block} />
      ))}
      <div ref={bottomRef} />
    </div>
  );
};

export default BlockList;
