<template>
  <div class="file-manager">
    <div class="file-manager-header">
      <h3>📁 File Manager</h3>
      <button class="btn btn-secondary" @click="selectFolder">
        📂 Select Folder
      </button>
    </div>
    
    <div class="file-manager-body">
      <div v-if="!selectedPath" class="no-folder">
        <div class="placeholder-icon">📁</div>
        <p>Select a folder to browse files</p>
      </div>
      
      <div v-else class="folder-content">
        <div class="current-path">
          <strong>Path:</strong> {{ selectedPath }}
        </div>
        
        <div class="file-list">
          <div 
            v-for="file in files" 
            :key="file.name"
            :class="['file-item', { selected: selectedFile === file.name }]"
            @click="selectFile(file)"
          >
            <span class="file-icon">{{ getFileIcon(file.extension) }}</span>
            <span class="file-name">{{ file.name }}</span>
            <span class="file-size">{{ formatSize(file.size) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'

export default {
  name: 'FileManager',
  setup() {
    const selectedPath = ref('')
    const selectedFile = ref('')
    const files = ref([])

    const selectFolder = async () => {
      // In a real implementation, this would use Tauri's file dialog
      // For now, we'll simulate it
      selectedPath.value = '/path/to/project'
      files.value = [
        { name: 'main.py', extension: 'py', size: 1024 },
        { name: 'utils.js', extension: 'js', size: 2048 },
        { name: 'config.json', extension: 'json', size: 512 },
        { name: 'README.md', extension: 'md', size: 1536 }
      ]
    }

    const selectFile = (file) => {
      selectedFile.value = file.name
      // Emit event to load file content for analysis
      console.log('Selected file:', file.name)
    }

    const getFileIcon = (extension) => {
      const icons = {
        py: '🐍',
        js: '📜',
        ts: '📘',
        json: '📋',
        md: '📝',
        rs: '🦀',
        go: '🐹',
        java: '☕',
        cpp: '⚙️',
        c: '🔧'
      }
      return icons[extension] || '📄'
    }

    const formatSize = (bytes) => {
      if (bytes < 1024) return bytes + ' B'
      if (bytes < 1024 * 1024) return Math.round(bytes / 1024) + ' KB'
      return Math.round(bytes / (1024 * 1024)) + ' MB'
    }

    return {
      selectedPath,
      selectedFile,
      files,
      selectFolder,
      selectFile,
      getFileIcon,
      formatSize
    }
  }
}
</script>

<style scoped>
.file-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.file-manager-header {
  padding: 1.5rem;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.file-manager-header h3 {
  margin: 0;
  color: var(--primary-color);
}

.file-manager-body {
  flex: 1;
  padding: 1rem;
}

.no-folder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.current-path {
  padding: 0.75rem;
  background: var(--bg-secondary);
  border-radius: 0.375rem;
  margin-bottom: 1rem;
  font-size: 0.875rem;
}

.file-list {
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  overflow: hidden;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  border-bottom: 1px solid var(--border-color);
  cursor: pointer;
  transition: background-color 0.2s;
}

.file-item:hover {
  background: var(--bg-secondary);
}

.file-item.selected {
  background: var(--primary-color);
  color: white;
}

.file-item:last-child {
  border-bottom: none;
}

.file-icon {
  font-size: 1.25rem;
}

.file-name {
  flex: 1;
  font-weight: 500;
}

.file-size {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.file-item.selected .file-size {
  color: rgba(255, 255, 255, 0.8);
}
</style>
