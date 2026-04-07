# Proxmox LXC Configurations for WireGuard Stack

## CT-100: WireGuard Server

Create via Proxmox UI or CLI:

```bash
pct create 100 local:vztmpl/alpine-3.19-default_20240207_amd64.tar.xz \
  --hostname wg-server \
  --memory 256 \
  --cores 1 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.100/24,gw=192.168.12.1 \
  --storage local-lvm \
  --rootfs local-lvm:4 \
  --unprivileged 0 \
  --features nesting=1 \
  --onboot 1 \
  --start 1
```

Add to `/etc/pve/lxc/100.conf`:
```
lxc.cgroup.devices.allow: c 10:200 rwm
lxc.mount.entry: /dev/net dev/net none bind,create=dir
```

---

## CT-101: WireGuard Client + TinyProxy

> Must be PRIVILEGED with TUN device access

```bash
pct create 101 local:vztmpl/alpine-3.19-default_20240207_amd64.tar.xz \
  --hostname wg-proxy \
  --memory 256 \
  --cores 1 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.101/24,gw=192.168.12.1 \
  --storage local-lvm \
  --rootfs local-lvm:4 \
  --unprivileged 0 \
  --features nesting=1,keyctl=1 \
  --onboot 1 \
  --start 1
```

Add to `/etc/pve/lxc/101.conf`:
```
lxc.cgroup.devices.allow: c 10:200 rwm
lxc.mount.entry: /dev/net dev/net none bind,create=dir
lxc.apparmor.profile: unconfined
lxc.mount.auto: proc:rw sys:rw
```

---

## Deployment Order

1. Create and start CT-100
2. `pct exec 100 -- sh` then run `setup-wg-server.sh`
3. Copy client config: `pct exec 100 -- cat /etc/wireguard/clients/ct101-wg-proxy.conf`
4. Create and start CT-101
5. `pct exec 101 -- sh`, save client config to `/etc/wireguard/wg0.conf`
6. Run `setup-gluetun-client.sh` in CT-101
7. Test: `curl -x http://192.168.12.101:8888 ifconfig.me`
8. Configure qBittorrent + Prowlarr to use proxy `192.168.12.101:8888`

---

## Kill Switch Verification

qBittorrent is protected because:
- All its traffic goes through the HTTP proxy at `192.168.12.101:8888`
- CT-101's internet access routes 100% through the WireGuard tunnel to CT-100
- If WireGuard drops → CT-101 loses internet → proxy returns errors → qBittorrent cannot download
- Your real IP is **never exposed**
