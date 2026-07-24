// Ported from warpdotdev/warp (AGPL-3.0) — command palette overlay.
// Opened by Ctrl/Cmd+P. Fuzzy-searches commands, tabs, and actions.
import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import {
  selectAllTabs,
  setActiveTab,
  createTab,
  closeTab,
} from '../store/slices/terminalTabSlice';
import { ShellType } from '../types/terminal';

interface PaletteItem {
  id: string;
  label: string;
  description?: string;
  icon: string;
  category: 'tab' | 'action' | 'command';
  action: () => void;
}

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
}

const CommandPalette: React.FC<CommandPaletteProps> = ({ open, onClose }) => {
  const dispatch = useDispatch();
  const tabs = useSelector(selectAllTabs);
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // ── Build the full item list ───────────────────────────────────────────────
  const allItems = useCallback((): PaletteItem[] => {
    const items: PaletteItem[] = [];

    // Tab items
    tabs.forEach(tab => {
      items.push({
        id: `tab:${tab.id}`,
        label: tab.title,
        description: tab.workingDirectory,
        icon: '🖥️',
        category: 'tab',
        action: () => {
          dispatch(setActiveTab(tab.id));
          onClose();
        },
      });
    });

    // Actions
    items.push(
      {
        id: 'action:new-tab-fish',
        label: 'New Terminal Tab — Fish',
        description: 'Open a new Fish shell tab',
        icon: '🐟',
        category: 'action',
        action: () => {
          dispatch(createTab({ shell: ShellType.FISH, workingDirectory: '~' }));
          onClose();
        },
      },
      {
        id: 'action:new-tab-bash',
        label: 'New Terminal Tab — Bash',
        description: 'Open a new Bash shell tab',
        icon: '🐚',
        category: 'action',
        action: () => {
          dispatch(createTab({ shell: ShellType.BASH, workingDirectory: '~' }));
          onClose();
        },
      },
      {
        id: 'action:close-tab',
        label: 'Close Current Tab',
        description: 'Close the active terminal tab',
        icon: '✕',
        category: 'action',
        action: () => {
          const active = tabs.find(t => t.isActive);
          if (active) dispatch(closeTab(active.id));
          onClose();
        },
      },
    );

    // Recent commands from all tab histories
    const seen = new Set<string>();
    tabs.forEach(tab => {
      tab.terminalHistory.slice(-20).forEach(entry => {
        if (!seen.has(entry.command)) {
          seen.add(entry.command);
          items.push({
            id: `cmd:${entry.command}`,
            label: entry.command,
            description: `Last run in ${tab.title}`,
            icon: '⌨️',
            category: 'command',
            action: () => {
              // Just copy to clipboard — the user can then paste into the input bar
              navigator.clipboard?.writeText(entry.command).catch(() => {});
              onClose();
            },
          });
        }
      });
    });

    return items;
  }, [tabs, dispatch, onClose]);

  // ── Fuzzy filter ──────────────────────────────────────────────────────────
  const filtered = query
    ? allItems().filter(item => {
        const q = query.toLowerCase();
        return (
          item.label.toLowerCase().includes(q) ||
          (item.description ?? '').toLowerCase().includes(q)
        );
      })
    : allItems();

  // ── Reset state when opening ──────────────────────────────────────────────
  useEffect(() => {
    if (open) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [open]);

  // Clamp selected index when filter results change
  useEffect(() => {
    if (selectedIndex >= filtered.length) {
      setSelectedIndex(Math.max(0, filtered.length - 1));
    }
  }, [filtered.length, selectedIndex]);

  // ── Keyboard navigation ───────────────────────────────────────────────────
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(i => Math.min(i + 1, filtered.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(i => Math.max(i - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        filtered[selectedIndex]?.action();
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  }, [filtered, selectedIndex, onClose]);

  if (!open) return null;

  const categoryLabel: Record<string, string> = {
    tab: 'Tabs',
    action: 'Actions',
    command: 'Recent Commands',
  };

  // Group items by category for display
  let lastCategory = '';

  return (
    <>
      {/* Backdrop */}
      <div
        style={{
          position: 'fixed',
          inset: 0,
          background: 'rgba(0,0,0,0.55)',
          zIndex: 999,
        }}
        onClick={onClose}
      />

      {/* Palette panel */}
      <div
        style={{
          position: 'fixed',
          top: '15%',
          left: '50%',
          transform: 'translateX(-50%)',
          width: 560,
          maxWidth: 'calc(100vw - 32px)',
          background: '#1c1c1e',
          border: '1px solid #374151',
          borderRadius: 10,
          boxShadow: '0 24px 80px rgba(0,0,0,0.7)',
          zIndex: 1000,
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
        }}
        onKeyDown={handleKeyDown}
      >
        {/* Search input */}
        <div style={{ display: 'flex', alignItems: 'center', padding: '10px 14px', borderBottom: '1px solid #374151', gap: 10 }}>
          <span style={{ color: '#6b7280', fontSize: 16 }}>⌘</span>
          <input
            ref={inputRef}
            value={query}
            onChange={e => { setQuery(e.target.value); setSelectedIndex(0); }}
            placeholder="Search commands, tabs, actions…"
            style={{
              flex: 1,
              background: 'transparent',
              border: 'none',
              outline: 'none',
              color: '#f9fafb',
              fontSize: 14,
              fontFamily: 'var(--font-mono)',
            }}
          />
          <kbd style={{ fontSize: 10, color: '#4b5563', background: '#374151', borderRadius: 3, padding: '2px 5px' }}>ESC</kbd>
        </div>

        {/* Results list */}
        <div style={{ maxHeight: 360, overflowY: 'auto' }}>
          {filtered.length === 0 ? (
            <div style={{ padding: '20px', textAlign: 'center', color: '#6b7280', fontSize: 13 }}>
              No results for "{query}"
            </div>
          ) : (
            filtered.map((item, idx) => {
              const showHeader = item.category !== lastCategory;
              lastCategory = item.category;

              return (
                <React.Fragment key={item.id}>
                  {showHeader && (
                    <div style={{
                      padding: '6px 14px 2px',
                      fontSize: 10,
                      fontWeight: 600,
                      color: '#4b5563',
                      textTransform: 'uppercase',
                      letterSpacing: '0.08em',
                    }}>
                      {categoryLabel[item.category] ?? item.category}
                    </div>
                  )}
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 10,
                      padding: '8px 14px',
                      cursor: 'pointer',
                      background: idx === selectedIndex ? 'rgba(59,130,246,0.18)' : 'transparent',
                      borderLeft: `2px solid ${idx === selectedIndex ? '#3b82f6' : 'transparent'}`,
                      transition: 'background 0.1s',
                    }}
                    onMouseEnter={() => setSelectedIndex(idx)}
                    onClick={item.action}
                  >
                    <span style={{ fontSize: 15, flexShrink: 0 }}>{item.icon}</span>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div style={{
                        fontSize: 13,
                        color: '#e5e7eb',
                        fontFamily: item.category === 'command' ? 'var(--font-mono)' : 'inherit',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                      }}>
                        {item.label}
                      </div>
                      {item.description && (
                        <div style={{
                          fontSize: 11,
                          color: '#6b7280',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          whiteSpace: 'nowrap',
                          marginTop: 1,
                        }}>
                          {item.description}
                        </div>
                      )}
                    </div>
                  </div>
                </React.Fragment>
              );
            })
          )}
        </div>

        {/* Footer hint */}
        <div style={{
          padding: '6px 14px',
          borderTop: '1px solid #1f2937',
          display: 'flex',
          gap: 16,
          fontSize: 10,
          color: '#4b5563',
        }}>
          <span><kbd style={{ background: '#374151', borderRadius: 3, padding: '1px 4px' }}>↑↓</kbd> navigate</span>
          <span><kbd style={{ background: '#374151', borderRadius: 3, padding: '1px 4px' }}>↵</kbd> select</span>
          <span><kbd style={{ background: '#374151', borderRadius: 3, padding: '1px 4px' }}>Esc</kbd> close</span>
        </div>
      </div>
    </>
  );
};

export default CommandPalette;
