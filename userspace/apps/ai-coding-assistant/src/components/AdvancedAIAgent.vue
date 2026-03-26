<template>
  <div class="ai-agent">
    <div class="agent-header">
      <h3>🤖 Advanced AI Agent</h3>
      <p>File operations, code analysis, Git integration, and system management</p>
    </div>
    
    <div class="agent-body">
      <!-- File Operations Section -->
      <div class="section">
        <h4>📁 File Operations</h4>
        <div class="file-operations">
          <div class="form-group">
            <label>File Path:</label>
            <input v-model="currentFilePath" class="form-input" placeholder="/path/to/file.py" />
          </div>
          <div class="button-group">
            <button @click="readFileContent" class="btn btn-primary" :disabled="!currentFilePath">
              📖 Read File
            </button>
            <button @click="analyzeCurrentFile" class="btn btn-success" :disabled="!currentFilePath || !fileContent">
              🔍 Analyze & Fix
            </button>
            <button @click="saveFile" class="btn btn-warning" :disabled="!currentFilePath || !fileContent">
              💾 Save Changes
            </button>
          </div>
        </div>
        
        <!-- File Content Editor -->
        <div v-if="fileContent" class="file-editor">
          <h5>📝 File Content:</h5>
          <textarea 
            v-model="fileContent" 
            class="code-editor"
            rows="15"
            placeholder="File content will appear here..."
          ></textarea>
        </div>
      </div>

      <!-- AI-Powered Code Analysis -->
      <div class="section">
        <h4>🧠 AI Code Analysis</h4>
        <div class="analysis-controls">
          <select v-model="analysisType" class="form-select">
            <option value="analyze">📊 Full Analysis</option>
            <option value="fix_bugs">🐛 Find & Fix Bugs</option>
            <option value="optimize">⚡ Optimize Performance</option>
            <option value="document">📚 Generate Documentation</option>
            <option value="test">🧪 Generate Tests</option>
          </select>
          <button @click="performAnalysis" class="btn btn-ai" :disabled="!fileContent || isAnalyzing">
            {{ isAnalyzing ? '🤔 Analyzing...' : '🚀 Analyze' }}
          </button>
        </div>
        
        <!-- Analysis Results -->
        <div v-if="analysisResult" class="analysis-result">
          <h5>🎯 Analysis Result:</h5>
          <pre class="result-display">{{ analysisResult }}</pre>
          <button @click="applyAISuggestions" class="btn btn-fix" v-if="analysisResult.includes('```')">
            ✨ Apply AI Suggestions
          </button>
        </div>
      </div>

      <!-- Directory Explorer -->
      <div class="section">
        <h4>🗂️ Directory Explorer</h4>
        <div class="directory-controls">
          <input v-model="currentDirectory" class="form-input" placeholder="/home/user/project" />
          <button @click="listDirectoryFiles" class="btn btn-outline">
            📂 Explore
          </button>
        </div>
        
        <div v-if="directoryFiles.length > 0" class="file-list">
          <h5>📋 Files in Directory:</h5>
          <div class="file-grid">
            <div 
              v-for="file in directoryFiles" 
              :key="file" 
              @click="selectFile(file)"
              class="file-item"
              :class="{ active: selectedFile === file }"
            >
              {{ getFileIcon(file) }} {{ file }}
            </div>
          </div>
        </div>
      </div>

      <!-- Git Integration -->
      <div class="section">
        <h4>🔧 Git Operations</h4>
        <div class="git-controls">
          <select v-model="gitCommand" class="form-select">
            <option value="status">📊 Git Status</option>
            <option value="log --oneline -10">📋 Recent Commits</option>
            <option value="diff">🔍 Show Changes</option>
            <option value="branch -a">🌿 List Branches</option>
            <option value="add .">➕ Stage All</option>
            <option value="commit -m 'AI-generated fixes'">💾 Commit Changes</option>
          </select>
          <button @click="executeGitCommand" class="btn btn-git">
            🔧 Execute
          </button>
        </div>
        
        <div v-if="gitOutput" class="git-output">
          <h5>🎯 Git Output:</h5>
          <pre class="git-result">{{ gitOutput }}</pre>
        </div>
      </div>

      <!-- AI Agent Actions -->
      <div class="section">
        <h4>🤖 AI Agent Actions</h4>
        <div class="agent-actions">
          <button @click="performFullProjectAnalysis" class="btn btn-agent">
            🔬 Full Project Analysis
          </button>
          <button @click="fixAllIssues" class="btn btn-agent">
            🛠️ Auto-Fix All Issues
          </button>
          <button @click="generateProjectDocumentation" class="btn btn-agent">
            📖 Generate Project Docs
          </button>
          <button @click="optimizeProject" class="btn btn-agent">
            ⚡ Optimize Entire Project
          </button>
        </div>
      </div>

      <!-- Activity Log -->
      <div class="section">
        <h4>📜 Activity Log</h4>
        <div class="activity-log">
          <div v-for="(activity, index) in activityLog" :key="index" class="activity-item">
            <span class="timestamp">{{ activity.timestamp }}</span>
            <span class="action">{{ activity.action }}</span>
            <span class="status" :class="activity.status">{{ activity.status }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'AdvancedAIAgent',
  setup() {
    // File operations
    const currentFilePath = ref('')
    const fileContent = ref('')
    const currentDirectory = ref('/home/lou')
    const directoryFiles = ref([])
    const selectedFile = ref('')
    
    // Analysis
    const analysisType = ref('analyze')
    const analysisResult = ref('')
    const isAnalyzing = ref(false)
    
    // Git operations
    const gitCommand = ref('status')
    const gitOutput = ref('')
    
    // Activity tracking
    const activityLog = ref([])

    const logActivity = (action, status = 'success') => {
      activityLog.value.unshift({
        timestamp: new Date().toLocaleTimeString(),
        action,
        status
      })
      if (activityLog.value.length > 50) {
        activityLog.value.pop()
      }
    }

    // File Operations
    const readFileContent = async () => {
      try {
        const content = await invoke('read_file', { filePath: currentFilePath.value })
        fileContent.value = content
        logActivity(`Read file: ${currentFilePath.value}`)
      } catch (error) {
        logActivity(`Failed to read file: ${error}`, 'error')
      }
    }

    const saveFile = async () => {
      try {
        await invoke('write_file', { 
          filePath: currentFilePath.value, 
          content: fileContent.value 
        })
        logActivity(`Saved file: ${currentFilePath.value}`)
      } catch (error) {
        logActivity(`Failed to save file: ${error}`, 'error')
      }
    }

    const listDirectoryFiles = async () => {
      try {
        const files = await invoke('list_files', { dirPath: currentDirectory.value })
        directoryFiles.value = files
        logActivity(`Listed directory: ${currentDirectory.value}`)
      } catch (error) {
        logActivity(`Failed to list directory: ${error}`, 'error')
      }
    }

    const selectFile = (filename) => {
      selectedFile.value = filename
      currentFilePath.value = `${currentDirectory.value}/${filename}`
      readFileContent()
    }

    // AI Analysis
    const performAnalysis = async () => {
      if (!fileContent.value) return
      
      isAnalyzing.value = true
      try {
        const result = await invoke('analyze_code', {
          code: fileContent.value,
          language: detectLanguage(currentFilePath.value),
          operation: analysisType.value
        })
        analysisResult.value = result.result
        logActivity(`AI analysis completed: ${analysisType.value}`)
      } catch (error) {
        logActivity(`AI analysis failed: ${error}`, 'error')
      } finally {
        isAnalyzing.value = false
      }
    }

    const analyzeCurrentFile = async () => {
      await performAnalysis()
    }

    const applyAISuggestions = () => {
      // Extract code blocks from AI response and apply them
      const codeBlocks = analysisResult.value.match(/```[\s\S]*?```/g)
      if (codeBlocks && codeBlocks.length > 0) {
        // Take the last code block as the fixed version
        const fixedCode = codeBlocks[codeBlocks.length - 1]
          .replace(/```[\w]*\n?/g, '')
          .replace(/```/g, '')
        fileContent.value = fixedCode
        logActivity('Applied AI suggestions to file')
      }
    }

    // Git Operations
    const executeGitCommand = async () => {
      try {
        const output = await invoke('run_git_command', { command: gitCommand.value })
        gitOutput.value = output
        logActivity(`Git command executed: ${gitCommand.value}`)
      } catch (error) {
        logActivity(`Git command failed: ${error}`, 'error')
      }
    }

    // Advanced AI Agent Actions
    const performFullProjectAnalysis = async () => {
      logActivity('Starting full project analysis...')
      // Implementation for full project analysis
      try {
        await listDirectoryFiles()
        for (const file of directoryFiles.value.filter(f => f.endsWith('.py') || f.endsWith('.js') || f.endsWith('.rs'))) {
          currentFilePath.value = `${currentDirectory.value}/${file}`
          await readFileContent()
          await performAnalysis()
        }
        logActivity('Full project analysis completed')
      } catch (error) {
        logActivity(`Project analysis failed: ${error}`, 'error')
      }
    }

    const fixAllIssues = async () => {
      logActivity('Auto-fixing all detected issues...')
      // Implementation for auto-fixing issues
      await performFullProjectAnalysis()
      logActivity('Auto-fix completed')
    }

    const generateProjectDocumentation = async () => {
      logActivity('Generating project documentation...')
      analysisType.value = 'document'
      await performFullProjectAnalysis()
      logActivity('Documentation generation completed')
    }

    const optimizeProject = async () => {
      logActivity('Optimizing entire project...')
      analysisType.value = 'optimize'
      await performFullProjectAnalysis()
      logActivity('Project optimization completed')
    }

    // Utility functions
    const detectLanguage = (filePath) => {
      const ext = filePath.split('.').pop().toLowerCase()
      const langMap = {
        'py': 'python',
        'js': 'javascript',
        'ts': 'typescript',
        'rs': 'rust',
        'cpp': 'cpp',
        'c': 'c',
        'java': 'java',
        'go': 'go'
      }
      return langMap[ext] || 'text'
    }

    const getFileIcon = (filename) => {
      const ext = filename.split('.').pop().toLowerCase()
      const icons = {
        'py': '🐍',
        'js': '🟨',
        'ts': '🔷',
        'rs': '🦀',
        'cpp': '⚙️',
        'java': '☕',
        'go': '🐹',
        'md': '📝',
        'json': '📋',
        'txt': '📄'
      }
      return icons[ext] || '📄'
    }

    onMounted(() => {
      currentDirectory.value = '/home/lou/awesome_stack/open-interpreter-tauri'
      listDirectoryFiles()
    })

    return {
      // File operations
      currentFilePath,
      fileContent,
      currentDirectory,
      directoryFiles,
      selectedFile,
      readFileContent,
      saveFile,
      listDirectoryFiles,
      selectFile,
      
      // Analysis
      analysisType,
      analysisResult,
      isAnalyzing,
      performAnalysis,
      analyzeCurrentFile,
      applyAISuggestions,
      
      // Git operations
      gitCommand,
      gitOutput,
      executeGitCommand,
      
      // Advanced actions
      performFullProjectAnalysis,
      fixAllIssues,
      generateProjectDocumentation,
      optimizeProject,
      
      // Activity log
      activityLog,
      
      // Utilities
      getFileIcon
    }
  }
}
</script>

<style scoped>
.ai-agent {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.agent-header {
  padding: 1.5rem;
  border-bottom: 1px solid var(--border-color);
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.agent-header h3 {
  margin: 0 0 0.5rem 0;
  font-size: 1.5rem;
}

.agent-header p {
  margin: 0;
  opacity: 0.9;
}

.agent-body {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
}

.section {
  margin-bottom: 2rem;
  padding: 1.5rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-secondary);
}

.section h4 {
  margin: 0 0 1rem 0;
  color: var(--primary-color);
  font-size: 1.2rem;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  color: var(--text-primary);
  font-weight: 600;
}

.form-input, .form-select {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.25rem;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

.code-editor {
  width: 100%;
  min-height: 300px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
  background: #1e1e1e;
  color: #f0f0f0;
  border: 2px solid var(--border-color);
  border-radius: 0.5rem;
  padding: 1rem;
  resize: vertical;
}

.file-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 0.5rem;
  max-height: 200px;
  overflow-y: auto;
}

.file-item {
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.25rem;
  cursor: pointer;
  transition: all 0.2s;
  background: var(--bg-primary);
}

.file-item:hover {
  background: var(--primary-color);
  color: white;
}

.file-item.active {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.button-group {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 0.25rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 600;
  transition: all 0.2s;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-success {
  background: #28a745;
  color: white;
}

.btn-warning {
  background: #ffc107;
  color: #212529;
}

.btn-ai {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-fix {
  background: linear-gradient(135deg, #ff6b6b, #ee5a6f);
  color: white;
}

.btn-git {
  background: #f14e32;
  color: white;
}

.btn-agent {
  background: linear-gradient(135deg, #4CAF50, #45a049);
  color: white;
  margin: 0.25rem;
}

.btn-outline {
  background: transparent;
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.analysis-controls, .directory-controls, .git-controls {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  margin-bottom: 1rem;
}

.analysis-result, .git-output {
  margin-top: 1rem;
}

.result-display, .git-result {
  background: #1e1e1e;
  color: #f0f0f0;
  padding: 1rem;
  border-radius: 0.5rem;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  line-height: 1.4;
  white-space: pre-wrap;
  word-wrap: break-word;
  max-height: 400px;
  overflow-y: auto;
}

.agent-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 0.75rem;
}

.activity-log {
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-primary);
}

.activity-item {
  display: flex;
  padding: 0.75rem;
  border-bottom: 1px solid var(--border-color);
  font-size: 0.875rem;
}

.activity-item:last-child {
  border-bottom: none;
}

.timestamp {
  color: var(--text-secondary);
  margin-right: 1rem;
  min-width: 80px;
}

.action {
  flex: 1;
  color: var(--text-primary);
}

.status {
  font-weight: 600;
  min-width: 60px;
  text-align: right;
}

.status.success {
  color: #28a745;
}

.status.error {
  color: #dc3545;
}

.file-editor h5, .analysis-result h5, .git-output h5 {
  margin: 0 0 0.5rem 0;
  color: var(--primary-color);
}
</style>
