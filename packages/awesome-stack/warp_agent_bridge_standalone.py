#!/usr/bin/env python3
"""
Warp Agent Bridge - Standalone version without external dependencies
Provides HTTP API to communicate with Warp terminal agent
"""

import http.server
import socketserver
import json
import urllib.request
import urllib.parse
import urllib.error
import threading
import logging
import sys
import os
from datetime import datetime

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/tmp/warp_bridge.log'),
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger(__name__)

class WarpAgentBridge:
    def __init__(self, warp_host='192.168.122.1', warp_port=7777, bridge_port=8080):
        self.warp_host = warp_host
        self.warp_port = warp_port
        self.bridge_port = bridge_port
        self.warp_url = f"http://{warp_host}:{warp_port}"
        
    def send_to_warp(self, endpoint, data=None, method='GET'):
        """Send request to Warp agent"""
        try:
            url = f"{self.warp_url}/{endpoint.lstrip('/')}"
            
            if data:
                if isinstance(data, dict):
                    data = json.dumps(data).encode('utf-8')
                elif isinstance(data, str):
                    data = data.encode('utf-8')
                    
                req = urllib.request.Request(url, data=data, method=method)
                req.add_header('Content-Type', 'application/json')
            else:
                req = urllib.request.Request(url, method=method)
                
            with urllib.request.urlopen(req, timeout=10) as response:
                content = response.read().decode('utf-8')
                return {
                    'success': True,
                    'status_code': response.status,
                    'data': content
                }
                
        except urllib.error.HTTPError as e:
            error_msg = e.read().decode('utf-8') if e.fp else str(e)
            logger.error(f"HTTP Error {e.code}: {error_msg}")
            return {
                'success': False,
                'error': f"HTTP {e.code}: {error_msg}",
                'status_code': e.code
            }
        except urllib.error.URLError as e:
            logger.error(f"URL Error: {e}")
            return {
                'success': False,
                'error': f"Connection failed: {e}",
                'status_code': 0
            }
        except Exception as e:
            logger.error(f"Unexpected error: {e}")
            return {
                'success': False,
                'error': str(e),
                'status_code': 0
            }

class BridgeRequestHandler(http.server.BaseHTTPRequestHandler):
    def __init__(self, *args, bridge=None, **kwargs):
        self.bridge = bridge
        super().__init__(*args, **kwargs)
        
    def log_message(self, format, *args):
        """Override to use our logger"""
        logger.info(f"{self.address_string()} - {format % args}")
        
    def do_GET(self):
        """Handle GET requests"""
        if self.path == '/health':
            self._send_response(200, {'status': 'healthy', 'timestamp': datetime.now().isoformat()})
        elif self.path == '/status':
            # Test connection to Warp agent
            result = self.bridge.send_to_warp('health')
            if result['success']:
                self._send_response(200, {'bridge': 'healthy', 'warp_agent': 'connected'})
            else:
                self._send_response(503, {'bridge': 'healthy', 'warp_agent': 'disconnected', 'error': result['error']})
        else:
            # Forward to Warp agent
            result = self.bridge.send_to_warp(self.path.lstrip('/'))
            status = result.get('status_code', 500)
            if result['success']:
                try:
                    data = json.loads(result['data']) if result['data'] else {}
                except json.JSONDecodeError:
                    data = {'raw_response': result['data']}
                self._send_response(status, data)
            else:
                self._send_response(status or 500, {'error': result['error']})
                
    def do_POST(self):
        """Handle POST requests"""
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length).decode('utf-8') if content_length > 0 else None
        
        result = self.bridge.send_to_warp(self.path.lstrip('/'), post_data, 'POST')
        status = result.get('status_code', 500)
        
        if result['success']:
            try:
                data = json.loads(result['data']) if result['data'] else {}
            except json.JSONDecodeError:
                data = {'raw_response': result['data']}
            self._send_response(status, data)
        else:
            self._send_response(status or 500, {'error': result['error']})
            
    def _send_response(self, status_code, data):
        """Send JSON response"""
        self.send_response(status_code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()
        
        response = json.dumps(data, indent=2)
        self.wfile.write(response.encode('utf-8'))

def create_handler(bridge):
    """Create request handler with bridge instance"""
    def handler(*args, **kwargs):
        return BridgeRequestHandler(*args, bridge=bridge, **kwargs)
    return handler

def main():
    # Parse command line arguments
    warp_host = sys.argv[1] if len(sys.argv) > 1 else '192.168.122.1'
    warp_port = int(sys.argv[2]) if len(sys.argv) > 2 else 7777
    bridge_port = int(sys.argv[3]) if len(sys.argv) > 3 else 8080
    
    logger.info(f"Starting Warp Agent Bridge...")
    logger.info(f"Warp Agent: {warp_host}:{warp_port}")
    logger.info(f"Bridge Port: {bridge_port}")
    
    # Create bridge instance
    bridge = WarpAgentBridge(warp_host, warp_port, bridge_port)
    
    # Test connection to Warp agent
    logger.info("Testing connection to Warp agent...")
    test_result = bridge.send_to_warp('health')
    if test_result['success']:
        logger.info("✅ Warp agent connection successful")
    else:
        logger.warning(f"⚠️  Warp agent connection failed: {test_result['error']}")
        logger.info("Bridge will start anyway and retry connections...")
    
    # Start HTTP server
    handler = create_handler(bridge)
    httpd = socketserver.TCPServer(("", bridge_port), handler)
    httpd.allow_reuse_address = True
    
    logger.info(f"🚀 Warp Agent Bridge started on port {bridge_port}")
    logger.info("Available endpoints:")
    logger.info("  GET  /health  - Bridge health check")
    logger.info("  GET  /status  - Bridge and Warp agent status")
    logger.info("  GET  /*       - Forward to Warp agent")
    logger.info("  POST /*       - Forward to Warp agent")
    
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        logger.info("Shutting down Warp Agent Bridge...")
        httpd.shutdown()
        httpd.server_close()

if __name__ == "__main__":
    main()
