#!/usr/bin/env python3

import http.server
import socketserver
import json
import subprocess
from urllib.parse import urlparse, parse_qs
import requests

# --- Configuration ---
PORT = 8082
PROXMOX_IP = "192.168.122.9"
PROXMOX_USER = "root"


class HomelabAPI(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path.startswith("/api/check-service"):
            self.handle_check_service()
        elif self.path.startswith("/api/system-stats"):
            self.handle_system_stats()
        else:
            self.send_error(404, "Not Found")

    def do_POST(self):
        if self.path.startswith("/api/execute-command"):
            self.handle_execute_command()
        else:
            self.send_error(404, "Not Found")

    def handle_check_service(self):
        query_components = parse_qs(urlparse(self.path).query)
        url = query_components.get("url", [None])[0]

        if not url:
            self.send_response(400)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"error": "URL parameter is required"}).encode())
            return

        try:
            response = requests.get(url, timeout=5, verify=False)
            status_code = response.status_code
        except requests.exceptions.RequestException:
            status_code = 500

        self.send_response(200 if status_code < 400 else 503)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        self.wfile.write(
            json.dumps({"url": url, "online": status_code < 400, "status_code": status_code})
            .encode()
        )

    def handle_system_stats(self):
        # --- System Monitoring ---
        # Note: These commands are for demonstration purposes.
        #       Replace with more robust monitoring tools if needed.

        cpu_usage = subprocess.getoutput("top -bn1 | grep \"Cpu(s)\" | sed \"s/.*, *\\([0-9.]*\\)%* id.*/\\1/\" | awk '{print 100 - $1}\'%'"
        )
        ram_usage = subprocess.getoutput("free -m | awk 'NR==2{printf \"%.2f%%\", $3*100/$2 }'")
        disk_usage = subprocess.getoutput("df -h / | awk 'NR==2{print $5}'")
        temp = subprocess.getoutput("sensors | grep 'Package id 0' | awk '{print $4}' | sed 's/+//'")

        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        self.wfile.write(
            json.dumps(
                {
                    "cpu": cpu_usage.strip(),
                    "ram": ram_usage.strip(),
                    "disk": disk_usage.strip(),
                    "temp": temp.strip(),
                }
            ).encode()
        )

    def handle_execute_command(self):
        content_length = int(self.headers["Content-Length"])
        post_data = self.rfile.read(content_length)
        command = json.loads(post_data)["command"]

        try:
            # Execute the command in the default terminal
            subprocess.Popen(["konsole", "-e", f"bash -c '{command}; exec bash'"])
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"status": "Command executed"}).encode())
        except Exception as e:
            self.send_response(500)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"error": str(e)}).encode())


if __name__ == "__main__":
    with socketserver.TCPServer(("", PORT), HomelabAPI) as httpd:
        print(f"Homelab API server started at http://localhost:{PORT}")
        httpd.serve_forever()

