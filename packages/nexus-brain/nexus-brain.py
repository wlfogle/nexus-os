#!/usr/bin/env python3
"""
nexus-brain — Phase 1 (self-contained idea/thought capture + search).

A NexusOS package that is ALSO runnable standalone for testing. Pure Python
standard library only (http.server + sqlite3), matching the unified-guide
service pattern: no pip install, no venv, no external services required.

  Run standalone:   python3 nexus-brain.py
  Then open:        http://127.0.0.1:8700
  Self-test:        python3 nexus-brain.py --selftest

Phase 2 (Ollama embeddings -> semantic search/chat) and Phase 3 (promote an
idea to an n8n workflow) bolt on later WITHOUT the core depending on them.

Config (CLI overrides env):
  --host / NEXUS_BRAIN_HOST   default 127.0.0.1
  --port / NEXUS_BRAIN_PORT   default 8700
  --db   / NEXUS_BRAIN_DB     default ./nexus-brain.db (standalone) ;
                              the systemd unit points this at a stable path.

License: MIT.
"""
from __future__ import annotations

import argparse
import json
import os
import sqlite3
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs

FTS_AVAILABLE = True  # decided at init_db() time

# ----------------------------------------------------------------------------- db
def get_db(path: str) -> sqlite3.Connection:
    db = sqlite3.connect(path, timeout=10)
    db.row_factory = sqlite3.Row
    db.execute("PRAGMA journal_mode=WAL")
    db.execute("PRAGMA synchronous=NORMAL")
    db.execute("PRAGMA foreign_keys=ON")
    return db


def init_db(path: str) -> None:
    global FTS_AVAILABLE
    os.makedirs(os.path.dirname(os.path.abspath(path)) or ".", exist_ok=True)
    db = get_db(path)
    db.executescript(
        """
        CREATE TABLE IF NOT EXISTS notes (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            text       TEXT NOT NULL,
            tags       TEXT NOT NULL DEFAULT '',
            source     TEXT NOT NULL DEFAULT 'web',
            status     TEXT NOT NULL DEFAULT 'inbox',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_notes_status  ON notes(status);
        CREATE INDEX IF NOT EXISTS idx_notes_created ON notes(created_at);
        """
    )
    # Full-text search is a nice-to-have: degrade to LIKE if FTS5 is missing.
    try:
        db.executescript(
            """
            CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts
                USING fts5(text, tags, content='notes', content_rowid='id');
            CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
                INSERT INTO notes_fts(rowid, text, tags) VALUES (new.id, new.text, new.tags);
            END;
            CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, text, tags)
                    VALUES('delete', old.id, old.text, old.tags);
            END;
            CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, text, tags)
                    VALUES('delete', old.id, old.text, old.tags);
                INSERT INTO notes_fts(rowid, text, tags) VALUES (new.id, new.text, new.tags);
            END;
            """
        )
        db.commit()
    except sqlite3.OperationalError:
        FTS_AVAILABLE = False
    db.close()


def add_note(path, text, tags="", source="web"):
    now = int(time.time())
    db = get_db(path)
    cur = db.execute(
        "INSERT INTO notes(text, tags, source, status, created_at, updated_at) "
        "VALUES (?,?,?,'inbox',?,?)",
        (text.strip(), tags.strip(), source.strip() or "web", now, now),
    )
    db.commit()
    rid = cur.lastrowid
    db.close()
    return rid


def set_status(path, note_id, status):
    db = get_db(path)
    db.execute("UPDATE notes SET status=?, updated_at=? WHERE id=?",
               (status, int(time.time()), note_id))
    db.commit()
    changed = db.total_changes
    db.close()
    return changed


def list_notes(path, limit=50, status=None):
    db = get_db(path)
    if status:
        rows = db.execute(
            "SELECT * FROM notes WHERE status=? ORDER BY created_at DESC LIMIT ?",
            (status, limit),
        ).fetchall()
    else:
        rows = db.execute(
            "SELECT * FROM notes ORDER BY created_at DESC LIMIT ?", (limit,)
        ).fetchall()
    db.close()
    return [dict(r) for r in rows]


def search_notes(path, query, limit=50):
    query = (query or "").strip()
    if not query:
        return list_notes(path, limit)
    db = get_db(path)
    try:
        if FTS_AVAILABLE:
            tokens = [t for t in query.replace('"', " ").split() if t]
            match = " ".join('"%s"' % t for t in tokens) or '""'
            rows = db.execute(
                "SELECT n.* FROM notes_fts f JOIN notes n ON n.id=f.rowid "
                "WHERE notes_fts MATCH ? ORDER BY rank LIMIT ?",
                (match, limit),
            ).fetchall()
        else:
            like = f"%{query}%"
            rows = db.execute(
                "SELECT * FROM notes WHERE text LIKE ? OR tags LIKE ? "
                "ORDER BY created_at DESC LIMIT ?",
                (like, like, limit),
            ).fetchall()
    except sqlite3.OperationalError:
        like = f"%{query}%"
        rows = db.execute(
            "SELECT * FROM notes WHERE text LIKE ? OR tags LIKE ? "
            "ORDER BY created_at DESC LIMIT ?",
            (like, like, limit),
        ).fetchall()
    db.close()
    return [dict(r) for r in rows]


# ----------------------------------------------------------------------------- web UI
INDEX_HTML = """<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>nexus-brain</title>
<style>
 :root{color-scheme:dark}
 body{font-family:system-ui,sans-serif;max-width:820px;margin:0 auto;padding:1rem;
      background:#0f1115;color:#e6e6e6}
 h1{font-size:1.25rem;color:#67d3ff;margin:.2rem 0 1rem}
 textarea,input{width:100%;box-sizing:border-box;background:#171a21;color:#e6e6e6;
      border:1px solid #2a2f3a;border-radius:8px;padding:.6rem;font-size:1rem}
 textarea{height:5rem;resize:vertical}
 .row{display:flex;gap:.5rem;margin:.5rem 0}
 button{background:#1f6feb;color:#fff;border:0;border-radius:8px;padding:.6rem 1rem;
      font-size:1rem;cursor:pointer}
 button.secondary{background:#2a2f3a}
 .note{border:1px solid #2a2f3a;border-radius:8px;padding:.6rem;margin:.5rem 0;background:#13161c}
 .meta{font-size:.75rem;color:#8b97a7;margin-top:.35rem}
 .tag{display:inline-block;background:#22303f;color:#7fd7ff;border-radius:6px;
      padding:0 .4rem;margin-right:.3rem;font-size:.7rem}
 .status{float:right;font-size:.7rem;color:#8b97a7}
</style></head>
<body>
 <h1>nexus-brain</h1>
 <textarea id="t" placeholder="Dump a thought or idea... (Ctrl+Enter to save)"></textarea>
 <div class="row">
   <input id="tags" placeholder="tags (comma separated, optional)">
   <button onclick="capture()">Capture</button>
 </div>
 <div class="row">
   <input id="q" placeholder="search..." oninput="debounced()">
   <button class="secondary" onclick="load()">Recent</button>
 </div>
 <div id="list"></div>
<script>
const el=id=>document.getElementById(id);
async function capture(){
  const text=el('t').value.trim(); if(!text)return;
  await fetch('/api/capture',{method:'POST',headers:{'Content-Type':'application/json'},
    body:JSON.stringify({text,tags:el('tags').value})});
  el('t').value='';el('tags').value='';load();
}
async function load(){render(await(await fetch('/api/list')).json());}
async function search(){
  const q=el('q').value.trim();
  render(await(await fetch('/api/search?q='+encodeURIComponent(q))).json());
}
let timer;function debounced(){clearTimeout(timer);timer=setTimeout(search,200);}
function render(items){
  el('list').innerHTML=(items.notes||[]).map(n=>{
    const tags=(n.tags||'').split(',').filter(x=>x.trim())
      .map(t=>'<span class="tag">'+t.trim()+'</span>').join('');
    const when=new Date(n.created_at*1000).toLocaleString();
    return '<div class="note"><span class="status">'+n.status+'</span>'+
      esc(n.text).replace(/\\n/g,'<br>')+
      '<div class="meta">'+tags+' '+when+' · #'+n.id+'</div></div>';
  }).join('')||'<p style="color:#8b97a7">nothing yet</p>';
}
function esc(s){return s.replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));}
el('t').addEventListener('keydown',e=>{if(e.ctrlKey&&e.key==='Enter')capture();});
load();
</script>
</body></html>"""


# ----------------------------------------------------------------------------- http
class Handler(BaseHTTPRequestHandler):
    db_path = "nexus-brain.db"

    def _send(self, code, body, ctype="application/json"):
        data = body if isinstance(body, bytes) else body.encode()
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def _json(self, code, obj):
        self._send(code, json.dumps(obj), "application/json")

    def log_message(self, fmt, *args):  # quieter logging
        sys.stderr.write("nexus-brain %s - %s\n" % (self.address_string(), fmt % args))

    def do_GET(self):
        u = urlparse(self.path)
        if u.path == "/" or u.path == "/index.html":
            return self._send(200, INDEX_HTML, "text/html; charset=utf-8")
        if u.path == "/healthz":
            return self._json(200, {"ok": True, "fts": FTS_AVAILABLE})
        if u.path == "/api/list":
            q = parse_qs(u.query)
            limit = int((q.get("limit", ["50"])[0]) or 50)
            status = (q.get("status", [None])[0])
            return self._json(200, {"notes": list_notes(self.db_path, limit, status)})
        if u.path == "/api/search":
            q = parse_qs(u.query)
            return self._json(200, {"notes": search_notes(self.db_path, q.get("q", [""])[0])})
        return self._json(404, {"error": "not found"})

    def _read_json(self):
        length = int(self.headers.get("Content-Length", 0) or 0)
        if not length:
            return {}
        try:
            return json.loads(self.rfile.read(length).decode() or "{}")
        except json.JSONDecodeError:
            return {}

    def do_POST(self):
        u = urlparse(self.path)
        if u.path == "/api/capture":
            body = self._read_json()
            text = (body.get("text") or "").strip()
            if not text:
                return self._json(400, {"error": "text required"})
            rid = add_note(self.db_path, text, body.get("tags", ""),
                           body.get("source", "web"))
            return self._json(200, {"ok": True, "id": rid})
        if u.path.startswith("/api/note/") and u.path.endswith("/status"):
            try:
                note_id = int(u.path.split("/")[3])
            except (IndexError, ValueError):
                return self._json(400, {"error": "bad id"})
            status = (self._read_json().get("status") or "").strip()
            if status not in ("inbox", "idea", "actionable", "done", "archived"):
                return self._json(400, {"error": "bad status"})
            set_status(self.db_path, note_id, status)
            return self._json(200, {"ok": True})
        return self._json(404, {"error": "not found"})


# ----------------------------------------------------------------------------- main
def selftest(db_path):
    if os.path.exists(db_path):
        os.remove(db_path)
    init_db(db_path)
    a = add_note(db_path, "wire jellyfin transcode idea", "media,jellyfin")
    add_note(db_path, "grocery list", "personal")
    hits = search_notes(db_path, "jellyfin")
    assert any(n["id"] == a for n in hits), "search failed to find note"
    assert len(list_notes(db_path)) == 2, "list count wrong"
    set_status(db_path, a, "actionable")
    assert list_notes(db_path, status="actionable")[0]["id"] == a, "status filter failed"
    os.remove(db_path)
    print("SELFTEST OK (fts=%s)" % FTS_AVAILABLE)
    return 0


def main():
    ap = argparse.ArgumentParser(description="nexus-brain idea capture/search service")
    ap.add_argument("--host", default=os.environ.get("NEXUS_BRAIN_HOST", "127.0.0.1"))
    ap.add_argument("--port", type=int, default=int(os.environ.get("NEXUS_BRAIN_PORT", "8700")))
    ap.add_argument("--db", default=os.environ.get("NEXUS_BRAIN_DB", "nexus-brain.db"))
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()

    if args.selftest:
        return selftest(args.db)

    init_db(args.db)
    Handler.db_path = args.db
    httpd = ThreadingHTTPServer((args.host, args.port), Handler)
    print(f"nexus-brain on http://{args.host}:{args.port}  (db={args.db}, fts={FTS_AVAILABLE})",
          file=sys.stderr)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        httpd.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
