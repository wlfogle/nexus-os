#!/bin/bash
# ============================================================
# Authentik SSO Setup — CT-107 (Ubuntu, 192.168.12.107)
# Requires: CT-105 (Valkey) and CT-106 (PostgreSQL) running
# Run inside CT-107 after container creation
# ============================================================
set -e

echo "==> Installing Docker..."
apt-get update -qq
apt-get install -y curl ca-certificates
curl -fsSL https://get.docker.com | sh

echo "==> Creating Authentik directory structure..."
mkdir -p /opt/authentik/{media,custom-templates,certs}

echo "==> Generating secret key..."
SECRET_KEY=$(openssl rand -base64 36)

# Prompt for PostgreSQL password
if [ -z "$AUTHENTIK_PG_PASSWORD" ]; then
  read -rsp "PostgreSQL password for authentik user: " AUTHENTIK_PG_PASSWORD
  echo
fi

echo "==> Writing docker-compose.yml..."
cat > /opt/authentik/docker-compose.yml <<EOF
---
services:
  server:
    image: ghcr.io/goauthentik/server:latest
    restart: unless-stopped
    command: server
    environment:
      AUTHENTIK_REDIS__HOST: 192.168.12.105
      AUTHENTIK_POSTGRESQL__HOST: 192.168.12.106
      AUTHENTIK_POSTGRESQL__USER: authentik
      AUTHENTIK_POSTGRESQL__NAME: authentik
      AUTHENTIK_POSTGRESQL__PASSWORD: ${AUTHENTIK_PG_PASSWORD}
      AUTHENTIK_SECRET_KEY: ${SECRET_KEY}
      AUTHENTIK_ERROR_REPORTING__ENABLED: "false"
    volumes:
      - ./media:/media
      - ./custom-templates:/templates
    ports:
      - 9000:9000
      - 9443:9443

  worker:
    image: ghcr.io/goauthentik/server:latest
    restart: unless-stopped
    command: worker
    user: root
    environment:
      AUTHENTIK_REDIS__HOST: 192.168.12.105
      AUTHENTIK_POSTGRESQL__HOST: 192.168.12.106
      AUTHENTIK_POSTGRESQL__USER: authentik
      AUTHENTIK_POSTGRESQL__NAME: authentik
      AUTHENTIK_POSTGRESQL__PASSWORD: ${AUTHENTIK_PG_PASSWORD}
      AUTHENTIK_SECRET_KEY: ${SECRET_KEY}
      AUTHENTIK_ERROR_REPORTING__ENABLED: "false"
    volumes:
      - ./media:/media
      - ./certs:/certs
      - /var/run/docker.sock:/var/run/docker.sock
EOF

echo "==> Pulling and starting Authentik..."
cd /opt/authentik
docker compose pull
docker compose up -d

echo ""
echo "=== Authentik SSO Setup Complete ==="
echo ""
echo "  Server:  http://192.168.12.107:9000"
echo "  Admin:   http://192.168.12.107:9000/if/flow/initial-setup/"
echo ""
echo "  Secret key saved in docker-compose.yml"
echo "  Back up /opt/authentik/docker-compose.yml — the secret key is not recoverable"
echo ""
