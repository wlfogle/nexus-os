# 🚀 AI & Home Automation Integration Guide

## 🏠 Home Assistant (VM 500)

### **Status: ✅ Online & Accessible**

- **IP Address**: `192.168.122.52`
- **Port**: `8123` (or other detected port)
- **VM Configuration**:
  - 2 cores (host CPU)
  - 2GB RAM

### **Recommendations for Home Assistant:**
1.  **Secure Access with HTTPS**:
    ```yaml
    # configuration.yaml
    http:
      ssl_certificate: /ssl/fullchain.pem
      ssl_key: /ssl/privkey.pem
    ```
    - Use the Let's Encrypt add-on for free SSL certificates.

2.  **Resource Optimization**:
    - Monitor resource usage in Proxmox and adjust CPU/RAM as needed.
    - Use VirtIO drivers for better disk and network performance.

3.  **Automated Backups**:
    - Use the Home Assistant Google Drive Backup add-on for automated cloud backups.

## 🤖 AI Services (CT 900)

### **Status: ✅ Online & Accessible**

- **IP Address**: `192.168.122.172`
- **Ollama API**: `http://192.168.122.172:11434`
- **Container Configuration**:
  - 4 cores
  - 4GB RAM
  - Python 3 and Ollama installed

### **Recommendations for AI Services:**

1.  **Open-Interpreter Integration**:
    - You can now use Open-Interpreter to interact with your local Ollama models.
    - Example script:
      ```python
      # script.py
      import interpreter
      interpreter.offline = True
      interpreter.llm.model = "ollama/mistral"
      interpreter.llm.api_base = "http://192.168.122.172:11434"
      interpreter.chat("What are the first 5 prime numbers?")
      ```

2.  **Model Management**:
    - Use `ollama list` to see available models.
    - Download new models with `ollama pull <model_name>`.

3.  **GPU Passthrough (for enhanced performance)**:
    - If you have a GPU, consider passing it through to CT 900 for faster AI model inference.
    - This involves editing the LXC configuration file:
      ```
      lxc.cgroup2.devices.allow: c 226:0 rwm
      lxc.mount.entry: /dev/dri dev/dri none bind,optional,create=dir
      ```

## 🔌 **Integration: Home Assistant + Ollama**

You can now integrate your local Ollama instance with Home Assistant for powerful local AI automations!

### **1. Home Assistant Configuration**

Add the following to your `configuration.yaml` in Home Assistant:

```yaml
# configuration.yaml
ollama:
  - name: "Local AI"
    host: 192.168.122.172
    port: 11434

sensor:
  - platform: ollama
    name: "AI Temperature Sensor"
    prompt: "What is the current temperature in my home?"
    model: "mistral"
```

### **2. Example Automation**

Create an automation that uses Ollama to control your smart home:

```yaml
# automations.yaml
- alias: "AI Morning Routine"
  trigger:
    platform: time
    at: "07:00:00"
  action:
    - service: conversation.process
      data:
        agent_id: ollama
        text: "Good morning! Turn on the lights and start the coffee maker."
```

## 🔒 **Security Recommendations**

- **Network Segmentation**: Keep your AI and Home Automation services on a separate VLAN if possible.
- **Firewall Rules**: Use Proxmox firewall to restrict access to these services.
- **Regular Updates**: Keep all software (Proxmox, Home Assistant, Ollama) updated.

## 📊 **Monitoring**

- Use Prometheus and Grafana (CT 260/261) to monitor the resource usage of both CT 900 and VM 500.
- Add scrape configs to your Prometheus configuration:

```yaml
# prometheus.yml
- job_name: 'home-assistant'
  static_configs:
    - targets: ['192.168.122.52:8123']
  metrics_path: /api/prometheus

- job_name: 'ollama'
  static_configs:
    - targets: ['192.168.122.172:11434']
```

---
*Last Updated: July 30, 2025*
*All services are running and accessible for integration.*

