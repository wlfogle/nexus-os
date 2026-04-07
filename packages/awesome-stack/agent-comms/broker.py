#!/usr/bin/env python3
"""
Simple Message Broker for Agent Communication
A lightweight HTTP/WebSocket-based message broker for inter-agent communication
"""

import json
import threading
import time
from datetime import datetime, timedelta
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import sqlite3
import os

class MessageBroker:
    def __init__(self, db_path="/opt/agent-comms/messages.db"):
        self.db_path = db_path
        self.init_database()
        self.lock = threading.Lock()
        
    def init_database(self):
        """Initialize SQLite database for message storage"""
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        conn = sqlite3.connect(self.db_path)
        conn.execute('''
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                sender TEXT NOT NULL,
                message TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                read_by TEXT DEFAULT ''
            )
        ''')
        conn.execute('''
            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
                status TEXT DEFAULT 'online'
            )
        ''')
        conn.commit()
        conn.close()

    def post_message(self, channel, sender, message):
        """Post a message to a channel"""
        with self.lock:
            conn = sqlite3.connect(self.db_path)
            conn.execute(
                "INSERT INTO messages (channel, sender, message) VALUES (?, ?, ?)",
                (channel, sender, message)
            )
            # Update agent last seen
            conn.execute(
                "INSERT OR REPLACE INTO agents (id, last_seen) VALUES (?, CURRENT_TIMESTAMP)",
                (sender,)
            )
            conn.commit()
            conn.close()
            
    def get_messages(self, channel, since=None, limit=50):
        """Get messages from a channel"""
        with self.lock:
            conn = sqlite3.connect(self.db_path)
            if since:
                cursor = conn.execute(
                    "SELECT id, sender, message, timestamp FROM messages WHERE channel = ? AND timestamp > ? ORDER BY timestamp DESC LIMIT ?",
                    (channel, since, limit)
                )
            else:
                cursor = conn.execute(
                    "SELECT id, sender, message, timestamp FROM messages WHERE channel = ? ORDER BY timestamp DESC LIMIT ?",
                    (channel, limit)
                )
            messages = cursor.fetchall()
            conn.close()
            return [{"id": m[0], "sender": m[1], "message": m[2], "timestamp": m[3]} for m in messages]
    
    def get_channels(self):
        """Get list of active channels"""
        with self.lock:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.execute("SELECT DISTINCT channel FROM messages ORDER BY channel")
            channels = [row[0] for row in cursor.fetchall()]
            conn.close()
            return channels
    
    def get_agents(self):
        """Get list of active agents"""
        with self.lock:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.execute("SELECT id, last_seen, status FROM agents ORDER BY last_seen DESC")
            agents = [{"id": a[0], "last_seen": a[1], "status": a[2]} for a in cursor.fetchall()]
            conn.close()
            return agents

class MessageBrokerHandler(BaseHTTPRequestHandler):
    def __init__(self, *args, broker=None, **kwargs):
        self.broker = broker
        super().__init__(*args, **kwargs)
    
    def do_GET(self):
        """Handle GET requests - retrieve messages"""
        try:
            parsed_path = urlparse(self.path)
            path_parts = parsed_path.path.strip('/').split('/')
            query_params = parse_qs(parsed_path.query)
            
            if path_parts[0] == 'messages' and len(path_parts) >= 2:
                channel = path_parts[1]
                since = query_params.get('since', [None])[0]
                limit = int(query_params.get('limit', [50])[0])
                
                messages = self.broker.get_messages(channel, since, limit)
                self.send_json_response({"messages": messages})
                
            elif path_parts[0] == 'channels':
                channels = self.broker.get_channels()
                self.send_json_response({"channels": channels})
                
            elif path_parts[0] == 'agents':
                agents = self.broker.get_agents()
                self.send_json_response({"agents": agents})
                
            elif path_parts[0] == 'status':
                self.send_json_response({
                    "status": "running",
                    "timestamp": datetime.now().isoformat(),
                    "channels": len(self.broker.get_channels()),
                    "agents": len(self.broker.get_agents())
                })
            else:
                self.send_error(404, "Not Found")
                
        except Exception as e:
            self.send_error(500, f"Internal Server Error: {str(e)}")
    
    def do_POST(self):
        """Handle POST requests - send messages"""
        try:
            content_length = int(self.headers.get('Content-Length', 0))
            post_data = self.rfile.read(content_length).decode('utf-8')
            data = json.loads(post_data)
            
            parsed_path = urlparse(self.path)
            path_parts = parsed_path.path.strip('/').split('/')
            
            if path_parts[0] == 'messages' and len(path_parts) >= 2:
                channel = path_parts[1]
                sender = data.get('sender', 'anonymous')
                message = data.get('message', '')
                
                if message:
                    self.broker.post_message(channel, sender, message)
                    self.send_json_response({"status": "success", "message": "Message posted"})
                else:
                    self.send_error(400, "Bad Request: Message required")
            else:
                self.send_error(404, "Not Found")
                
        except json.JSONDecodeError:
            self.send_error(400, "Bad Request: Invalid JSON")
        except Exception as e:
            self.send_error(500, f"Internal Server Error: {str(e)}")
    
    def send_json_response(self, data):
        """Send JSON response"""
        response = json.dumps(data, indent=2)
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', len(response))
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(response.encode('utf-8'))
    
    def log_message(self, format, *args):
        """Custom log format"""
        print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] {format % args}")

def create_handler(broker):
    """Create handler with broker instance"""
    def handler(*args, **kwargs):
        return MessageBrokerHandler(*args, broker=broker, **kwargs)
    return handler

def main():
    print("🚀 Starting Agent Communication Broker...")
    
    # Initialize message broker
    broker = MessageBroker()
    
    # Create HTTP server
    handler = create_handler(broker)
    server = HTTPServer(('0.0.0.0', 8080), handler)
    
    print("📨 Message Broker running on http://0.0.0.0:8080")
    print("\nEndpoints:")
    print("  GET  /status                    - Broker status")
    print("  GET  /channels                  - List channels")
    print("  GET  /agents                    - List agents")
    print("  GET  /messages/{channel}        - Get messages from channel")
    print("  POST /messages/{channel}        - Post message to channel")
    print("\nExample usage:")
    print("  curl -X POST http://192.168.122.86:8080/messages/general \\")
    print("       -H 'Content-Type: application/json' \\")
    print("       -d '{\"sender\":\"agent1\",\"message\":\"Hello from agent 1!\"}'")
    print("\n  curl http://192.168.122.86:8080/messages/general")
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n🛑 Shutting down broker...")
        server.shutdown()

if __name__ == "__main__":
    main()
