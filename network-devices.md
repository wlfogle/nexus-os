# Network Device Documentation

## Router Information
- **Model**: Archer AX55 Pro
- **Bands**: 2.4G, 5G
- **Network Range**: 192.168.12.0/24

## Connected Devices

### Fire TV - Living Room (Amazon Device)
- **Device Name**: Android-2 (as shown on router)
- **IP Address**: 192.168.12.207
- **MAC Address**: EC-A1-38-3F-1F-C0
- **Device Type**: Amazon Fire TV
- **Connection Duration**: 38+ minutes (as of last check)
- **Notes**: 
  - Has ADB debugging enabled over network
  - Automatically discovered by `adb devices` command
  - Accessible via `adb connect 192.168.12.207:5555` (if ADB debugging is enabled)

### Fire TV - Bedroom (Amazon Device)
- **IP Address**: 192.168.12.142
- **Device Type**: Amazon Fire TV
- **Notes**: 
  - Likely has ADB debugging enabled over network
  - Accessible via `adb connect 192.168.12.142:5555` (if ADB debugging is enabled)

### Echo Spot (Amazon Device)
- **MAC Address**: 0c:dc:91:35:33:8b
- **Device Type**: Amazon Echo Spot

### Echo Frames (Amazon Device)
- **Serial Number**: g002bc04434500te
- **Device Type**: Amazon Echo Frames

### Echo Show (Amazon Device)
- **MAC Address**: a4:08:01:0a:2b:99
- **Device Type**: Amazon Echo Show

## Security Notes
- Fire TV has network ADB debugging enabled, making it discoverable to ADB commands
- Consider disabling network ADB debugging if not needed for development purposes

---
*Last updated: 2025-08-02T22:13:54Z*
