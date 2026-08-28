#!/usr/bin/env python3
"""
nexus-brain — Phase 1 (self-contained idea/thought capture + search).

A NexusOS package that is ALSO runnable standalone for testing. Pure Python
standard library only (http.server + sqlite3), matching the unified-guide
service pattern: no pip install, no venv, no external services required.

  Run standalone:   python3 nexus-brain.py
  Then open:        http://127.0.0.1:8700
  Self-test:        python3 nexus-brain.py --selftest

Phase 2 (Ollama embeddings -> semantic search/chat) is a future bolt-on.
Phase 3 (promote an idea: fire an n8n webhook, then draft an n8n workflow via
Ollama) is implemented below as an optional bolt-on too -- with both unset,
promoting a note just flips its status; nothing calls out over the network.

Config (CLI overrides env):
  --host           / NEXUS_BRAIN_HOST            default 127.0.0.1
  --port           / NEXUS_BRAIN_PORT            default 8700
  --db             / NEXUS_BRAIN_DB              default ./nexus-brain.db (standalone) ;
                                                  the systemd unit points this at a stable path.
  --n8n-webhook    / NEXUS_BRAIN_N8N_WEBHOOK     default "" (disabled)
  --ollama-url     / NEXUS_BRAIN_OLLAMA_URL      default http://127.0.0.1:11434 ; "" disables it
  --ollama-model   / NEXUS_BRAIN_OLLAMA_MODEL    default qwen2.5-coder:7b
  --n8n-timeout    / NEXUS_BRAIN_N8N_TIMEOUT     default 10 (seconds)
  --ollama-timeout / NEXUS_BRAIN_OLLAMA_TIMEOUT  default 90 (seconds)

License: MIT.
"""
from __future__ import annotations

import argparse
import json
import os
import sqlite3
import sys
import threading
import time
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs

FTS_AVAILABLE = True  # decided at init_db() time
DEFAULT_OLLAMA_URL = "http://127.0.0.1:11434"
DEFAULT_OLLAMA_MODEL = "qwen2.5-coder:7b"

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
    # Phase 3 columns, added later: migrate existing DBs in place.
    for col, decl in (("workflow_json", "TEXT"), ("promoted_at", "INTEGER")):
        try:
            db.execute("ALTER TABLE notes ADD COLUMN %s %s" % (col, decl))
        except sqlite3.OperationalError:
            pass  # column already exists
    db.commit()
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


def get_note(path, note_id):
    db = get_db(path)
    row = db.execute("SELECT * FROM notes WHERE id=?", (note_id,)).fetchone()
    db.close()
    return dict(row) if row else None


def save_workflow(path, note_id, workflow_json, promoted_at):
    db = get_db(path)
    db.execute(
        "UPDATE notes SET workflow_json=?, promoted_at=?, updated_at=? WHERE id=?",
        (workflow_json, promoted_at, promoted_at, note_id),
    )
    db.commit()
    db.close()


# ----------------------------------------------------------------------------- phase 3
# Promote an idea -> fire an n8n webhook, then draft an importable n8n workflow
# JSON for it via a local Ollama model. Both steps are optional bolt-ons: an
# empty webhook URL or Ollama URL just skips that step, so promoting a note
# always succeeds even with n8n/Ollama down or never configured.
WORKFLOW_SYSTEM_PROMPT = (
    "You draft minimal, importable n8n workflow JSON from a short idea. "
    "Reply with ONLY a JSON object shaped exactly like: "
    '{"name": string, "nodes": [{"name": string, "type": string, '
    '"typeVersion": number, "position": [number, number], "parameters": object}], '
    '"connections": object}. The first node must be a Manual Trigger '
    '(type "n8n-nodes-base.manualTrigger"). Keep it to 2-4 nodes total that '
    "plausibly act on the idea. No prose, no markdown fences, JSON only."
)


def _post_json(url, payload, timeout):
    """POST JSON with stdlib only. Returns (ok, status, body_text, error)."""
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url, data=data, method="POST",
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return True, resp.status, resp.read().decode("utf-8", "replace"), None
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8", "replace") if e.fp else ""
        return False, e.code, body, "HTTP %s: %s" % (e.code, body[:200] or e.reason)
    except (urllib.error.URLError, OSError, ValueError) as e:
        return False, None, None, str(e)


def fire_n8n_webhook(url, note, timeout=10):
    """Notify n8n that an idea was promoted. A no-op when url is falsy."""
    if not url:
        return {"attempted": False, "ok": False, "skipped": "no webhook configured"}
    payload = {
        "id": note["id"],
        "text": note["text"],
        "tags": note["tags"],
        "source": note["source"],
        "status": note["status"],
        "created_at": note["created_at"],
    }
    ok, status, _body, err = _post_json(url, payload, timeout)
    return {"attempted": True, "ok": ok, "status": status, "error": err}


def _extract_json_object(raw):
    """Recover a JSON object even if the model wrapped it in prose/fences."""
    raw = (raw or "").strip()
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        pass
    start, end = raw.find("{"), raw.rfind("}")
    if start == -1 or end == -1 or end < start:
        raise ValueError("no JSON object in model reply: %r" % raw[:200])
    return json.loads(raw[start:end + 1])


def draft_workflow_with_ollama(base_url, model, note, timeout=90):
    """Ask a local Ollama model to draft an n8n workflow for this idea."""
    if not base_url:
        return {"attempted": False, "ok": False, "skipped": "no ollama url configured"}
    prompt = "Idea:\n%s\n\nTags: %s\n\nDraft the n8n workflow JSON now." % (
        note["text"], note["tags"] or "(none)",
    )
    payload = {
        "model": model,
        "prompt": prompt,
        "system": WORKFLOW_SYSTEM_PROMPT,
        "format": "json",
        "stream": False,
        "keep_alive": "5m",
        "options": {"temperature": 0.2, "num_predict": 700},
    }
    ok, status, body, err = _post_json(
        base_url.rstrip("/") + "/api/generate", payload, timeout
    )
    if not ok:
        return {"attempted": True, "ok": False, "error": err or ("http %s" % status)}
    try:
        envelope = json.loads(body)
        workflow = _extract_json_object(envelope.get("response", ""))
    except (json.JSONDecodeError, ValueError) as e:
        return {"attempted": True, "ok": False, "error": "bad model output: %s" % e}
    if not isinstance(workflow, dict) or not isinstance(workflow.get("nodes"), list):
        return {"attempted": True, "ok": False, "error": "model reply missing 'nodes'"}
    return {"attempted": True, "ok": True, "workflow": workflow}


def promote_note(path, note_id, n8n_webhook, ollama_url, ollama_model,
                  n8n_timeout=10, ollama_timeout=90):
    """Promote an idea: mark it actionable, fire the n8n webhook, draft a
    workflow via Ollama, and persist whatever was produced. Returns None if
    the note does not exist."""
    note = get_note(path, note_id)
    if note is None:
        return None
    if note["status"] not in ("actionable", "done"):
        set_status(path, note_id, "actionable")
        note["status"] = "actionable"

    webhook_result = fire_n8n_webhook(n8n_webhook, note, n8n_timeout)
    workflow_result = draft_workflow_with_ollama(ollama_url, ollama_model, note, ollama_timeout)

    now = int(time.time())
    workflow_json = json.dumps(workflow_result["workflow"]) if workflow_result.get("ok") else None
    save_workflow(path, note_id, workflow_json, now)

    return {
        "id": note_id,
        "status": note["status"],
        "webhook": webhook_result,
        "workflow": workflow_result,
    }


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
 .meta a{color:#7fd7ff;text-decoration:none}
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
async function promote(id){
  const r=await fetch('/api/note/'+id+'/promote',{method:'POST'});
  const d=await r.json();
  let msg='status: '+d.status;
  msg+='\\nwebhook: '+(d.webhook.attempted?(d.webhook.ok?'sent':'failed - '+d.webhook.error):'not configured');
  msg+='\\nworkflow draft: '+(d.workflow.attempted?(d.workflow.ok?'ready (click "workflow" to view)':'failed - '+d.workflow.error):'not configured');
  alert(msg);
  load();
}
async function viewWorkflow(id){
  const r=await fetch('/api/note/'+id);
  const d=await r.json();
  alert(d.note.workflow?JSON.stringify(d.note.workflow,null,2):'no workflow drafted yet');
}
function render(items){
  el('list').innerHTML=(items.notes||[]).map(n=>{
    const tags=(n.tags||'').split(',').filter(x=>x.trim())
      .map(t=>'<span class="tag">'+t.trim()+'</span>').join('');
    const when=new Date(n.created_at*1000).toLocaleString();
    return '<div class="note"><span class="status">'+n.status+'</span>'+
      esc(n.text).replace(/\\n/g,'<br>')+
      '<div class="meta">'+tags+' '+when+' · #'+n.id+
      ' · <a href="#" onclick="promote('+n.id+');return false;">promote → n8n</a>'+
      (n.workflow_json?' · <a href="#" onclick="viewWorkflow('+n.id+');return false;">workflow</a>':'')+
      '</div></div>';
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
    n8n_webhook = ""
    ollama_url = DEFAULT_OLLAMA_URL
    ollama_model = DEFAULT_OLLAMA_MODEL
    n8n_timeout = 10
    ollama_timeout = 90

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
        if u.path.startswith("/api/note/"):
            try:
                note_id = int(u.path.rstrip("/").rsplit("/", 1)[-1])
            except ValueError:
                return self._json(400, {"error": "bad id"})
            note = get_note(self.db_path, note_id)
            if note is None:
                return self._json(404, {"error": "not found"})
            if note.get("workflow_json"):
                try:
                    note["workflow"] = json.loads(note["workflow_json"])
                except json.JSONDecodeError:
                    note["workflow"] = None
            return self._json(200, {"note": note})
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
        if u.path.startswith("/api/note/") and u.path.endswith("/promote"):
            try:
                note_id = int(u.path.split("/")[3])
            except (IndexError, ValueError):
                return self._json(400, {"error": "bad id"})
            result = promote_note(
                self.db_path, note_id,
                self.n8n_webhook, self.ollama_url, self.ollama_model,
                self.n8n_timeout, self.ollama_timeout,
            )
            if result is None:
                return self._json(404, {"error": "not found"})
            return self._json(200, dict({"ok": True}, **result))
        return self._json(404, {"error": "not found"})


# ----------------------------------------------------------------------------- selftest helpers
class _FakeWebhookHandler(BaseHTTPRequestHandler):
    """Stands in for an n8n webhook during --selftest (no live infra needed)."""
    received = []

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0) or 0)
        body = self.rfile.read(length)
        _FakeWebhookHandler.received.append(json.loads(body.decode()))
        payload = b'{"ok":true}'
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, fmt, *args):
        pass


class _FakeOllamaHandler(BaseHTTPRequestHandler):
    """Stands in for Ollama's /api/generate during --selftest."""

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0) or 0)
        self.rfile.read(length)
        workflow = {
            "name": "Test workflow",
            "nodes": [
                {"name": "Manual Trigger", "type": "n8n-nodes-base.manualTrigger",
                 "typeVersion": 1, "position": [0, 0], "parameters": {}},
                {"name": "Set", "type": "n8n-nodes-base.set",
                 "typeVersion": 1, "position": [200, 0], "parameters": {}},
            ],
            "connections": {},
        }
        payload = json.dumps({"response": json.dumps(workflow), "done": True}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, fmt, *args):
        pass


def _run_fake_server(handler_cls):
    httpd = ThreadingHTTPServer(("127.0.0.1", 0), handler_cls)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    return httpd


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

    # Phase 3: promoting with nothing configured must still succeed cleanly.
    result = promote_note(db_path, a, "", "", DEFAULT_OLLAMA_MODEL)
    assert result["webhook"]["attempted"] is False, "webhook must be skipped when unset"
    assert result["workflow"]["attempted"] is False, "ollama must be skipped when unset"

    # Phase 3: promoting for real against fake local n8n/Ollama servers.
    _FakeWebhookHandler.received.clear()
    webhook_httpd = _run_fake_server(_FakeWebhookHandler)
    ollama_httpd = _run_fake_server(_FakeOllamaHandler)
    try:
        webhook_url = "http://127.0.0.1:%d/webhook/idea" % webhook_httpd.server_address[1]
        ollama_url = "http://127.0.0.1:%d" % ollama_httpd.server_address[1]
        result = promote_note(db_path, a, webhook_url, ollama_url, DEFAULT_OLLAMA_MODEL)
        assert result["webhook"]["ok"] is True, "webhook should have fired: %r" % result["webhook"]
        assert _FakeWebhookHandler.received and _FakeWebhookHandler.received[-1]["id"] == a, \
            "webhook payload should carry the promoted note"
        assert result["workflow"]["ok"] is True, "workflow draft should have succeeded: %r" % result["workflow"]
        assert result["workflow"]["workflow"]["nodes"], "drafted workflow must have nodes"
        stored = get_note(db_path, a)
        assert stored["workflow_json"], "workflow json should be persisted on the note"
        assert json.loads(stored["workflow_json"])["name"] == "Test workflow"
    finally:
        webhook_httpd.shutdown()
        webhook_httpd.server_close()
        ollama_httpd.shutdown()
        ollama_httpd.server_close()

    # An unreachable Ollama must fail gracefully rather than raising.
    bad = draft_workflow_with_ollama("http://127.0.0.1:1", DEFAULT_OLLAMA_MODEL, get_note(db_path, a))
    assert bad["attempted"] is True and bad["ok"] is False, "unreachable ollama must report a clean failure"

    os.remove(db_path)
    print("SELFTEST OK (fts=%s)" % FTS_AVAILABLE)
    return 0


def main():
    ap = argparse.ArgumentParser(description="nexus-brain idea capture/search service")
    ap.add_argument("--host", default=os.environ.get("NEXUS_BRAIN_HOST", "127.0.0.1"))
    ap.add_argument("--port", type=int, default=int(os.environ.get("NEXUS_BRAIN_PORT", "8700")))
    ap.add_argument("--db", default=os.environ.get("NEXUS_BRAIN_DB", "nexus-brain.db"))
    ap.add_argument("--n8n-webhook",
                     default=os.environ.get("NEXUS_BRAIN_N8N_WEBHOOK", ""),
                     help="n8n webhook URL fired when an idea is promoted (default: disabled)")
    ap.add_argument("--ollama-url",
                     default=os.environ.get("NEXUS_BRAIN_OLLAMA_URL", DEFAULT_OLLAMA_URL),
                     help='Ollama base URL used to draft a workflow on promote; "" disables it')
    ap.add_argument("--ollama-model",
                     default=os.environ.get("NEXUS_BRAIN_OLLAMA_MODEL", DEFAULT_OLLAMA_MODEL))
    ap.add_argument("--n8n-timeout", type=float,
                     default=float(os.environ.get("NEXUS_BRAIN_N8N_TIMEOUT", "10")))
    ap.add_argument("--ollama-timeout", type=float,
                     default=float(os.environ.get("NEXUS_BRAIN_OLLAMA_TIMEOUT", "90")))
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()

    if args.selftest:
        return selftest(args.db)

    init_db(args.db)
    Handler.db_path = args.db
    Handler.n8n_webhook = args.n8n_webhook
    Handler.ollama_url = args.ollama_url
    Handler.ollama_model = args.ollama_model
    Handler.n8n_timeout = args.n8n_timeout
    Handler.ollama_timeout = args.ollama_timeout
    httpd = ThreadingHTTPServer((args.host, args.port), Handler)
    print(
        "nexus-brain on http://%s:%s  (db=%s, fts=%s, n8n=%s, ollama=%s)"
        % (args.host, args.port, args.db, FTS_AVAILABLE,
           "on" if args.n8n_webhook else "off",
           args.ollama_model if args.ollama_url else "off"),
        file=sys.stderr,
    )
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        httpd.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
