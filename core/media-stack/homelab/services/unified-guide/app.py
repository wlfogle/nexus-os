#!/usr/bin/env python3
"""
TiamatsGuide — Unified Media Guide Service (port 8088)
Aggregates: HDHomeRun OTA channels, Plex EPG + DVR recordings,
            Jellyfin library, Sonarr/Radarr calendar
"""

import logging
import os
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timedelta

import requests
from flask import Flask, jsonify, render_template
from flask_cors import CORS

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
log = logging.getLogger("tiamats-guide")

app = Flask(__name__)
CORS(app)

# ── Config from environment ──────────────────────────────────────────────────
HDHR_IP          = os.environ.get("HDHR_IP", "")
PLEX_URL         = os.environ.get("PLEX_URL", "http://plex:32400").rstrip("/")
PLEX_TOKEN       = os.environ.get("PLEX_TOKEN", "")
JELLYFIN_URL     = os.environ.get("JELLYFIN_URL", "http://jellyfin:8096").rstrip("/")
JELLYFIN_KEY     = os.environ.get("JELLYFIN_API_KEY", "")
JELLYFIN_USER_ID = os.environ.get("JELLYFIN_USER_ID", "")  # scoped library + watched state
SONARR_URL       = os.environ.get("SONARR_URL", "http://sonarr:8989").rstrip("/")
SONARR_KEY       = os.environ.get("SONARR_API_KEY", "")
RADARR_URL       = os.environ.get("RADARR_URL", "http://radarr:7878").rstrip("/")
RADARR_KEY       = os.environ.get("RADARR_API_KEY", "")

TIMEOUT = 6  # seconds per upstream request

# ── Startup config summary ────────────────────────────────────────────────────
log.info("TiamatsGuide starting on :8088")
log.info("  HDHomeRun  : %s", HDHR_IP or "NOT SET")
log.info("  Plex       : %s  token=%s", PLEX_URL, "SET" if PLEX_TOKEN else "NOT SET")
log.info("  Jellyfin   : %s  key=%s  user=%s",
         JELLYFIN_URL, "SET" if JELLYFIN_KEY else "NOT SET",
         JELLYFIN_USER_ID or "not set (system scope)")
log.info("  Sonarr     : %s  key=%s", SONARR_URL, "SET" if SONARR_KEY else "NOT SET")
log.info("  Radarr     : %s  key=%s", RADARR_URL, "SET" if RADARR_KEY else "NOT SET")

# ── TTL in-memory cache ───────────────────────────────────────────────────────
# Avoids hammering upstream services on every page load / auto-refresh.
_cache: dict = {}
_CACHE_TTL = {
    "guide":      120,   # 2 min  — live TV changes infrequently
    "library":    300,   # 5 min  — new content added rarely
    "calendar":   300,   # 5 min  — Sonarr/Radarr schedule
    "recordings": 180,   # 3 min  — DVR recordings
    "status":      30,   # 30 s   — health badges refresh fast
}


def cache_get(key: str):
    entry = _cache.get(key)
    if entry and (time.monotonic() - entry["ts"]) < _CACHE_TTL.get(key, 120):
        log.debug("cache HIT  %s", key)
        return entry["data"]
    return None


def cache_set(key: str, data):
    _cache[key] = {"ts": time.monotonic(), "data": data}


def cache_bust(key: str):
    _cache.pop(key, None)


# ── HTTP helper ───────────────────────────────────────────────────────────────

def safe_get(url, headers=None, params=None):
    """GET that returns parsed JSON or None; logs failures at WARNING level."""
    try:
        r = requests.get(
            url,
            headers=headers or {},
            params=params or {},
            timeout=TIMEOUT,
        )
        r.raise_for_status()
        return r.json()
    except requests.exceptions.ConnectionError:
        log.warning("Connection refused: %s", url)
    except requests.exceptions.Timeout:
        log.warning("Timeout after %ds: %s", TIMEOUT, url)
    except requests.exceptions.HTTPError as exc:
        log.warning("HTTP %s from %s", exc.response.status_code, url)
    except Exception as exc:  # noqa: BLE001
        log.warning("safe_get error %s — %s", url, exc)
    return None


# ── Frontend ─────────────────────────────────────────────────────────────────

@app.route("/")
def index():
    return render_template("index.html")


# ── API: OTA Guide (HDHomeRun channels + Plex EPG) ───────────────────────────

@app.route("/api/guide")
def api_guide():
    channels = []
    epg = {}

    # 1. HDHomeRun channel lineup
    if HDHR_IP:
        lineup = safe_get(f"http://{HDHR_IP}/lineup.json")
        if lineup:
            for ch in lineup:
                channels.append({
                    "number":   ch.get("GuideNumber", ""),
                    "name":     ch.get("GuideName", ""),
                    "hd":       bool(ch.get("HD")),
                    "favorite": bool(ch.get("Favorite")),
                    "signal":   ch.get("SignalStrength", 0),
                    "quality":  ch.get("SignalQuality", 0),
                    "url":      ch.get("URL", ""),
                })

    # 2. Plex EPG grid (next 4 hours)
    if PLEX_URL and PLEX_TOKEN:
        plex_headers = {
            "X-Plex-Token": PLEX_TOKEN,
            "Accept":       "application/json",
        }
        now_ts  = int(datetime.utcnow().timestamp())
        end_ts  = now_ts + (4 * 3600)

        # Attempt to fetch live-TV guide from Plex
        guide_data = safe_get(
            f"{PLEX_URL}/livetv/dvr/guide",
            headers=plex_headers,
            params={"start": now_ts, "end": end_ts},
        )
        if guide_data:
            try:
                for container in guide_data.get("MediaContainer", {}) \
                                           .get("MediaContainer", []):
                    ch_number = container.get("GuideNumber", "")
                    slots = []
                    for meta in container.get("Metadata", []):
                        slots.append({
                            "title":    meta.get("title", ""),
                            "start":    meta.get("Metadata", {}).get("beginsAt", ""),
                            "end":      meta.get("Metadata", {}).get("endsAt", ""),
                            "summary":  (meta.get("summary") or "")[:100],
                        })
                    if ch_number:
                        epg[ch_number] = slots
            except Exception:
                epg = {}

    return jsonify({
        "channels": channels,
        "epg":      epg,
        "now_ts":   int(datetime.utcnow().timestamp()),
    })


# ── API: Jellyfin Library ─────────────────────────────────────────────────────

@app.route("/api/library")
def api_library():
    if not (JELLYFIN_URL and JELLYFIN_KEY):
        return jsonify({"items": [], "total": 0, "error": "Jellyfin not configured"})

    headers = {"X-Emby-Token": JELLYFIN_KEY}

    data = safe_get(
        f"{JELLYFIN_URL}/Items",
        headers=headers,
        params={
            "IncludeItemTypes":  "Movie,Series",
            "Recursive":         "true",
            "Limit":             120,
            "SortBy":            "DateCreated",
            "SortOrder":         "Descending",
            "Fields":            "Overview,Genres,ProductionYear,CommunityRating,"
                                 "OfficialRating,RunTimeTicks",
            "ImageTypeLimit":    1,
            "EnableImageTypes":  "Primary,Backdrop,Thumb",
        },
    )

    if not data:
        return jsonify({"items": [], "total": 0, "error": "Jellyfin unreachable"})

    items = []
    for item in data.get("Items", []):
        item_id = item.get("Id", "")
        thumb = None
        if item.get("ImageTags", {}).get("Primary"):
            thumb = (f"{JELLYFIN_URL}/Items/{item_id}/Images/Primary"
                     f"?maxWidth=280&quality=80&api_key={JELLYFIN_KEY}")

        runtime_min = None
        ticks = item.get("RunTimeTicks")
        if ticks:
            runtime_min = int(ticks / 600_000_000)

        items.append({
            "id":         item_id,
            "name":       item.get("Name", ""),
            "type":       item.get("Type", ""),
            "year":       item.get("ProductionYear"),
            "rating":     round(item.get("CommunityRating") or 0, 1) or None,
            "mpaa":       item.get("OfficialRating", ""),
            "overview":   (item.get("Overview") or "")[:220],
            "genres":     item.get("Genres", [])[:3],
            "thumb":      thumb,
            "runtime":    runtime_min,
        })

    return jsonify({"items": items, "total": data.get("TotalRecordCount", len(items))})


# ── API: Sonarr + Radarr Calendar ─────────────────────────────────────────────

@app.route("/api/calendar")
def api_calendar():
    today = datetime.utcnow().date().isoformat()
    end   = (datetime.utcnow() + timedelta(days=14)).date().isoformat()
    events = []

    # Sonarr — upcoming episodes
    if SONARR_URL and SONARR_KEY:
        data = safe_get(
            f"{SONARR_URL}/api/v3/calendar",
            headers={"X-Api-Key": SONARR_KEY},
            params={"start": today, "end": end, "unmonitored": "false"},
        )
        if data:
            for ep in data:
                series = ep.get("series", {})
                events.append({
                    "type":          "episode",
                    "title":         series.get("title") or ep.get("title", "Unknown"),
                    "subtitle":      ep.get("title", ""),
                    "episode":       f"S{ep.get('seasonNumber', 0):02d}"
                                     f"E{ep.get('episodeNumber', 0):02d}",
                    "air_date":      ep.get("airDateUtc", ""),
                    "has_file":      ep.get("hasFile", False),
                    "network":       series.get("network", ""),
                    "poster":        None,
                })

    # Radarr — upcoming movies
    if RADARR_URL and RADARR_KEY:
        data = safe_get(
            f"{RADARR_URL}/api/v3/calendar",
            headers={"X-Api-Key": RADARR_KEY},
            params={"start": today, "end": end, "unmonitored": "false"},
        )
        if data:
            for movie in data:
                air = (movie.get("physicalRelease")
                       or movie.get("digitalRelease")
                       or movie.get("inCinemas", ""))
                events.append({
                    "type":     "movie",
                    "title":    movie.get("title", "Unknown"),
                    "subtitle": movie.get("studio", ""),
                    "episode":  "",
                    "air_date": air,
                    "has_file": movie.get("hasFile", False),
                    "year":     movie.get("year"),
                    "network":  "",
                    "poster":   None,
                })

    events.sort(key=lambda x: x.get("air_date") or "")
    return jsonify({"events": events})


# ── API: Plex DVR Recordings ──────────────────────────────────────────────────

@app.route("/api/recordings")
def api_recordings():
    if not (PLEX_URL and PLEX_TOKEN):
        return jsonify({"recordings": [], "error": "Plex not configured"})

    plex_headers = {
        "X-Plex-Token": PLEX_TOKEN,
        "Accept":       "application/json",
    }

    recordings = []

    # Primary: /livetv/recordings
    data = safe_get(f"{PLEX_URL}/livetv/recordings", headers=plex_headers)
    if data:
        for item in data.get("MediaContainer", {}).get("Metadata", []):
            thumb_path = item.get("thumb") or item.get("art") or ""
            recordings.append({
                "key":      item.get("key", ""),
                "title":    item.get("title", ""),
                "subtitle": item.get("grandparentTitle", ""),
                "type":     item.get("type", ""),
                "year":     item.get("year"),
                "summary":  (item.get("summary") or "")[:200],
                "added_at": item.get("addedAt"),
                "duration": int((item.get("duration") or 0) / 60000),  # ms→min
                "thumb":    f"{PLEX_URL}{thumb_path}?X-Plex-Token={PLEX_TOKEN}"
                            if thumb_path else None,
                "plex_url": f"{PLEX_URL}/web/index.html#!/server/"
                            f"?key={item.get('key', '')}",
            })

    # Fallback: scan library sections labelled as DVR
    if not recordings:
        sections_data = safe_get(
            f"{PLEX_URL}/library/sections", headers=plex_headers
        )
        if sections_data:
            for sec in sections_data.get("MediaContainer", {}).get("Directory", []):
                if "DVR" in (sec.get("title") or ""):
                    sec_key = sec.get("key")
                    items_data = safe_get(
                        f"{PLEX_URL}/library/sections/{sec_key}/all",
                        headers=plex_headers,
                    )
                    if items_data:
                        for item in items_data.get("MediaContainer", {}) \
                                              .get("Metadata", []):
                            thumb_path = item.get("thumb") or ""
                            recordings.append({
                                "key":      item.get("key", ""),
                                "title":    item.get("title", ""),
                                "subtitle": "",
                                "type":     item.get("type", ""),
                                "year":     item.get("year"),
                                "summary":  (item.get("summary") or "")[:200],
                                "added_at": item.get("addedAt"),
                                "duration": int((item.get("duration") or 0) / 60000),
                                "thumb":    f"{PLEX_URL}{thumb_path}"
                                            f"?X-Plex-Token={PLEX_TOKEN}"
                                            if thumb_path else None,
                                "plex_url": f"{PLEX_URL}/web/index.html",
                            })

    return jsonify({"recordings": recordings})


# ── API: Service Health Status ────────────────────────────────────────────────

@app.route("/api/status")
def api_status():
    services = {}

    if HDHR_IP:
        d = safe_get(f"http://{HDHR_IP}/discover.json")
        services["hdhr"] = {
            "ok":   d is not None,
            "name": (d or {}).get("FriendlyName", "HDHomeRun"),
            "ip":   HDHR_IP,
        }

    if PLEX_URL and PLEX_TOKEN:
        d = safe_get(
            f"{PLEX_URL}/",
            headers={"X-Plex-Token": PLEX_TOKEN, "Accept": "application/json"},
        )
        services["plex"] = {
            "ok":  d is not None,
            "url": PLEX_URL,
        }

    if JELLYFIN_URL and JELLYFIN_KEY:
        d = safe_get(
            f"{JELLYFIN_URL}/System/Info",
            headers={"X-Emby-Token": JELLYFIN_KEY},
        )
        services["jellyfin"] = {
            "ok":      d is not None,
            "version": (d or {}).get("Version", ""),
            "url":     JELLYFIN_URL,
        }

    if SONARR_URL and SONARR_KEY:
        d = safe_get(
            f"{SONARR_URL}/api/v3/system/status",
            headers={"X-Api-Key": SONARR_KEY},
        )
        services["sonarr"] = {
            "ok":  d is not None,
            "url": SONARR_URL,
        }

    if RADARR_URL and RADARR_KEY:
        d = safe_get(
            f"{RADARR_URL}/api/v3/system/status",
            headers={"X-Api-Key": RADARR_KEY},
        )
        services["radarr"] = {
            "ok":  d is not None,
            "url": RADARR_URL,
        }

    return jsonify({"services": services, "time": datetime.utcnow().isoformat() + "Z"})


# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8088, debug=False)
