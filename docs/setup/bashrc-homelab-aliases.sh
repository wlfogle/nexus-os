export BUN_INSTALL="$HOME/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"
. "$HOME/.cargo/env"

# ── Homelab aliases ──
# SSH shortcuts — just type the machine name
alias tiamat='ssh tiamat'
alias ziggy='ssh ziggy'
alias ct-media='ssh ct-media'
alias ct-adguard='ssh ct-adguard'
alias ct-wg='ssh ct-wg'
alias ct-proxy='ssh ct-proxy'
alias ct-firetv='ssh ct-firetv'

# lou-laptop — show this machine's identity + IP
alias lou-laptop='echo "lou-laptop | $(hostname) | $(hostname -I | awk "{print \$1}") | Pop!_OS 22.04 | i9-13900HX 62.5GB RTX4080"'

# Quick homelab ops
alias pct-list='ssh tiamat pct list'
alias stack-ps='ssh ct-media docker ps'
alias stack-logs='ssh ct-media docker compose -f /opt/homelab-media-stack/media-stack/docker-compose.yml logs --tail 50'
alias stack-restart='ssh ct-media "cd /opt/homelab-media-stack/media-stack && docker compose restart"'
alias stack-update='ssh ct-media "cd /opt/homelab-media-stack/media-stack && docker compose pull && docker compose up -d"'
alias adguard-logs='ssh ct-adguard docker logs adguardhome --tail 50'
alias wg-status='ssh ct-wg wg show'
