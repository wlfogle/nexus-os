/**
 * Command Routing — unit tests
 *
 * Covers the synchronous fallback path (used when Tauri is not running) and
 * the NL→shell direct-translation map.  Tauri's invoke and the logger are
 * mocked so these tests run in a plain Node environment via vitest.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// ── Mocks (must be declared before any import that triggers them) ──────────────

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

vi.mock('../utils/logger', () => ({
  routingLogger: { debug: vi.fn(), error: vi.fn(), info: vi.fn(), warn: vi.fn() },
  terminalLogger: { debug: vi.fn(), error: vi.fn(), info: vi.fn(), warn: vi.fn() },
  commandRoutingLogger: { debug: vi.fn(), error: vi.fn(), info: vi.fn(), warn: vi.fn() },
  createServiceLogger: () => ({ debug: vi.fn(), error: vi.fn(), info: vi.fn(), warn: vi.fn() }),
  createComponentLogger: () => ({ debug: vi.fn(), error: vi.fn(), info: vi.fn(), warn: vi.fn() }),
}));

// ── Imports (after mocks) ─────────────────────────────────────────────────────

import { translateNLToShell, isShellCommand, CommandRoutingService } from './commandRouting';

// ── translateNLToShell ────────────────────────────────────────────────────────

describe('translateNLToShell', () => {
  it('maps "list files" → ls -la', () => {
    expect(translateNLToShell('list files')).toBe('ls -la');
  });

  it('maps "show files" → ls -la', () => {
    expect(translateNLToShell('show files')).toBe('ls -la');
  });

  it('maps "show me what is here" → ls -la', () => {
    expect(translateNLToShell("show me what's here")).toBe('ls -la');
  });

  it('maps "where am i" → pwd', () => {
    expect(translateNLToShell('where am i')).toBe('pwd');
  });

  it('maps "current dir" → pwd', () => {
    expect(translateNLToShell('current dir')).toBe('pwd');
  });

  it('maps "check disk space" → df -h', () => {
    expect(translateNLToShell('check disk space')).toBe('df -h');
  });

  it('maps "how much disk usage" → df -h', () => {
    expect(translateNLToShell('how much disk usage')).toBe('df -h');
  });

  it('maps "show memory" → free -h', () => {
    expect(translateNLToShell('show memory')).toBe('free -h');
  });

  it('maps "ram usage" → free -h', () => {
    expect(translateNLToShell('ram usage')).toBe('free -h');
  });

  it('maps "what is running" → ps aux', () => {
    expect(translateNLToShell('what is running')).toBe('ps aux');
  });

  it('maps "show network" → ip -brief addr', () => {
    expect(translateNLToShell('show network')).toBe('ip -brief addr');
  });

  it('maps "git status" → git status --short --branch', () => {
    expect(translateNLToShell('git status')).toBe('git status --short --branch');
  });

  it('maps "recent commits" → git log', () => {
    expect(translateNLToShell('recent commits')).toBe('git --no-pager log --oneline -10');
  });

  it('maps "cpu" → uptime', () => {
    expect(translateNLToShell('cpu')).toBe('uptime');
  });

  it('maps "top processes" → ps aux sorted', () => {
    expect(translateNLToShell('top processes')).toBe('ps aux --sort=-%cpu | head -10');
  });

  it('returns null for unrecognised phrases', () => {
    expect(translateNLToShell('explain rust lifetimes')).toBeNull();
    expect(translateNLToShell('fix the build')).toBeNull();
    expect(translateNLToShell('scan system and optimize')).toBeNull();
  });
});

// ── isShellCommand (synchronous fallback) ────────────────────────────────────

describe('isShellCommand — shell commands correctly identified', () => {
  it('ls -la → shell', () => expect(isShellCommand('ls -la')).toBe(true));
  it('ls → shell', () => expect(isShellCommand('ls')).toBe(true));
  it('pwd → shell', () => expect(isShellCommand('pwd')).toBe(true));
  it('cd /home → shell', () => expect(isShellCommand('cd /home')).toBe(true));
  it('git status → shell', () => expect(isShellCommand('git status')).toBe(true));
  it('git commit -m "msg" → shell', () => expect(isShellCommand('git commit -m "msg"')).toBe(true));
  it('cargo check → shell', () => expect(isShellCommand('cargo check')).toBe(true));
  it('npm run build → shell', () => expect(isShellCommand('npm run build')).toBe(true));
  it('docker ps → shell', () => expect(isShellCommand('docker ps')).toBe(true));
  it('sudo systemctl restart nginx → shell', () => expect(isShellCommand('sudo systemctl restart nginx')).toBe(true));
  it('./run.sh → shell', () => expect(isShellCommand('./run.sh')).toBe(true));
  it('/usr/bin/env node → shell', () => expect(isShellCommand('/usr/bin/env node')).toBe(true));
  it('cat src/main.rs → shell', () => expect(isShellCommand('cat src/main.rs')).toBe(true));
  it('grep -r TODO . → shell', () => expect(isShellCommand('grep -r TODO .')).toBe(true));
  it('ps aux | grep ollama → shell (pipe)', () => expect(isShellCommand('ps aux | grep ollama')).toBe(true));
  it('FOO=bar command → shell (env var)', () => expect(isShellCommand('FOO=bar command')).toBe(true));
});

describe('isShellCommand — natural language correctly rejected', () => {
  it('"list files" → AI', () => expect(isShellCommand('list files')).toBe(false));
  it('"what is running" → AI', () => expect(isShellCommand('what is running')).toBe(false));
  it('"how do I fix this error" → AI', () => expect(isShellCommand('how do I fix this error')).toBe(false));
  it('"explain rust lifetimes" → AI', () => expect(isShellCommand('explain rust lifetimes')).toBe(false));
  it('"help me debug this" → AI', () => expect(isShellCommand('help me debug this')).toBe(false));
  it('"why is my build failing" → AI', () => expect(isShellCommand('why is my build failing')).toBe(false));
  it('"what is in this directory" → AI', () => expect(isShellCommand('what is in this directory')).toBe(false));
  it('"can you fix the errors" → AI', () => expect(isShellCommand('can you fix the errors')).toBe(false));
  it('"scan system and optimize" → AI', () => expect(isShellCommand('scan system and optimize')).toBe(false));
  it('"optimize memory usage" → AI', () => expect(isShellCommand('optimize memory usage')).toBe(false));
  it('"show me how to use git" → AI', () => expect(isShellCommand('show me how to use git')).toBe(false));
});

describe('isShellCommand — edge cases', () => {
  it('empty string → false', () => expect(isShellCommand('')).toBe(false));
  it('whitespace-only → false', () => expect(isShellCommand('   ')).toBe(false));
  it('single "?" → AI', () => expect(isShellCommand('?')).toBe(false));
});

// ── Warp-aligned shell vs agent routing cases ─────────────────────────────
// These mirror the routing decisions in Warp's HeuristicClassifier.

describe('isShellCommand — Warp-aligned shell commands', () => {
  // Core unix commands
  it('cargo build → shell', () => expect(isShellCommand('cargo build')).toBe(true));
  it('cargo test → shell', () => expect(isShellCommand('cargo test')).toBe(true));
  it('cargo run → shell', () => expect(isShellCommand('cargo run')).toBe(true));
  it('make install → shell', () => expect(isShellCommand('make install')).toBe(true));
  it('vim src/main.rs → shell', () => expect(isShellCommand('vim src/main.rs')).toBe(true));
  it('ssh user@host → shell', () => expect(isShellCommand('ssh user@host')).toBe(true));
  it('curl https://example.com → shell', () => expect(isShellCommand('curl https://example.com')).toBe(true));
  it('kill -9 1234 → shell', () => expect(isShellCommand('kill -9 1234')).toBe(true));
  it('find . -name "*.rs" → shell', () => expect(isShellCommand('find . -name "*.rs"')).toBe(true));
  it('tar -czf archive.tar.gz src/ → shell', () => expect(isShellCommand('tar -czf archive.tar.gz src/')).toBe(true));
});

describe('isShellCommand — Warp-aligned natural language → agent', () => {
  // Natural language instructions that should go to AI, not shell
  it('write a function to sort an array → AI', () => {
    expect(isShellCommand('write a function to sort an array')).toBe(false);
  });
  it('what does cargo build do → AI', () => {
    expect(isShellCommand('what does cargo build do')).toBe(false);
  });
  it('how do I use git rebase → AI', () => {
    expect(isShellCommand('how do I use git rebase')).toBe(false);
  });
  it('generate a dockerfile for node → AI', () => {
    expect(isShellCommand('generate a dockerfile for node')).toBe(false);
  });
  it('review my last commit → AI', () => {
    expect(isShellCommand('review my last commit')).toBe(false);
  });
  it('tell me about rust ownership → AI', () => {
    expect(isShellCommand('tell me about rust ownership')).toBe(false);
  });
  it('show me how to write a makefile → AI', () => {
    expect(isShellCommand('show me how to write a makefile')).toBe(false);
  });
});

// ── CommandRoutingService.isShellCommand (instance method) ──────────────────

describe('CommandRoutingService.isShellCommand (instance)', () => {
  const svc = new CommandRoutingService();

  it('identifies systemctl as shell', () => {
    expect(svc.isShellCommand('systemctl status nginx')).toBe(true);
  });

  it('identifies ping as shell', () => {
    expect(svc.isShellCommand('ping google.com')).toBe(true);
  });

  it('rejects question → AI', () => {
    expect(svc.isShellCommand('what services are running?')).toBe(false);
  });
});
