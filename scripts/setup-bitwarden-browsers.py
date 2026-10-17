#!/usr/bin/env python3
"""Install public Bitwarden policies; never open browser vault data."""
import argparse
import copy
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


SERVER = "https://vaultwarden.lou-fogle-media-stack.duckdns.org"
CHROME_ID = "nngceckbapebfimnlniiiahkandclblb"
OPERA_ID = "ccnckbpmaceehanjmeomladnmlffdjgn"
FIREFOX_ID = "{446900e4-71c2-419f-a6a7-df9c091e268b}"
CHROME_UPDATE = "https://clients2.google.com/service/update2/crx"
OPERA_UPDATE = "https://extension-updates.opera.com/api/omaha/update/"
FIREFOX_UPDATE = (
    "https://addons.mozilla.org/firefox/downloads/latest/"
    "bitwarden-password-manager/latest.xpi"
)


def merge(original, changes):
    """Merge owned keys without replacing unrelated configuration."""
    result = copy.deepcopy(original)
    for key, value in changes.items():
        if isinstance(value, dict):
            existing = result.get(key, {})
            if not isinstance(existing, dict):
                raise ValueError(f"Expected a policy object at {key}")
            result[key] = merge(existing, value)
        else:
            result[key] = copy.deepcopy(value)
    return result


def chromium_policy(extension_id, update_url):
    return {
        "ExtensionSettings": {
            extension_id: {
                "installation_mode": "force_installed",
                "update_url": update_url,
            }
        },
        "3rdparty": {
            "extensions": {
                extension_id: {"environment": {"base": SERVER}}
            }
        },
    }


def policy_changes():
    chrome = chromium_policy(CHROME_ID, CHROME_UPDATE)
    opera = chromium_policy(OPERA_ID, OPERA_UPDATE)
    return {
        "etc/opt/chrome/policies/managed/nexus-bitwarden.json": chrome,
        "etc/brave/policies/managed/nexus-bitwarden.json": chrome,
        # Retain Chromium's native policy root as well as its Snap root.
        "etc/chromium/policies/managed/nexus-bitwarden.json": chrome,
        # Chromium Snap uses the current revision's system policy root.
        ("var/snap/chromium/current/policies/managed/"
         "nexus-bitwarden.json"): chrome,
        "etc/opt/opera/policies/managed/nexus-bitwarden.json": opera,
        "etc/opt/opera-gx/policies/managed/nexus-bitwarden.json": opera,
        "etc/firefox/policies/policies.json": {
            "policies": {
                "ExtensionSettings": {
                    FIREFOX_ID: {
                        "installation_mode": "force_installed",
                        "install_url": FIREFOX_UPDATE,
                    }
                },
                "3rdparty": {
                    "Extensions": {
                        FIREFOX_ID: {"environment": {"base": SERVER}}
                    }
                },
            }
        },
    }


def install(root):
    """Validate inputs before writing, and privately retain prior files."""
    prepared = []
    for relative, changes in policy_changes().items():
        target = root / relative
        if target.is_symlink():
            raise ValueError(f"Refusing policy file symlink: {target}")
        original = json.loads(target.read_text()) if target.exists() else {}
        if not isinstance(original, dict):
            raise ValueError(f"Expected JSON object: {target}")
        updated = merge(original, changes)
        if updated != original:
            prepared.append((relative, target, updated))
    if not prepared:
        print("All Bitwarden policies already match.")
        return None
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    backup = root / "var/backups" / f"nexus-bitwarden-{stamp}"
    backup.mkdir(parents=True, mode=0o700)
    records = []
    for relative, target, updated in prepared:
        existed = target.exists()
        if existed:
            saved = backup / relative
            saved.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(target, saved)
        target.parent.mkdir(parents=True, exist_ok=True, mode=0o755)
        fd, temporary = tempfile.mkstemp(
            prefix=".bitwarden-", dir=target.parent,
        )
        try:
            with os.fdopen(fd, "w") as stream:
                json.dump(updated, stream, indent=2)
                stream.write("\n")
                stream.flush()
                os.fsync(stream.fileno())
                os.fchmod(stream.fileno(), 0o644)
            os.replace(temporary, target)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)
        records.append({"path": relative, "existed": existed})
        (backup / "files.json").write_text(
            json.dumps(records, indent=2) + "\n",
        )
        print(f"Configured {target}")
    print(f"Rollback records: {backup}")
    return backup


def configure_opera_flatpak():
    """Enable packaged policy forwarding without granting host write access."""
    if os.geteuid() == 0:
        raise ValueError("Run --opera-flatpak as the desktop user, not root.")
    subprocess.run(["flatpak", "info", "com.opera.Opera"],
                   check=True, stdout=subprocess.DEVNULL)
    current = subprocess.run(
        ["flatpak", "override", "--user", "--show", "com.opera.Opera"],
        check=True, capture_output=True, text=True,
    )
    if "host-etc:ro;" in current.stdout:
        print("Opera Flatpak read-only host configuration access already set.")
        return
    data_home = Path(os.environ.get(
        "XDG_DATA_HOME", Path.home() / ".local/share",
    ))
    state_home = Path(os.environ.get(
        "XDG_STATE_HOME", Path.home() / ".local/state",
    ))
    override = data_home / "flatpak/overrides/com.opera.Opera"
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    backup = state_home / "nexus-bitwarden" / stamp
    backup.mkdir(parents=True, mode=0o700)
    if override.exists():
        shutil.copy2(override, backup / "opera-flatpak-override")
    (backup / "state.json").write_text(json.dumps({
        "override": str(override), "existed": override.exists(),
    }, indent=2) + "\n")
    subprocess.run([
        "flatpak", "override", "--user", "--filesystem=host-etc:ro",
        "com.opera.Opera",
    ], check=True)
    print(f"Opera Flatpak override backup: {backup}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--root", type=Path, default=Path("/"),
                      help="Alternate filesystem root for isolated testing")
    mode.add_argument(
        "--opera-flatpak", action="store_true",
        help="As desktop user, enable read-only policy forwarding",
    )
    args = parser.parse_args()
    if args.opera_flatpak:
        configure_opera_flatpak()
        return
    root = args.root.resolve()
    if root == Path("/") and os.geteuid() != 0:
        parser.error("System policy installation requires root.")
    install(root)
    if root == Path("/"):
        print("Policy files are not runtime verification.")
        print(
            "Brave Snap blocks host policy reads: use the official store and "
            "Bitwarden's self-hosted server settings UI for that profile."
        )


if __name__ == "__main__":
    main()
