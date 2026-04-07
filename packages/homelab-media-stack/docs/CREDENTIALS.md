# Media Stack Credentials & Access URLs

## Media Servers
| Service | URL | User | Pass |
|---|---|---|---|
| Jellyfin | http://192.168.12.231:8096 | jellyfin | jellyfin |
| Plex | http://192.168.12.230:32400/web | loufogle (Plex account) | — |

## Download Stack
| Service | URL | User/API Key |
|---|---|---|
| qBittorrent | http://192.168.12.212:8080 | admin / adminadmin |
| Prowlarr | http://192.168.12.210:9696 | API: 6719026a4a5042a99897597122fa4495 |
| Sonarr | http://192.168.12.214:8989 | API: 9e2127824e7446f6a2ddc5da67cfe693 |
| Radarr | http://192.168.12.225:7878 | API: cc7485c9f5a64f78bfd226ffe23e2991 |
| Readarr | http://192.168.12.217:8787 | API: 19566aa7fb90487ebd2c643ad8c6595d |
| Lidarr | http://192.168.12.218:8686 | API: (check config) |

## Media Management
| Service | URL | User | Pass |
|---|---|---|---|
| Seerr | http://192.168.12.151:5055 | jellyfin (Jellyfin login) | jellyfin |
| Tautulli | http://192.168.12.169:8181 | tautulli | tautulli |
| Bazarr | http://192.168.12.240:6767 | none | — |
| Calibre-Web | http://192.168.12.233:8083 | calibre | calibre |
| Audiobookshelf | http://192.168.12.232:13378 | (set on first login) | — |

## Dashboards
| Service | URL |
|---|---|
| Homarr | http://192.168.12.275:7575 |
| Homepage | http://192.168.12.276:3000 |
| Traefik | http://192.168.12.103:8080 |

## Infrastructure
| Service | URL | User | Pass |
|---|---|---|---|
| Vaultwarden | https://192.168.12.104 | (create account) | — |
| Proxmox | https://192.168.12.242:8006 | root | (your root pw) |
| DietPi Dashboard | http://192.168.12.244:5252 | (DietPi login) | — |
| AdGuard Home | http://192.168.12.244:8080 | adguard | (your pw) |
| Home Assistant | http://192.168.12.123:8123 | loufogle | homeassist |

## Remote Access
| Service | URL |
|---|---|
| Jellyseerr (public) | https://tiamat-tailscale.tail9d8b73.ts.net/ |

## VNC
| System | Address | Display |
|---|---|---|
| Tiamat | 192.168.12.242:5900 | :0 (x11vnc) |
| Tiamat | 192.168.12.242:5901 | :1 (Xtigervnc + Warp) |
| Bahamut | 192.168.12.244:5901 | :1 (Xtigervnc + Warp) |

## API Keys Reference
| Service | Key |
|---|---|
| Prowlarr | 6719026a4a5042a99897597122fa4495 |
| Sonarr | 9e2127824e7446f6a2ddc5da67cfe693 |
| Radarr | cc7485c9f5a64f78bfd226ffe23e2991 |
| Readarr | 19566aa7fb90487ebd2c643ad8c6595d |
| Plex Token | mixMERF9aEJxg9HrDzZW |
| TMDb | 47ef060c8451984321a70c2a07c63bce |
