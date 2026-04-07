#!/usr/bin/env python3
"""
API Masking Proxy Server
Masks API requests to prevent tracking by Warp Terminal or other monitoring
"""

import asyncio
import aiohttp
import random
import time
import hashlib
import json
import re
from urllib.parse import urlparse, parse_qs, urlencode
from aiohttp import web, ClientTimeout
from datetime import datetime, timedelta
import base64
import uuid

class APIMaskingProxy:
    def __init__(self, port=8080):
        self.port = port
        self.session_pool = []
        self.user_agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/121.0"
        ]
        
        # API endpoint mappings for different services
        self.api_routes = {
            'openai': ['api.openai.com', 'chat.openai.com'],
            'anthropic': ['api.anthropic.com', 'claude.ai'], 
            'google': ['generativelanguage.googleapis.com', 'ai.google.dev'],
            'mistral': ['api.mistral.ai'],
            'cohere': ['api.cohere.ai']
        }
        
        self.request_cache = {}
        self.last_request_time = {}
        
    def get_random_user_agent(self):
        """Return random user agent to avoid fingerprinting"""
        return random.choice(self.user_agents)
    
    def add_request_jitter(self):
        """Add random delay to break timing patterns"""
        delay = random.uniform(0.1, 2.0)  # 100ms to 2s jitter
        return delay
    
    def obfuscate_headers(self, headers, target_domain):
        """Modify headers to avoid detection"""
        masked_headers = {}
        
        # Always randomize user agent
        masked_headers['User-Agent'] = self.get_random_user_agent()
        
        # Add realistic browser headers
        masked_headers['Accept'] = 'application/json, text/plain, */*'
        masked_headers['Accept-Language'] = random.choice([
            'en-US,en;q=0.9', 'en-GB,en;q=0.9', 'en-CA,en;q=0.9'
        ])
        masked_headers['Accept-Encoding'] = 'gzip, deflate, br'
        masked_headers['DNT'] = '1'
        masked_headers['Connection'] = 'keep-alive'
        masked_headers['Sec-Fetch-Dest'] = 'empty'
        masked_headers['Sec-Fetch-Mode'] = 'cors'
        masked_headers['Sec-Fetch-Site'] = 'same-origin'
        
        # Preserve essential API headers but modify others
        essential_headers = ['authorization', 'content-type', 'anthropic-version', 'openai-organization']
        
        for key, value in headers.items():
            lower_key = key.lower()
            if lower_key in essential_headers:
                masked_headers[key] = value
            elif lower_key == 'referer':
                # Mask referer to appear as coming from the API site
                masked_headers['Referer'] = f"https://{target_domain}/"
            elif lower_key not in ['user-agent', 'host']:
                masked_headers[key] = value
                
        # Add random additional headers to vary fingerprint
        if random.random() < 0.3:  # 30% chance
            masked_headers['X-Request-ID'] = str(uuid.uuid4())
            
        return masked_headers
    
    def obfuscate_payload(self, payload, api_type):
        """Modify request payload to break content patterns"""
        if not payload:
            return payload
            
        try:
            data = json.loads(payload) if isinstance(payload, str) else payload
            
            # Add subtle variations to break pattern matching
            if api_type == 'openai' and 'messages' in data:
                # Add invisible variations to prompts
                for message in data.get('messages', []):
                    if 'content' in message and isinstance(message['content'], str):
                        # Add zero-width characters occasionally
                        if random.random() < 0.1:  # 10% chance
                            message['content'] += '\u200b'  # Zero-width space
                            
            elif api_type == 'anthropic' and 'messages' in data:
                # Similar obfuscation for Claude
                for message in data.get('messages', []):
                    if 'content' in message and isinstance(message['content'], str):
                        if random.random() < 0.1:
                            message['content'] += '\u200b'
            
            # Randomize temperature slightly
            if 'temperature' in data and random.random() < 0.2:  # 20% chance
                orig_temp = data['temperature']
                variation = random.uniform(-0.05, 0.05)
                data['temperature'] = max(0, min(2.0, orig_temp + variation))
            
            return json.dumps(data)
            
        except (json.JSONDecodeError, TypeError):
            return payload
    
    def detect_api_type(self, url):
        """Detect which API service is being called"""
        domain = urlparse(url).netloc.lower()
        
        for api_type, domains in self.api_routes.items():
            if any(d in domain for d in domains):
                return api_type
        return 'unknown'
    
    async def proxy_request(self, request):
        """Main proxy handler with masking"""
        target_url = request.match_info.get('path', '')
        
        if not target_url.startswith('http'):
            return web.Response(text="Invalid URL", status=400)
        
        api_type = self.detect_api_type(target_url)
        
        # Add jitter delay
        await asyncio.sleep(self.add_request_jitter())
        
        try:
            # Read request body
            body = await request.read()
            
            # Obfuscate headers and payload
            masked_headers = self.obfuscate_headers(dict(request.headers), urlparse(target_url).netloc)
            masked_payload = self.obfuscate_payload(body.decode('utf-8') if body else '', api_type)
            
            # Create session with random characteristics
            timeout = ClientTimeout(total=30)
            connector = aiohttp.TCPConnector(
                limit=10,
                limit_per_host=5,
                enable_cleanup_closed=True
            )
            
            async with aiohttp.ClientSession(
                timeout=timeout,
                connector=connector,
                headers={'User-Agent': masked_headers.get('User-Agent')}
            ) as session:
                
                # Make proxied request with masking
                async with session.request(
                    method=request.method,
                    url=target_url,
                    headers=masked_headers,
                    data=masked_payload.encode('utf-8') if masked_payload else None,
                    allow_redirects=False
                ) as response:
                    
                    # Read response
                    response_body = await response.read()
                    
                    # Create response with original headers (filtered)
                    proxy_response = web.Response(
                        body=response_body,
                        status=response.status,
                        reason=response.reason
                    )
                    
                    # Copy safe headers
                    safe_headers = ['content-type', 'content-length', 'content-encoding']
                    for header in safe_headers:
                        if header in response.headers:
                            proxy_response.headers[header] = response.headers[header]
                    
                    return proxy_response
                    
        except Exception as e:
            print(f"Proxy error: {e}")
            return web.Response(text=f"Proxy error: {str(e)}", status=500)
    
    async def handle_direct_api(self, request):
        """Handle direct API calls through proxy paths"""
        service = request.match_info.get('service', '')
        path = request.match_info.get('path', '')
        
        # Map service to actual API URLs
        service_urls = {
            'openai': 'https://api.openai.com',
            'claude': 'https://api.anthropic.com',
            'anthropic': 'https://api.anthropic.com',
            'google': 'https://generativelanguage.googleapis.com'
        }
        
        if service not in service_urls:
            return web.Response(text="Unsupported service", status=400)
        
        target_url = f"{service_urls[service]}/{path.lstrip('/')}"
        
        # Create new request object with target URL
        proxy_request = request.clone()
        proxy_request.match_info = {'path': target_url}
        
        return await self.proxy_request(proxy_request)
    
    def setup_routes(self, app):
        """Setup proxy routes"""
        # Direct proxy route
        app.router.add_route('*', '/proxy/{path:.*}', self.proxy_request)
        
        # Service-specific routes for easier usage
        app.router.add_route('*', '/api/{service}/{path:.*}', self.handle_direct_api)
        
        # Health check
        app.router.add_get('/health', lambda r: web.Response(text="OK"))
        
        # Status endpoint
        app.router.add_get('/status', self.status_handler)
    
    async def status_handler(self, request):
        """Proxy status endpoint"""
        status = {
            "proxy_active": True,
            "port": self.port,
            "supported_apis": list(self.api_routes.keys()),
            "uptime": datetime.now().isoformat()
        }
        return web.json_response(status)
    
    async def start_server(self):
        """Start the proxy server"""
        app = web.Application()
        self.setup_routes(app)
        
        runner = web.AppRunner(app)
        await runner.setup()
        
        site = web.TCPSite(runner, '127.0.0.1', self.port)
        await site.start()
        
        print(f"API Masking Proxy started on http://127.0.0.1:{self.port}")
        print(f"Usage examples:")
        print(f"  OpenAI: http://127.0.0.1:{self.port}/api/openai/v1/chat/completions")
        print(f"  Claude: http://127.0.0.1:{self.port}/api/claude/v1/messages")
        print(f"  Direct: http://127.0.0.1:{self.port}/proxy/https://api.openai.com/v1/chat/completions")
        
        # Keep server running
        try:
            await asyncio.Future()  # Run forever
        except KeyboardInterrupt:
            await runner.cleanup()

def main():
    proxy = APIMaskingProxy(port=8080)
    asyncio.run(proxy.start_server())

if __name__ == '__main__':
    main()
