

# 📄 **Project Documentation: Local Alexa Skill for Voice Control**

---

## **1. Goal & Architecture**

**Goal:** To establish a secure, self-hosted connection between the Amazon Alexa ecosystem and the Home Assistant instance (`192.168.122.113`) without relying on the Nabu Casa cloud subscription. This enables the Echo Frames and other Alexa devices to control the 47+ container homelab stack via custom voice commands processed locally.

**Updated Architecture (Using Tailscale):**
```
Your Voice → [Echo Frames] → [Amazon Alexa Cloud]
                                    ↓ (HTTPS/TLS)
[Internet] → [Tailscale Network] → [lou-eon17x (100.96.98.61)]
                                    ↓ (Subnet Route: 192.168.122.0/24)
            [Traefik Reverse Proxy (CT 103)] → [Home Assistant (VM 500)]
```

**Network Details:**
- **Tailscale Device**: `lou-eon17x` (IP: `100.96.98.61`)
- **Homelab Network**: `192.168.122.0/24` (advertised via Tailscale subnet route)
- **Home Assistant**: `192.168.122.113:8123`
- **Traefik**: `192.168.122.103:8080/8443`

---

## **2. Implementation Plan & Steps**

### **Step 1: Tailscale Network Setup** ✅ **COMPLETED SUCCESSFULLY**

**Implementation Summary:**
- ✅ Tailscale installed on `lou-eon17x` (IP: `100.96.98.61`)
- ✅ Subnet route `192.168.122.0/24` advertised and **APPROVED**
- ✅ IP forwarding enabled on Garuda host
- ✅ Network connectivity tested and verified

**Technical Details:**
```bash
# Installation command used:
sudo pacman -S tailscale --noconfirm
sudo systemctl enable --now tailscaled
sudo tailscale up --advertise-routes=192.168.122.0/24 --accept-routes

# IP forwarding configuration:
echo 'net.ipv4.ip_forward = 1' | sudo tee /etc/sysctl.d/99-tailscale.conf
echo 'net.ipv6.conf.all.forwarding = 1' | sudo tee -a /etc/sysctl.d/99-tailscale.conf
sudo sysctl -p /etc/sysctl.d/99-tailscale.conf
```

**Connectivity Tests:**
- ✅ Home Assistant accessible: `http://192.168.122.113:8123` (HTTP 200)
- ✅ Tailscale status: `100.96.98.61 lou-eon17x loufogle@ linux -`
- ✅ Subnet route approved in Tailscale admin console

**Result:** Your entire homelab network (`192.168.122.0/24`) is now securely accessible through the Tailscale network!

### **Step 2: Dynamic DNS with DuckDNS**
1.  **Create a DuckDNS Account:** Go to `https://www.duckdns.org/` and sign in with a provider.
2.  **Create a Domain:** Create a unique subdomain (e.g., `your-homelab-name.duckdns.org`).
3.  **Get Token:** Note down your DuckDNS token from the dashboard.
4.  **Set up Auto-Update:**
    *   Install the official **Duck DNS add-on** in Home Assistant from the Add-on Store.
    *   Configure the add-on with your new domain and token.
    *   This ensures your domain always points to your home's public IP address.

### **Step 2: SSL Certificate with Let's Encrypt**
1.  **Use Traefik:** Your existing Traefik setup (CT 103) is already configured to manage Let's Encrypt certificates automatically for your services.
2.  **Update Traefik Configuration:**
    *   Add a new router rule in your Traefik dynamic configuration file (e.g., `routers.yml`).
    *   This rule will route traffic from `your-homelab-name.duckdns.org` to the Home Assistant IP (`192.168.122.113:8123`).
    *   Ensure the `tls.certresolver` is set to your existing Let's Encrypt resolver. Traefik will handle the certificate procurement and renewal.

### **Step 3: Home Assistant Configuration**
1.  **Configure `http` Integration:** In your `configuration.yaml`, ensure the `http` section includes your Traefik reverse proxy's IP address in the `trusted_proxies` list. This is critical for security.
    ```yaml
    http:
      use_x_forwarded_for: true
      trusted_proxies:
        - 192.168.122.103  # IP of your Traefik container
    ```
2.  **Add Alexa Integration:**
    *   Go to **Settings > Integrations > Add Integration**.
    *   Search for and add **"Amazon Alexa"**.

### **Step 4: Amazon Developer Console & Skill Setup**
1.  **Create an Amazon Developer Account:** Sign up at `https://developer.amazon.com/`.
2.  **Create a New Alexa Skill:**
    *   Go to the Alexa Developer Console and click "Create Skill".
    *   **Name:** "Home Assistant" or similar.
    *   **Model:** "Smart Home".
3.  **Configure the Skill:**
    *   **Skill ID:** Note this down. You will need it in Home Assistant.
    *   **Default Endpoint:** This is the most critical part. Enter the HTTPS URL of your Home Assistant instance: `https://your-homelab-name.duckdns.org/api/alexa/smart_home`
    *   **Account Linking:**
        *   **Authorization URI:** `https://your-homelab-name.duckdns.org/auth/authorize`
        *   **Access Token URI:** `https://your-homelab-name.duckdns.org/auth/token`
        *   **Client ID:** `https://pitangui.amazon.com/` (for skills in the US) or the appropriate URL for your region.
        *   **Client Secret:** Create a long, random string.
        *   **Scope:** `smart_home`
4.  **Finalize Skill & Connect:**
    *   Link the skill to your personal Amazon account.
    *   In Home Assistant, add the Alexa Skill ID and the Client Secret you created.
    *   Discover devices by saying "Alexa, discover devices."

---

---

## **3. Echo Frames Integration Options**

With the Tailscale network successfully established, there are now **three viable approaches** for integrating your Echo Frames:

### **Option A: Direct Network Integration** ⭐ **RECOMMENDED**

**Concept:** Install Tailscale on the Echo Frames (if supported) to connect them directly to your homelab network.

**Advantages:**
- ✅ **Simplest setup** - No complex OAuth or Alexa skill development
- ✅ **Direct access** - Echo Frames become part of your homelab network
- ✅ **Low latency** - No routing through Amazon's servers
- ✅ **Full control** - All communication stays within your network
- ✅ **No external dependencies** - Works even if internet is down

**Implementation:**
1. Check if Echo Frames support Tailscale app installation
2. If yes, install Tailscale and connect to `loufogle@gmail.com` tailnet
3. Echo Frames get direct access to `192.168.122.113:8123` (Home Assistant)
4. Configure voice commands directly in Home Assistant

### **Option B: Custom Alexa Skill** (Original Plan)

**Concept:** Create a custom Alexa skill that uses your Tailscale-accessible Home Assistant endpoint.

**Advantages:**
- ✅ **Works with any Alexa device** - Not limited to Tailscale-compatible devices
- ✅ **Familiar Alexa experience** - Uses standard "Alexa, ..." commands
- ✅ **Amazon ecosystem integration** - Works with Alexa routines, etc.

**Implementation:**
- Continue with Steps 2-4 from the original plan
- Use your Tailscale network to provide secure HTTPS access
- Requires DuckDNS, SSL certificates, and Amazon Developer Console setup

### **Option C: IFTTT Integration** 🚀 **EASIEST**

**Concept:** Use IFTTT (If This Then That) to create simple applets that connect Alexa voice commands to Home Assistant actions.

**Advantages:**
- ✅ **Easiest setup** - No custom skill development or complex OAuth
- ✅ **Quick to implement** - Create applets in minutes
- ✅ **Works with any Alexa device** - Including Echo Frames
- ✅ **Visual interface** - Easy to manage and modify commands
- ✅ **Built-in Alexa integration** - Uses existing Alexa service

**Disadvantages:**
- ⚠️ **Cloud dependency** - Requires internet connection
- ⚠️ **Limited complexity** - Simple "if this, then that" logic only
- ⚠️ **Third-party service** - Relies on IFTTT's availability

**Implementation Steps:**
1. **Create IFTTT Account:** Sign up at [ifttt.com](https://ifttt.com)
2. **Connect Amazon Alexa Service:** 
   - In IFTTT, search for "Amazon Alexa" service
   - Connect it to your Amazon account (same one used with Echo Frames)
3. **Connect Home Assistant via Webhooks:**
   - Enable the "Webhooks" service in IFTTT
   - Configure Home Assistant to accept webhook calls through your Tailscale network
4. **Create Applets for Your 35+ Voice Commands:**
   - **Trigger:** "Alexa, movie night" → **Action:** Webhook to `http://192.168.122.113:8123/api/webhook/movie_night`
   - **Trigger:** "Alexa, system status" → **Action:** Webhook to `http://192.168.122.113:8123/api/webhook/system_status`
   - Repeat for all your existing Home Assistant scripts
5. **Test with Echo Frames:**
   - Say "Alexa, trigger movie night" to test the integration

**Architecture Flow:**
```
Echo Frames → ["Alexa, trigger X"] → Amazon Alexa Cloud → IFTTT → 
    Webhook → Tailscale Network → Home Assistant (192.168.122.113)
```

---

## **4. Project Status**

**Completed:**
- ✅ **Tailscale Network** - Full homelab access established
- ✅ **Network Testing** - Home Assistant connectivity verified
- ✅ **Documentation** - Complete with three integration options

**Integration Options Summary:**
- **Option A:** Direct Network (Tailscale on Echo Frames) - Most control, requires device compatibility
- **Option B:** Custom Alexa Skill - Most flexibility, complex setup
- **Option C:** IFTTT Integration - Easiest setup, cloud-dependent

**Next Decision:**
- **Choose Integration Approach:** A vs B vs C based on your priorities
- **For Option A:** Research Echo Frames Tailscale compatibility
- **For Option B:** Set up DuckDNS and SSL certificates
- **For Option C:** Create IFTTT account and test webhooks

**Current Recommendation:** Try **Option C (IFTTT)** first for quickest results, then explore **Option A** for maximum control.

