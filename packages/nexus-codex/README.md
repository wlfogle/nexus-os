# nexus-codex

**NexusOS Documentation Intelligence** — AI-powered desktop app that scans your laptop and all GitHub repos, classifies every doc file for freshness, and generates a non-destructive report. Never modifies source files.

Built with Tauri v2 (Rust + Svelte), using local Ollama models.

---

## Features

- **Full filesystem scan** — walks `$HOME` respecting `.gitignore`, finds `.md`, `.txt`, `.pdf`, `.rst`, `.adoc`
- **GitHub integration** — scans all repos via GitHub API using `gh auth token` (no credentials stored)
- **PDF extraction** — `pdf-extract` crate with `pdftotext` fallback
- **AI classification** — each doc classified as `current`, `stale`, `outdated`, `orphaned`, or `needs_review`
- **Auto model selection** — scores all local Ollama models, picks the best coding-capable one
- **Live progress UI** — real-time feed as each document is processed
- **Report viewer** — filterable table with evidence, suggested rewrites (read-only), confidence scores
- **Export** — Markdown or JSON report via file picker
- **Non-destructive** — zero writes to any scanned source file, ever

## Classification

| Status | Meaning |
|---|---|
| `current` | Accurately reflects current code/project state |
| `stale` | Partially outdated but salvageable |
| `outdated` | Significantly behind and misleading |
| `orphaned` | No related code or project found |
| `needs_review` | Confidence too low to classify reliably |

## Model auto-selection priority

`qwen` > `codestral` > `deepseek-coder` > `codellama` > `llama3.1` > `llama3` > `mistral` > largest other

Override in the Config tab if needed.

## Build

### Prerequisites

- Rust stable 1.75+
- Node.js 18+, npm
- Ollama running at `http://localhost:11434` with at least one model
- `gh` CLI authenticated (`gh auth login`)

```bash
cd packages/nexus-codex
npm install
npm run tauri dev      # development
npm run tauri build    # production
```

## Architecture

```
nexus-codex/
├── src-tauri/src/
│   ├── types.rs      — shared Serde types (IPC contract)
│   ├── state.rs      — scan registry + config mutex
│   ├── config.rs     — load/save (~/.config/nexus-codex/config.json)
│   ├── commands.rs   — 8 Tauri IPC commands
│   ├── scanner.rs    — .gitignore-aware local filesystem walk
│   ├── repo.rs       — git repo detection + file age via git log
│   ├── github.rs     — GitHub REST API (repos, tree, raw content)
│   ├── pdf.rs        — PDF text extraction + pdftotext fallback
│   ├── ollama.rs     — model scoring, /api/generate analysis
│   ├── analyzer.rs   — full scan orchestration, event emission
│   └── report.rs     — build_report, export_markdown, export_json
└── src/lib/
    ├── types.ts           — TypeScript mirror of Rust types
    ├── api.ts             — typed invoke() + event listeners
    ├── stores.svelte.ts   — Svelte 5 runes stores
    └── components/
        ├── ConfigView.svelte  — settings, model selector, folder picker
        ├── ScanView.svelte    — live progress, cancel
        └── ReportView.svelte  — filterable report, export
```

## NexusOS integration

`nexus-codex` is the documentation intelligence subsystem for NexusOS. Future: exposed as `nexus.codex` IPC service that personality servers can query for system documentation state.
