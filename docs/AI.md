# AI Services (Laptop RTX 4080 + CT-900)
Architecture:
- Laptop runs Ollama with RTX 4080 acceleration.
- CT-900 runs Open WebUI (frontend) and SearXNG (web search backend).
## Laptop: Ollama
Install and run Ollama listening on LAN:
```bash
curl -fsSL https://ollama.com/install.sh | sh
sudo systemctl enable --now ollama
```
Set host binding for LAN access:
```bash
sudo systemctl edit ollama
```
Drop-in:
```ini
[Service]
Environment="OLLAMA_HOST=0.0.0.0:11434"
```
Restart:
```bash
sudo systemctl daemon-reload
sudo systemctl restart ollama
```
Verify:
```bash
curl http://127.0.0.1:11434/api/tags
nvidia-smi
```
## CT-900: Open WebUI + SearXNG
`media-stack/docker-compose.yml` includes:
- `open-webui` on port `3000`
- `searxng` on port `8081`
Env values:
```bash
OPENWEBUI_PORT=3000
SEARXNG_PORT=8081
OLLAMA_HOST=http://192.168.12.172:11434
```
## Open WebUI Configuration
1. Open WebUI admin settings.
2. Confirm Ollama endpoint matches `OLLAMA_HOST`.
3. Enable web search and set SearXNG endpoint.
## Recommended Models for 12GB VRAM
- `llama3.1:8b`
- `qwen2.5:7b`
- `mistral:7b`
Use quantized variants for better performance.
## Notes
- If laptop IP changes, update `OLLAMA_HOST`.
- Keep model cache on fast storage.
