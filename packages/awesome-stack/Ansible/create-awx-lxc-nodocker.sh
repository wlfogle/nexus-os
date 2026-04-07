#!/bin/bash
# Proxmox LXC AWX (system install, no Docker) for Media, AI, Alexa/Home Assistant
# Run as root on your Proxmox node

CTID=140
HOSTNAME="awx-lxc"
TEMPLATE="local:vztmpl/ubuntu-22.04-standard_22.04-1_amd64.tar.zst"
STORAGE="local-lvm"
DISK_SIZE="20G"
MEMORY="4096"
CORES="2"
NET="name=eth0,bridge=vmbr0,ip=dhcp"
SSH_KEY="/root/.ssh/id_rsa.pub"

# Download template if needed
if ! pveam available | grep -q "${TEMPLATE##*/}"; then
    echo "Template not found locally, downloading..."
    pveam update
    pveam download local ${TEMPLATE##*/}
fi

pct create $CTID $TEMPLATE \
    -hostname $HOSTNAME \
    -storage $STORAGE \
    -rootfs $STORAGE:$DISK_SIZE \
    -memory $MEMORY \
    -cores $CORES \
    -net0 $NET \
    -features nesting=1 \
    -features keyctl=1 \
    -features fuse=1 \
    -unprivileged 1 \
    -password "changeme" \
    -description "AWX LXC for Media Stack, AI, Alexa/Home Assistant (no Docker)"

pct start $CTID
sleep 10

pct exec $CTID -- bash -c "apt-get update && apt-get install -y python3 python3-pip git gcc libffi-dev python3-dev build-essential libssl-dev libxml2-dev libxslt1-dev libpq-dev libyaml-dev libcurl4-openssl-dev libkrb5-dev libjpeg-dev libtool python3-venv python3-wheel python3-setuptools redis-server postgresql postgresql-contrib ansible curl"

pct exec $CTID -- useradd --create-home --shell /bin/bash awx
pct exec $CTID -- bash -c "mkdir -p /var/lib/awx /var/log/awx /etc/awx"
pct exec $CTID -- chown -R awx:awx /var/lib/awx /var/log/awx /etc/awx

pct exec $CTID -- bash -c "git clone -b devel --depth=1 https://github.com/ansible/awx.git /opt/awx"
pct exec $CTID -- bash -c "python3 -m venv /opt/awx/venv"
pct exec $CTID -- bash -c '/opt/awx/venv/bin/pip install --upgrade pip'
pct exec $CTID -- bash -c '/opt/awx/venv/bin/pip install -r /opt/awx/requirements/requirements.txt'

pct exec $CTID -- bash -c "sudo -u postgres psql -c \"CREATE USER awx WITH PASSWORD 'awxpass';\""
pct exec $CTID -- bash -c "sudo -u postgres psql -c \"CREATE DATABASE awx OWNER awx;\""

pct exec $CTID -- bash -c "echo 'export AWX_SECRET_KEY=$(openssl rand -base64 32)' >> /etc/awx/env"
pct exec $CTID -- bash -c "echo 'export DATABASE_USER=awx' >> /etc/awx/env"
pct exec $CTID -- bash -c "echo 'export DATABASE_PASSWORD=awxpass' >> /etc/awx/env"
pct exec $CTID -- bash -c "echo 'export DATABASE_NAME=awx' >> /etc/awx/env"
pct exec $CTID -- bash -c "echo 'export DATABASE_HOST=127.0.0.1' >> /etc/awx/env"
pct exec $CTID -- bash -c "echo 'export DATABASE_PORT=5432' >> /etc/awx/env"
pct exec $CTID -- bash -c "echo 'export REDIS_HOST=127.0.0.1' >> /etc/awx/env"
pct exec $CTID -- bash -c "chown awx:awx /etc/awx/env"

pct exec $CTID -- bash -c "cat > /etc/systemd/system/awx-web.service <<EOF
[Unit]
Description=AWX Web Service
After=network.target postgresql.service redis-server.service

[Service]
User=awx
EnvironmentFile=/etc/awx/env
WorkingDirectory=/opt/awx
ExecStart=/opt/awx/venv/bin/gunicorn --bind 0.0.0.0:8043 awx.wsgi:application
Restart=always

[Install]
WantedBy=multi-user.target
EOF"

pct exec $CTID -- systemctl daemon-reload
pct exec $CTID -- systemctl enable --now awx-web

if [ -f "$SSH_KEY" ]; then
  pct exec $CTID -- mkdir -p /root/.ssh
  pct push $CTID $SSH_KEY /root/.ssh/authorized_keys
  pct exec $CTID -- chmod 600 /root/.ssh/authorized_keys
  pct exec $CTID -- chown root:root /root/.ssh/authorized_keys
fi

echo "AWX LXC $CTID ($HOSTNAME) is ready!"
echo "AWX UI should be available at http://<container-ip>:8043"