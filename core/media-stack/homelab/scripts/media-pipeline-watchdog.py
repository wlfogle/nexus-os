#!/usr/bin/env python3
import json
import sys
import urllib.parse
import urllib.request
from datetime import datetime, timezone, timedelta
from pathlib import Path


BAD_PATTERNS = (
    ".exe",
    "executable file",
    "password protected",
    "rar password",
    "contains executable",
)


def log(msg):
    print(msg, flush=True)


def req(method, url, headers=None, data=None, timeout=20):
    body = None
    if data is not None:
        body = json.dumps(data).encode()
        headers = dict(headers or {})
        headers["Content-Type"] = "application/json"
    r = urllib.request.Request(url, data=body, headers=headers or {}, method=method)
    with urllib.request.urlopen(r, timeout=timeout) as resp:
        raw = resp.read()
        ctype = resp.headers.get("Content-Type", "")
        if "application/json" in ctype and raw:
            return json.loads(raw.decode())
        return raw.decode(errors="replace")


def api_get(base, version, path, key):
    return req("GET", f"{base}/api/{version}/{path}", {"X-Api-Key": key})


def api_post(base, version, path, key, payload):
    return req("POST", f"{base}/api/{version}/{path}", {"X-Api-Key": key}, payload)


def api_delete(base, version, path, key):
    return req("DELETE", f"{base}/api/{version}/{path}", {"X-Api-Key": key})


def find_qbit_schema(base, version, key):
    items = api_get(base, version, "downloadclient/schema", key)
    for item in items:
        if item.get("implementation") == "QBittorrent":
            return item
    return None


def field_value(name, media_kind, qbit):
    cat_map = qbit.get("categories", {})
    if name == "host":
        return qbit["host"]
    if name == "port":
        return qbit["port"]
    if name == "useSsl":
        return qbit.get("use_ssl", False)
    if name == "urlBase":
        return qbit.get("url_base", "")
    if name == "username":
        return qbit["username"]
    if name == "password":
        return qbit["password"]
    if name.endswith("Category") and not name.endswith("ImportedCategory"):
        return cat_map.get(media_kind, media_kind)
    if name.endswith("ImportedCategory"):
        return cat_map.get(media_kind, media_kind)
    if name in ("recentTvPriority", "olderTvPriority", "recentMoviePriority", "olderMoviePriority",
                "recentMusicPriority", "olderMusicPriority", "initialState"):
        return 0
    if name in ("sequentialOrder", "firstAndLast"):
        return False
    if name == "contentLayout":
        return "subfolder"
    return None


def ensure_qbit_client(app, qbit):
    clients = api_get(app["url"], app["api_version"], "downloadclient", app["api_key"])
    for client in clients:
        if client.get("implementation") == "QBittorrent":
            log(f"[OK] {app['name']}: qBittorrent client already configured")
            return
    schema = find_qbit_schema(app["url"], app["api_version"], app["api_key"])
    if not schema:
        log(f"[WARN] {app['name']}: no qBittorrent schema found")
        return
    payload = dict(schema)
    for k in ("id", "resource", "infoLink"):
        payload.pop(k, None)
    payload["name"] = "qBittorrent"
    payload["enable"] = True
    payload["priority"] = 1
    payload["protocol"] = "torrent"
    fields = []
    for field in payload.get("fields", []):
        f = dict(field)
        val = field_value(f.get("name"), app["media_kind"], qbit)
        if val is not None:
            f["value"] = val
        fields.append(f)
    payload["fields"] = fields
    api_post(app["url"], app["api_version"], "downloadclient", app["api_key"], payload)
    log(f"[FIXED] {app['name']}: qBittorrent client created")


def iso_to_dt(value):
    if not value:
        return None
    value = value.replace("Z", "+00:00")
    try:
        return datetime.fromisoformat(value)
    except Exception:
        return None


def queue_delete_path(app, item_id, remove_from_client=True, blocklist=True, skip_redownload=False):
    qs = urllib.parse.urlencode({
        "removeFromClient": str(remove_from_client).lower(),
        "blocklist": str(blocklist).lower(),
        "skipRedownload": str(skip_redownload).lower(),
    })
    return f"queue/{item_id}?{qs}"


def cleanup_queue(app, stale_hours):
    queue = api_get(app["url"], app["api_version"], "queue?page=1&pageSize=200", app["api_key"])
    now = datetime.now(timezone.utc)
    records = queue.get("records", [])
    if not records:
        log(f"[OK] {app['name']}: queue empty")
        return
    for rec in records:
        item_id = rec.get("id")
        title = rec.get("title", "?")
        joined = " ".join(
            msg
            for sm in rec.get("statusMessages", [])
            for msg in sm.get("messages", [])
        ).lower()
        added = iso_to_dt(rec.get("added"))
        age = (now - added) if added else timedelta()
        bad = any(pat in joined for pat in BAD_PATTERNS)
        stale = age > timedelta(hours=stale_hours) and rec.get("trackedDownloadStatus") in ("warning", "failed") or (
            age > timedelta(hours=stale_hours) and rec.get("trackedDownloadState") in ("importPending", "downloadFailed")
        )
        if bad:
            api_delete(app["url"], app["api_version"], queue_delete_path(app, item_id, True, True, False), app["api_key"])
            log(f"[FIXED] {app['name']}: removed poisoned release {title}")
        elif stale:
            api_delete(app["url"], app["api_version"], queue_delete_path(app, item_id, True, True, False), app["api_key"])
            log(f"[FIXED] {app['name']}: removed stale queue item {title}")


def maybe_run_hook(hook):
    try:
        if hook["method"].upper() == "POST":
            req("POST", hook["url"], hook.get("headers", {}), hook.get("body"))
        else:
            req("GET", hook["url"], hook.get("headers", {}))
        log(f"[OK] hook: {hook['name']}")
    except Exception as e:
        log(f"[WARN] hook {hook['name']}: {e}")


def main():
    if len(sys.argv) != 2:
        print("usage: media-pipeline-watchdog.py /path/to/config.json", file=sys.stderr)
        sys.exit(2)
    cfg_path = Path(sys.argv[1])
    cfg = json.loads(cfg_path.read_text())
    qbit = cfg["qbittorrent"]
    stale_hours = cfg.get("stale_hours", 24)
    for app in cfg.get("apps", []):
        try:
            ensure_qbit_client(app, qbit)
            cleanup_queue(app, stale_hours)
        except Exception as e:
            log(f"[WARN] {app['name']}: {e}")
    for hook in cfg.get("hooks", []):
        maybe_run_hook(hook)


if __name__ == "__main__":
    main()
