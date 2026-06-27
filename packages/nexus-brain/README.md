# nexus-brain

> A self-contained NexusOS package to **capture thoughts/ideas, search them, and
> (later) turn them into actions** (e.g. n8n workflows). Ships in the NexusOS
> install, but is built to **run standalone for testing**.
>
> Status flagged promising — parked for later. This README is the memory.

## Why
Offload "too much info to handle by myself": a personal second brain — capture
from any device, retrieve instantly, and promote the actionable ones into
automations. Self-contained first; AI and automation bolt on without the core
depending on them.

## Status
- **Phase 1 — implemented** (`nexus-brain.py`): pure-stdlib (`http.server` +
  `sqlite3`/FTS5), zero pip deps, embedded web UI. Capture + tag + full-text
  search + status (inbox/idea/actionable/done/archived). REST API + `/healthz`.
  - Verify: `python3 nexus-brain.py --selftest`
  - Run: `python3 nexus-brain.py` → http://127.0.0.1:8700
  - Config: `--host/--port/--db` or `NEXUS_BRAIN_{HOST,PORT,DB}`.

## Roadmap (later)
- **Phase 2 — AI layer (optional):** Ollama embeddings (`nomic-embed-text`) in
  SQLite → semantic search + chat-with-your-notes. Degrades gracefully if Ollama
  is down.
- **Phase 3 — action layer (optional):** promote an idea → fire an **n8n**
  webhook; then draft an n8n workflow JSON from the idea via Ollama.
- **Install integration:** systemd unit, served behind Caddy on CT-300; capture
  from phones via the Syncthing/file-share path; reuse n8n + Ollama already on
  CT-300.

## Design rule
Self-contained core (capture + search work with no external service). Ollama
(AI) and n8n (action) are optional plug-ins, never hard dependencies.
