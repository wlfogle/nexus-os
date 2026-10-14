#!/usr/bin/env python3
"""Scan the live homelab and report where the docs disagree with reality.

Why this exists
---------------
Docs in this ecosystem drift silently. Services move between hosts, CTs are
retired during consolidations, ports change to dodge collisions -- and the
markdown that describes all of it is updated by hand, or not at all. By
2026-10-13 that had produced six Caddy routes pointing at dead containers, two
docs contradicting each other about which host ran the WireGuard server, and a
hardcoded model-routing table referring to models that no longer existed.

This tool replaces "remember to update the docs" with a measurement. It:

  1. probes every known host, and enumerates what each one is *actually*
     listening on (via SSH where available);
  2. extracts every ``IP`` / ``IP:port`` reference from the tracked docs and
     configs of each repo;
  3. cross-references the two, so a reference to a dead endpoint is an error
     rather than a thing somebody notices months later;
  4. checks that every Ollama model named in code/config is actually installed;
  5. regenerates a timestamped, generated-not-handwritten state document.

Usage
-----
    infra-audit.py                  # human-readable drift report
    infra-audit.py --write          # also regenerate the state document
    infra-audit.py --check          # exit 1 if drift found (cron / CI)
    infra-audit.py --json           # machine-readable output

Exit codes: 0 = no drift, 1 = drift found (``--check`` only), 2 = scan error.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime as _dt
import ipaddress
import json
import re
import shutil
import socket
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

# --------------------------------------------------------------------------
# Inventory.
#
# Only *hosts* are declared. Services are discovered, never declared -- a
# hardcoded service list is the very thing that goes stale.
# --------------------------------------------------------------------------

VAULT = Path("/media/loufogle/Data/Repos")

# Repos are discovered, not listed. A hardcoded repo list is the same failure
# mode as a hardcoded service list.
SKIP_REPOS = {"_bundles"}

# Which repos describe the *live* system. Per NEXUS-OS-SOURCE-OF-TRUTH.md sec.1,
# everything else is a forerunner, a legacy snapshot, or a third-party clone.
#
# This distinction is the difference between a useful tool and a wall of noise:
# a reference to a retired container inside `homelab-media-stack` is not a bug,
# it is an accurate record of a topology that genuinely existed. Only findings
# in active repos are actionable, and only those gate --check and --fix.
ACTIVE_REPOS = {"nexus-os", "nexus-mediastack"}

# Files that deliberately name retired things, because recording what was
# removed is their entire purpose. Auto-rewriting these would destroy the
# audit trail -- MISTAKES.md would stop naming the mistake, and the action
# list would stop saying which models were deleted. Reported, never rewritten.
NEVER_REWRITE = {
    "MISTAKES.md",
    "AI-STACK-ACTIONS.md",
    "INFRASTRUCTURE-STATE.md",
    "NEXUS-OS-SOURCE-OF-TRUTH.md",
    "infra-audit.py",
    "CHANGELOG.md",
}


def is_active(path: Path) -> bool:
    """True when a path lives inside a repo that describes the live system."""
    try:
        rel = path.relative_to(VAULT)
    except ValueError:
        return False
    return bool(rel.parts) and rel.parts[0] in ACTIVE_REPOS


def discover_repos() -> list[Path]:
    """Every git repo directly under the vault."""
    if not VAULT.is_dir():
        return []
    out = []
    for p in sorted(VAULT.iterdir()):
        if not p.is_dir() or p.name in SKIP_REPOS:
            continue
        if (p / ".git").exists():
            out.append(p)
    return out


STATE_DOC = VAULT / "nexus-os/docs/INFRASTRUCTURE-STATE.md"


@dataclass(frozen=True)
class Host:
    ip: str
    name: str
    role: str
    # SSH alias, or None when the host cannot be shelled into.
    ssh: str | None = None
    # Probe these ports even if SSH enumeration is unavailable.
    probe_ports: tuple[int, ...] = ()


HOSTS: tuple[Host, ...] = (
    Host("192.168.12.242", "tiamat", "Proxmox VE host", ssh="tiamat",
         probe_ports=(8006, 22)),
    Host("192.168.12.30", "ct-300 mediastack", "Consolidated media stack LXC",
         ssh="mediastack"),
    Host("192.168.12.244", "bahamut", "Pi 4 edge node, Tailscale subnet router",
         ssh="bahamut"),
    Host("192.168.12.222", "vm-100 openwrt", "Primary router + WireGuard server",
         ssh="openwrt"),
    Host("192.168.12.123", "vm-990 haos", "Home Assistant OS",
         probe_ports=(8123,)),
    Host("192.168.12.254", "archer stella", "LAN gateway / DHCP",
         probe_ports=(80, 443)),
)

# Hosts that are expected to be absent. Keeps retired kit from being reported
# as drift forever, while still documenting that it is gone on purpose.
RETIRED: dict[str, str] = {
    "192.168.12.104": "vaultwarden CT - retired in the CT-300 consolidation",
    "192.168.12.107": "authentik CT - retired in the CT-300 consolidation",
    "192.168.12.100": "wireguard CT - server moved to OpenWrt VM-100",
    "192.168.12.210": "prowlarr CT - *arr stack replaced by Riven",
    "192.168.12.214": "sonarr CT - *arr stack replaced by Riven",
    "192.168.12.225": "radarr CT - *arr stack replaced by Riven",
    "192.168.12.251": "CT-501 HABridge - retired in the CT-300 consolidation",
    "192.168.12.250": "CT-900 ziggy - stopped",
}

# Files worth scanning for endpoint references.
SCAN_SUFFIXES = {".md", ".yaml", ".yml", ".json", ".xml", ".kt", ".sh", ".py",
                 ".rs", ".toml", ".conf"}
SCAN_EXTRA_NAMES = {"Caddyfile"}

# Directories never worth walking.
SKIP_DIRS = {".git", "node_modules", "target", "build", "dist", "__pycache__",
             ".venv", "venv", "legacy", "_consolidate", "buildiso-chroots",
             "iso_root", ".cargo", "limine"}

IP_RE = re.compile(r"\b(192\.168\.\d{1,3}\.\d{1,3})(?::(\d{1,5}))?\b")
# Ollama model references: `name:tag` where tag is a size/latest style token.
MODEL_RE = re.compile(
    r"\b([a-z0-9][a-z0-9._-]{1,40}:(?:latest|[0-9]+(?:\.[0-9]+)?[bB]|"
    r"[0-9]+x[0-9]+[bB]))\b"
)

TCP_TIMEOUT = 3.0
# Generous: six hosts are probed in parallel and a loaded LXC can take a while
# to accept. Too tight a timeout makes enumeration fail silently and fall back
# to port probing, which looks like "this host only runs 3 services".
SSH_TIMEOUT = 30

# --------------------------------------------------------------------------
# Remap table: stale reference -> verified current reference.
#
# Only entries with a *confident, verified* replacement belong here. A dead
# endpoint with no obvious successor is reported, never silently rewritten --
# guessing a replacement would bake a new wrong fact into the docs, which is
# the problem this tool exists to solve.
#
# Format: "old" -> ("new", "why")
# Keys may be bare IPs or IP:port. IP:port keys are matched first.
# --------------------------------------------------------------------------

ENDPOINT_REMAP: dict[str, tuple[str, str]] = {
    "192.168.12.251:7655": ("192.168.12.30:7655",
                            "Pulse moved to CT-300 in the consolidation"),
    "192.168.12.244:3000": ("192.168.12.244:8081",
                            "AdGuard admin is :8081; :3000 is setup-only"),
    "192.168.12.104:443": ("192.168.12.244:8080",
                           "Vaultwarden runs natively on Bahamut"),
    "192.168.12.104": ("192.168.12.244",
                       "Vaultwarden runs natively on Bahamut"),
}

MODEL_REMAP: dict[str, tuple[str, str]] = {
    "llama3.2:3b":         ("qwen3.5:4b", "pruned 2026-10-13; 4b is the fast tier"),
    "qwen2.5:7b":          ("qwen3.5:9b", "pruned 2026-10-13"),
    "qwen2.5-coder:7b":    ("qwen3.5:9b", "pruned 2026-10-13"),
    "llama3.2-vision:11b": ("qwen3.5:9b", "pruned 2026-10-13; qwen3.5 is multimodal"),
    "codestral:22b":       ("qwen3.5:9b", "pruned 2026-10-13"),
    "phi4:latest":         ("qwen3.5:9b", "pruned 2026-10-13"),
}


# --------------------------------------------------------------------------
# Probing
# --------------------------------------------------------------------------

@dataclass
class Listener:
    port: int
    process: str


@dataclass
class HostState:
    host: Host
    reachable: bool = False
    listeners: list[Listener] = field(default_factory=list)
    enumerated: bool = False      # True when SSH enumeration succeeded
    note: str = ""

    @property
    def ports(self) -> set[int]:
        return {l.port for l in self.listeners}


def tcp_open(ip: str, port: int, timeout: float = TCP_TIMEOUT) -> bool:
    try:
        with socket.create_connection((ip, port), timeout=timeout):
            return True
    except OSError:
        return False


def _ssh(alias: str, remote_cmd: str) -> str | None:
    """Run a command over SSH. Returns stdout, or None if unreachable."""
    if shutil.which("ssh") is None:
        return None
    try:
        p = subprocess.run(
            ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=6",
             "-o", "StrictHostKeyChecking=accept-new", alias, remote_cmd],
            capture_output=True, text=True, timeout=SSH_TIMEOUT,
        )
    except (subprocess.TimeoutExpired, OSError):
        return None
    return p.stdout if p.returncode == 0 else None


_SS_RE = re.compile(r":(\d+)\s")
_PROC_RE = re.compile(r'users:\(\("([^"]+)"')


def enumerate_listeners(alias: str) -> list[Listener] | None:
    """Enumerate listening TCP sockets on a host.

    Tries `ss` with process names, then plain `ss`, then `netstat` for BusyBox
    hosts such as OpenWrt where `ss` may be absent. Returns None only when no
    method produced output, so the caller can distinguish "nothing listening"
    from "could not look".
    """
    out = _ssh(alias, "ss -ltnpH 2>/dev/null || ss -ltnH 2>/dev/null || "
                      "ss -ltn 2>/dev/null || netstat -ltn 2>/dev/null")
    if out is None or not out.strip():
        return None
    found: dict[int, str] = {}
    for line in out.splitlines():
        # Local address is the 4th column for `ss -ltn` output.
        parts = line.split()
        if len(parts) < 4:
            continue
        m = _SS_RE.search(parts[3] + " ")
        if not m:
            continue
        port = int(m.group(1))
        proc = _PROC_RE.search(line)
        name = proc.group(1) if proc else "?"
        # Prefer a named process over a placeholder.
        if port not in found or found[port] == "?":
            found[port] = name
    return [Listener(p, n) for p, n in sorted(found.items())]


def probe_host(h: Host) -> HostState:
    st = HostState(host=h)

    # Reachability: any open port, or a successful SSH, counts. ICMP alone is
    # unreliable (OpenWrt and some CTs drop it).
    if h.ssh:
        listeners = enumerate_listeners(h.ssh)
        if listeners is not None:
            st.reachable = True
            st.enumerated = True
            st.listeners = listeners
            return st
        st.note = "SSH enumeration failed; fell back to port probing"

    ports = set(h.probe_ports) | {22, 80, 443, 8006, 8123}
    for port in sorted(ports):
        if tcp_open(h.ip, port):
            st.reachable = True
            st.listeners.append(Listener(port, "?"))
    if not st.reachable:
        st.note = st.note or "no response on probed ports"
    return st


def scan_hosts() -> dict[str, HostState]:
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(HOSTS)) as ex:
        results = list(ex.map(probe_host, HOSTS))
    return {st.host.ip: st for st in results}


# --------------------------------------------------------------------------
# Reference extraction
# --------------------------------------------------------------------------

@dataclass(frozen=True)
class Ref:
    ip: str
    port: int | None
    path: Path
    line: int
    text: str
    commented: bool


def _is_commented(line: str) -> bool:
    s = line.lstrip()
    return s.startswith(("#", "//", "<!--", "*", "- #"))


def iter_scan_files(repo: Path):
    for p in repo.rglob("*"):
        if not p.is_file():
            continue
        if any(part in SKIP_DIRS for part in p.parts):
            continue
        if p.suffix.lower() in SCAN_SUFFIXES or p.name in SCAN_EXTRA_NAMES:
            yield p


def extract_refs(repo: Path) -> list[Ref]:
    refs: list[Ref] = []
    for path in iter_scan_files(repo):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for lineno, line in enumerate(text.splitlines(), 1):
            for m in IP_RE.finditer(line):
                ip, port = m.group(1), m.group(2)
                try:
                    ipaddress.ip_address(ip)
                except ValueError:
                    continue
                refs.append(Ref(
                    ip=ip,
                    port=int(port) if port else None,
                    path=path,
                    line=lineno,
                    text=line.strip()[:160],
                    commented=_is_commented(line),
                ))
    return refs


# --------------------------------------------------------------------------
# Ollama model checks
# --------------------------------------------------------------------------

def installed_models() -> set[str] | None:
    if shutil.which("ollama") is None:
        return None
    try:
        p = subprocess.run(["ollama", "list"], capture_output=True,
                           text=True, timeout=20)
    except (subprocess.TimeoutExpired, OSError):
        return None
    if p.returncode != 0:
        return None
    out = set()
    for line in p.stdout.splitlines()[1:]:
        if line.strip():
            out.add(line.split()[0])
    return out


# Tokens that match the model regex but are not Ollama models.
_MODEL_FALSE_POSITIVES = re.compile(
    r"^(?:sha256|version|v[0-9]|cuda|python|node|debian|ubuntu|alpine|"
    r"postgres|redis|nginx|caddy|http|https|ftp|ssh|ollama|image|tag|"
    r"latest|docker|ghcr|lscr)[:.]", re.I
)

# A model name inside code or config is a runtime dependency -- if the model is
# gone, that code breaks. The same name inside prose is just text, frequently a
# historical list, and flagging it drowns the real findings. Only the former is
# treated as drift.
_MODEL_CONFIG_SUFFIXES = {".rs", ".py", ".json", ".toml", ".yaml", ".yml",
                         ".sh", ".kt", ".conf", ".env"}


def extract_model_refs(repo: Path) -> list[Ref]:
    """Find Ollama model references.

    Restricted to files that actually mention Ollama. Without that gate the
    `name:tag` pattern matches an enormous amount of unrelated text -- docker
    tags, YAML keys, markdown anchors -- and buries the real findings. A model
    name only means something in a file that talks to a model server.
    """
    refs: list[Ref] = []
    for path in iter_scan_files(repo):
        if path.suffix.lower() not in _MODEL_CONFIG_SUFFIXES:
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if "ollama" not in text.lower():
            continue
        for lineno, line in enumerate(text.splitlines(), 1):
            for m in MODEL_RE.finditer(line):
                token = m.group(1)
                if _MODEL_FALSE_POSITIVES.match(token):
                    continue
                refs.append(Ref(ip=token, port=None, path=path, line=lineno,
                                text=line.strip()[:160],
                                commented=_is_commented(line)))
    return refs


# --------------------------------------------------------------------------
# Analysis
# --------------------------------------------------------------------------

@dataclass
class Finding:
    severity: str          # "dead" | "wrong-port" | "unknown-host" | "model"
    detail: str
    ref: Ref

    @property
    def actionable(self) -> bool:
        """Findings in archived repos are history, not drift."""
        return is_active(self.ref.path)


def analyse(states: dict[str, HostState], refs: list[Ref]) -> list[Finding]:
    findings: list[Finding] = []
    for r in refs:
        if r.commented:
            continue  # already acknowledged as historical
        if r.ip in RETIRED:
            findings.append(Finding(
                "dead", f"{r.ip} is retired ({RETIRED[r.ip]})", r))
            continue
        st = states.get(r.ip)
        if st is None:
            continue  # not a host we track; could be a printer, TV, etc.
        if not st.reachable:
            findings.append(Finding(
                "dead", f"{r.ip} ({st.host.name}) did not respond", r))
            continue
        if r.port is not None and st.enumerated and r.port not in st.ports:
            findings.append(Finding(
                "wrong-port",
                f"{r.ip} ({st.host.name}) is up but nothing is listening on "
                f"{r.port}", r))
    return findings


def analyse_models(model_refs: list[Ref],
                   installed: set[str] | None) -> list[Finding]:
    if installed is None:
        return []
    findings = []
    for r in model_refs:
        if r.commented:
            continue
        if r.ip not in installed:
            findings.append(Finding(
                "model", f"model '{r.ip}' is referenced but not installed", r))
    return findings


# --------------------------------------------------------------------------
# Rewriting
# --------------------------------------------------------------------------

@dataclass
class Rewrite:
    path: Path
    line: int
    old: str
    new: str
    why: str


# Opt-out markers. Some source legitimately *names* retired things as data --
# a remap table, or a test asserting that retired names do not survive. Those
# lines must survive the rewriter, or it destroys the very mapping that makes
# it correct. Filename-based exemption is too blunt here (`config.rs` is not a
# name worth blanket-skipping), so the opt-out is explicit and greppable.
IGNORE_LINE = "infra-audit: ignore"
IGNORE_START = "infra-audit: ignore-start"
IGNORE_END = "infra-audit: ignore-end"


def ignored_lines(text: str) -> set[int]:
    """1-based line numbers the rewriter must leave alone."""
    out: set[int] = set()
    inside = False
    for lineno, line in enumerate(text.splitlines(), 1):
        if IGNORE_START in line:
            inside = True
            out.add(lineno)
            continue
        if IGNORE_END in line:
            inside = False
            out.add(lineno)
            continue
        if inside or IGNORE_LINE in line:
            out.add(lineno)
    return out


def _plan_line(line: str) -> tuple[str, list[tuple[str, str, str]]]:
    """Apply every applicable remap to one line.

    Returns the rewritten line and the list of (old, new, why) substitutions.
    IP:port keys are applied before bare-IP keys so the more specific mapping
    wins; otherwise rewriting ``.104`` first would strip the port context that
    ``.104:443`` needs to match.
    """
    subs: list[tuple[str, str, str]] = []
    out = line

    endpoint_keys = sorted(ENDPOINT_REMAP, key=lambda k: (":" not in k, -len(k)))
    for old in endpoint_keys:
        if old in out:
            new, why = ENDPOINT_REMAP[old]
            out = out.replace(old, new)
            subs.append((old, new, why))

    for old, (new, why) in MODEL_REMAP.items():
        # Word-boundary match so `qwen2.5:7b` does not also hit a longer token.
        pat = re.compile(rf"(?<![\w:.-]){re.escape(old)}(?![\w.-])")
        if pat.search(out):
            out = pat.sub(new, out)
            subs.append((old, new, why))

    return out, subs


def plan_rewrites(repos: list[Path], all_repos: bool = False) -> list[Rewrite]:
    """Find every line a confident remap applies to.

    Restricted to active repos unless `all_repos` is set. Rewriting an archived
    repo would corrupt an accurate historical record.
    """
    plans: list[Rewrite] = []
    for repo in repos:
        if not all_repos and repo.name not in ACTIVE_REPOS:
            continue
        for path in iter_scan_files(repo):
            if path.name in NEVER_REWRITE:
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError):
                continue
            skip = ignored_lines(text)
            for lineno, line in enumerate(text.splitlines(), 1):
                if lineno in skip:
                    continue
                _, subs = _plan_line(line)
                for old, new, why in subs:
                    plans.append(Rewrite(path=path, line=lineno, old=old,
                                         new=new, why=why))
    return plans


def apply_rewrites(plans: list[Rewrite]) -> int:
    """Rewrite the planned files in place. Returns the number of files changed.

    Files are rewritten whole, once, preserving the trailing newline. Relies on
    git for undo -- this must only ever be run in a clean-ish working tree.
    """
    by_file: dict[Path, list[Rewrite]] = {}
    for p in plans:
        by_file.setdefault(p.path, []).append(p)

    changed = 0
    for path in by_file:
        if path.name in NEVER_REWRITE:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        had_trailing_nl = text.endswith("\n")
        skip = ignored_lines(text)
        lines = text.splitlines()
        dirty = False
        for i, line in enumerate(lines):
            if (i + 1) in skip:
                continue
            new_line, subs = _plan_line(line)
            if subs and new_line != line:
                lines[i] = new_line
                dirty = True
        if not dirty:
            continue
        out = "\n".join(lines) + ("\n" if had_trailing_nl else "")
        try:
            path.write_text(out, encoding="utf-8")
        except OSError:
            continue
        changed += 1
    return changed


# --------------------------------------------------------------------------
# Output
# --------------------------------------------------------------------------

def render_state_doc(states: dict[str, HostState],
                     installed: set[str] | None,
                     findings: list[Finding]) -> str:
    now = _dt.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    L: list[str] = []
    L.append("# Infrastructure State (generated)")
    L.append("")
    L.append("> **Do not edit by hand.** Regenerate with")
    L.append("> `python3 nexus-os/scripts/infra-audit.py --write`.")
    L.append("> Everything below was measured, not remembered.")
    L.append(f"> Last scan: {now}")
    L.append("")

    L.append("## Hosts")
    L.append("")
    for h in HOSTS:
        st = states[h.ip]
        status = "UP" if st.reachable else "**UNREACHABLE**"
        how = "ss over ssh" if st.enumerated else "tcp probe"
        L.append(f"### {h.name} - `{h.ip}`")
        L.append(f"- Role: {h.role}")
        L.append(f"- Status: {status} ({how})")
        if st.note:
            L.append(f"- Note: {st.note}")
        if st.listeners:
            L.append("- Listening:")
            for l in st.listeners:
                proc = f" ({l.process})" if l.process != "?" else ""
                L.append(f"  - `{l.port}`{proc}")
        L.append("")

    L.append("## Retired hosts")
    L.append("")
    L.append("Deliberately gone. Referencing these in live config is a bug.")
    L.append("")
    for ip, why in sorted(RETIRED.items()):
        L.append(f"- `{ip}` - {why}")
    L.append("")

    L.append("## Ollama models installed")
    L.append("")
    if installed is None:
        L.append("_ollama not available on this machine._")
    else:
        for m in sorted(installed):
            L.append(f"- `{m}`")
    L.append("")

    L.append("## Drift")
    L.append("")
    if not findings:
        L.append("None. Every tracked reference resolves to something live.")
    else:
        L.append(f"{len(findings)} stale reference(s) found:")
        L.append("")
        for f in findings:
            L.append(f"- **{f.severity}** `{f.ref.path}:{f.ref.line}` - {f.detail}")
    L.append("")
    return "\n".join(L)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--write", action="store_true",
                    help="regenerate the generated state document")
    ap.add_argument("--check", action="store_true",
                    help="exit 1 when drift is found")
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    ap.add_argument("--fix", action="store_true",
                    help="rewrite stale references that have a verified "
                         "replacement (dry run unless --apply is also given)")
    ap.add_argument("--apply", action="store_true",
                    help="with --fix, actually write the changes to disk")
    ap.add_argument("--all-repos", action="store_true",
                    help="include archived/forerunner repos when fixing "
                         "(default: only " + ", ".join(sorted(ACTIVE_REPOS)) + ")")
    args = ap.parse_args()

    repos = discover_repos()
    if not repos:
        print(f"error: no git repos found under {VAULT}", file=sys.stderr)
        return 2
    if not args.json:
        print(f"repos: {len(repos)} discovered under {VAULT}")
        for r in repos:
            print(f"  - {r.name}")
        print()

    states = scan_hosts()

    refs: list[Ref] = []
    model_refs: list[Ref] = []
    for repo in repos:
        refs.extend(extract_refs(repo))
        model_refs.extend(extract_model_refs(repo))

    installed = installed_models()
    findings = analyse(states, refs) + analyse_models(model_refs, installed)

    if args.json:
        print(json.dumps({
            "scanned": _dt.datetime.now().isoformat(timespec="seconds"),
            "hosts": {
                ip: {
                    "name": st.host.name,
                    "reachable": st.reachable,
                    "enumerated": st.enumerated,
                    "ports": sorted(st.ports),
                } for ip, st in states.items()
            },
            "models_installed": sorted(installed) if installed else None,
            "findings": [
                {"severity": f.severity, "detail": f.detail,
                 "file": str(f.ref.path), "line": f.ref.line}
                for f in findings
            ],
        }, indent=2))
    else:
        up = sum(1 for s in states.values() if s.reachable)
        print(f"hosts: {up}/{len(states)} reachable")
        degraded = 0
        for ip, st in states.items():
            mark = "ok  " if st.reachable else "DOWN"
            how = "ssh" if st.enumerated else "probe"
            if st.host.ssh and not st.enumerated:
                how = "PROBE-ONLY"
                degraded += 1
            print(f"  [{mark}] {st.host.name:20s} {ip:16s} "
                  f"{len(st.listeners):3d} listening  ({how})")
        if degraded:
            print(f"  warning: {degraded} host(s) have an ssh alias but could "
                  f"not be enumerated; their port lists are incomplete and")
            print("           wrong-port findings are suppressed for them.")
        print(f"references scanned: {len(refs)} endpoint, {len(model_refs)} model")
        if installed is not None:
            print(f"models installed: {len(installed)}")
        print()
        live = [f for f in findings if f.actionable]
        hist = [f for f in findings if not f.actionable]
        if live:
            print(f"ACTIONABLE DRIFT: {len(live)} stale reference(s) in "
                  f"{', '.join(sorted(ACTIVE_REPOS))}")
            by_sev: dict[str, list[Finding]] = {}
            for f in live:
                by_sev.setdefault(f.severity, []).append(f)
            for sev in sorted(by_sev):
                items = by_sev[sev]
                if sev == "model":
                    # One missing model referenced 80 times is one problem,
                    # not 80. Group by model so the real count is visible.
                    grouped: dict[str, list[Finding]] = {}
                    for f in items:
                        grouped.setdefault(f.ref.ip, []).append(f)
                    print(f"\n  {sev}: {len(grouped)} missing model(s), "
                          f"{len(items)} reference(s)")
                    for model in sorted(grouped):
                        hits = grouped[model]
                        print(f"    {model}  ({len(hits)} refs)")
                        for f in hits[:3]:
                            print(f"      {f.ref.path}:{f.ref.line}")
                        if len(hits) > 3:
                            print(f"      ... and {len(hits) - 3} more")
                    continue
                print(f"\n  {sev} ({len(items)}):")
                for f in items:
                    print(f"    {f.ref.path}:{f.ref.line}")
                    print(f"      {f.detail}")
        else:
            print("no actionable drift found")
        if hist:
            print(f"\n({len(hist)} further reference(s) in archived/forerunner "
                  "repos - accurate history, not drift. Not reported above.)")

    if args.fix:
        plans = plan_rewrites(repos, all_repos=args.all_repos)
        if not plans:
            print("\nnothing to rewrite: no stale reference has a verified "
                  "replacement")
        else:
            by_file: dict[Path, list[Rewrite]] = {}
            for p in plans:
                by_file.setdefault(p.path, []).append(p)
            verb = "rewriting" if args.apply else "would rewrite"
            print(f"\n{verb} {len(plans)} reference(s) across "
                  f"{len(by_file)} file(s):")
            for path in sorted(by_file):
                print(f"\n  {path}")
                for it in by_file[path]:
                    print(f"    L{it.line}: {it.old} -> {it.new}")
                    print(f"           ({it.why})")
            if args.apply:
                n = apply_rewrites(plans)
                print(f"\nrewrote {n} file(s). Review with: git diff")
            else:
                print("\ndry run. Re-run with --fix --apply to write changes.")

    if args.write:
        STATE_DOC.parent.mkdir(parents=True, exist_ok=True)
        STATE_DOC.write_text(render_state_doc(states, installed, findings),
                             encoding="utf-8")
        print(f"\nwrote {STATE_DOC}")

    if args.check and any(f.actionable for f in findings):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
