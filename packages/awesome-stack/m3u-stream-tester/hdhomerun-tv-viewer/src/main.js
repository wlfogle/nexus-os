const { invoke } = window.__TAURI__.core;

let currentDeviceId = null;
let channels = [];

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
    
    if (devices.length === 0) {
      setStatus('No HDHomeRun devices found', true);
      return;
    }
    
    // Auto-select first device
    const device = devices[0];
    currentDeviceId = device.id;
    setStatus(`Found device: ${device.id} - ${device.model}`);
    
    // Auto-load channels
    await scanChannels();
  } catch (error) {
    setStatus(`Error discovering devices: ${error}`, true);
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
    channels = await invoke('scan_channels', { deviceId: currentDeviceId });
    
    if (channels.length === 0) {
      setStatus('No channels found', true);
      return;
    }
    
    displayChannels(channels);
    setStatus(`Loaded ${channels.length} channel(s)`);
  } catch (error) {
    setStatus(`Error loading channels: ${error}`, true);
  }
}

// Display channels function
function displayChannels(channelData) {
  const channelsList = document.getElementById('channels-list');
  channelsList.innerHTML = '';
  
  channelData.forEach((channel, index) => {
    const channelEl = document.createElement('div');
    channelEl.className = 'channel-item';
    
    // Create individual elements
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
    
    // Add click handler
    watchButton.addEventListener('click', () => {
      const streamUrl = channel.stream_url || `http://192.168.12.215:5004/auto/v${channel.number}`;
      playChannel(streamUrl, channel.number, channel.name);
    });
    
    // Assemble elements
    channelInfo.appendChild(channelNumber);
    channelInfo.appendChild(channelName);
    channelInfo.appendChild(channelEpg);
    
    channelEl.appendChild(channelInfo);
    channelEl.appendChild(watchButton);
    
    channelsList.appendChild(channelEl);
    
    // Load EPG
    setTimeout(() => {
      loadEPG(channel.number, index);
    }, 100 + (index * 50));
  });
}

function loadEPG(channelNumber, index) {
  const epgElement = document.getElementById(`epg-${index}`);
  if (!epgElement) return;
  
  const hour = new Date().getHours();
  let program = 'Current Programming';
  
  if (hour >= 6 && hour < 12) program = 'Morning Shows';
  else if (hour >= 12 && hour < 17) program = 'Afternoon Programming';
  else if (hour >= 17 && hour < 22) program = 'Evening News';
  else program = 'Late Night Programming';
  
  epgElement.textContent = `${hour}:00 - ${program}`;
  epgElement.classList.add('epg-loaded');
}

function playChannel(streamUrl, channelNumber, channelName) {
  console.log(`Playing ${channelNumber} - ${channelName}: ${streamUrl}`);
  
  const videoPlayer = document.getElementById('video-player');
  const currentChannelEl = document.getElementById('current-channel');
  
  // Set volume to 200%
  videoPlayer.volume = 2.0;
  
  // Set source
  videoPlayer.src = streamUrl;
  videoPlayer.load();
  
  // Update display
  if (currentChannelEl) {
    currentChannelEl.innerHTML = `
      <h3>Now Playing:</h3>
      <p><strong>${channelNumber} - ${channelName}</strong></p>
      <p>Stream: ${streamUrl}</p>
    `;
  }
}

// Volume control functions
function updateVolume(value) {
  const videoPlayer = document.getElementById('video-player');
  const volumeDisplay = document.getElementById('volume-display');
  
  videoPlayer.volume = value / 100;
  volumeDisplay.textContent = value + '%';
}

function setMaxVolume() {
  const videoPlayer = document.getElementById('video-player');
  const volumeSlider = document.getElementById('volume-slider');
  const volumeDisplay = document.getElementById('volume-display');
  
  videoPlayer.volume = 2.0;
  volumeSlider.value = 200;
  volumeDisplay.textContent = '200%';
  
  setStatus('Volume set to maximum (200%)');
}

// Event listeners
window.addEventListener('DOMContentLoaded', () => {
  // Set default volume to maximum on startup
  const videoPlayer = document.getElementById('video-player');
  videoPlayer.volume = 2.0;
  
  // Volume control event listeners
  const volumeSlider = document.getElementById('volume-slider');
  const maxVolumeBtn = document.getElementById('max-volume-btn');
  
  volumeSlider.addEventListener('input', (e) => {
    updateVolume(e.target.value);
  });
  
  maxVolumeBtn.addEventListener('click', setMaxVolume);
  
  // Auto-discover devices on startup
  discoverDevices();
});
