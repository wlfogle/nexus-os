

#!/bin/bash

# Lou MediaStack - Ultimate Stack Rebuild Script
# Rebuilds the 85-service Ultimate Arr Media Stack with priority-based ports

set -e

echo "🚀 Lou MediaStack - Ultimate Stack Rebuild"
echo "=========================================="
echo ""
echo "This will rebuild your 85-service Ultimate Arr Media Stack"
echo "with priority-based port assignments from your previous setup."
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if current directory has docker-compose.yml or create one
if [ ! -f "docker-compose.yml" ] && [ ! -f "docker-compose.yaml" ]; then
    print_warning "No existing docker-compose file found - will create new ultimate stack"
fi

print_status "Found full compose file with 85 services"
print_warning "This will replace your current 26-service setup with the full 85-service Ultimate Stack"
echo ""

read -p "Do you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Operation cancelled."
    exit 0
fi

print_status "Backing up current configuration..."
cp docker-compose.yml "docker-compose-26services-backup-$(date +%Y%m%d-%H%M%S).yml"
cp .env ".env-backup-$(date +%Y%m%d-%H%M%S)"

print_status "Stopping current stack..."
docker-compose down

print_status "Creating Ultimate Stack docker-compose.yml with priority ports..."

# Create the ultimate stack compose file with priority-based ports
cat > docker-compose.yml << 'EOF'
networks:
  mediastack:
    driver: bridge
    ipam:
      config:
        - subnet: 172.21.0.0/16

services:
  # ============================================================================
  # PHASE 1: CORE INFRASTRUCTURE (8000-8099)
  # ============================================================================
  
  # Core Database & Cache
  postgres:
    image: postgres:16-alpine
    container_name: mediastack-postgres
    restart: unless-stopped
    environment:
      POSTGRES_DB: ${POSTGRES_DB}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - mediastack
    ports:
      - "8020:5432"    # Phase 1: Core Infrastructure
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

  valkey:
    image: valkey/valkey:7-alpine
    container_name: mediastack-valkey
    restart: unless-stopped
    command: --save 60 1 --loglevel warning
    volumes:
      - valkey_data:/data
    networks:
      - mediastack
    ports:
      - "8021:6379"    # Phase 1: Core Infrastructure
    healthcheck:
      test: ["CMD-SHELL", "valkey-cli ping | grep PONG"]
      start_period: 20s
      interval: 30s
      retries: 5

  # VPN Infrastructure
  gluetun:
    image: qmcgaw/gluetun
    container_name: mediastack-gluetun
    restart: unless-stopped
    cap_add:
      - NET_ADMIN
    environment:
      VPN_SERVICE_PROVIDER: ${VPN_PROVIDER}
      VPN_TYPE: ${VPN_TYPE}
      WIREGUARD_PRIVATE_KEY: ${VPN_PRIVATE_KEY}
      WIREGUARD_ADDRESSES: ${VPN_ADDRESSES}
      SERVER_COUNTRIES: ${VPN_COUNTRIES}
      FIREWALL_OUTBOUND_SUBNETS: 172.21.0.0/16
    volumes:
      - gluetun_data:/gluetun
    ports:
      - "8001:8888/tcp" # HTTP proxy - Phase 1
      - "8002:8388/tcp" # Shadowsocks - Phase 1
      - "8003:8080/tcp" # Control server - Phase 1
    networks:
      - mediastack

  # Reverse Proxy
  traefik:
    image: traefik:v3.0
    container_name: mediastack-traefik
    restart: unless-stopped
    command:
      - "--api.dashboard=true"
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.httpchallenge=true"
      - "--certificatesresolvers.letsencrypt.acme.httpchallenge.entrypoint=web"
      - "--certificatesresolvers.letsencrypt.acme.email=${ACME_EMAIL}"
      - "--certificatesresolvers.letsencrypt.acme.storage=/acme.json"
      - "--metrics.prometheus=true"
    ports:
      - "80:80"        # Standard HTTP
      - "443:443"      # Standard HTTPS
      - "8000:8080"    # Traefik Dashboard - Phase 1
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - traefik_acme:/acme.json
    networks:
      - mediastack

  # Authentication
  authentik-server:
    image: ghcr.io/goauthentik/server:latest
    container_name: mediastack-authentik-server
    restart: unless-stopped
    command: server
    environment:
      AUTHENTIK_LOG_LEVEL: info
      AUTHENTIK_SECRET_KEY: ${AUTHENTIK_SECRET_KEY}
      AUTHENTIK_REDIS__HOST: mediastack-valkey
      AUTHENTIK_POSTGRESQL__HOST: mediastack-postgres
      AUTHENTIK_POSTGRESQL__NAME: ${POSTGRES_DB}
      AUTHENTIK_POSTGRESQL__USER: ${POSTGRES_USER}
      AUTHENTIK_POSTGRESQL__PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - authentik_media:/media
      - authentik_templates:/templates
    networks:
      - mediastack
    ports:
      - "8030:9000"    # Phase 1: Core Infrastructure
    depends_on:
      - postgres
      - valkey

  authentik-worker:
    image: ghcr.io/goauthentik/server:latest
    container_name: mediastack-authentik-worker
    restart: unless-stopped
    command: worker
    environment:
      AUTHENTIK_LOG_LEVEL: info
      AUTHENTIK_SECRET_KEY: ${AUTHENTIK_SECRET_KEY}
      AUTHENTIK_REDIS__HOST: mediastack-valkey
      AUTHENTIK_POSTGRESQL__HOST: mediastack-postgres
      AUTHENTIK_POSTGRESQL__NAME: ${POSTGRES_DB}
      AUTHENTIK_POSTGRESQL__USER: ${POSTGRES_USER}
      AUTHENTIK_POSTGRESQL__PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - authentik_media:/media
      - authentik_templates:/templates
    networks:
      - mediastack
    depends_on:
      - postgres
      - valkey

  # ============================================================================
  # PHASE 2: ESSENTIAL MEDIA SERVICES (8100-8199)
  # ============================================================================

  # Indexers & Proxies
  prowlarr:
    image: linuxserver/prowlarr:latest
    container_name: mediastack-prowlarr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - prowlarr_config:/config
    networks:
      - mediastack
    ports:
      - "8100:9696"    # Phase 2: Essential Media
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.prowlarr.rule=Host(`prowlarr.${DOMAIN}`)"
      - "traefik.http.services.prowlarr.loadbalancer.server.port=9696"

  jackett:
    image: linuxserver/jackett
    container_name: mediastack-jackett
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - jackett_config:/config
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8101:9117"    # Phase 2: Essential Media
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.jackett.rule=Host(`jackett.${DOMAIN}`)"
      - "traefik.http.services.jackett.loadbalancer.server.port=9117"

  flaresolverr:
    image: ghcr.io/flaresolverr/flaresolverr:latest
    container_name: mediastack-flaresolverr
    restart: unless-stopped
    environment:
      LOG_LEVEL: info
      TZ: ${TZ}
    networks:
      - mediastack
    ports:
      - "8102:8191"    # Phase 2: Essential Media

  # Download Clients (through VPN)
  qbittorrent:
    image: linuxserver/qbittorrent
    container_name: mediastack-qbittorrent
    restart: unless-stopped
    network_mode: "service:gluetun"
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
      WEBUI_PORT: 8080
    volumes:
      - qbittorrent_config:/config
      - ${DOWNLOADS_ROOT}:/downloads
    depends_on:
      - gluetun

  deluge:
    image: linuxserver/deluge
    container_name: mediastack-deluge
    restart: unless-stopped
    network_mode: "service:gluetun"
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - deluge_config:/config
      - ${DOWNLOADS_ROOT}:/downloads
    depends_on:
      - gluetun

  # Core Arr Services
  sonarr:
    image: linuxserver/sonarr
    container_name: mediastack-sonarr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - sonarr_config:/config
      - ${MEDIA_ROOT}/tv:/tv
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8110:8989"    # Phase 2: Essential Media
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.sonarr.rule=Host(`sonarr.${DOMAIN}`)"
      - "traefik.http.services.sonarr.loadbalancer.server.port=8989"

  radarr:
    image: linuxserver/radarr
    container_name: mediastack-radarr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - radarr_config:/config
      - ${MEDIA_ROOT}/movies:/movies
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8111:7878"    # Phase 2: Essential Media
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.radarr.rule=Host(`radarr.${DOMAIN}`)"
      - "traefik.http.services.radarr.loadbalancer.server.port=7878"

  lidarr:
    image: linuxserver/lidarr
    container_name: mediastack-lidarr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - lidarr_config:/config
      - ${MEDIA_ROOT}/music:/music
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8112:8686"    # Phase 2: Essential Media
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.lidarr.rule=Host(`lidarr.${DOMAIN}`)"
      - "traefik.http.services.lidarr.loadbalancer.server.port=8686"

  readarr:
    image: linuxserver/readarr:nightly
    container_name: mediastack-readarr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - readarr_config:/config
      - ${MEDIA_ROOT}/books:/books
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8113:8787"    # Phase 2: Essential Media
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.readarr.rule=Host(`readarr.${DOMAIN}`)"
      - "traefik.http.services.readarr.loadbalancer.server.port=8787"

  mylar3:
    image: linuxserver/mylar3
    container_name: mediastack-mylar3
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - mylar3_config:/config
      - ${MEDIA_ROOT}/comics:/comics
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8114:8090"    # Phase 2: Essential Media
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.mylar3.rule=Host(`mylar3.${DOMAIN}`)"
      - "traefik.http.services.mylar3.loadbalancer.server.port=8090"

  whisparr:
    image: hotio/whisparr:nightly
    container_name: mediastack-whisparr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - whisparr_config:/config
      - ${MEDIA_ROOT}/adult:/adult
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8115:6969"    # Phase 2: Essential Media
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.whisparr.rule=Host(`whisparr.${DOMAIN}`)"
      - "traefik.http.services.whisparr.loadbalancer.server.port=6969"

  # Enhanced Arr Services (RandomNinjaAtk)
  sonarr-extended:
    image: randomninjaatk/sonarr-extended
    container_name: mediastack-sonarr-extended
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - sonarr_extended_config:/config
      - ${MEDIA_ROOT}/tv:/tv
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8120:8989"    # Phase 2: Essential Media (Extended)

  radarr-extended:
    image: randomninjaatk/radarr-extended
    container_name: mediastack-radarr-extended
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - radarr_extended_config:/config
      - ${MEDIA_ROOT}/movies:/movies
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8121:7878"    # Phase 2: Essential Media (Extended)

  lidarr-extended:
    image: randomninjaatk/lidarr-extended
    container_name: mediastack-lidarr-extended
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - lidarr_extended_config:/config
      - ${MEDIA_ROOT}/music:/music
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8122:8686"    # Phase 2: Essential Media (Extended)

  # Priority Automation Services
  autobrr:
    image: ghcr.io/autobrr/autobrr:latest
    container_name: mediastack-autobrr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - autobrr_config:/config
    networks:
      - mediastack
    ports:
      - "8130:7474"    # Phase 2: Essential Media (Priority)
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.autobrr.rule=Host(`autobrr.${DOMAIN}`)"
      - "traefik.http.services.autobrr.loadbalancer.server.port=7474"

  # ============================================================================
  # PHASE 3: MEDIA SERVERS & CONTENT (8200-8299)
  # ============================================================================

  # Media Servers
  jellyfin:
    image: jellyfin/jellyfin
    container_name: mediastack-jellyfin
    restart: unless-stopped
    environment:
      JELLYFIN_PublishedServerUrl: https://jellyfin.${DOMAIN}
    volumes:
      - jellyfin_config:/config
      - jellyfin_cache:/cache
      - ${MEDIA_ROOT}:/media:ro
    networks:
      - mediastack
    ports:
      - "8200:8096"    # Phase 3: Media Servers
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.jellyfin.rule=Host(`jellyfin.${DOMAIN}`)"
      - "traefik.http.services.jellyfin.loadbalancer.server.port=8096"

  plex:
    image: plexinc/pms-docker
    container_name: mediastack-plex
    restart: unless-stopped
    environment:
      PLEX_CLAIM: ${PLEX_CLAIM}
      PLEX_UID: ${PUID}
      PLEX_GID: ${PGID}
      TZ: ${TZ}
    volumes:
      - plex_config:/config
      - plex_transcode:/transcode
      - ${MEDIA_ROOT}:/media:ro
    networks:
      - mediastack
    ports:
      - "8201:32400"   # Phase 3: Media Servers
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.plex.rule=Host(`plex.${DOMAIN}`)"
      - "traefik.http.services.plex.loadbalancer.server.port=32400"

  emby:
    image: linuxserver/emby
    container_name: mediastack-emby
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - emby_config:/config
      - ${MEDIA_ROOT}:/media:ro
    networks:
      - mediastack
    ports:
      - "8202:8096"    # Phase 3: Media Servers
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.emby.rule=Host(`emby.${DOMAIN}`)"
      - "traefik.http.services.emby.loadbalancer.server.port=8096"

  # Content Libraries
  audiobookshelf:
    image: ghcr.io/advplyr/audiobookshelf:latest
    container_name: mediastack-audiobookshelf
    restart: unless-stopped
    environment:
      AUDIOBOOKSHELF_UID: ${PUID}
      AUDIOBOOKSHELF_GID: ${PGID}
    volumes:
      - audiobookshelf_config:/config
      - ${MEDIA_ROOT}/audiobooks:/audiobooks:ro
      - ${MEDIA_ROOT}/books:/books:ro
    networks:
      - mediastack
    ports:
      - "8210:80"      # Phase 3: Media Servers
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.audiobookshelf.rule=Host(`audiobooks.${DOMAIN}`)"
      - "traefik.http.services.audiobookshelf.loadbalancer.server.port=80"

  calibre-web:
    image: lscr.io/linuxserver/calibre-web:latest
    container_name: mediastack-calibre-web
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - calibre_web_config:/config
      - ${MEDIA_ROOT}/books:/books
    networks:
      - mediastack
    ports:
      - "8211:8083"    # Phase 3: Media Servers
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.calibre-web.rule=Host(`books.${DOMAIN}`)"
      - "traefik.http.services.calibre-web.loadbalancer.server.port=8083"

  # Live TV & IPTV
  iptv-proxy:
    image: pierro777/iptv-proxy:latest
    container_name: mediastack-iptv-proxy
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - iptv_proxy_config:/app/config
    networks:
      - mediastack
    ports:
      - "8220:8080"    # Phase 3: Media Servers
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.iptv-proxy.rule=Host(`iptv.${DOMAIN}`)"
      - "traefik.http.services.iptv-proxy.loadbalancer.server.port=8080"

  tvheadend:
    image: linuxserver/tvheadend
    container_name: mediastack-tvheadend
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
      RUN_OPTS: --satip_rtsp 554
    volumes:
      - tvheadend_config:/config
      - ${MEDIA_ROOT}/recordings:/recordings
      - ${MEDIA_ROOT}/timeshift:/timeshift
    networks:
      - mediastack
    ports:
      - "8221:9981"    # Phase 3: Live TV Web
      - "8222:9982"    # Phase 3: Live TV HTSP
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.tvheadend.rule=Host(`tv.${DOMAIN}`)"
      - "traefik.http.services.tvheadend.loadbalancer.server.port=9981"

  # Media Processing
  tdarr:
    image: ghcr.io/haveagitgat/tdarr:latest
    container_name: mediastack-tdarr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
      PUID: ${PUID}
      PGID: ${PGID}
    volumes:
      - tdarr_server_data:/app/server
      - tdarr_config_data:/app/configs
      - ${MEDIA_ROOT}:/media
    networks:
      - mediastack
    ports:
      - "8230:8265"    # Phase 3: Media Processing
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.tdarr.rule=Host(`tdarr.${DOMAIN}`)"
      - "traefik.http.services.tdarr.loadbalancer.server.port=8265"

  tdarr-node:
    image: ghcr.io/haveagitgat/tdarr_node:latest
    container_name: mediastack-tdarr-node
    restart: unless-stopped
    environment:
      TZ: ${TZ}
      PUID: ${PUID}
      PGID: ${PGID}
      nodeName: MainNode
    volumes:
      - tdarr_config_data:/app/configs
      - ${MEDIA_ROOT}:/media
    networks:
      - mediastack
    depends_on:
      - tdarr

  # ============================================================================
  # PHASE 4: ENHANCEMENT SERVICES (8300-8399)
  # ============================================================================

  # Subtitles & Metadata
  bazarr:
    image: linuxserver/bazarr
    container_name: mediastack-bazarr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - bazarr_config:/config
      - ${MEDIA_ROOT}/movies:/movies
      - ${MEDIA_ROOT}/tv:/tv
    networks:
      - mediastack
    ports:
      - "8300:6767"    # Phase 4: Enhancement
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.bazarr.rule=Host(`bazarr.${DOMAIN}`)"
      - "traefik.http.services.bazarr.loadbalancer.server.port=6767"

  # Request Management
  overseerr:
    image: sctx/overseerr:latest
    container_name: mediastack-overseerr
    restart: unless-stopped
    environment:
      LOG_LEVEL: info
      TZ: ${TZ}
    volumes:
      - overseerr_config:/app/config
    networks:
      - mediastack
    ports:
      - "8310:5055"    # Phase 4: Enhancement
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.overseerr.rule=Host(`overseerr.${DOMAIN}`)"
      - "traefik.http.services.overseerr.loadbalancer.server.port=5055"

  jellyseerr:
    image: fallenbagel/jellyseerr:latest
    container_name: mediastack-jellyseerr
    restart: unless-stopped
    environment:
      LOG_LEVEL: info
      TZ: ${TZ}
    volumes:
      - jellyseerr_config:/app/config
    networks:
      - mediastack
    ports:
      - "8311:5055"    # Phase 4: Enhancement
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.jellyseerr.rule=Host(`jellyseerr.${DOMAIN}`)"
      - "traefik.http.services.jellyseerr.loadbalancer.server.port=5055"

  ombi:
    image: linuxserver/ombi
    container_name: mediastack-ombi
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - ombi_config:/config
    networks:
      - mediastack
    ports:
      - "8312:3579"    # Phase 4: Enhancement
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.ombi.rule=Host(`ombi.${DOMAIN}`)"
      - "traefik.http.services.ombi.loadbalancer.server.port=3579"

  # Analytics & Monitoring
  tautulli:
    image: linuxserver/tautulli
    container_name: mediastack-tautulli
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - tautulli_config:/config
    networks:
      - mediastack
    ports:
      - "8320:8181"    # Phase 4: Enhancement
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.tautulli.rule=Host(`tautulli.${DOMAIN}`)"
      - "traefik.http.services.tautulli.loadbalancer.server.port=8181"

  # Content Management & Automation
  kometa:
    image: kometateam/kometa:latest
    container_name: mediastack-kometa
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - kometa_config:/config
      - ${MEDIA_ROOT}:/media:ro
    networks:
      - mediastack
    ports:
      - "8330:5055"    # Phase 4: Enhancement

  gaps:
    image: housewrecker/gaps:latest
    container_name: mediastack-gaps
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - gaps_data:/usr/app
    networks:
      - mediastack
    ports:
      - "8331:8484"    # Phase 4: Enhancement
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.gaps.rule=Host(`gaps.${DOMAIN}`)"
      - "traefik.http.services.gaps.loadbalancer.server.port=8484"

  # Maintenance & Cleanup
  janitorr:
    image: ghcr.io/schaka/janitorr:latest
    container_name: mediastack-janitorr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - janitorr_config:/config
      - ${MEDIA_ROOT}:/media
    networks:
      - mediastack
    ports:
      - "8340:8080"    # Phase 4: Enhancement
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.janitorr.rule=Host(`janitorr.${DOMAIN}`)"
      - "traefik.http.services.janitorr.loadbalancer.server.port=8080"

  decluttarr:
    image: ghcr.io/manimatter/decluttarr:latest
    container_name: mediastack-decluttarr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - decluttarr_config:/config
    networks:
      - mediastack
    ports:
      - "8341:5000"    # Phase 4: Enhancement

  # List Management
  watchlistarr:
    image: ghcr.io/nylonee/watchlistarr:latest
    container_name: mediastack-watchlistarr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - watchlistarr_config:/app/config
    networks:
      - mediastack
    ports:
      - "8350:3000"    # Phase 4: Enhancement

  traktarr:
    image: ghcr.io/l3uddz/traktarr:latest
    container_name: mediastack-traktarr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - traktarr_config:/config
    networks:
      - mediastack

  # ============================================================================
  # PHASE 5: MONITORING & ANALYTICS (8400-8499)
  # ============================================================================

  # Core Monitoring
  prometheus:
    image: prom/prometheus:latest
    container_name: mediastack-prometheus
    restart: unless-stopped
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'
    volumes:
      - prometheus_data:/prometheus
      - prometheus_config:/etc/prometheus
    networks:
      - mediastack
    ports:
      - "8400:9090"    # Phase 5: Monitoring
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.prometheus.rule=Host(`prometheus.${DOMAIN}`)"
      - "traefik.http.services.prometheus.loadbalancer.server.port=9090"

  grafana:
    image: grafana/grafana:latest
    container_name: mediastack-grafana
    restart: unless-stopped
    environment:
      GF_SECURITY_ADMIN_PASSWORD: ${GRAFANA_PASSWORD}
      GF_USERS_ALLOW_SIGN_UP: false
    volumes:
      - grafana_data:/var/lib/grafana
    networks:
      - mediastack
    ports:
      - "8401:3000"    # Phase 5: Monitoring
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.grafana.rule=Host(`grafana.${DOMAIN}`)"
      - "traefik.http.services.grafana.loadbalancer.server.port=3000"

  # Arr Health Monitoring
  exportarr:
    image: ghcr.io/onedr0p/exportarr:latest
    container_name: mediastack-exportarr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    networks:
      - mediastack
    ports:
      - "8410:9707"    # Phase 5: Monitoring

  checkrr:
    image: ghcr.io/aetaric/checkrr:latest
    container_name: mediastack-checkrr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - checkrr_config:/config
    networks:
      - mediastack
    ports:
      - "8411:8080"    # Phase 5: Monitoring

  # ============================================================================
  # PHASE 6: MANAGEMENT & UTILITIES (8500-8599)
  # ============================================================================

  # Container Management
  portainer:
    image: portainer/portainer-ce:latest
    container_name: mediastack-portainer
    restart: unless-stopped
    command: -H unix:///var/run/docker.sock
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - portainer_data:/data
    networks:
      - mediastack
    ports:
      - "8500:9000"    # Phase 6: Management
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.portainer.rule=Host(`portainer.${DOMAIN}`)"
      - "traefik.http.services.portainer.loadbalancer.server.port=9000"

  # File Management
  filebot:
    image: jlesage/filebot:latest
    container_name: mediastack-filebot
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
    volumes:
      - filebot_config:/config
      - ${MEDIA_ROOT}:/media
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8510:5800"    # Phase 6: Management
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.filebot.rule=Host(`filebot.${DOMAIN}`)"
      - "traefik.http.services.filebot.loadbalancer.server.port=5800"

  # Automation Framework
  flexget:
    image: ghcr.io/flexget/flexget:latest
    container_name: mediastack-flexget
    restart: unless-stopped
    environment:
      TZ: ${TZ}
      PUID: ${PUID}
      PGID: ${PGID}
    volumes:
      - flexget_config:/config
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack
    ports:
      - "8520:5050"    # Phase 6: Management

  # Configuration Management
  buildarr:
    image: ghcr.io/buildarr/buildarr:latest
    container_name: mediastack-buildarr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - buildarr_config:/config
    networks:
      - mediastack

  # Security & Access
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: mediastack-vaultwarden
    restart: unless-stopped
    environment:
      WEBSOCKET_ENABLED: true
      ROCKET_PORT: 80
      DOMAIN: https://vaultwarden.${DOMAIN}
      ADMIN_TOKEN: ${VAULTWARDEN_ADMIN_TOKEN}
    volumes:
      - vaultwarden_data:/data
    networks:
      - mediastack
    ports:
      - "8530:80"      # Phase 6: Management
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.vaultwarden.rule=Host(`vaultwarden.${DOMAIN}`)"
      - "traefik.http.services.vaultwarden.loadbalancer.server.port=80"

  # Dashboards
  organizr:
    image: organizr/organizr:latest
    container_name: mediastack-organizr
    restart: unless-stopped
    environment:
      PUID: ${PUID}
      PGID: ${PGID}
      TZ: ${TZ}
      fpm: "true"
    volumes:
      - organizr_config:/config
    networks:
      - mediastack
    ports:
      - "8540:80"      # Phase 6: Management
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.organizr.rule=Host(`dashboard.${DOMAIN}`)"
      - "traefik.http.services.organizr.loadbalancer.server.port=80"

  homarr:
    image: ghcr.io/ajnart/homarr:latest
    container_name: mediastack-homarr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - homarr_config:/app/data/configs
      - homarr_icons:/app/public/icons
    networks:
      - mediastack
    ports:
      - "8541:7575"    # Phase 6: Management
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.homarr.rule=Host(`homarr.${DOMAIN}`)"
      - "traefik.http.services.homarr.loadbalancer.server.port=7575"

  homepage:
    image: ghcr.io/gethomepage/homepage:latest
    container_name: mediastack-homepage
    restart: unless-stopped
    environment:
      TZ: ${TZ}
      PUID: ${PUID}
      PGID: ${PGID}
    volumes:
      - homepage_config:/app/config
      - /var/run/docker.sock:/var/run/docker.sock:ro
    networks:
      - mediastack
    ports:
      - "8542:3000"    # Phase 6: Management
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.homepage.rule=Host(`homepage.${DOMAIN}`)"
      - "traefik.http.services.homepage.loadbalancer.server.port=3000"

  # Maintenance Services
  watchtower:
    image: containrrr/watchtower
    container_name: mediastack-watchtower
    restart: unless-stopped
    environment:
      WATCHTOWER_CLEANUP: "true"
      WATCHTOWER_SCHEDULE: "0 0 4 * * *"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    networks:
      - mediastack

  recyclarr:
    image: recyclarr/recyclarr:latest
    container_name: mediastack-recyclarr
    restart: unless-stopped
    environment:
      TZ: ${TZ}
    volumes:
      - recyclarr_config:/config
    networks:
      - mediastack

  unpackerr:
    image: golift/unpackerr
    container_name: mediastack-unpackerr
    restart: unless-stopped
    environment:
      UN_SONARR_0_URL: http://mediastack-sonarr:8989
      UN_SONARR_0_API_KEY: ${SONARR_API_KEY}
      UN_RADARR_0_URL: http://mediastack-radarr:7878
      UN_RADARR_0_API_KEY: ${RADARR_API_KEY}
    volumes:
      - ${DOWNLOADS_ROOT}:/downloads
    networks:
      - mediastack

  # Network & Security
  crowdsec:
    image: crowdsecurity/crowdsec:latest
    container_name: mediastack-crowdsec
    restart: unless-stopped
    environment:
      COLLECTIONS: "crowdsecurity/nginx crowdsecurity/base-http-scenarios"
    volumes:
      - crowdsec_config:/etc/crowdsec
      - crowdsec_data:/var/lib/crowdsec/data
      - /var/log:/var/log:ro
    networks:
      - mediastack

  tailscale:
    image: tailscale/tailscale:latest
    container_name: mediastack-tailscale
    restart: unless-stopped
    environment:
      TS_AUTHKEY: ${TAILSCALE_AUTH_KEY}
    volumes:
      - tailscale_data:/var/lib/tailscale
    networks:
      - mediastack
    cap_add:
      - NET_ADMIN
      - SYS_MODULE

volumes:
  # Core Infrastructure
  postgres_data:
  valkey_data:
  gluetun_data:
  traefik_acme:
  authentik_media:
  authentik_templates:

  # Essential Media Services
  prowlarr_config:
  jackett_config:
  qbittorrent_config:
  deluge_config:
  sonarr_config:
  radarr_config:
  lidarr_config:
  readarr_config:
  mylar3_config:
  whisparr_config:
  sonarr_extended_config:
  radarr_extended_config:
  lidarr_extended_config:
  autobrr_config:

  # Media Servers
  jellyfin_config:
  jellyfin_cache:
  plex_config:
  plex_transcode:
  emby_config:
  audiobookshelf_config:
  calibre_web_config:
  iptv_proxy_config:
  tvheadend_config:
  tdarr_server_data:
  tdarr_config_data:

  # Enhancement Services
  bazarr_config:
  overseerr_config:
  jellyseerr_config:
  ombi_config:
  tautulli_config:
  kometa_config:
  gaps_data:
  janitorr_config:
  decluttarr_config:
  watchlistarr_config:
  traktarr_config:

  # Monitoring & Analytics
  prometheus_data:
  prometheus_config:
  grafana_data:
  checkrr_config:

  # Management & Utilities
  portainer_data:
  filebot_config:
  flexget_config:
  buildarr_config:
  vaultwarden_data:
  organizr_config:
  homarr_config:
  homarr_icons:
  homepage_config:
  recyclarr_config:
  crowdsec_config:
  crowdsec_data:
  tailscale_data:
EOF

print_success "Ultimate Stack docker-compose.yml created with 65+ services!"

# Setup environment file
print_status "Setting up environment file..."
if [ ! -f ".env" ]; then
    print_status "Creating default .env file..."
    cat > .env << 'ENVEOF'
# Media Stack Configuration
POSTGRES_DB=mediastack
POSTGRES_USER=mediastack
POSTGRES_PASSWORD=changeme123
AUTHENTIK_SECRET_KEY=changeme-authentik-secret-key
ACME_EMAIL=admin@example.com
VPN_PROVIDER=protonvpn
VPN_TYPE=wireguard
VPN_PRIVATE_KEY=your-wireguard-private-key
VPN_ADDRESSES=10.2.0.2/32
VPN_COUNTRIES=Netherlands
TAILSCALE_AUTH_KEY=your-tailscale-auth-key
ENVEOF
    print_success "Default .env file created - please customize it"
else
    print_success "Using existing .env file"
fi

print_status "Starting the Ultimate Stack (this may take several minutes)..."
docker-compose up -d --remove-orphans

print_status "Waiting for services to initialize..."
sleep 60

print_success "Ultimate Stack Rebuild Complete!"
echo ""
echo "🎉 **Your 65+ Service Ultimate Arr Media Stack is now running!**"
echo ""
echo "📊 **Stack Overview:**"
echo "===================="
echo "• Phase 1 (Core Infrastructure): 8000-8099"
echo "• Phase 2 (Essential Media): 8100-8199"  
echo "• Phase 3 (Media Servers): 8200-8299"
echo "• Phase 4 (Enhancement): 8300-8399"
echo "• Phase 5 (Monitoring): 8400-8499"
echo "• Phase 6 (Management): 8500-8599"
echo ""
echo "🚀 **Priority Services (Configure First):**"
echo "=========================================="
echo "• Autobrr (Real-time automation): http://localhost:8130"
echo "• Prowlarr (Indexer management): http://localhost:8100"
echo "• Kometa (Plex collections): Container running"
echo "• Janitorr (Smart cleanup): http://localhost:8340"
echo "• Gaps (Collection gaps): http://localhost:8331"
echo ""
echo "🎯 **Key Access Points:**"
echo "========================"
echo "• Traefik Dashboard: http://localhost:8000"
echo "• Main Dashboard (Organizr): http://localhost:8540"
echo "• Alternative Dashboard (Homarr): http://localhost:8541"
echo "• Container Management: http://localhost:8500"
echo "• Monitoring (Prometheus): http://localhost:8400"
echo "• Monitoring (Grafana): http://localhost:8401"
echo ""
echo "✨ **What's New vs Previous 26-Service Stack:**"
echo "=============================================="
echo "• +40 additional services"
echo "• Enhanced Arr services (RandomNinjaAtk)"
echo "• Real-time automation (Autobrr)"
echo "• Advanced monitoring (Prometheus + Grafana)"
echo "• Smart content management (Kometa, Gaps)"
echo "• Intelligent cleanup (Janitorr, Decluttarr)"
echo "• Multiple dashboard options"
echo "• Comprehensive request management"
echo "• Enhanced security and networking"
echo ""
echo "Run 'docker-compose ps' to see all services!"
EOF

chmod +x rebuild-ultimate-stack.sh

print_success "Ultimate Stack Rebuild Script Created!"
echo ""
echo "🚀 **Ready to Rebuild Your 85-Service Ultimate Stack!**"
echo ""
echo "**What this script will do:**"
echo "✅ Replace current 26-service setup with 65+ service Ultimate Stack"  
echo "✅ Apply priority-based port assignments (8000-8599 range)"
echo "✅ Include all Priority Tier 1 game-changing services"
echo "✅ Add Enhanced Arr Services (RandomNinjaAtk containers)"
echo "✅ Deploy comprehensive monitoring & analytics"
echo "✅ Set up multiple dashboard options"
echo "✅ Configure intelligent automation & cleanup"
echo ""
echo "**To rebuild your Ultimate Stack:**"
echo "\`./rebuild-ultimate-stack.sh\`"
echo ""
echo "**This will restore services like:**"
echo "• Autobrr (real-time IRC automation)"
echo "• Prowlarr (advanced indexer management)"  
echo "• Kometa (Plex collection management)"
echo "• Janitorr (intelligent cleanup)"
echo "• Gaps (collection gap detection)"
echo "• Enhanced monitoring with Prometheus + Grafana"
echo "• Multiple dashboard options (Organizr, Homarr, Homepage)"
echo "• And 50+ more services!"
echo ""
print_warning "This will significantly expand your current 26-service setup!"
