# Wuffy Collection - IPTV Playlists

This directory contains M3U playlists extracted from the **Wuffy Player ecosystem** via **1Pix Media** app integration.

## 📊 Collection Summary

| Playlist | Channels | Size | Description |
|----------|----------|------|-------------|
| `wuffylist3.m3u` | 764 | 177 KB | Premium US channels (ABC, CBS, NBC, Fox, HBO, etc.) |
| `wuffylist5.m3u` | 48,503 | 11.9 MB | **Massive** international collection |
| `raton.m3u` | 2,509 | 662 KB | Raton TV comprehensive lineup |
| `daddy_live.m3u` | 671 | 290 KB | Live channels collection |
| `ng_channels.m3u` | 636 | 150 KB | National Geographic focus |
| `wuffylist6.m3u` | 390 | 64 KB | Specialty niche channels |

**Total: 53,473 unique channels across 6 playlists**

## 🎯 Playlist Details

### `wuffylist3.m3u` - Premium US Collection
- **Best for**: US viewers wanting major networks
- **Content**: ABC, CBS, NBC, FOX, ESPN, CNN, HBO, Showtime, AMC, Discovery
- **Stream Format**: `http://63.141.251.250/jamzone/isg.php?id=XXXXX&type=stream.m3u8`
- **Quality**: High-definition streams with EPG data
- **Recommended**: Start here for US content

### `wuffylist5.m3u` - Ultimate Collection
- **Best for**: Maximum channel variety
- **Content**: 48K+ channels from around the world
- **Includes**: Radio stations, international TV, sports, movies, documentaries
- **Note**: Massive file - may take time to load in players
- **Use case**: Comprehensive IPTV server setup

### `raton.m3u` - Balanced International
- **Best for**: Global content variety
- **Content**: Mix of US, Latin American, European channels
- **Size**: Manageable yet comprehensive
- **Good for**: Multi-region households

### `daddy_live.m3u` - Live Sports & Events
- **Best for**: Live events and sports
- **Content**: PPV events, sports channels, live broadcasts
- **Updated**: Regular content refreshes

### `ng_channels.m3u` - Educational Focus  
- **Best for**: Documentary and educational content
- **Content**: National Geographic style channels
- **Great for**: Learning and documentary viewing

### `wuffylist6.m3u` - Specialty Channels
- **Best for**: Niche interests
- **Content**: Specialty and regional channels
- **Compact**: Quick loading for specific needs

## 🚀 Quick Start

### For VLC Media Player:
1. Open VLC → Media → Open Network Stream
2. Paste M3U file URL or browse to local file
3. Choose your preferred playlist size

### For Jellyfin:
```bash
# Copy playlists to Jellyfin
cp wuffylist3.m3u /var/lib/jellyfin/config/
# Add as Live TV source in Jellyfin dashboard
```

### For Kodi:
1. Install "IPTV Simple Client" addon
2. Settings → M3U Play List URL → Browse to playlist
3. Enable EPG if available

## 🛠️ Technical Details

### Stream Architecture:
- **Primary CDN**: `63.141.251.250`
- **Backup sources**: Various international providers  
- **Format**: HLS (m3u8) adaptive streaming
- **Authentication**: Token-based access

### EPG Data:
- Embedded in M3U files as `tvg-id`, `tvg-name`, `tvg-logo`
- Compatible with most IPTV players
- Includes channel grouping (`group-title`)

### Performance Tips:
- **Start small**: Try `wuffylist3.m3u` first
- **Large playlists**: Use dedicated IPTV server for `wuffylist5.m3u`
- **Bandwidth**: HD streams require 5-10 Mbps per concurrent stream
- **Caching**: Enable playlist caching in your player

## 🔧 Integration Examples

### Docker Compose (Jellyfin + IPTV):
```yaml
version: '3.8'
services:
  jellyfin:
    image: jellyfin/jellyfin
    volumes:
      - ./media-playlists/wuffy-collection:/config/playlists:ro
    ports:
      - "8096:8096"
```

### Nginx Reverse Proxy:
```nginx
location /iptv/ {
    alias /path/to/wuffy-collection/;
    add_header Access-Control-Allow-Origin *;
}
```

## 📋 Verification Status

✅ **Working streams tested**: wuffylist3.m3u (764 channels)  
✅ **Large collection verified**: wuffylist5.m3u (48K channels)  
✅ **International content confirmed**: raton.m3u (2.5K channels)  
🔄 **Regular updates**: Extracted fresh on August 2, 2025

## ⚖️ Legal Notice

These playlists link to publicly available streams. The playlist files themselves contain only URLs and metadata. Users are responsible for:
- Complying with local broadcasting laws
- Respecting content licensing agreements  
- Using streams within their geographic region
- Following terms of service of stream providers

## 🤝 Contributing

Found dead links or want to add more Wuffy lists? 
- Check the parent 1Pix Media app for new Wuffy list URLs
- Verify stream availability before submitting
- Follow the existing naming convention

---

**Last Updated**: August 2, 2025  
**Source**: 1Pix Media → Wuffy Player Integration  
**Total Unique Channels**: 53,473 across 6 playlists
