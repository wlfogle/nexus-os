<template>
  <div class="settings">
    <div class="settings-header">
      <h3>⚙️ Settings</h3>
    </div>
    
    <div class="settings-body">
      <div class="settings-section">
        <h4>AI Container Configuration</h4>
        <div class="form-group">
          <label class="form-label">Container IP Address</label>
          <input 
            v-model="containerIp" 
            type="text" 
            class="form-input"
            placeholder="192.168.122.172"
          />
        </div>
        <div class="form-group">
          <label class="form-label">Container Port</label>
          <input 
            v-model="containerPort" 
            type="number" 
            class="form-input"
            placeholder="11434"
          />
        </div>
        <button class="btn btn-primary" @click="testConnection">
          Test Connection
        </button>
      </div>

      <div class="settings-section">
        <h4>Analysis Settings</h4>
        <div class="form-group">
          <label class="form-label">Default Language</label>
          <select v-model="defaultLanguage" class="form-input">
            <option value="auto">Auto Detect</option>
            <option value="python">Python</option>
            <option value="javascript">JavaScript</option>
            <option value="rust">Rust</option>
            <option value="go">Go</option>
            <option value="java">Java</option>
            <option value="c++">C++</option>
          </select>
        </div>
        <div class="form-group">
          <label class="form-label">
            <input 
              v-model="autoAnalyze" 
              type="checkbox"
              style="margin-right: 0.5rem;"
            />
            Auto-analyze on code change
          </label>
        </div>
      </div>

      <div class="settings-section">
        <h4>🤖 AI Assistant</h4>
        <div class="form-group">
          <label class="form-label">Ask the AI anything</label>
          <textarea 
            v-model="aiQuery" 
            class="form-input ai-textarea"
            placeholder="Ask about system optimization, troubleshooting, configuration, or anything else..."
            rows="3"
          ></textarea>
        </div>
        <button class="btn btn-primary" @click="askAI" :disabled="!aiQuery.trim()">
          Ask AI
        </button>
        <div v-if="aiResponse" class="response-box">
          <h5>AI Response:</h5>
          <pre>{{ aiResponse }}</pre>
        </div>
      </div>

      <div class="settings-section">
        <h4>About</h4>
        <div class="about-info">
          <p><strong>AI Coding Assistant</strong></p>
          <p>Version: 1.0.0</p>
          <p>Built with Tauri + Vue 3 + Rust</p>
          <p>Connects to AI LXC container at {{ containerIp }}:{{ containerPort }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'Settings',
  setup() {
    const containerIp = ref('192.168.122.172')
    const containerPort = ref(11434)
    const defaultLanguage = ref('auto')
    const autoAnalyze = ref(false)
    const aiQuery = ref('')
    const aiResponse = ref('')

    const testConnection = async () => {
      try {
        const result = await invoke('check_ai_connection')
        if (result) {
          alert('✅ Connection successful!')
        } else {
          alert('❌ Connection failed. Check your settings.')
        }
      } catch (error) {
        alert('❌ Connection error: ' + error)
      }
    }

    const askAI = async () => {
      if (!aiQuery.value.trim()) return
      
      try {
        aiResponse.value = 'Thinking...'
        const result = await invoke('general_ai_query', { query: aiQuery.value })
        aiResponse.value = result
      } catch (error) {
        aiResponse.value = 'Error: ' + error
      }
    }

    return {
      containerIp,
      containerPort,
      defaultLanguage,
      autoAnalyze,
      aiQuery,
      aiResponse,
      testConnection,
      askAI
    }
  }
}
</script>

<style scoped>
.settings {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.settings-header {
  padding: 1.5rem;
  border-bottom: 1px solid var(--border-color);
}

.settings-header h3 {
  margin: 0;
  color: var(--primary-color);
}

.settings-body {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
}

.settings-section {
  margin-bottom: 2rem;
  padding: 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-secondary);
}

.settings-section h4 {
  margin: 0 0 1rem 0;
  color: var(--text-primary);
}

.about-info p {
  margin: 0.5rem 0;
  color: var(--text-secondary);
}

.ai-textarea {
  min-height: 80px;
  resize: vertical;
  font-family: 'Courier New', monospace;
}

.response-box {
  margin-top: 1rem;
  padding: 1rem;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  max-height: 400px;
  overflow-y: auto;
}

.response-box h5 {
  margin: 0 0 0.5rem 0;
  color: var(--primary-color);
}

.response-box pre {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'Courier New', monospace;
  font-size: 0.9rem;
  line-height: 1.4;
  color: var(--text-primary);
}
</style>
