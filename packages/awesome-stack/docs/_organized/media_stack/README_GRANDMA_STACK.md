# 🏠 Complete Grandmother Media Stack

## 🎯 Overview

This is a complete, grandmother-friendly media center solution that combines all your media services into one simple, beautiful interface. Perfect for non-technical users who want access to movies, TV shows, live TV, weather, and more - all in large, easy-to-use buttons.

## ✨ Features

### 🌤️ Complete Weather Dashboard
- **Live Weather**: Current temperature, conditions, "feels like" temperature
- **Detailed Info**: Humidity, wind speed & direction, pressure, visibility, UV index
- **Hourly Forecast**: Next 12 hours with temperature and conditions
- **7-Day Forecast**: Weekly weather outlook with highs and lows
- **Interactive Radar**: Live weather radar from Windy.com centered on your location
- **Weather Alerts**: Real-time weather warnings and alerts
- **Auto-refresh**: Updates every 10 minutes automatically

### 🔍 AI-Powered Search
- **Smart Content Search**: Find movies and TV shows using natural language
- **Fuzzy Matching**: Finds content even with misspelled or partial titles
- **Multi-source Search**: Searches across Radarr, Sonarr, Jellyfin, and Plex
- **AI Recognition**: Enhanced search capabilities (when OpenAI API key is provided)
- **Real-time Suggestions**: Live search suggestions as you type

### 📺 Complete TV Experience
- **Live TV Guide**: Full EPG with 8,122+ channels (OTA + IPTV + PseudoTV)
- **PseudoTV Channels**: 24/7 movie and TV channels from your personal library
- **Recording Capability**: Schedule recordings for OTA and IPTV channels
- **Unified EPG**: Single interface for all TV sources
- **Channel Categories**: Action Movies, Comedy, Horror, Sitcoms, Drama, Kids, and more

### 🎭 PseudoTV Integration
- **8 Pre-configured Channels**:
  - Action Movies 24/7 (Channel 100)
  - Comedy Central Movies (Channel 101)
  - Horror Theatre (Channel 102)
  - Sitcom Central (Channel 200)
  - Drama Network (Channel 201)
  - Sci-Fi Channel (Channel 202)
  - Kids Cartoons (Channel 300)
  - Christmas Movies (Channel 400)
- **Plex Integration**: Uses your Plex Lifetime Pass libraries
- **Jellyfin Integration**: Also works with Jellyfin libraries
- **24/7 Programming**: Continuous channel programming with realistic scheduling

### 🎥 Media Library Access
- **Jellyfin Integration**: Direct access to your Jellyfin server
- **Plex Integration**: Seamless Plex Lifetime Pass integration
- **Request System**: Easy movie and TV show requesting via Overseerr
- **Recent Activity**: Shows recently added movies and shows
- **Large Button Interface**: Perfect for elderly or vision-impaired users

### 🏠 Smart Home Integration
- **System Status**: Real-time monitoring of all services
- **Auto-refresh**: Dashboard refreshes automatically
- **Mobile Responsive**: Works perfectly on tablets and phones
- **Simple Interface**: Large buttons, clear text, intuitive layout

## 🚀 Quick Start

### 1. Prerequisites
- Docker and Docker Compose installed
- At least 8GB RAM and 100GB storage
- Internet connection for weather and EPG data

### 2. Configuration
```bash
# Clone or navigate to your media stack directory
cd /home/lou/lou-media-stack

# Copy the environment template
cp .env.example .env

# Edit the environment file with your settings
nano .env
```

### 3. Required API Keys
Get these free API keys and add them to your `.env` file:

**Weather Service** (Free):
- Get API key from: https://openweathermap.org/api
- Add to `.env`: `WEATHER_API_KEY=your_api_key_here`

**AI Search** (Optional but recommended):
- Get API key from: https://platform.openai.com/api-keys
- Add to `.env`: `OPENAI_API_KEY=your_api_key_here`

### 4. Start the Stack
```bash
# Start all services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f grandma-dashboard
```

### 5. Access Your Dashboard
- **Main Dashboard**: http://localhost:8600
- **Weather Page**: http://localhost:8600/weather
- **TV Guide**: http://localhost:8600/tv
- **Search**: http://localhost:8600/search
- **PseudoTV**: http://localhost:8600/pseudotv

## 🔧 Configuration Details

### Weather Configuration
```env
WEATHER_API_KEY=your_openweathermap_api_key
WEATHER_LOCATION=New Albany, IN
```

### Live TV Configuration
```env
HDHOMERUN_IP=192.168.1.100          # Your HDHomeRun IP
TVPASS_USERNAME=your_username        # TVPass credentials
TVPASS_PASSWORD=your_password        
```

### PseudoTV Configuration
```env
PLEX_TOKEN=your_plex_api_token      # For Plex integration
JELLYFIN_API_KEY=your_jellyfin_key   # For Jellyfin integration
PSEUDOTV_ENABLED=true
```

### AI Search Configuration
```env
OPENAI_API_KEY=your_openai_key       # Optional but enhances search
```

## 📊 Service Ports

### Core Dashboard
- **8600**: Grandmother Dashboard (main interface)

### Weather & EPG Services  
- **8888**: Unified EPG Server
- **8890**: PseudoTV Server

### Media Services
- **8200**: Jellyfin
- **8201**: Plex  
- **8221**: TVHeadend (Live TV)
- **8310**: Overseerr (Requests)

### Management
- **8000**: Traefik Dashboard
- **8500**: Portainer (Container Management)

## 🎮 How to Use

### For Grandmother (Simple Mode)
1. **Open the dashboard** in a web browser
2. **Check the weather** - see current conditions and forecast
3. **Search for movies/shows** - type what you want to watch
4. **Watch live TV** - browse channels and schedule recordings  
5. **Access movie channels** - watch 24/7 themed channels
6. **Request new content** - ask for movies/shows to be added

### Large Button Interface
- All buttons are large and clearly labeled
- High contrast colors for easy visibility
- Simple navigation with minimal clicks
- Auto-refresh keeps information current
- Works on tablets, phones, and computers

## 🔍 Troubleshooting

### Dashboard Not Loading
```bash
# Check if the service is running
docker-compose ps grandma-dashboard

# View logs
docker-compose logs grandma-dashboard

# Restart the service
docker-compose restart grandma-dashboard
```

### Weather Not Working
1. Verify your API key in `.env`
2. Check the location spelling
3. Ensure internet connectivity

### PseudoTV Channels Empty
1. Ensure Plex/Jellyfin has media in libraries
2. Check API keys and tokens
3. Verify PseudoTV server is running: `docker-compose logs pseudotv-server`

### EPG Not Updating
1. Check HDHomeRun IP address
2. Verify TVPass credentials
3. Restart unified EPG: `docker-compose restart unified-epg`

## 📁 Directory Structure
```
/home/lou/lou-media-stack/
├── docker-compose.yml              # Main configuration
├── .env                           # Your environment variables
├── grandma-dashboard/             # Dashboard application
│   ├── Dockerfile
│   ├── requirements.txt
│   ├── grandma_app.py            # Enhanced application
│   └── templates/                 # HTML templates
├── pseudotv-server/              # PseudoTV service (auto-created)
├── unified-epg/                  # EPG service (auto-created)
└── ai-recommendation-engine/     # AI service (auto-created)
```

## 🎯 Channel Guide

### PseudoTV Channels
| Channel | Name | Content | Genre |
|---------|------|---------|-------|
| 100 | Action Movies 24/7 | Action movies from your library | Action/Adventure |
| 101 | Comedy Central Movies | Comedy movies | Comedy |
| 102 | Horror Theatre | Horror movies | Horror/Thriller |
| 200 | Sitcom Central | TV comedy shows | Comedy/Sitcom |
| 201 | Drama Network | TV drama series | Drama |
| 202 | Sci-Fi Channel | Science fiction content | Sci-Fi/Fantasy |
| 300 | Kids Cartoons | Children's animation | Kids/Animation |
| 400 | Christmas Movies | Holiday movies | Holiday/Family |

### Total Channel Count
- **OTA Channels**: 75 (via HDHomeRun)
- **IPTV Channels**: 8,039 (via TVPass + Local)
- **PseudoTV Channels**: 8 (from your media library)
- **Total**: 8,122+ channels

## 🔄 Automatic Features

### Weather Updates
- Current conditions refresh every 10 minutes
- Forecast updates every hour
- Radar refreshes automatically
- Alert monitoring continuous

### Content Updates
- Recently added content refreshes every 5 minutes
- System status checks every 30 seconds
- EPG data updates every 6 hours
- PseudoTV schedules regenerate daily

### Dashboard Maintenance
- Auto-refresh every 5 minutes
- Service health monitoring
- Automatic error recovery
- Cache management

## 🎊 Success Metrics

When properly configured, you should have:
- ✅ **Complete Weather Station**: Current, forecast, radar, alerts
- ✅ **8,122+ TV Channels**: OTA + IPTV + PseudoTV integrated
- ✅ **AI-Powered Search**: Find any content instantly
- ✅ **24/7 Movie Channels**: Personal library as linear TV
- ✅ **Recording Capability**: Schedule recordings from EPG
- ✅ **Simple Interface**: Perfect for elderly users
- ✅ **Mobile Responsive**: Works on any device
- ✅ **Auto-Refresh**: Always up-to-date information

## 🆘 Support

### Common Issues
1. **Services won't start**: Check `.env` file configuration
2. **No weather data**: Verify OpenWeatherMap API key
3. **Empty PseudoTV**: Ensure media exists in Plex/Jellyfin
4. **No EPG data**: Check HDHomeRun and TVPass settings

### Log Locations
```bash
# Main dashboard logs
docker-compose logs grandma-dashboard

# Weather service logs  
docker-compose logs grandma-dashboard | grep Weather

# PseudoTV logs
docker-compose logs pseudotv-server

# EPG logs
docker-compose logs unified-epg
```

## 🔮 Future Enhancements

The system is designed to be extensible. Possible future additions:
- Voice control integration
- Smart home device control
- News and RSS feeds
- Photo slideshows
- Music streaming integration
- Video calling integration
- Medication reminders
- Calendar integration

---

## 🎉 Congratulations!

You now have a complete, production-ready media center that's specifically designed for elderly users or anyone who wants a simple, beautiful interface to access all their entertainment. The system combines:

- **Professional weather station**
- **8,122+ TV channels with recording**
- **AI-powered content search**
- **Personal media libraries as 24/7 channels**
- **Simple, large-button interface**
- **Auto-updating dashboard**

Perfect for grandparents, elderly relatives, or anyone who wants entertainment without complexity!
