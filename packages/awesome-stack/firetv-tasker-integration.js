/**
 * Fire TV Tasker Integration Script
 * 
 * This JavaScript can be used with Tasker (Android) to control Fire TV
 * Copy this script to Tasker and modify the server URL as needed
 * 
 * Usage in Tasker:
 * 1. Create new Task
 * 2. Add "JavaScript" action
 * 3. Paste this code
 * 4. Modify the firetvCommand function call with desired command
 * 5. Set up triggers (widgets, voice commands, etc.)
 */

// Configuration
const FIRETV_SERVER = "http://192.168.1.150:5000";

/**
 * Send command to Fire TV Controller API
 */
function firetvCommand(command, params = {}) {
    const url = `${FIRETV_SERVER}/command`;
    const data = JSON.stringify({
        command: command,
        params: params
    });
    
    // Make HTTP POST request
    try {
        const request = new XMLHttpRequest();
        request.open('POST', url, false); // Synchronous for Tasker compatibility
        request.setRequestHeader('Content-Type', 'application/json');
        request.send(data);
        
        if (request.status === 200) {
            const response = JSON.parse(request.responseText);
            if (response.success) {
                flash(`Fire TV: ${command} sent successfully`);
                return true;
            } else {
                flash(`Fire TV Error: ${response.error}`);
                return false;
            }
        } else {
            flash(`Fire TV: Connection failed (${request.status})`);
            return false;
        }
    } catch (error) {
        flash(`Fire TV: Network error - ${error.message}`);
        return false;
    }
}

/**
 * Get Fire TV status
 */
function getFireTVStatus() {
    const url = `${FIRETV_SERVER}/status`;
    
    try {
        const request = new XMLHttpRequest();
        request.open('GET', url, false);
        request.send();
        
        if (request.status === 200) {
            const status = JSON.parse(request.responseText);
            
            if (status.connected) {
                const statusText = `Fire TV: ${status.state || 'Unknown'}`;
                const appText = status.current_app ? ` (${status.current_app})` : '';
                flash(statusText + appText);
                
                // Set Tasker variables
                setGlobal('FIRETV_STATE', status.state || 'unknown');
                setGlobal('FIRETV_APP', status.current_app || '');
                setGlobal('FIRETV_VOLUME', status.volume || '0');
                setGlobal('FIRETV_CONNECTED', 'true');
            } else {
                flash('Fire TV: Disconnected');
                setGlobal('FIRETV_CONNECTED', 'false');
            }
            
            return status;
        } else {
            flash(`Fire TV Status: Connection failed (${request.status})`);
            return null;
        }
    } catch (error) {
        flash(`Fire TV Status: Network error - ${error.message}`);
        return null;
    }
}

/**
 * Launch specific apps with friendly names
 */
function launchApp(appName) {
    const appMap = {
        'netflix': 'com.netflix.ninja',
        'prime': 'com.amazon.avod', 
        'amazon': 'com.amazon.avod',
        'youtube': 'com.google.android.youtube.tv',
        'plex': 'com.plexapp.android',
        'disney': 'com.disney.disneyplus',
        'disneyplus': 'com.disney.disneyplus',
        'hulu': 'com.hulu.livingroomplus',
        'hbo': 'com.hbo.hbonow',
        'spotify': 'com.spotify.tv.android',
        'twitch': 'tv.twitch.android.app',
        'kodi': 'org.xbmc.kodi'
    };
    
    const appId = appMap[appName.toLowerCase()] || appName;
    return firetvCommand('launch_app', { app_id: appId });
}

/**
 * Quick action functions for common commands
 */
function powerToggle() {
    return firetvCommand('power');
}

function goHome() {
    return firetvCommand('home');
}

function playPause() {
    return firetvCommand('play_pause');
}

function volumeUp() {
    return firetvCommand('volume_up');
}

function volumeDown() {
    return firetvCommand('volume_down');
}

function navigateUp() {
    return firetvCommand('up');
}

function navigateDown() {
    return firetvCommand('down');
}

function navigateLeft() {
    return firetvCommand('left');
}

function navigateRight() {
    return firetvCommand('right');
}

function selectOK() {
    return firetvCommand('center');
}

function goBack() {
    return firetvCommand('back');
}

function showMenu() {
    return firetvCommand('menu');
}

function sendText(text) {
    if (!text) {
        flash('Fire TV: No text to send');
        return false;
    }
    return firetvCommand('text', { text: text });
}

/**
 * Smart automation functions
 */
function startMovie() {
    // Turn on Fire TV, go home, launch Plex
    if (powerToggle()) {
        setTimeout(() => {
            goHome();
            setTimeout(() => {
                launchApp('plex');
            }, 2000);
        }, 3000);
    }
}

function startNetflix() {
    if (powerToggle()) {
        setTimeout(() => {
            launchApp('netflix');
        }, 3000);
    }
}

function bedtimeMode() {
    // Lower volume and pause whatever is playing
    volumeDown();
    volumeDown();
    volumeDown();
    setTimeout(() => {
        playPause();
    }, 1000);
}

/**
 * Voice command handler
 * Use this with Tasker's AutoVoice or Google Assistant integration
 */
function handleVoiceCommand(command) {
    const cmd = command.toLowerCase();
    
    if (cmd.includes('turn on') || cmd.includes('power on')) {
        return powerToggle();
    } else if (cmd.includes('turn off') || cmd.includes('power off')) {
        return powerToggle();
    } else if (cmd.includes('home')) {
        return goHome();
    } else if (cmd.includes('play') || cmd.includes('pause')) {
        return playPause();
    } else if (cmd.includes('volume up') || cmd.includes('louder')) {
        return volumeUp();
    } else if (cmd.includes('volume down') || cmd.includes('quieter')) {
        return volumeDown();
    } else if (cmd.includes('back')) {
        return goBack();
    } else if (cmd.includes('netflix')) {
        return launchApp('netflix');
    } else if (cmd.includes('youtube')) {
        return launchApp('youtube');
    } else if (cmd.includes('plex')) {
        return launchApp('plex');
    } else if (cmd.includes('disney')) {
        return launchApp('disney');
    } else if (cmd.includes('prime') || cmd.includes('amazon')) {
        return launchApp('prime');
    } else if (cmd.includes('spotify')) {
        return launchApp('spotify');
    } else if (cmd.includes('status')) {
        return getFireTVStatus();
    } else if (cmd.includes('search')) {
        // Extract search term (everything after "search for")
        const searchTerm = cmd.replace(/.*search for\s+/i, '');
        if (searchTerm) {
            // Go to home, wait, then send search text
            goHome();
            setTimeout(() => {
                // Fire TV search is usually accessible via voice button or search icon
                sendText(searchTerm);
            }, 2000);
        }
    } else {
        flash(`Fire TV: Unknown command - ${command}`);
        return false;
    }
}

/**
 * Example usage for different Tasker scenarios:
 */

// Example 1: Simple power toggle
// firetvCommand('power');

// Example 2: Launch Netflix
// launchApp('netflix');

// Example 3: Get status and set Tasker variables
// getFireTVStatus();

// Example 4: Handle voice command from AutoVoice
// handleVoiceCommand('%avcommnofilter'); // %avcommnofilter is the voice input variable

// Example 5: Send custom text
// sendText('Game of Thrones');

// Example 6: Smart morning routine
// startMovie(); // This will turn on Fire TV and launch Plex

// Example 7: Navigate and select
// navigateDown();
// setTimeout(() => { selectOK(); }, 1000);

/**
 * TASKER SETUP EXAMPLES:
 * 
 * Widget Button - Power Toggle:
 * - Create widget
 * - Task: JavaScript with: firetvCommand('power');
 * 
 * Voice Command - "Fire TV Netflix":
 * - AutoVoice Recognized: "Fire TV Netflix"
 * - Task: JavaScript with: launchApp('netflix');
 * 
 * Home Screen Shortcut - Fire TV Remote:
 * - Create task with multiple JavaScript actions for different buttons
 * - Each action calls different firetvCommand() functions
 * 
 * Scheduled Task - Bedtime:
 * - Time trigger: 10:00 PM
 * - Task: JavaScript with: bedtimeMode();
 * 
 * NFC Tag - Living Room:
 * - NFC trigger
 * - Task: JavaScript with: startMovie();
 * 
 * Location Based - Arriving Home:
 * - GPS trigger: Arrive home
 * - Task: JavaScript with: powerToggle();
 */
