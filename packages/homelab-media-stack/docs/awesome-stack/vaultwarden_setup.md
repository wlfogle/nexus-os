# Vaultwarden Setup Documentation

## Domain Configuration

Setup for Vaultwarden running with Traefik:

- **Domain**: `vaultwarden.lou-fogle-media-stack.duckdns.org`
- **SSL/TLS**: Enabled via Let's Encrypt in Traefik

## Traefik Configuration

Traefik configuration file located at `/etc/traefik/dynamic/vaultwarden.yml`:

```yaml
http:
  routers:
    vaultwarden:
      rule: "Host(`vaultwarden.lou-fogle-media-stack.duckdns.org`)"
      service: vaultwarden
      tls:
        certResolver: letsencrypt
      
  services:
    vaultwarden:
      loadBalancer:
        servers:
          - url: "http://192.168.122.104:8080"
```

## Vaultwarden Service

- **Access URL**: `https://vaultwarden.lou-fogle-media-stack.duckdns.org`
- **Secure Context**: HTTPS provides Subtle Crypto API for cryptographic functions

## Desktop Access

### Via External Domain (Original Setup)
1. Open FireDragon browser
2. Navigate to `https://vaultwarden.lou-fogle-media-stack.duckdns.org`
3. Accept any security exceptions if prompted
4. Log in with your Vaultwarden credentials

### Via Local HTTPS Proxy (Current Setup)
1. HTTPS proxy running on: `https://localhost:8443`
2. Proxy forwards to Vaultwarden container: `192.168.122.104:8080`
3. FireDragon browser extension configured to use local proxy
4. Self-signed certificate provides secure context for WebCrypto API

## Mobile Device Access

### Proxy Configuration
- **HTTPS Proxy**: `https://192.168.12.204:8444` (for Chrome/Samsung Internet)
- **HTTP Proxy**: `http://192.168.12.204:8080` (fallback for certificate issues)
- **Target Server**: `192.168.122.104:8080` (Vaultwarden container)

### Samsung Galaxy Note 9 (SM-N960U1)
**Device ID**: `2aaaca00251c7ece`

**Working Browsers:**
- ✅ **Chrome Mobile**: HTTPS proxy at `https://192.168.12.204:8444`
- ✅ **Bitwarden App**: Available for autofill
- ❌ **Opera/Firefox**: Spinning circle issues (browser compatibility)

**Setup Commands:**
```bash
# Clear Chrome data
adb -s 2aaaca00251c7ece shell pm clear com.android.chrome

# Open Chrome with HTTPS proxy
adb -s 2aaaca00251c7ece shell am start -a android.intent.action.VIEW -d "https://192.168.12.204:8444" com.android.chrome

# Launch Bitwarden app
adb -s 2aaaca00251c7ece shell am start -n com.x8bit.bitwarden/.MainActivity
```

### Samsung Galaxy A23 5G (SM-A236U)
**Device ID**: `R5CW800ME4D`

**Working Browsers:**
- ✅ **Chrome Mobile**: HTTPS proxy at `https://192.168.12.204:8444`
- ✅ **Samsung Internet**: HTTPS proxy at `https://192.168.12.204:8444`
- ✅ **Bitwarden App**: Available for autofill
- ❌ **Opera GX**: Spinning circle issues (browser compatibility)

**Setup Commands:**
```bash
# Clear browser data
adb -s R5CW800ME4D shell pm clear com.android.chrome
adb -s R5CW800ME4D shell pm clear com.sec.android.app.sbrowser

# Open Chrome with HTTPS proxy
adb -s R5CW800ME4D shell am start -a android.intent.action.VIEW -d "https://192.168.12.204:8444" com.android.chrome

# Open Samsung Internet with HTTPS proxy
adb -s R5CW800ME4D shell am start -a android.intent.action.VIEW -d "https://192.168.12.204:8444" com.sec.android.app.sbrowser

# Launch Bitwarden app
adb -s R5CW800ME4D shell am start -n com.x8bit.bitwarden/.MainActivity
```

## Proxy Services

### HTTPS Proxy (Port 8444)
```bash
# Start HTTPS proxy with self-signed certificate
socat openssl-listen:8444,cert=/tmp/mobile-vaultwarden.pem,verify=0,fork tcp:192.168.122.104:8080 &
```

### HTTP Proxy (Port 8080)
```bash
# Start HTTP proxy for fallback
socat tcp-listen:8080,bind=192.168.12.204,fork tcp:192.168.122.104:8080 &
```

## Troubleshooting

### Browser Compatibility Issues
- **Chrome**: ✅ Works reliably with HTTPS proxy
- **Samsung Internet**: ✅ Works reliably with HTTPS proxy  
- **Firefox Mobile**: ❌ Spinning circle issues with WebCrypto API
- **Opera/Opera GX**: ❌ Spinning circle issues with WebCrypto API

### Certificate Issues
- Accept self-signed certificate warnings in browsers
- Use HTTP proxy as fallback if HTTPS fails
- Bitwarden app handles certificates better than browsers

### Data Import
- Imported from Bitwarden export: `/home/lou/Downloads/bitwarden_export_20250804125949.json`
- All passwords and vault data successfully migrated
- Desktop and mobile devices synchronized with same vault

This setup ensures that the WebCrypto API is accessible on both desktop and mobile devices, providing secure password management across all platforms.
