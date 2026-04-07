const { invoke } = window.__TAURI__.core;
// Try different API paths for dialog
let dialogAPI;
try {
  dialogAPI = window.__TAURI__.plugins?.dialog || window.__TAURI__.dialog;
} catch (e) {
  console.error('Dialog API not found:', e);
}

const open = dialogAPI?.open;

let folderInput;
let outputInput;
let timeoutInput;
let concurrentInput;
let startButton;
let stopButton;
let clearButton;
let progressText;
let progressFill;
let totalStreams;
let testedStreams;
let workingStreams;
let failedStreams;
let logContent;
let categoriesContainer;
let saveByCategoryButton;

let isRunning = false;
let currentResults = [];
let currentCategories = [];

function formatTime() {
  return new Date().toLocaleTimeString();
}

function addLogEntry(message, type = 'info') {
  const entry = document.createElement('div');
  entry.className = `log-entry ${type}`;
  entry.textContent = `[${formatTime()}] ${message}`;
  logContent.appendChild(entry);
  logContent.scrollTop = logContent.scrollHeight;
}

function updateProgress(tested, total, working) {
  const percentage = total > 0 ? (tested / total) * 100 : 0;
  progressFill.style.width = `${percentage}%`;
  progressText.textContent = `Testing ${tested}/${total} streams (${working} working)`;
  
  totalStreams.textContent = total;
  testedStreams.textContent = tested;
  workingStreams.textContent = working;
  failedStreams.textContent = tested - working;
}

function resetProgress() {
  progressFill.style.width = '0%';
  progressText.textContent = 'Ready';
  totalStreams.textContent = '0';
  testedStreams.textContent = '0';
  workingStreams.textContent = '0';
  failedStreams.textContent = '0';
}

function setRunning(running) {
  isRunning = running;
  startButton.disabled = running;
  stopButton.disabled = !running;
  folderInput.disabled = running;
  outputInput.disabled = running;
  timeoutInput.disabled = running;
  concurrentInput.disabled = running;
}

async function selectFolder() {
  try {
    addLogEntry('Opening folder dialog...', 'info');
    console.log('Full Tauri API:', window.__TAURI__);
    console.log('Dialog API:', dialogAPI);
    console.log('Open function:', open);
    
    if (!open) {
      addLogEntry('Dialog API not available', 'error');
      return;
    }
    
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select folder containing M3U files'
    });
    
    console.log('Selected:', selected);
    
    if (selected) {
      folderInput.value = selected;
      addLogEntry(`Selected folder: ${selected}`);
    } else {
      addLogEntry('No folder selected', 'warning');
    }
  } catch (error) {
    console.error('Dialog error:', error);
    addLogEntry(`Error selecting folder: ${error}`, 'error');
  }
}

async function selectOutputFile() {
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      title: 'Select output file location',
      filters: [{
        name: 'M3U8 files',
        extensions: ['m3u8']
      }]
    });
    
    if (selected) {
      outputInput.value = selected;
      addLogEntry(`Selected output file: ${selected}`);
    }
  } catch (error) {
    addLogEntry(`Error selecting output file: ${error}`, 'error');
  }
}

async function startTesting() {
  const folderPath = folderInput.value.trim();
  const outputPath = outputInput.value.trim() || 'working_streams.m3u8';
  const timeoutSeconds = parseInt(timeoutInput.value) || 10;
  const maxConcurrent = parseInt(concurrentInput.value) || 10;
  
  if (!folderPath) {
    addLogEntry('Please select a folder containing M3U files', 'error');
    return;
  }
  
  setRunning(true);
  resetProgress();
  addLogEntry('Starting stream testing...', 'info');
  
  try {
    const results = await invoke('test_streams_from_folder', {
      folderPath,
      outputPath,
      timeoutSeconds,
      maxConcurrent
    });
    
    currentResults = results;
    const workingCount = results.filter(r => r.working).length;
    const totalCount = results.length;
    
    updateProgress(totalCount, totalCount, workingCount);
    
    addLogEntry(`Testing completed!`, 'success');
    addLogEntry(`Results: ${workingCount}/${totalCount} streams are working`, 'success');
    addLogEntry(`Working streams saved to: ${outputPath}`, 'success');
    
    // Log summary of detailed results (not every single one to avoid spam)
    const failedCount = totalCount - workingCount;
    addLogEntry(`Summary: ${workingCount} working, ${failedCount} failed streams`, 'info');
    
  } catch (error) {
    addLogEntry(`Error during testing: ${error}`, 'error');
  } finally {
    setRunning(false);
  }
}

async function stopTesting() {
  if (isRunning) {
    addLogEntry('Stopping test...', 'warning');
    try {
      await invoke('stop_testing');
      addLogEntry('Stop signal sent', 'warning');
    } catch (error) {
      addLogEntry(`Error stopping: ${error}`, 'error');
    }
  }
}

function clearLog() {
  logContent.innerHTML = '';
  addLogEntry('Log cleared', 'info');
}

function displayCategories(categories) {
  console.log('=== displayCategories called ===');
  console.log('Categories received:', categories);
  console.log('Categories length:', categories ? categories.length : 'undefined');
  
  const container = document.getElementById('categories-container');
  console.log('Container element:', container);
  
  if (!container) {
    console.error('Categories container not found!');
    return;
  }
  
  container.innerHTML = '';
  
  if (!categories || categories.length === 0) {
    console.log('No categories to display');
    container.innerHTML = '<p class="no-categories">No categories found in the tested streams.</p>';
    saveByCategoryButton.disabled = true;
    return;
  }
  
  console.log('Processing', categories.length, 'categories');
  
  categories.forEach((category, index) => {
    console.log(`Processing category ${index}:`, category);
    
    const categoryDiv = document.createElement('div');
    categoryDiv.className = 'category-item';
    
    categoryDiv.innerHTML = `
      <div class="category-name">${category.name || 'Unknown'}</div>
      <div class="category-stats">
        <span class="category-stat total">Total: ${category.total || 0}</span>
        <span class="category-stat working">Working: ${category.working || 0}</span>
        <span class="category-stat failed">Failed: ${category.failed || 0}</span>
      </div>
    `;
    
    container.appendChild(categoryDiv);
    
    console.log(`Added category ${index} to DOM:`, categoryDiv);
  });
  
  console.log('Final container contents:', container.innerHTML);
  console.log('=== displayCategories completed ===');
  
  saveByCategoryButton.disabled = false;
}

async function saveByCategoryAction() {
  if (currentResults.length === 0) {
    addLogEntry('No results to save', 'error');
    return;
  }
  
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select folder to save category files'
    });
    
    if (selected) {
      addLogEntry('Saving streams by category...', 'info');
      
      const savedFiles = await invoke('save_streams_by_category', {
        streams: currentResults,
        outputFolder: selected,
        onlyWorking: true
      });
      
      addLogEntry(`Saved ${savedFiles.length} category files:`, 'success');
      savedFiles.forEach(file => {
        addLogEntry(`- ${file}`, 'success');
      });
    }
  } catch (error) {
    addLogEntry(`Error saving by category: ${error}`, 'error');
  }
}

window.addEventListener('DOMContentLoaded', () => {
  // Get DOM elements
  folderInput = document.querySelector('#folder-input');
  outputInput = document.querySelector('#output-input');
  timeoutInput = document.querySelector('#timeout-input');
  concurrentInput = document.querySelector('#concurrent-input');
  startButton = document.querySelector('#start-button');
  stopButton = document.querySelector('#stop-button');
  clearButton = document.querySelector('#clear-button');
  progressText = document.querySelector('#progress-text');
  progressFill = document.querySelector('#progress-fill');
  totalStreams = document.querySelector('#total-streams');
  testedStreams = document.querySelector('#tested-streams');
  workingStreams = document.querySelector('#working-streams');
  failedStreams = document.querySelector('#failed-streams');
  logContent = document.querySelector('#log-content');
  categoriesContainer = document.querySelector('#categories-container');
  saveByCategoryButton = document.querySelector('#save-by-category');
  
  // Set up event listeners
  document.querySelector('#folder-button').addEventListener('click', selectFolder);
  document.querySelector('#output-button').addEventListener('click', selectOutputFile);
  startButton.addEventListener('click', startTesting);
  stopButton.addEventListener('click', stopTesting);
  clearButton.addEventListener('click', clearLog);
  
  // Listen for progress events from Tauri
  window.__TAURI__.event.listen('progress', (event) => {
    const progress = event.payload;
    updateProgress(progress.tested, progress.total, progress.working);
    if (progress.current_stream) {
      addLogEntry(`Testing: ${progress.current_stream}`, 'info');
    }
  });
  
  // Listen for category events from Tauri
  window.__TAURI__.event.listen('categories', (event) => {
    console.log('Categories event received:', event);
    console.log('Categories payload:', event.payload);
    const summary = event.payload;
    console.log('Summary categories:', summary.categories);
    currentCategories = summary.categories;
    displayCategories(summary.categories);
    addLogEntry(`Found ${summary.categories.length} categories`, 'info');
  });
  
  // Add save by category button handler
  saveByCategoryButton.addEventListener('click', saveByCategoryAction);
  
  // Set default output filename
  outputInput.value = 'working_streams.m3u8';
  
  // Initial log message
  addLogEntry('M3U Stream Tester ready', 'info');
});
