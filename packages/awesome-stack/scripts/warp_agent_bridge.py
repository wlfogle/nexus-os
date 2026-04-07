#!/usr/bin/env python3
"""
Warp Agent Bridge Daemon

This daemon acts as a local proxy for Warp terminal agent requests,
forwarding them to the custom broker system at CT-950.
"""

import json
import threading
import time
import requests
import logging
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import socket
import subprocess
import sys
from collections import deque
from datetime import datetime

# Configuration
BROKER_HOST = "192.168.122.86"
BROKER_PORT = 8080
LOCAL_PORT = 9090
AGENT_ID = socket.gethostname()
POLL_INTERVAL = 30  # Poll every 30 seconds
MAX_MESSAGE_HISTORY = 100  # Keep last 100 messages

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/home/alexa/warp_bridge.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

# Global message queue and polling state
message_queue = deque(maxlen=MAX_MESSAGE_HISTORY)
last_poll_time = None
polling_active = False
polling_thread = None
new_message_notifications = []
last_message_count = 0

class WarpBridgeHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        """Override to use our logger instead of stderr"""
        logger.info(f"{self.address_string()} - {format % args}")
    
    def do_GET(self):
        """Handle GET requests from Warp terminal"""
        try:
            parsed_path = urlparse(self.path)
            
            if parsed_path.path == '/health':
                self._send_health_check()
            elif parsed_path.path == '/messages':
                self._handle_get_messages()
            elif parsed_path.path == '/status':
                self._handle_get_status()
            else:
                self._send_404()
                
        except Exception as e:
            logger.error(f"Error handling GET request: {e}")
            self._send_error(500, str(e))
    
    def do_POST(self):
        """Handle POST requests from Warp terminal"""
        try:
            content_length = int(self.headers.get('Content-Length', 0))
            post_data = self.rfile.read(content_length).decode('utf-8')
            
            parsed_path = urlparse(self.path)
            
            if parsed_path.path == '/send':
                self._handle_send_message(post_data)
            elif parsed_path.path == '/execute':
                self._handle_execute_command(post_data)
            else:
                self._send_404()
                
        except Exception as e:
            logger.error(f"Error handling POST request: {e}")
            self._send_error(500, str(e))
    
    def _send_health_check(self):
        """Send health check response"""
        response = {
            "status": "ok",
            "agent_id": AGENT_ID,
            "broker": f"{BROKER_HOST}:{BROKER_PORT}",
            "timestamp": int(time.time())
        }
        self._send_json_response(200, response)
    
    def _handle_get_messages(self):
        """Get messages from broker"""
        try:
            # Use the chat.sh script to get messages
            result = subprocess.run([
                '/home/alexa/chat.sh', 'read', 'general'
            ], capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0:
                messages = self._parse_chat_output(result.stdout)
                response = {
                    "messages": messages,
                    "status": "success"
                }
                self._send_json_response(200, response)
            else:
                logger.error(f"Chat script failed: {result.stderr}")
                self._send_error(500, "Failed to retrieve messages")
                
        except subprocess.TimeoutExpired:
            logger.error("Chat script timeout")
            self._send_error(504, "Timeout retrieving messages")
        except Exception as e:
            logger.error(f"Error getting messages: {e}")
            self._send_error(500, str(e))
    
    def _handle_send_message(self, post_data):
        """Send message via broker"""
        try:
            data = json.loads(post_data)
            message = data.get('message', '')
            channel = data.get('channel', 'general')
            
            # Use the chat.sh script to send message  
            result = subprocess.run([
                '/home/alexa/chat.sh', 'send', channel, message
            ], capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0:
                response = {
                    "status": "sent",
                    "message_id": int(time.time()),
                    "channel": channel
                }
                self._send_json_response(200, response)
            else:
                logger.error(f"Failed to send message: {result.stderr}")
                self._send_error(500, "Failed to send message")
                
        except json.JSONDecodeError:
            self._send_error(400, "Invalid JSON")
        except subprocess.TimeoutExpired:
            logger.error("Chat script timeout")
            self._send_error(504, "Timeout sending message")
        except Exception as e:
            logger.error(f"Error sending message: {e}")
            self._send_error(500, str(e))
    
    def _handle_execute_command(self, post_data):
        """Execute command and return result"""
        try:
            data = json.loads(post_data)
            command = data.get('command', '')
            
            if not command:
                self._send_error(400, "No command provided")
                return
            
            # For security, only allow specific commands
            allowed_commands = ['ls', 'pwd', 'whoami', 'date', 'uptime']
            cmd_parts = command.strip().split()
            
            if not cmd_parts or cmd_parts[0] not in allowed_commands:
                self._send_error(403, "Command not allowed")
                return
            
            result = subprocess.run(
                command, shell=True, capture_output=True, text=True, timeout=30
            )
            
            response = {
                "command": command,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "return_code": result.returncode,
                "status": "completed"
            }
            self._send_json_response(200, response)
            
        except json.JSONDecodeError:
            self._send_error(400, "Invalid JSON")
        except subprocess.TimeoutExpired:
            self._send_error(504, "Command timeout")
        except Exception as e:
            logger.error(f"Error executing command: {e}")
            self._send_error(500, str(e))
    
    def _handle_get_status(self):
        """Get broker and system status"""
        try:
            # Check broker status
            result = subprocess.run([
                '/home/alexa/chat.sh', 'status'
            ], capture_output=True, text=True, timeout=10)
            
            broker_status = "online" if result.returncode == 0 else "offline"
            
            response = {
                "agent_id": AGENT_ID,
                "broker_status": broker_status,
                "local_time": int(time.time()),
                "uptime": self._get_uptime()
            }
            self._send_json_response(200, response)
            
        except Exception as e:
            logger.error(f"Error getting status: {e}")
            self._send_error(500, str(e))
    
    def _parse_chat_output(self, output):
        """Parse chat.sh output into structured messages"""
        messages = []
        for line in output.strip().split('\n'):
            if line.strip():
                # Basic parsing - adjust based on actual chat.sh output format
                parts = line.split(':', 2)
                if len(parts) >= 2:
                    messages.append({
                        "timestamp": parts[0].strip(),
                        "sender": parts[1].strip() if len(parts) > 2 else "system",
                        "content": parts[-1].strip()
                    })
        return messages
    
    def _get_uptime(self):
        """Get system uptime"""
        try:
            with open('/proc/uptime', 'r') as f:
                return float(f.read().split()[0])
        except:
            return 0
    
    def _send_json_response(self, status_code, data):
        """Send JSON response"""
        self.send_response(status_code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode('utf-8'))
    
    def _send_error(self, status_code, message):
        """Send error response"""
        response = {
            "error": message,
            "status_code": status_code,
            "timestamp": int(time.time())
        }
        self._send_json_response(status_code, response)
    
    def _send_404(self):
        """Send 404 response"""
        self._send_error(404, "Not Found")

def check_broker_connectivity():
    """Check if broker is reachable"""
    try:
        response = requests.get(f"http://{BROKER_HOST}:{BROKER_PORT}/health", timeout=5)
        return response.status_code == 200
    except:
        return False

def message_polling_thread():
    """Background thread for polling messages"""
    global last_message_count
    logger.info("📡 Message polling started")
    
    while polling_active:
        try:
            # Simulate retrieving messages
            result = subprocess.run([
                '/home/alexa/chat.sh', 'read', 'general'
            ], capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0:
                messages = result.stdout.strip().split('\n')
                current_count = len(messages)

                if current_count > last_message_count:
                    new_messages = messages[last_message_count:current_count]
                    last_message_count = current_count
                    new_message_notifications.extend(new_messages)
                    logger.info(f"New messages from Garuda: {new_messages}")

            time.sleep(POLL_INTERVAL)
        except subprocess.TimeoutExpired:
            logger.warning("Polling timeout")
        except Exception as e:
            logger.error(f"Polling failed: {e}")



def main():
    """Main daemon function"""
    logger.info(f"Starting Warp Agent Bridge on port {LOCAL_PORT}")
    logger.info(f"Broker: {BROKER_HOST}:{BROKER_PORT}")
    logger.info(f"Agent ID: {AGENT_ID}")
    
    # Check broker connectivity
    if check_broker_connectivity():
        logger.info("✅ Broker connectivity confirmed")
    else:
        logger.warning("⚠️  Broker not reachable - will use chat.sh fallback")
    
    # Start message polling
    global polling_active, polling_thread
    polling_active = True
    polling_thread = threading.Thread(target=message_polling_thread)
    polling_thread.start()

    # Start HTTP server
    try:
        server = HTTPServer(('127.0.0.1', LOCAL_PORT), WarpBridgeHandler)
        logger.info(f"🚀 Bridge daemon listening on http://127.0.0.1:{LOCAL_PORT}")
        
        # Register with broker if possible
        try:
            subprocess.run([
                '/home/alexa/chat.sh', 'send', 'system', 
                f'Warp bridge started on {AGENT_ID}:{LOCAL_PORT}'
            ], timeout=5)
        except:
            pass
        
        server.serve_forever()
        
    except KeyboardInterrupt:
        logger.info("🛑 Shutting down bridge daemon")
        sys.exit(0)
    except Exception as e:
        logger.error(f"❌ Failed to start server: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
