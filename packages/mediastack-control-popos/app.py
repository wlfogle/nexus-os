#!/usr/bin/env python3
import json
import os
import threading
import tkinter as tk
from tkinter import ttk, messagebox
from urllib import request, error
from urllib.parse import urlparse, urlencode
import webbrowser


CONFIG_DIR = os.path.expanduser("~/.config/mediastack-control")
CONFIG_PATH = os.path.join(CONFIG_DIR, "config.json")
DEFAULT_URL = "http://127.0.0.1:9900"


class MediaControlDesktopApp:
    def __init__(self, root: tk.Tk):
        self.root = root
        self.root.title("MediaStack Control (Pop!_OS)")
        self.root.geometry("1280x760")
        self.root.minsize(980, 620)

        self.service_map = {}
        self.category_nodes = {}
        self.auto_refresh_ms = 10000

        self.server_url = tk.StringVar(value=self._load_url())
        self.stats_var = tk.StringVar(value="Running: -  Stopped: -  Total: -")
        self.system_var = tk.StringVar(value="CPU -%   MEM -%   DISK -%")
        self.status_var = tk.StringVar(value="Ready")
        self.lines_var = tk.IntVar(value=300)

        self._build_ui()
        self.refresh_data()
        self._schedule_auto_refresh()

    # ---------- config ----------
    def _load_url(self) -> str:
        try:
            with open(CONFIG_PATH, "r", encoding="utf-8") as f:
                data = json.load(f)
                return data.get("server_url", DEFAULT_URL)
        except Exception:
            return DEFAULT_URL

    def _save_url(self):
        os.makedirs(CONFIG_DIR, exist_ok=True)
        with open(CONFIG_PATH, "w", encoding="utf-8") as f:
            json.dump({"server_url": self.server_url.get().strip()}, f, indent=2)
        self.status_var.set(f"Saved server URL: {self.server_url.get().strip()}")

    # ---------- HTTP ----------
    def _base(self) -> str:
        return self.server_url.get().strip().rstrip("/")

    def _json_get(self, path: str):
        url = f"{self._base()}{path}"
        req = request.Request(url, method="GET")
        with request.urlopen(req, timeout=20) as resp:
            return json.loads(resp.read().decode("utf-8"))

    def _json_post(self, path: str, payload=None):
        payload = payload or {}
        body = json.dumps(payload).encode("utf-8")
        req = request.Request(
            f"{self._base()}{path}",
            data=body,
            method="POST",
            headers={"Content-Type": "application/json"},
        )
        with request.urlopen(req, timeout=20) as resp:
            return json.loads(resp.read().decode("utf-8"))

    # ---------- UI ----------
    def _build_ui(self):
        root = self.root

        top = ttk.Frame(root, padding=10)
        top.pack(fill=tk.X)

        ttk.Label(top, text="Server URL:").pack(side=tk.LEFT)
        url_entry = ttk.Entry(top, textvariable=self.server_url, width=54)
        url_entry.pack(side=tk.LEFT, padx=(8, 8))

        ttk.Button(top, text="Save", command=self._save_url).pack(side=tk.LEFT, padx=(0, 6))
        ttk.Button(top, text="Refresh", command=self.refresh_data).pack(side=tk.LEFT, padx=(0, 6))
        ttk.Button(top, text="Open Dashboard", command=self.open_dashboard).pack(side=tk.LEFT, padx=(0, 6))

        stats_frame = ttk.Frame(root, padding=(10, 0, 10, 8))
        stats_frame.pack(fill=tk.X)
        ttk.Label(stats_frame, textvariable=self.stats_var, font=("Sans", 11, "bold")).pack(side=tk.LEFT)
        ttk.Label(stats_frame, text="   |   ").pack(side=tk.LEFT)
        ttk.Label(stats_frame, textvariable=self.system_var).pack(side=tk.LEFT)

        center = ttk.Frame(root, padding=(10, 0, 10, 6))
        center.pack(fill=tk.BOTH, expand=True)

        columns = ("name", "status", "ports", "image")
        self.tree = ttk.Treeview(center, columns=columns, show="tree headings", height=20)
        self.tree.heading("#0", text="Category / Service")
        self.tree.heading("name", text="Service Name")
        self.tree.heading("status", text="Status")
        self.tree.heading("ports", text="Ports")
        self.tree.heading("image", text="Image")
        self.tree.column("#0", width=220, anchor=tk.W)
        self.tree.column("name", width=220, anchor=tk.W)
        self.tree.column("status", width=130, anchor=tk.W)
        self.tree.column("ports", width=260, anchor=tk.W)
        self.tree.column("image", width=280, anchor=tk.W)
        self.tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        yscroll = ttk.Scrollbar(center, orient=tk.VERTICAL, command=self.tree.yview)
        self.tree.configure(yscroll=yscroll.set)
        yscroll.pack(side=tk.RIGHT, fill=tk.Y)

        self.tree.bind("<<TreeviewSelect>>", self._on_select)

        action = ttk.Frame(root, padding=(10, 6, 10, 6))
        action.pack(fill=tk.X)
        self.btn_start = ttk.Button(action, text="Start", command=lambda: self.service_action("start"))
        self.btn_stop = ttk.Button(action, text="Stop", command=lambda: self.service_action("stop"))
        self.btn_restart = ttk.Button(action, text="Restart", command=lambda: self.service_action("restart"))
        self.btn_logs = ttk.Button(action, text="Logs", command=self.show_logs)
        self.btn_open = ttk.Button(action, text="Open UI", command=self.open_selected_ui)
        for b in (self.btn_start, self.btn_stop, self.btn_restart, self.btn_logs, self.btn_open):
            b.pack(side=tk.LEFT, padx=(0, 8))
            b.state(["disabled"])

        log_line = ttk.Frame(root, padding=(10, 0, 10, 8))
        log_line.pack(fill=tk.X)
        ttk.Label(log_line, text="Logs lines:").pack(side=tk.LEFT)
        ttk.Combobox(
            log_line,
            textvariable=self.lines_var,
            values=[100, 300, 500, 1000, 2000, 5000],
            width=6,
            state="readonly",
        ).pack(side=tk.LEFT, padx=(6, 0))

        status = ttk.Frame(root, padding=(10, 0, 10, 10))
        status.pack(fill=tk.X)
        ttk.Label(status, textvariable=self.status_var).pack(anchor=tk.W)

    def _schedule_auto_refresh(self):
        self.root.after(self.auto_refresh_ms, self._auto_refresh_tick)

    def _auto_refresh_tick(self):
        self.refresh_data(silent=True)
        self._schedule_auto_refresh()

    # ---------- data loading ----------
    def refresh_data(self, silent=False):
        def worker():
            try:
                services = self._json_get("/api/services")
                system = self._json_get("/api/system")
                self.root.after(0, lambda: self._apply_data(services, system, silent))
            except Exception as e:
                self.root.after(0, lambda: self.status_var.set(f"Refresh failed: {e}"))

        if not silent:
            self.status_var.set("Refreshing...")
        threading.Thread(target=worker, daemon=True).start()

    def _apply_data(self, services, system, silent=False):
        self.tree.delete(*self.tree.get_children())
        self.service_map.clear()
        self.category_nodes.clear()

        stats = services.get("stats", {})
        self.stats_var.set(
            f"Running: {stats.get('running', 0)}   "
            f"Stopped: {stats.get('stopped', 0)}   "
            f"Total: {stats.get('total', 0)}"
        )

        cpu = (system.get("cpu") or {}).get("percent", 0)
        mem = (system.get("memory") or {}).get("percent", 0)
        disk = (system.get("disk") or {}).get("percent", 0)
        self.system_var.set(f"CPU {cpu:.1f}%   MEM {mem:.1f}%   DISK {disk:.1f}%")

        for group in services.get("groups", []):
            cat = group.get("category", "Other")
            cat_id = f"cat::{cat}"
            self.category_nodes[cat] = self.tree.insert("", tk.END, iid=cat_id, text=cat, open=True)
            for svc in group.get("services", []):
                name = svc.get("name", "")
                display = svc.get("display_name", name)
                status = svc.get("status", "unknown")
                health = svc.get("health") or ""
                status_txt = status if not health else f"{status} ({health})"
                ports = ", ".join(svc.get("ports", [])) if svc.get("ports") else "-"
                image = svc.get("image", "-")
                iid = f"svc::{name}"
                self.service_map[iid] = svc
                self.tree.insert(
                    cat_id,
                    tk.END,
                    iid=iid,
                    text=svc.get("icon", "📦"),
                    values=(display, status_txt, ports, image),
                )

        if not silent:
            self.status_var.set("Refresh complete")

    # ---------- actions ----------
    def _on_select(self, _event=None):
        svc = self._selected_service()
        enabled = "normal" if svc else "disabled"
        for b in (self.btn_start, self.btn_stop, self.btn_restart, self.btn_logs):
            b.state(["!disabled"] if enabled == "normal" else ["disabled"])
        if svc and svc.get("web_url"):
            self.btn_open.state(["!disabled"])
        else:
            self.btn_open.state(["disabled"])

    def _selected_service(self):
        sel = self.tree.selection()
        if not sel:
            return None
        return self.service_map.get(sel[0])

    def service_action(self, action: str):
        svc = self._selected_service()
        if not svc:
            return
        name = svc.get("name")

        def worker():
            try:
                self._json_post(f"/api/services/{name}/{action}", {})
                self.root.after(0, lambda: self.status_var.set(f"{name}: {action} OK"))
                self.root.after(50, lambda: self.refresh_data(silent=True))
            except Exception as e:
                self.root.after(0, lambda: messagebox.showerror("Action failed", f"{name}: {e}"))

        threading.Thread(target=worker, daemon=True).start()

    def show_logs(self):
        svc = self._selected_service()
        if not svc:
            return
        name = svc.get("name")
        lines = max(1, int(self.lines_var.get()))

        win = tk.Toplevel(self.root)
        win.title(f"Logs - {svc.get('display_name', name)}")
        win.geometry("1000x600")

        controls = ttk.Frame(win, padding=8)
        controls.pack(fill=tk.X)
        txt_state = tk.StringVar(value="Loading logs...")
        ttk.Label(controls, textvariable=txt_state).pack(side=tk.LEFT)

        text = tk.Text(win, wrap=tk.NONE, font=("Monospace", 10))
        text.pack(fill=tk.BOTH, expand=True)
        yscroll = ttk.Scrollbar(win, orient=tk.VERTICAL, command=text.yview)
        text.configure(yscrollcommand=yscroll.set)
        yscroll.place(relx=1.0, rely=0, relheight=1.0, anchor="ne")

        def load_logs():
            try:
                data = self._json_get(f"/api/services/{name}/logs?lines={lines}")
                logs = data.get("logs", "")
                self.root.after(0, lambda: _set_text(logs))
            except Exception as e:
                self.root.after(0, lambda: txt_state.set(f"Load failed: {e}"))

        def _set_text(content):
            text.delete("1.0", tk.END)
            text.insert("1.0", content if content else "(no logs)")
            txt_state.set(f"{len(content.splitlines())} lines")

        ttk.Button(controls, text="Reload", command=lambda: threading.Thread(target=load_logs, daemon=True).start()).pack(side=tk.RIGHT)
        threading.Thread(target=load_logs, daemon=True).start()

    def open_dashboard(self):
        webbrowser.open(self._base())

    def _rewrite_localhost(self, url: str) -> str:
        parsed = urlparse(url)
        if parsed.hostname not in ("localhost", "127.0.0.1"):
            return url
        server = urlparse(self._base())
        host = server.hostname or "127.0.0.1"
        scheme = server.scheme or "http"
        port = parsed.port
        path = parsed.path or ""
        query = f"?{parsed.query}" if parsed.query else ""
        return f"{scheme}://{host}:{port}{path}{query}" if port else f"{scheme}://{host}{path}{query}"

    def open_selected_ui(self):
        svc = self._selected_service()
        if not svc:
            return
        web_url = svc.get("web_url")
        if not web_url:
            messagebox.showinfo("No Web UI", "This service does not expose a web URL.")
            return
        webbrowser.open(self._rewrite_localhost(web_url))


def main():
    root = tk.Tk()
    app = MediaControlDesktopApp(root)
    root.mainloop()


if __name__ == "__main__":
    main()
