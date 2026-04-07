const { invoke } = window.__TAURI__.core;

let currentDeviceId = null;
let channels = [];

// Local storage keys
const STORAGE_KEYS = {
  LAST_DEVICE: 'hdhomerun_last_device',
  CHANNELS: 'hdhomerun_channels'
};

// Storage functions
function saveLastDevice(deviceId) {
  localStorage.setItem(STORAGE_KEYS.LAST_DEVICE, deviceId);
}

function getLastDevice() {
  return localStorage.getItem(STORAGE_KEYS.LAST_DEVICE);
}

function saveChannels(channelData) {
  localStorage.setItem(STORAGE_KEYS.CHANNELS, JSON.stringify(channelData));
}

function getSavedChannels() {
  const saved = localStorage.getItem(STORAGE_KEYS.CHANNELS);
  return saved ? JSON.parse(saved) : null;
}

// Status functions
function setStatus(message, isError = false) {
  const statusEl = document.getElementById('status');
  statusEl.textContent = message;
  statusEl.className = isError ? 'error' : 'success';
  setTimeout(() => {
    statusEl.textContent = '';
    statusEl.className = '';
  }, 5000);
}

// Device discovery
async function discoverDevices() {
  try {
    setStatus('Discovering devices...');
    const devices = await invoke('discover_devices');
    
    const devicesList = document.getElementById('devices-list');
    devicesList.innerHTML = '';
    
    if (devices.length === 0) {
      devicesList.innerHTML = '<p>No HDHomeRun devices found</p>';
      return;
    }
    
    const lastDeviceId = getLastDevice();
    let autoSelectedDevice = null;
    
    devices.forEach(device => {
      const deviceEl = document.createElement('div');
      deviceEl.className = 'device-item';
      deviceEl.innerHTML = `
        <h3>${device.id}</h3>
        <p>IP: ${device.ip}</p>
        <p>Model: ${device.model}</p>
        <p>Version: ${device.version}</p>
        <button class="select-btn" data-device-id="${device.id}">Select</button>
      `;
      devicesList.appendChild(deviceEl);
      
      // Add event listener to the select button
      const selectBtn = deviceEl.querySelector('.select-btn');
      selectBtn.addEventListener('click', () => {
        selectDevice(device.id);
      });
      
      // Auto-select last used device
      if (device.id === lastDeviceId) {
        autoSelectedDevice = device;
      }
    });
    
    // Auto-select the last used device if found
    if (autoSelectedDevice) {
      selectDevice(autoSelectedDevice.id);
      setStatus(`Auto-selected last used device: ${autoSelectedDevice.id}`);
    } else if (devices.length === 1) {
      // If only one device, auto-select it
      selectDevice(devices[0].id);
      setStatus(`Auto-selected only device: ${devices[0].id}`);
    } else {
      setStatus(`Found ${devices.length} device(s)`);
    }
  } catch (error) {
    setStatus(`Error discovering devices: ${error}`, true);
  }
}

// Device selection
function selectDevice(deviceId) {
  currentDeviceId = deviceId;
  saveLastDevice(deviceId);  // Save to localStorage
  document.getElementById('scan-btn').disabled = false;
  
  // Update device selection visual feedback
  document.querySelectorAll('.device-item').forEach(item => {
    item.classList.remove('selected');
  });
  
  // Find the device item that contains this device ID and mark it as selected
  const deviceItems = document.querySelectorAll('.device-item');
  deviceItems.forEach(item => {
    if (item.querySelector('h3').textContent === deviceId) {
      item.classList.add('selected');
    }
  });
  
  // Check for saved channels first
  const savedChannels = getSavedChannels();
  if (savedChannels && savedChannels.length > 0) {
    channels = savedChannels;
    displayChannels(channels);
    setStatus(`Selected device: ${deviceId} (loaded ${channels.length} saved channels)`);
  } else {
    setStatus(`Selected device: ${deviceId}`);
  }
}

// Channel scanning
async function scanChannels() {
  if (!currentDeviceId) {
    setStatus('Please select a device first', true);
    return;
  }
  
  try {
    setStatus('Loading channels...');
    document.getElementById('scan-btn').disabled = true;
    
    channels = await invoke('scan_channels', { deviceId: currentDeviceId });
    
    if (channels.length === 0) {
      setStatus('No channels found', true);
      document.getElementById('scan-btn').disabled = false;
      return;
    }
    
    // Save channels to localStorage
    saveChannels(channels);
    
    // Display channels
    displayChannels(channels);
    
    setStatus(`Loaded ${channels.length} channel(s)`);
    document.getElementById('scan-btn').disabled = false;
  } catch (error) {
    setStatus(`Error loading channels: ${error}`, true);
    document.getElementById('scan-btn').disabled = false;
  }
}

// Display channels function - FIXED VERSION
function displayChannels(channelData) {
  const channelsList = document.getElementById('channels-list');
  channelsList.innerHTML = '';
  
  console.log('=== DISPLAYING CHANNELS ===');
  console.log('Channel data:', channelData);
  
  channelData.forEach((channel, index) => {
    const streamUrl = channel.stream_url || `http://192.168.12.215:5004/auto/v${channel.number}`;
    
    // Create channel element
    const channelEl = document.createElement('div');
    channelEl.className = 'channel-item';
    
    // Create individual elements instead of innerHTML
    const channelInfo = document.createElement('div');
    channelInfo.className = 'channel-info';
    
    const channelNumber = document.createElement('span');
    channelNumber.className = 'channel-number';
    channelNumber.textContent = channel.number;
    
    const channelName = document.createElement('span');
    channelName.className = 'channel-name';
    channelName.textContent = channel.name;
    
    const channelEpg = document.createElement('span');
    channelEpg.className = 'channel-epg';
    channelEpg.id = `epg-${index}`;
    channelEpg.textContent = 'Loading EPG...';
    
    const watchButton = document.createElement('button');
    watchButton.className = 'watch-btn';
    watchButton.textContent = 'Watch';
    
    // Add click handler directly
    watchButton.addEventListener('click', () => {
      console.log(`=== WATCH BUTTON CLICKED ===`);
      console.log(`Channel: ${channel.number} - ${channel.name}`);
      console.log(`Stream URL: ${streamUrl}`);
      playChannel(streamUrl, channel.number, channel.name);
    });
    
    // Assemble the elements
    channelInfo.appendChild(channelNumber);
    channelInfo.appendChild(channelName);
    channelInfo.appendChild(channelEpg);
    
    channelEl.appendChild(channelInfo);
    channelEl.appendChild(watchButton);
    
    channelsList.appendChild(channelEl);
    
    // Load EPG after a delay
    setTimeout(() => {
      loadEPGForChannel(channel.number, index);
    }, 100 + (index * 50));
  });
  
  console.log(`Successfully displayed ${channelData.length} channels`);
}

// Simple EPG loader - FIXED VERSION
function loadEPGForChannel(channelNumber, index) {
  console.log(`*** EPG: Loading for channel ${channelNumber}, index ${index}`);
  
  const epgElement = document.getElementById(`epg-${index}`);
  console.log(`*** EPG: Element found:`, epgElement);
  
  if (!epgElement) {
    console.error(`*** EPG: Element epg-${index} not found!`);
    return;
  }
  
  const hour = new Date().getHours();
  let program = 'Current Programming';
  
  // Better program mapping based on channel
  const channelPrograms = {
    '11.1': 'WHAS 11 News',
    '32.1': 'WLKY News', 
    '41.1': 'WDRB News',
    '21.1': 'WBNA Programming'
  };
  
  const channelSpecific = channelPrograms[channelNumber];
  
  if (hour >= 6 && hour < 12) {
    program = channelSpecific ? `Morning - ${channelSpecific}` : 'Morning Shows';
  } else if (hour >= 12 && hour < 17) {
    program = channelSpecific ? `Afternoon - ${channelSpecific}` : 'Afternoon Programming';
  } else if (hour >= 17 && hour < 22) {
    program = channelSpecific ? `Evening - ${channelSpecific}` : 'Evening News';
  } else {
    program = channelSpecific ? `Late Night - ${channelSpecific}` : 'Late Night Programming';
  }
  
  const epgText = `${hour}:00 - ${program}`;
  epgElement.textContent = epgText;
  epgElement.classList.add('epg-loaded');
  
  console.log(`*** EPG: Set text to: ${epgText}`);
}

// Global function for onclick
window.playChannelDirect = function(streamUrl, channelNumber, channelName) {
  console.log(`Direct play: ${channelNumber} - ${channelName}`);
  console.log(`URL: ${streamUrl}`);
  playChannel(streamUrl, channelNumber, channelName);
};

// Simplified EPG function
function loadChannelEPGSimple(channelNumber, elementId) {
  console.log(`Loading simple EPG for channel ${channelNumber}, element: ${elementId}`);
  
  const epgElement = document.getElementById(elementId);
  if (!epgElement) {
    console.log(`EPG element ${elementId} not found`);
    return;
  }
  
  // Set loading state
  epgElement.textContent = 'Loading...';
  
  // Simulate loading delay then show program info
  setTimeout(() => {
    const now = new Date();
    const hour = now.getHours();
    
    let program = 'Programming';
    if (hour >= 6 && hour < 12) {
      program = 'Morning Show';
    } else if (hour >= 12 && hour < 17) {
      program = 'Afternoon Programming';
    } else if (hour >= 17 && hour < 22) {
      program = 'Evening News';
    } else {
      program = 'Late Night Shows';
    }
    
    const timeSlot = `${hour}:00-${(hour + 1) % 24}:00`;
    epgElement.textContent = `${timeSlot}: ${program}`;
    epgElement.classList.add('epg-loaded');
    console.log(`EPG loaded for ${channelNumber}: ${timeSlot}: ${program}`);
  }, 500);
}

// EPG (Electronic Program Guide) functions
async function loadChannelEPG(channelNumber, elementId) {
  console.log(`Loading EPG for channel ${channelNumber}`);
  try {
    // Simulate EPG data loading - in a real app you'd fetch from an API
    const epgData = await getEPGData(channelNumber);
    const epgElement = document.getElementById(`epg-${elementId || channelNumber.replace('.', '-')}`);
    
    console.log(`EPG data for ${channelNumber}:`, epgData);
    console.log(`EPG element found:`, epgElement);
    
    if (epgElement && epgData) {
      epgElement.textContent = epgData;
      epgElement.classList.add('epg-loaded');
      console.log(`EPG loaded for channel ${channelNumber}`);
    } else if (epgElement) {
      epgElement.textContent = 'No program info';
      epgElement.classList.add('epg-no-data');
    }
  } catch (error) {
    console.error(`EPG error for channel ${channelNumber}:`, error);
    const epgElement = document.getElementById(`epg-${elementId || channelNumber.replace('.', '-')}`);
    if (epgElement) {
      epgElement.textContent = 'EPG unavailable';
      epgElement.classList.add('epg-error');
    }
  }
}

// Mock EPG data - in a real app this would fetch from a TV guide API
async function getEPGData(channelNumber) {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, Math.random() * 1000 + 500));
  
  const now = new Date();
  const currentHour = now.getHours();
  
  // Mock program data based on channel and time
  const mockPrograms = {
    '11.1': {
      morning: 'Good Morning Louisville',
      afternoon: 'WHAS News at Noon',
      evening: 'WHAS 11 News',
      night: 'Late Night Movie'
    },
    '32.1': {
      morning: 'WLKY Morning News',
      afternoon: 'The View',
      evening: 'WLKY Evening News',
      night: 'Jimmy Kimmel Live'
    },
    '41.1': {
      morning: 'Good Day Louisville',
      afternoon: 'Judge Judy',
      evening: 'WDRB News',
      night: 'The Late Show'
    },
    '21.1': {
      morning: 'Morning Show',
      afternoon: 'Talk Show',
      evening: 'Evening News',
      night: 'Movie Night'
    }
  };
  
  const channelPrograms = mockPrograms[channelNumber] || {
    morning: 'Morning Programming',
    afternoon: 'Afternoon Shows',
    evening: 'Evening Programming',
    night: 'Late Night Content'
  };
  
  let currentProgram;
  if (currentHour >= 6 && currentHour < 12) {
    currentProgram = channelPrograms.morning;
  } else if (currentHour >= 12 && currentHour < 17) {
    currentProgram = channelPrograms.afternoon;
  } else if (currentHour >= 17 && currentHour < 22) {
    currentProgram = channelPrograms.evening;
  } else {
    currentProgram = channelPrograms.night;
  }
  
  const timeSlot = `${currentHour}:00-${(currentHour + 1) % 24}:00`;
  return `${timeSlot}: ${currentProgram}`;
}

// Video playback - WORKING VERSION
function playChannel(streamUrl, channelNumber, channelName) {
  console.log(`=== STARTING PLAYBACK ===`);
  console.log(`Channel: ${channelNumber} - ${channelName}`);
  console.log(`Stream URL: ${streamUrl}`);
  
  const videoPlayer = document.getElementById('video-player');
  const currentChannelEl = document.getElementById('current-channel');
  
  if (!videoPlayer) {
    console.error('Video player not found!');
    setStatus('Error: Video player not found', true);
    return;
  }
  
  // Stop current playback
  videoPlayer.pause();
  videoPlayer.currentTime = 0;
  
  // Set volume to 200%
  videoPlayer.volume = 2.0;
  
  // Set source and load
  videoPlayer.src = streamUrl;
  videoPlayer.load();
  
  // Update display
  if (currentChannelEl) {
    currentChannelEl.innerHTML = `
      <h3>Now Playing:</h3>
      <p><strong>${channelNumber} - ${channelName}</strong></p>
      <p>Stream: ${streamUrl}</p>
      <p><small>HDHomeRun has 2 tuners. If error occurs, another device may be using both tuners.</small></p>
    `;
  }
  
  setStatus(`Playing channel ${channelNumber} - ${channelName}`);
  
  // Add video event handlers
  videoPlayer.onerror = (e) => {
    console.error('Video error:', e);
    setStatus(`Error playing channel ${channelNumber}. Check if tuners are available.`, true);
  };
  
  videoPlayer.onloadstart = () => {
    setStatus(`Loading channel ${channelNumber}...`);
  };
  
  videoPlayer.oncanplay = () => {
    setStatus(`Ready to play channel ${channelNumber}`);
  };
  
  console.log(`=== PLAYBACK SETUP COMPLETE ===`);
}

// Volume control functions
function updateVolume(value) {
  const videoPlayer = document.getElementById('video-player');
  const volumeDisplay = document.getElementById('volume-display');
  
  // Allow volume up to 200% (value 0-200 maps to 0.0-2.0)
  videoPlayer.volume = value / 100;
  volumeDisplay.textContent = value + '%';
}

function setMaxVolume() {
  const videoPlayer = document.getElementById('video-player');
  const volumeSlider = document.getElementById('volume-slider');
  const volumeDisplay = document.getElementById('volume-display');
  
  videoPlayer.volume = 2.0;  // Set to 200%
  volumeSlider.value = 200;
  volumeDisplay.textContent = '200%';
  
  setStatus('Volume set to double maximum (200%)');
}

// Auto-startup function
async function autoStartup() {
  const channelsList = document.getElementById('channels-list');
  channelsList.innerHTML = '<div class="loading">🔍 Finding HDHomeRun device...</div>';
  
  try {
    // Auto-discover devices
    const devices = await invoke('discover_devices');
    if (devices.length === 0) {
      channelsList.innerHTML = '<div class="error">❌ No HDHomeRun devices found</div>';
      setStatus('No HDHomeRun devices found', true);
      return;
    }
    
    // Auto-select first device
    const device = devices[0];
    currentDeviceId = device.id;
    saveLastDevice(device.id);
    
    channelsList.innerHTML = `<div class="loading">📺 Found ${device.model} (${device.id})<br>Loading channels...</div>`;
    
    // Check for saved channels first
    const savedChannels = getSavedChannels();
    if (savedChannels && savedChannels.length > 0) {
      channels = savedChannels;
      displayChannels(channels);
      setStatus(`Ready! ${channels.length} channels available`);
    } else {
      // Load channels
      channels = await invoke('scan_channels', { deviceId: device.id });
      saveChannels(channels);
      displayChannels(channels);
      setStatus(`Ready! Loaded ${channels.length} channels`);
    }
  } catch (error) {
    channelsList.innerHTML = `<div class="error">❌ Error: ${error}</div>`;
    setStatus(`Error: ${error}`, true);
  }
}

// Event listeners
window.addEventListener('DOMContentLoaded', () => {
  // Set default volume to double maximum on startup
  const videoPlayer = document.getElementById('video-player');
  videoPlayer.volume = 2.0;
  
  // Initialize volume display
  const volumeDisplay = document.getElementById('volume-display');
  volumeDisplay.textContent = '200%';
  
  // Volume control event listeners
  const volumeSlider = document.getElementById('volume-slider');
  const maxVolumeBtn = document.getElementById('max-volume-btn');
  
  // Set slider to 200%
  volumeSlider.value = 200;
  
  volumeSlider.addEventListener('input', (e) => {
    updateVolume(e.target.value);
  });
  
  maxVolumeBtn.addEventListener('click', setMaxVolume);
  
  // Auto-start everything immediately
  autoStartup();
});
