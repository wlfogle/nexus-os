const { invoke } = window.__TAURI__.core;

// Simple channel data with EPG
const CHANNELS = [
  { number: '11.1', name: 'WHAS-HD', program: 'Evening News' },
  { number: '11.2', name: 'Crime', program: 'Criminal Minds' },
  { number: '11.3', name: 'Quest', program: 'How It\'s Made' },
  { number: '32.1', name: 'WLKY-HD', program: 'Local News' },
  { number: '32.2', name: 'ME TV', program: 'Classic TV' },
  { number: '41.1', name: 'WDRB', program: 'Fox News' },
  { number: '41.2', name: 'Ant.TV', program: 'Comedy Shows' },
  { number: '21.1', name: 'WBNA-DT', program: 'CW Programming' },
  { number: '21.2', name: 'StartTV', program: 'Action Series' },
  { number: '24.1', name: 'Laff', program: 'Comedy Movies' },
  { number: '24.2', name: 'DEFY', program: 'Adventure Shows' },
  { number: '50.1', name: 'TheWalk', program: 'Religious Programming' },
  { number: '50.2', name: 'AVoice', program: 'Talk Shows' },
  { number: '58.1', name: 'WBKI-CW', program: 'CW Shows' },
  { number: '58.2', name: 'COZI', program: 'Classic Movies' }
];

function createTVGuide() {
  const tvGuide = document.getElementById('tv-guide');
  const currentTime = new Date();
  const hour = currentTime.getHours();
  const timeSlot = `${hour}:00 - ${(hour + 1) % 24}:00`;
  
  let html = `<div class="time-header">📅 ${timeSlot}</div>`;
  
  CHANNELS.forEach(channel => {
    const streamUrl = `http://192.168.12.215:5004/auto/v${channel.number}`;
    
    html += `
      <div class="guide-item">
        <div class="channel-info">
          <span class="channel-number">${channel.number}</span>
          <span class="channel-name">${channel.name}</span>
        </div>
        <div class="program-info">
          <span class="program-name">${channel.program}</span>
          <span class="program-time">${timeSlot}</span>
        </div>
        <button class="watch-now-btn" onclick="watchChannel('${streamUrl}', '${channel.number}', '${channel.name}')">
          ▶️ Watch Now
        </button>
      </div>
    `;
  });
  
  tvGuide.innerHTML = html;
}

window.watchChannel = function(streamUrl, channelNumber, channelName) {
  console.log(`Playing ${channelNumber} - ${channelName}`);
  
  const videoPlayer = document.getElementById('video-player');
  const currentChannelEl = document.getElementById('current-channel');
  
  // Set volume to 200%
  videoPlayer.volume = 2.0;
  
  // Set source
  videoPlayer.src = streamUrl;
  videoPlayer.load();
  
  // Update display
  currentChannelEl.innerHTML = `
    <h3>▶️ Now Watching:</h3>
    <p><strong>${channelNumber} - ${channelName}</strong></p>
    <p>Stream: ${streamUrl}</p>
  `;
  
  console.log(`Started playing ${channelNumber}`);
};

// Volume controls
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
}

// Initialize
window.addEventListener('DOMContentLoaded', () => {
  console.log('=== INITIALIZING TV GUIDE ===');
  
  // Create TV guide immediately
  createTVGuide();
  
  // Set up volume controls
  const videoPlayer = document.getElementById('video-player');
  videoPlayer.volume = 2.0;
  
  const volumeSlider = document.getElementById('volume-slider');
  const maxVolumeBtn = document.getElementById('max-volume-btn');
  
  volumeSlider.addEventListener('input', (e) => {
    updateVolume(e.target.value);
  });
  
  maxVolumeBtn.addEventListener('click', setMaxVolume);
  
  console.log('=== TV GUIDE READY ===');
});
