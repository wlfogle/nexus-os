// Browser API Masking Script
// Intercepts and masks API requests to prevent tracking
// Deploy this in browser console or as userscript

(function() {
    'use strict';
    
    console.log('🛡️ API Masking Extension Loaded');
    
    // Configuration
    const config = {
        proxyServer: 'http://127.0.0.1:8080',
        maskingEnabled: true,
        debug: false
    };
    
    // Random values for masking
    const userAgents = [
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    ];
    
    const acceptLanguages = [
        'en-US,en;q=0.9',
        'en-GB,en;q=0.9', 
        'en-CA,en;q=0.9'
    ];
    
    // API endpoint detection
    const apiEndpoints = {
        'api.openai.com': 'openai',
        'chat.openai.com': 'openai',
        'api.anthropic.com': 'anthropic',
        'claude.ai': 'anthropic',
        'generativelanguage.googleapis.com': 'google',
        'api.mistral.ai': 'mistral',
        'api.cohere.ai': 'cohere'
    };
    
    // Store original fetch and XMLHttpRequest
    const originalFetch = window.fetch;
    const originalXHR = window.XMLHttpRequest;
    
    // Random delay generator
    function getRandomDelay() {
        return Math.random() * 2000 + 100; // 100-2100ms
    }
    
    // Generate random request ID
    function generateRequestId() {
        return 'req_' + Math.random().toString(36).substr(2, 16);
    }
    
    // Detect if URL is an AI API
    function isAIAPI(url) {
        try {
            const hostname = new URL(url).hostname;
            return Object.keys(apiEndpoints).some(domain => hostname.includes(domain));
        } catch {
            return false;
        }
    }
    
    // Add subtle content variations
    function addContentVariation(content) {
        if (typeof content !== 'string') return content;
        
        // 10% chance to add zero-width space
        if (Math.random() < 0.1) {
            return content + '\u200b';
        }
        
        return content;
    }
    
    // Obfuscate request payload
    function obfuscatePayload(data, apiType) {
        if (!data) return data;
        
        try {
            const payload = typeof data === 'string' ? JSON.parse(data) : data;
            
            // Add subtle variations based on API type
            if (payload.messages && Array.isArray(payload.messages)) {
                payload.messages.forEach(message => {
                    if (message.content) {
                        message.content = addContentVariation(message.content);
                    }
                });
            }
            
            // Slightly randomize temperature
            if (payload.temperature && Math.random() < 0.2) {
                const variation = (Math.random() - 0.5) * 0.1; // ±0.05
                payload.temperature = Math.max(0, Math.min(2, payload.temperature + variation));
            }
            
            // Add random system fingerprint variation
            if (apiType === 'openai' && Math.random() < 0.3) {
                payload.user = generateRequestId();
            }
            
            return JSON.stringify(payload);
        } catch {
            return data;
        }
    }
    
    // Mask request headers
    function maskHeaders(headers, apiType) {
        const maskedHeaders = new Headers();
        
        // Copy essential headers
        const essentialHeaders = [
            'authorization', 'content-type', 'anthropic-version', 
            'openai-organization', 'x-api-key'
        ];
        
        if (headers instanceof Headers) {
            headers.forEach((value, key) => {
                if (essentialHeaders.some(h => key.toLowerCase().includes(h))) {
                    maskedHeaders.set(key, value);
                }
            });
        } else if (typeof headers === 'object') {
            Object.entries(headers).forEach(([key, value]) => {
                if (essentialHeaders.some(h => key.toLowerCase().includes(h))) {
                    maskedHeaders.set(key, value);
                }
            });
        }
        
        // Add randomized browser headers
        maskedHeaders.set('User-Agent', userAgents[Math.floor(Math.random() * userAgents.length)]);
        maskedHeaders.set('Accept', 'application/json, text/plain, */*');
        maskedHeaders.set('Accept-Language', acceptLanguages[Math.floor(Math.random() * acceptLanguages.length)]);
        maskedHeaders.set('Accept-Encoding', 'gzip, deflate, br');
        maskedHeaders.set('DNT', '1');
        maskedHeaders.set('Sec-Fetch-Dest', 'empty');
        maskedHeaders.set('Sec-Fetch-Mode', 'cors');
        maskedHeaders.set('Sec-Fetch-Site', 'same-origin');
        
        // Add random request ID
        if (Math.random() < 0.3) {
            maskedHeaders.set('X-Request-ID', generateRequestId());
        }
        
        return maskedHeaders;
    }
    
    // Enhanced fetch with masking
    async function maskedFetch(url, options = {}) {
        if (!config.maskingEnabled || !isAIAPI(url)) {
            return originalFetch(url, options);
        }
        
        if (config.debug) {
            console.log('🎭 Masking API request:', url);
        }
        
        // Add random delay to break timing patterns
        await new Promise(resolve => setTimeout(resolve, getRandomDelay()));
        
        // Detect API type
        const hostname = new URL(url).hostname;
        const apiType = Object.entries(apiEndpoints).find(([domain]) => 
            hostname.includes(domain)
        )?.[1] || 'unknown';
        
        // Prepare masked options
        const maskedOptions = { ...options };
        
        // Mask headers
        maskedOptions.headers = maskHeaders(options.headers || {}, apiType);
        
        // Mask payload
        if (maskedOptions.body) {
            maskedOptions.body = obfuscatePayload(maskedOptions.body, apiType);
        }
        
        // Route through proxy if configured
        if (config.proxyServer) {
            const proxyUrl = `${config.proxyServer}/proxy/${encodeURIComponent(url)}`;
            return originalFetch(proxyUrl, maskedOptions);
        }
        
        return originalFetch(url, maskedOptions);
    }
    
    // Enhanced XMLHttpRequest with masking
    function MaskedXMLHttpRequest() {
        const xhr = new originalXHR();
        const originalOpen = xhr.open;
        const originalSend = xhr.send;
        const originalSetRequestHeader = xhr.setRequestHeader;
        
        let requestUrl = '';
        let requestMethod = '';
        let requestHeaders = {};
        
        xhr.open = function(method, url, ...args) {
            requestUrl = url;
            requestMethod = method;
            return originalOpen.call(this, method, url, ...args);
        };
        
        xhr.setRequestHeader = function(header, value) {
            requestHeaders[header] = value;
            return originalSetRequestHeader.call(this, header, value);
        };
        
        xhr.send = function(data) {
            if (!config.maskingEnabled || !isAIAPI(requestUrl)) {
                return originalSend.call(this, data);
            }
            
            if (config.debug) {
                console.log('🎭 Masking XHR request:', requestUrl);
            }
            
            // Add delay
            setTimeout(() => {
                // Detect API type
                const hostname = new URL(requestUrl).hostname;
                const apiType = Object.entries(apiEndpoints).find(([domain]) => 
                    hostname.includes(domain)
                )?.[1] || 'unknown';
                
                // Clear existing headers and set masked ones
                const maskedHeaders = maskHeaders(requestHeaders, apiType);
                maskedHeaders.forEach((value, key) => {
                    originalSetRequestHeader.call(xhr, key, value);
                });
                
                // Mask data
                const maskedData = data ? obfuscatePayload(data, apiType) : data;
                
                // Route through proxy if configured
                if (config.proxyServer) {
                    const proxyUrl = `${config.proxyServer}/proxy/${encodeURIComponent(requestUrl)}`;
                    originalOpen.call(xhr, requestMethod, proxyUrl);
                }
                
                originalSend.call(xhr, maskedData);
            }, getRandomDelay());
        };
        
        return xhr;
    }
    
    // Replace global functions
    window.fetch = maskedFetch;
    window.XMLHttpRequest = MaskedXMLHttpRequest;
    
    // Additional masking for WebSocket connections
    const originalWebSocket = window.WebSocket;
    window.WebSocket = function(url, protocols) {
        if (isAIAPI(url)) {
            console.log('🎭 WebSocket API connection detected:', url);
            // Add delay for WebSocket connections too
            setTimeout(() => {
                return new originalWebSocket(url, protocols);
            }, getRandomDelay());
        }
        return new originalWebSocket(url, protocols);
    };
    
    // Browser fingerprinting protection
    Object.defineProperty(navigator, 'userAgent', {
        get: function() {
            return userAgents[Math.floor(Math.random() * userAgents.length)];
        }
    });
    
    // Mask timing attacks
    const originalNow = performance.now;
    performance.now = function() {
        return originalNow.call(this) + (Math.random() - 0.5) * 10; // ±5ms jitter
    };
    
    // API for controlling masking
    window.APIMask = {
        enable: () => { config.maskingEnabled = true; console.log('🛡️ API Masking enabled'); },
        disable: () => { config.maskingEnabled = false; console.log('🛡️ API Masking disabled'); },
        setProxy: (url) => { config.proxyServer = url; console.log('🛡️ Proxy set to:', url); },
        debug: (enable) => { config.debug = enable; console.log('🛡️ Debug mode:', enable); },
        status: () => { console.log('🛡️ API Masking status:', config); }
    };
    
    console.log('🛡️ API Masking ready. Use APIMask.status() to check configuration.');
    
})();
