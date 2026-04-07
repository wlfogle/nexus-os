# Warp Terminal Auto-Login Script

This script automates the process of logging into Warp terminal using a temporary email address, bypassing the need for a permanent email account.

## Features

- 🔄 **Automatic temporary email generation** from 10minutemail.com
- 🚀 **Automated Warp login** at app.warp.dev/login/remote
- 📧 **Email monitoring** for Firebase verification links
- 🔐 **Auth token capture** from the logged_in page
- 🔗 **Automatic token passing** to Warp terminal application

## Prerequisites

Install the required dependencies:

```bash
# Install Python packages
pip install selenium pyperclip

# Install browser and driver (Garuda Linux/Arch)
sudo pacman -S firefox geckodriver

# Or for other distros:
# sudo apt install firefox-geckodriver  # Ubuntu/Debian
# sudo dnf install firefox geckodriver  # Fedora
```

## Usage

### Basic Usage

```bash
python3 /home/lou/github/awesome-stack/scripts/warp_auto_login.py
```

### Headless Mode (no browser window)

```bash
python3 /home/lou/github/awesome-stack/scripts/warp_auto_login.py --headless
```

## How It Works

1. **Email Generation**: Opens 10minutemail.com and captures a fresh temporary email address
2. **Warp Login**: Navigates to app.warp.dev/login/remote and enters the temporary email
3. **Email Verification**: Monitors the temporary email inbox for Firebase verification links
4. **Auth Completion**: Automatically clicks the verification link to complete authentication
5. **Token Capture**: Extracts the Warp auth token from the logged_in page
6. **Token Delivery**: Passes the token to Warp terminal via:
   - System URL handler (`warp://login/TOKEN`)
   - Direct command line arguments
   - Manual copy-paste (fallback)

## Token Extraction Methods

The script tries multiple methods to capture the auth token:

- **Page source regex patterns** for JSON tokens
- **Local storage** inspection for stored auth tokens
- **"Here" link parsing** for `warp://` protocol URLs
- **DOM element inspection** for token containers

## Manual Fallback

If automatic processing fails, the script:

- Keeps the browser open for manual verification
- Copies the token to clipboard for manual pasting
- Provides clear instructions for manual completion

## Troubleshooting

### Common Issues

1. **Selenium/Geckodriver not found**:
   ```bash
   sudo pacman -S firefox geckodriver
   ```

2. **Dependencies missing**:
   ```bash
   pip install selenium pyperclip
   ```

3. **Firewall/Network issues**:
   - Disable ad blockers for `app.warp.dev`
   - Check firewall rules for `*.googleapis.com` and `*.segment.io`

4. **Email verification timeout**:
   - The script waits 3 minutes by default
   - Check the email tab manually if needed
   - Some emails may take longer to arrive

### Debug Mode

For troubleshooting, you can run without headless mode to see what's happening:

```bash
python3 /home/lou/github/awesome-stack/scripts/warp_auto_login.py
```

### Token Manual Entry

If automatic token passing fails:

1. The token is automatically copied to your clipboard
2. Open Warp terminal manually
3. Paste the token when prompted
4. Or use the `warp://login/TOKEN` URL directly

## Security Notes

- 🔐 **Temporary email**: Uses disposable email addresses that expire after 10 minutes
- 🚫 **No persistent storage**: No credentials are saved locally
- 🔄 **Session-based**: Each run creates a fresh authentication session
- 📋 **Clipboard only**: Token is only stored in clipboard temporarily

## Script Output Example

```
🚀 Starting Warp Auto-Login Process
==================================================
🔄 Getting temporary email from 10minutemail...
✅ Got temporary email: example@10minutemail.com
🔄 Navigating to main Warp login page...
✅ Entered email on login page: example@10minutemail.com
✅ Clicked login button
📧 Waiting for verification email...
🔄 Checking temporary email for verification link...
✅ Found 1 email(s) in inbox
✅ Found Firebase verification link: https://astral-field-294621.firebaseapp.com/__/auth/action?...
🔄 Completing Firebase authentication...
✅ Firebase auth completed, current URL: https://app.warp.dev/logged_in/remote
🔄 Capturing Warp authentication token...
✅ Found auth token: eyJhbGciOiJSUzI1...xyz123
✅ Token copied to clipboard
🎉 Successfully captured auth token!

Do you want to automatically open Warp with this token? (y/n): y
✅ Opened Warp login URL with system handler
✅ Token passed to Warp successfully!
✅ Process completed successfully!
```

This script provides a complete automation solution for Warp terminal authentication without requiring a permanent email address or manual intervention.
