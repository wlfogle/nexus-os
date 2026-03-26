<template>
  <div id="app">
    <!-- Top Navigation Bar -->
    <nav class="top-nav">
      <div class="nav-brand">
        <div class="logo">🤖</div>
        <h1>AI Coding Assistant</h1>
      </div>
      <div class="nav-controls">
        <ConnectionStatus />
        <ModelSelector v-model="selectedModel" />
        <button class="settings-btn" @click="showSettings = !showSettings" :class="{ active: showSettings }">
          ⚙️ Settings
        </button>
      </div>
    </nav>

    <!-- Settings Panel -->
    <Settings v-if="showSettings" @close="showSettings = false" />

    <!-- Main Content -->
    <div class="main-container">
      <!-- Sidebar Navigation -->
      <aside class="sidebar">
        <nav class="sidebar-nav">
          <button 
            v-for="tab in tabs" 
            :key="tab.id"
            @click="activeTab = tab.id" 
            :class="['nav-item', { active: activeTab === tab.id }]"
            :title="tab.description"
          >
            <span class="nav-icon">{{ tab.icon }}</span>
            <span class="nav-label">{{ tab.name }}</span>
          </button>
        </nav>
        
        <!-- Quick Actions -->
        <div class="quick-actions">
          <button class="quick-action" @click="clearAllData" title="Clear All Data">
            🗑️
          </button>
          <button class="quick-action" @click="exportData" title="Export Data">
            📤
          </button>
          <button class="quick-action" @click="importData" title="Import Data">
            📥
          </button>
        </div>
      </aside>

      <!-- Main Content Area -->
      <main class="content-area">
        <!-- Conversation Tab -->
        <ConversationTab 
          v-if="activeTab === 'conversation'"
          :selected-model="selectedModel"
          @model-change="selectedModel = $event"
        />
        
        <!-- Code Analyzer Tab -->
        <OptimizedCodeAnalyzer 
          v-if="activeTab === 'analyzer'"
          :selected-model="selectedModel"
        />
        
        <!-- File Manager Tab -->
        <FileManager 
          v-if="activeTab === 'files'"
        />
        
        <!-- AI Assistant Tab -->
        <AIAssistant 
          v-if="activeTab === 'assistant'"
          :selected-model="selectedModel"
        />
        
        <!-- Advanced AI Agent Tab -->
        <AdvancedAIAgent 
          v-if="activeTab === 'agent'"
          :selected-model="selectedModel"
        />
      </main>
    </div>

    <!-- Status Bar -->
    <footer class="status-bar">
      <div class="status-left">
        <span class="status-item">🟢 Connected</span>
        <span class="status-item">Model: {{ selectedModel }}</span>
        <span class="status-item">Tab: {{ getCurrentTabName() }}</span>
      </div>
      <div class="status-right">
        <span class="status-item">{{ getCurrentTime() }}</span>
      </div>
    </footer>
  </div>
</template>

<script>
import { ref, onMounted, onUnmounted } from 'vue'
import ConnectionStatus from './components/ConnectionStatus.vue'
import ModelSelector from './components/ModelSelector.vue'
import Settings from './components/Settings.vue'
import ConversationTab from './components/ConversationTab.vue'
import OptimizedCodeAnalyzer from './components/OptimizedCodeAnalyzer.vue'
import FileManager from './components/FileManager.vue'
import AIAssistant from './components/AIAssistant.vue'
import AdvancedAIAgent from './components/AdvancedAIAgent.vue'

export default {
  name: 'App',
  components: {
    ConnectionStatus,
    ModelSelector,
    Settings,
    ConversationTab,
    OptimizedCodeAnalyzer,
    FileManager,
    AIAssistant,
    AdvancedAIAgent
  },
  setup() {
    // State management
    const activeTab = ref('conversation')
    const selectedModel = ref('codellama:7b')
    const showSettings = ref(false)
    const currentTime = ref(new Date().toLocaleTimeString())
    
    // Tab configuration
    const tabs = ref([
      {
        id: 'conversation',
        name: 'Chat',
        icon: '💬',
        description: 'AI-powered conversations and assistance'
      },
      {
        id: 'analyzer',
        name: 'Analyzer',
        icon: '🔍',
        description: 'Advanced code analysis and optimization'
      },
      {
        id: 'files',
        name: 'Files',
        icon: '📁',
        description: 'File management and operations'
      },
      {
        id: 'assistant',
        name: 'Assistant',
        icon: '🤖',
        description: 'AI assistant for coding tasks'
      },
      {
        id: 'agent',
        name: 'Agent',
        icon: '🚀',
        description: 'Advanced AI agent with multi-step reasoning'
      }
    ])
    
    // Time update interval
    let timeInterval = null
    
    // Lifecycle methods
    onMounted(() => {
      // Update time every second
      timeInterval = setInterval(() => {
        currentTime.value = new Date().toLocaleTimeString()
      }, 1000)
      
      // Load saved preferences
      loadPreferences()
    })
    
    onUnmounted(() => {
      if (timeInterval) {
        clearInterval(timeInterval)
      }
    })
    
    // Methods
    const getCurrentTabName = () => {
      const tab = tabs.value.find(t => t.id === activeTab.value)
      return tab ? tab.name : 'Unknown'
    }
    
    const getCurrentTime = () => {
      return currentTime.value
    }
    
    const clearAllData = async () => {
      if (confirm('Are you sure you want to clear all data? This action cannot be undone.')) {
        // Clear localStorage
        localStorage.clear()
        // Reset state
        activeTab.value = 'conversation'
        selectedModel.value = 'codellama:7b'
        showSettings.value = false
        // Emit clear event to all components
        window.dispatchEvent(new CustomEvent('clear-all-data'))
      }
    }
    
    const exportData = () => {
      try {
        const data = {
          preferences: {
            activeTab: activeTab.value,
            selectedModel: selectedModel.value
          },
          conversations: JSON.parse(localStorage.getItem('conversations') || '[]'),
          codeAnalysis: JSON.parse(localStorage.getItem('codeAnalysis') || '[]'),
          timestamp: new Date().toISOString()
        }
        
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = `ai-assistant-export-${new Date().toISOString().split('T')[0]}.json`
        document.body.appendChild(a)
        a.click()
        document.body.removeChild(a)
        URL.revokeObjectURL(url)
      } catch (error) {
        console.error('Export failed:', error)
        alert('Export failed. Please try again.')
      }
    }
    
    const importData = () => {
      const input = document.createElement('input')
      input.type = 'file'
      input.accept = '.json'
      input.onchange = (event) => {
        const file = event.target.files[0]
        if (file) {
          const reader = new FileReader()
          reader.onload = (e) => {
            try {
              const data = JSON.parse(e.target.result)
              if (data.preferences) {
                activeTab.value = data.preferences.activeTab || 'conversation'
                selectedModel.value = data.preferences.selectedModel || 'codellama:7b'
              }
              if (data.conversations) {
                localStorage.setItem('conversations', JSON.stringify(data.conversations))
              }
              if (data.codeAnalysis) {
                localStorage.setItem('codeAnalysis', JSON.stringify(data.codeAnalysis))
              }
              alert('Data imported successfully!')
              // Refresh components
              window.dispatchEvent(new CustomEvent('data-imported'))
            } catch (error) {
              console.error('Import failed:', error)
              alert('Import failed. Please check the file format.')
            }
          }
          reader.readAsText(file)
        }
      }
      input.click()
    }
    
    const loadPreferences = () => {
      try {
        const savedTab = localStorage.getItem('activeTab')
        const savedModel = localStorage.getItem('selectedModel')
        
        if (savedTab && tabs.value.find(t => t.id === savedTab)) {
          activeTab.value = savedTab
        }
        
        if (savedModel) {
          selectedModel.value = savedModel
        }
      } catch (error) {
        console.error('Failed to load preferences:', error)
      }
    }
    
    const savePreferences = () => {
      try {
        localStorage.setItem('activeTab', activeTab.value)
        localStorage.setItem('selectedModel', selectedModel.value)
      } catch (error) {
        console.error('Failed to save preferences:', error)
      }
    }
    
    // Watch for changes and save preferences
    const watchActiveTab = (newTab) => {
      savePreferences()
    }
    
    const watchSelectedModel = (newModel) => {
      savePreferences()
    }
    
    return {
      // State
      activeTab,
      selectedModel,
      showSettings,
      tabs,
      
      // Methods
      getCurrentTabName,
      getCurrentTime,
      clearAllData,
      exportData,
      importData,
      
      // Watchers
      watchActiveTab,
      watchSelectedModel
    }
  },
  
  watch: {
    activeTab: 'watchActiveTab',
    selectedModel: 'watchSelectedModel'
  }
}</script>

<style scoped>
#app {
  padding: 20px;
  font-family: Arial, sans-serif;
}

.debug-info {
  position: fixed;
  top: 10px;
  right: 10px;
  background: #ff0000;
  color: white;
  padding: 10px;
  border-radius: 5px;
  z-index: 1000;
  font-weight: bold;
}

.simple-layout {
  display: flex;
  height: 600px;
  border: 1px solid #ccc;
  border-radius: 8px;
  overflow: hidden;
}

.sidebar {
  width: 200px;
  background: #f5f5f5;
  padding: 20px;
  border-right: 1px solid #ddd;
}

.sidebar button {
  display: block;
  width: 100%;
  padding: 10px;
  margin: 5px 0;
  border: 1px solid #ccc;
  background: white;
  cursor: pointer;
  border-radius: 4px;
}

.sidebar button:hover {
  background: #e9e9e9;
}

.sidebar button.active {
  background: #007bff;
  color: white;
}

.main-content {
  flex: 1;
  padding: 20px;
  background: white;
}

.chat-area {
  height: 400px;
  display: flex;
  flex-direction: column;
}

.messages {
  flex: 1;
  border: 1px solid #ddd;
  padding: 10px;
  overflow-y: auto;
  margin-bottom: 10px;
  background: #fafafa;
}

.message {
  margin: 5px 0;
  padding: 8px;
  background: white;
  border-radius: 4px;
  border: 1px solid #eee;
}

.message.user {
  background: #e3f2fd;
  border-color: #2196f3;
  margin-left: 50px;
}

.message.ai {
  background: #f3e5f5;
  border-color: #9c27b0;
  margin-right: 50px;
}

.message.system {
  background: #fff3e0;
  border-color: #ff9800;
  font-style: italic;
}

.message.error {
  background: #ffebee;
  border-color: #f44336;
  color: #d32f2f;
}

.message.typing {
  background: #f0f0f0;
  border-color: #999;
  font-style: italic;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}

.input-area {
  display: flex;
  gap: 10px;
}

.input-area input {
  flex: 1;
  padding: 10px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.input-area button {
  padding: 10px 20px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.input-area button:hover {
  background: #0056b3;
}
</style>
