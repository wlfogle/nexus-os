# Fire TV iOS Shortcuts Integration

This guide shows how to create iOS Shortcuts to control your Fire TV from any iOS device (iPhone, iPad, Apple Watch).

## Prerequisites

1. Fire TV Controller service running on your Proxmox LXC
2. iOS device on the same network or connected via VPN
3. iOS Shortcuts app (pre-installed on iOS 13+)

## Basic Shortcut Template

Here's the basic template for all Fire TV shortcuts:

### 1. Create New Shortcut

1. Open Shortcuts app
2. Tap "+" to create new shortcut
3. Add "Get Contents of URL" action
4. Configure as follows:

**URL:** `http://192.168.1.150:5000/command`  
**Method:** POST  
**Headers:**
- Content-Type: application/json

**Request Body:**
```json
{
  "command": "COMMAND_NAME",
  "params": {}
}
```

## Individual Shortcut Configurations

### Power Toggle
```json
{
  "command": "power",
  "params": {}
}
```
**Shortcut Name:** "Fire TV Power"  
**Icon:** Power symbol  
**Color:** Red

### Navigation Shortcuts

#### Home
```json
{
  "command": "home",
  "params": {}
}
```

#### Back
```json
{
  "command": "back",
  "params": {}
}
```

#### Play/Pause
```json
{
  "command": "play_pause",
  "params": {}
}
```

#### Volume Up
```json
{
  "command": "volume_up",
  "params": {}
}
```

#### Volume Down
```json
{
  "command": "volume_down",
  "params": {}
}
```

### App Launch Shortcuts

#### Netflix
```json
{
  "command": "launch_app",
  "params": {
    "app_id": "com.netflix.ninja"
  }
}
```

#### YouTube
```json
{
  "command": "launch_app",
  "params": {
    "app_id": "com.google.android.youtube.tv"
  }
}
```

#### Plex
```json
{
  "command": "launch_app",
  "params": {
    "app_id": "com.plexapp.android"
  }
}
```

#### Amazon Prime Video
```json
{
  "command": "launch_app",
  "params": {
    "app_id": "com.amazon.avod"
  }
}
```

#### Disney+
```json
{
  "command": "launch_app",
  "params": {
    "app_id": "com.disney.disneyplus"
  }
}
```

#### Spotify
```json
{
  "command": "launch_app",
  "params": {
    "app_id": "com.spotify.tv.android"
  }
}
```

## Advanced Shortcuts

### Fire TV Status Checker

**Steps:**
1. Get Contents of URL
   - URL: `http://192.168.1.150:5000/status`
   - Method: GET
2. Get Value from Input
   - Key: state
3. Speak Text
   - Text: "Fire TV is [state]"

### Text Input with Voice

**Steps:**
1. Dictate Text
2. Get Contents of URL
   - URL: `http://192.168.1.150:5000/command`
   - Method: POST
   - Request Body:
   ```json
   {
     "command": "text",
     "params": {
       "text": "DICTATED_TEXT"
     }
   }
   ```
3. Replace "DICTATED_TEXT" with "Provided Input" from step 1

### Smart Scene Shortcuts

#### Movie Night
**Steps:**
1. Fire TV Power On
2. Wait 3 seconds
3. Launch Plex
4. Speak: "Movie night is ready!"

#### Bedtime Mode
**Steps:**
1. Volume Down (repeat 3 times)
2. Play/Pause (to pause content)
3. Speak: "Good night!"

#### Morning News
**Steps:**
1. Fire TV Power On
2. Wait 3 seconds
3. Launch YouTube
4. Wait 2 seconds
5. Send Text: "morning news"

### Fire TV Remote Control Menu

Create a master shortcut that presents multiple options:

**Steps:**
1. Choose from Menu
   - Options: 
     - Power Toggle
     - Netflix
     - YouTube
     - Plex
     - Home
     - Back
     - Volume Up
     - Volume Down
     - Play/Pause
2. If "Power Toggle" → Run "Fire TV Power" shortcut
3. If "Netflix" → Run "Fire TV Netflix" shortcut
4. (Continue for all options...)

## Siri Integration

All shortcuts automatically work with Siri. You can invoke them by saying:

- "Hey Siri, Fire TV Power"
- "Hey Siri, Launch Netflix on Fire TV"
- "Hey Siri, Fire TV Home"
- "Hey Siri, Movie Night"

### Custom Siri Phrases

Edit any shortcut and add custom phrases:
- "Turn on the TV" → Fire TV Power
- "Start Netflix" → Launch Netflix
- "TV Home" → Fire TV Home
- "Pause TV" → Play/Pause

## Apple Watch Integration

All shortcuts automatically sync to Apple Watch. Create complications for quick access:

1. Open Watch app on iPhone
2. Go to "Complications"
3. Add Fire TV shortcuts to watch face
4. Use Siri on watch: "Launch Netflix"

## Widget Integration

Add shortcuts to your home screen or Today View:

1. Long press home screen
2. Tap "+" in top corner
3. Search "Shortcuts"
4. Add "Siri Shortcuts" widget
5. Configure with your Fire TV shortcuts

## Automation Examples

### Location-Based Automation
1. Open Shortcuts app
2. Go to "Automation" tab
3. Create "Arrive Home" automation
4. Add "Run Shortcut" → "Fire TV Power"

### Time-Based Automation
1. Create "Time of Day" automation
2. Set to 7:00 PM daily
3. Add "Run Shortcut" → "Movie Night"

### CarPlay Integration
Fire TV shortcuts appear in CarPlay when you arrive home, allowing you to turn on TV before entering the house.

## Error Handling

For better user experience, add error handling to shortcuts:

**Steps:**
1. Get Contents of URL (your Fire TV command)
2. If (Has Any Value)
   - Speak: "Command sent successfully"
3. Otherwise
   - Speak: "Fire TV connection failed"

## Sharing Shortcuts

Share your Fire TV shortcuts with family:

1. Open any shortcut
2. Tap settings icon
3. Tap "Share"
4. Send via AirDrop, Messages, etc.
5. Recipients can add to their Shortcuts app

## Security Considerations

- Only works on your local network (or VPN)
- Consider setting up authentication if exposing externally
- Use HTTPS if available for secure communication

## Troubleshooting

**Shortcut not working?**
1. Check network connection
2. Verify Fire TV Controller service is running
3. Test URL in Safari: `http://192.168.1.150:5000/status`
4. Ensure Fire TV is powered on and connected

**Siri not recognizing commands?**
1. Re-record Siri phrase
2. Check for similar phrases in other shortcuts
3. Use more distinctive command names

## Export/Import

**To export all shortcuts:**
1. Select multiple shortcuts
2. Tap "Share"
3. Save as .shortcut files

**To import:**
1. Tap .shortcut file
2. Review and add to library

This gives you complete Fire TV control from any iOS device, with voice commands, widgets, automations, and Apple Watch integration!
