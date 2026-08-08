# Ecosystem Librarian: Advanced Enhancements & Optimization Blueprint

This document aggregates the architectural, operational, and safety enhancements designed to transform your self-hosted "Librarian Janitor" into a professional-grade, repo-aware ecosystem organizer for your Pop!_OS workstation.

---

## 1. Core Ingestion & Performance Pipeline
* **Two-Stage Ingestion:** 
  * *Stage A (Lightweight):* Scan paths, extensions, sizes, and mtimes.
  * *Stage B (Deep Extraction):* Invoke embeddings/text extraction only on high-confidence candidates to minimize CPU usage.
* **Incremental Indexing & Deduplication:** 
  * Use a local SQLite state database for path/mtime/hash tracking. 
  * Skip reprocessing unchanged files; detect moved files via hash matching to re-attach existing metadata.
* **Inotify-Driven Real-Time Watcher:** 
  * Utilize `inotify` / `watchfiles` to trigger incremental indexing on file system events, keeping the librarian perpetually synchronized without massive re-scans.
* **Entropy-Based Noise Filtering:** 
  * Discard binary bloat, build artifacts (`node_modules`, `target`, `.git/objects`), and system noise using path deny-lists and file entropy analysis before deep processing.

## 2. Repo-Aware Relevance & Categorization
* **Zero-Copy Symbolic Indexing:** 
  * Before physical movement, create a virtual overlay using symlinks to provide a categorized logical view (`~/Library/Librarian/`) without disturbing original file locations.
* **Hybrid Retrieval (BM25 + Vector):** 
  * Combine keyword-based sparse search (for technical terms/code) with vector-based semantic search (for conceptual ideas) to minimize false positives.
* **Git-Aware Contextual Weighting:** 
  * Dynamically weight file relevance based on Git commit activity. Files linked to active repositories receive higher priority and lower "staleness" scores.
* **Canonical Source & Supersession Graph:** 
  * Maintain a tracking graph that identifies "canonical" sources (in-repo) vs. auxiliary notes, allowing you to explicitly label/identify outdated information.

## 3. Safe Operations & Safeguards
* **Quarantine-First Lifecycle:** 
  * Direct all deletion requests to `~/Library/Librarian/Quarantine/`. Implement an automated 30-day purge timer with pre-purge notifications.
* **Deterministic Rollback Journals:** 
  * All batch operations are executed via a SQLite-backed transaction journal. Every plan is atomic; if an interruption occurs, the journal enables an exact reversal to the previous state.
* **Path Normalization Layer:** 
  * Automatically identify and rewrite internal references (symlinks, local imports, hard-coded script paths) during moves to prevent broken functionality.

## 4. Interaction & Maintenance
* **Sidecar Summarization:** 
  * Leverage a local LLM endpoint to generate concise structural summaries and tags for files classified as `UNKNOWN` or `RELEVANT_STALE`, enhancing searchability without external data exposure.
* **Local Web-UI Dashboard:** 
  * A local-only web dashboard (bound to `127.0.0.1`) that visualizes confidence scores, category distributions, and enables one-click approval of categorization plans.
* **Human-in-the-Loop Feedback:** 
  * Record acceptance/rejection decisions to refine classification weights over time, allowing the system to learn your organizational preferences.
