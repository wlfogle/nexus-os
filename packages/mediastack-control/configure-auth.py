#!/usr/bin/env python3
"""
configure-auth.py
Reads service passwords from a local file and configures authentication
for *arr services, Jackett, and qBittorrent via their APIs.

Usage:
    python3 configure-auth.py [--password-file /path/to/file] [--username name]

Requires: services must be running (started at least once to generate configs).
"""

import argparse
import base64
import hashlib
import json
import os
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from urllib.request import Request, urlopen
from urllib.error import URLError, HTTPError

# ---------------------------------------------------------------------------
# Service definitions
# ---------------------------------------------------------------------------

ARR_SERVICES = {
    "sonarr":   {"port": 8989, "api": "v3"},
    "radarr":   {"port": 7878, "api": "v3"},
    "lidarr":   {"port": 8686, "api": "v1"},
    "readarr":  {"port": 8787, "api": "v1"},
    "prowlarr": {"port": 9696, "api": "v1"},
}

OTHER_SERVICES = {
    "jackett":     {"port": 9117},
    "qbittorrent": {"port": 8080},
}

# All service keywords we look for when parsing the password file
ALL_SERVICES = list(ARR_SERVICES) + list(OTHER_SERVICES)


# ---------------------------------------------------------------------------
# Password file parser
# ---------------------------------------------------------------------------

def parse_passwords(filepath):
    """
    Parse service passwords from the password file.
    Expected line formats:
        <password> <service_name> [password]
        <service_name> <password> <service_name>   (jellyfin style)
    Returns dict of {service_name: password}.
    """
    passwords = {}
    try:
        with open(filepath, "r") as fh:
            for raw in fh:
                line = raw.rstrip("\n")
                if not line.strip():
                    continue
                low = line.lower()
                for svc in ALL_SERVICES:
                    if svc not in low:
                        continue
                    idx = low.index(svc)
                    pw = line[:idx].strip()
                    if pw:
                        passwords[svc] = pw
                    break
    except FileNotFoundError:
        print(f"ERROR  Password file not found: {filepath}")
        sys.exit(1)
    return passwords


# ---------------------------------------------------------------------------
# Docker helpers
# ---------------------------------------------------------------------------

def container_running(name):
    r = subprocess.run(
        ["docker", "inspect", "-f", "{{.State.Running}}", name],
        capture_output=True, text=True, timeout=5,
    )
    return r.returncode == 0 and "true" in r.stdout.lower()


def get_api_key(container):
    """Read the ApiKey from an *arr container's config.xml."""
    try:
        r = subprocess.run(
            ["docker", "exec", container, "cat", "/config/config.xml"],
            capture_output=True, text=True, timeout=10,
        )
        if r.returncode != 0:
            return None
        root = ET.fromstring(r.stdout)
        el = root.find("ApiKey")
        return el.text.strip() if el is not None and el.text else None
    except Exception:
        return None


# ---------------------------------------------------------------------------
# HTTP helpers (stdlib only — no extra deps)
# ---------------------------------------------------------------------------

def http_get(url, headers=None):
    req = Request(url, headers=headers or {})
    try:
        with urlopen(req, timeout=10) as resp:
            return resp.status, json.loads(resp.read().decode())
    except HTTPError as e:
        return e.code, None
    except Exception:
        return 0, None


def http_put(url, data, headers=None):
    body = json.dumps(data).encode()
    hdrs = {"Content-Type": "application/json", "Accept": "application/json"}
    if headers:
        hdrs.update(headers)
    req = Request(url, data=body, method="PUT", headers=hdrs)
    try:
        with urlopen(req, timeout=10) as resp:
            return resp.status, json.loads(resp.read().decode())
    except HTTPError as e:
        return e.code, None
    except Exception:
        return 0, None


def http_post_form(url, form_data, cookies=None):
    """POST application/x-www-form-urlencoded."""
    from urllib.parse import urlencode
    body = urlencode(form_data).encode()
    hdrs = {"Content-Type": "application/x-www-form-urlencoded"}
    if cookies:
        hdrs["Cookie"] = cookies
    req = Request(url, data=body, method="POST", headers=hdrs)
    try:
        resp = urlopen(req, timeout=10)
        cookie_header = resp.headers.get("Set-Cookie", "")
        return resp.status, resp.read().decode(), cookie_header
    except HTTPError as e:
        return e.code, "", ""
    except Exception:
        return 0, "", ""


# ---------------------------------------------------------------------------
# *arr configurator (Sonarr, Radarr, Lidarr, Readarr, Prowlarr)
# ---------------------------------------------------------------------------

def configure_arr(name, port, api_ver, username, password):
    if not container_running(name):
        return False, "container not running"

    api_key = get_api_key(name)
    if not api_key:
        return False, "could not read API key from config.xml"

    base = f"http://localhost:{port}/api/{api_ver}"
    hdr = {"X-Api-Key": api_key, "Accept": "application/json"}

    # Fetch current host config
    status, config = http_get(f"{base}/config/host", hdr)
    if not config:
        return False, f"failed to fetch config (HTTP {status})"

    # Set authentication fields
    config["authenticationMethod"] = "forms"
    config["authenticationRequired"] = "enabled"
    config["username"] = username
    config["password"] = password
    config["passwordConfirmation"] = password

    hdr_put = {"X-Api-Key": api_key}
    status, result = http_put(f"{base}/config/host", config, hdr_put)
    if status in (200, 202):
        return True, "auth configured"
    return False, f"PUT failed (HTTP {status})"


# ---------------------------------------------------------------------------
# Jackett configurator
# ---------------------------------------------------------------------------

def aspnet_identity_hash(password):
    """Generate an ASP.NET Core Identity V3 compatible password hash.
    Format: 0x01 | prf(4B) | iter(4B) | saltLen(4B) | salt | subkey
    All big-endian, base64 encoded."""
    import struct
    prf = 1         # HMACSHA256
    iterations = 100000
    salt_len = 16
    subkey_len = 32
    salt = os.urandom(salt_len)
    subkey = hashlib.pbkdf2_hmac("sha256", password.encode(), salt, iterations, dklen=subkey_len)
    blob = (bytes([0x01])
            + struct.pack(">I", prf)
            + struct.pack(">I", iterations)
            + struct.pack(">I", salt_len)
            + salt + subkey)
    return base64.b64encode(blob).decode()


def configure_jackett(port, password):
    if not container_running("jackett"):
        return False, "container not running"

    conf_path = "/config/Jackett/ServerConfig.json"

    # Read current config
    try:
        r = subprocess.run(
            ["docker", "exec", "jackett", "cat", conf_path],
            capture_output=True, text=True, timeout=10,
        )
        if r.returncode != 0:
            return False, "could not read ServerConfig.json"
        cfg = json.loads(r.stdout)
    except Exception as e:
        return False, f"config read error: {e}"

    # Hash the password in ASP.NET Identity V3 format
    cfg["AdminPassword"] = aspnet_identity_hash(password)

    new_conf = json.dumps(cfg, indent=2)

    # Write back
    try:
        w = subprocess.run(
            ["docker", "exec", "-i", "jackett", "tee", conf_path],
            input=new_conf, capture_output=True, text=True, timeout=10,
        )
        if w.returncode != 0:
            return False, "failed to write config"
    except Exception as e:
        return False, f"config write error: {e}"

    # Restart to pick up changes
    try:
        subprocess.run(["docker", "restart", "jackett"], timeout=30,
                       capture_output=True)
    except Exception:
        pass

    return True, "admin password set (container restarted)"


# ---------------------------------------------------------------------------
# qBittorrent configurator
# ---------------------------------------------------------------------------

def qbt_pbkdf2_hash(password):
    """Generate a qBittorrent PBKDF2-HMAC-SHA512 password hash."""
    salt = os.urandom(16)
    key = hashlib.pbkdf2_hmac("sha512", password.encode(), salt, 100000, dklen=64)
    salt_b64 = base64.b64encode(salt).decode()
    key_b64 = base64.b64encode(key).decode()
    return f"@ByteArray({salt_b64}:{key_b64})"


def configure_qbittorrent(port, username, password):
    if not container_running("qbittorrent"):
        return False, "container not running"

    conf_path = "/config/qBittorrent/qBittorrent.conf"

    # Read current config
    try:
        r = subprocess.run(
            ["docker", "exec", "qbittorrent", "cat", conf_path],
            capture_output=True, text=True, timeout=10,
        )
        if r.returncode != 0:
            return False, "could not read qBittorrent.conf"
        conf = r.stdout
    except Exception as e:
        return False, f"config read error: {e}"

    # Generate new PBKDF2 hash
    pw_hash = qbt_pbkdf2_hash(password)

    # Update username and password lines
    lines = conf.split("\n")
    new_lines = []
    found_user = False
    found_pass = False
    for line in lines:
        if line.startswith("WebUI\\Username="):
            new_lines.append(f"WebUI\\Username={username}")
            found_user = True
        elif line.startswith("WebUI\\Password_PBKDF2="):
            new_lines.append(f'WebUI\\Password_PBKDF2="{pw_hash}"')
            found_pass = True
        else:
            new_lines.append(line)

    # Add if not present
    if not found_user:
        new_lines.append(f"WebUI\\Username={username}")
    if not found_pass:
        new_lines.append(f'WebUI\\Password_PBKDF2="{pw_hash}"')

    new_conf = "\n".join(new_lines)

    # Write back via docker exec
    try:
        w = subprocess.run(
            ["docker", "exec", "-i", "qbittorrent", "tee", conf_path],
            input=new_conf, capture_output=True, text=True, timeout=10,
        )
        if w.returncode != 0:
            return False, "failed to write config"
    except Exception as e:
        return False, f"config write error: {e}"

    # Restart qBittorrent to pick up changes
    try:
        subprocess.run(["docker", "restart", "qbittorrent"], timeout=30,
                       capture_output=True)
    except Exception:
        pass

    return True, "credentials updated (container restarted)"


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Configure auth for media stack services")
    parser.add_argument("--password-file", default="/media/usb0/arr_passwords",
                        help="Path to the password file")
    parser.add_argument("--username", default="loufogle",
                        help="Username to set for all services")
    args = parser.parse_args()

    print("━━━ MediaStack Auth Configurator ━━━\n")

    passwords = parse_passwords(args.password_file)
    found = [s for s in ALL_SERVICES if s in passwords]
    if not found:
        print("No service passwords found in the file.")
        sys.exit(1)

    print(f"Passwords found for: {', '.join(found)}\n")

    # --- *arr services ---
    for name, cfg in ARR_SERVICES.items():
        if name not in passwords:
            print(f"  SKIP   {name:15s}  no password in file")
            continue
        ok, msg = configure_arr(name, cfg["port"], cfg["api"], args.username, passwords[name])
        tag = "  OK  " if ok else " FAIL "
        print(f"  {tag}  {name:15s}  :{cfg['port']}  {msg}")

    # --- Jackett ---
    if "jackett" in passwords:
        ok, msg = configure_jackett(9117, passwords["jackett"])
        tag = "  OK  " if ok else " FAIL "
        print(f"  {tag}  {'jackett':15s}  :9117  {msg}")
    else:
        print(f"  SKIP   {'jackett':15s}  no password in file")

    # --- qBittorrent ---
    if "qbittorrent" in passwords:
        ok, msg = configure_qbittorrent(8080, args.username, passwords["qbittorrent"])
        tag = "  OK  " if ok else " FAIL "
        print(f"  {tag}  {'qbittorrent':15s}  :8080  {msg}")
    else:
        print(f"  SKIP   {'qbittorrent':15s}  no password in file")

    print("\n━━━ Done ━━━")
    print("Tip: restart services for auth changes to take full effect:")
    svc_list = " ".join(s for s in found if container_running(s))
    if svc_list:
        print(f"  docker restart {svc_list}")


if __name__ == "__main__":
    main()
