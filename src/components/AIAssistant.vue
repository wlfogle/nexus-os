<template>
  <div class="ai-assistant">
    <div class="assistant-header">
      <h3>🤖 AI Assistant</h3>
      <p>Ask me anything about system optimization, troubleshooting, or general questions!</p>
    </div>
    
    <div class="assistant-body">
      <div class="query-section">
        <div class="form-group">
          <label class="form-label">Your Question</label>
          <textarea 
            v-model="aiQuery" 
            class="form-input ai-textarea"
            placeholder="Examples:
• How can I optimize my Garuda Linux system for better performance?
• What are the best security practices for Linux?
• How do I troubleshoot high CPU usage?
• Explain how to configure my firewall
• What's the best way to manage disk space?"
            rows="4"
          ></textarea>
        </div>
        <div class="button-group">
          <button class="btn btn-primary" @click="askAI" :disabled="!aiQuery.trim() || isLoading">
            {{ isLoading ? '🤔 Thinking...' : '💬 Ask AI' }}
          </button>
          <button class="btn btn-secondary" @click="clearConversation" v-if="conversation.length > 0">
            🗑️ Clear
          </button>
        </div>
      </div>

      <div class="conversation-section" v-if="conversation.length > 0">
        <h4>💬 Conversation</h4>
        <div class="conversation-history">
          <div v-for="(item, index) in conversation" :key="index" class="conversation-item">
            <div class="question">
              <strong>You:</strong> {{ item.question }}
            </div>
            <div class="response">
              <strong>AI:</strong>
              <pre>{{ item.response }}</pre>
            </div>
          </div>
        </div>
      </div>

      <div class="ai-actions-section">
        <h4>🤖 AI Actions</h4>
        <div class="action-grid">
          <button class="btn btn-action" @click="askWithExecution('Check my system performance and show me what processes are using the most resources')">🔍 System Analysis</button>
          <button class="btn btn-action" @click="askWithExecution('Show me my disk usage and help me find large files I can clean up')">💾 Disk Cleanup</button>
          <button class="btn btn-action" @click="askWithExecution('Check my memory usage and show what is consuming RAM')">🧠 Memory Check</button>
          <button class="btn btn-action" @click="askWithExecution('Show me my network connections and check for any issues')">🌐 Network Status</button>
          <button class="btn btn-action" @click="askWithExecution('Check system logs for any errors or warnings')">📋 System Logs</button>
          <button class="btn btn-action" @click="askWithExecution('Show me information about my hardware and drivers')">🖥️ Hardware Info</button>
        </div>
      </div>

      <div class="fix-code-section">
        <h4>🔧 Fix Code/Text</h4>
        <p class="section-description">Paste code, error messages, or text that needs fixing:</p>
        <div class="form-group">
          <textarea 
            v-model="codeToFix" 
            class="form-input code-textarea"
            placeholder="Paste your code, error message, or text here...

Examples:
• Broken shell script
• Error messages from terminal
• Configuration files
• Code snippets with bugs
• System logs with issues"
            rows="8"
          ></textarea>
        </div>
        <div class="button-group">
          <button class="btn btn-fix" @click="fixCode" :disabled="!codeToFix.trim() || isLoading">
            {{ isLoading ? '🔧 Analyzing...' : '🛠️ Fix' }}
          </button>
          <button class="btn btn-secondary" @click="clearFixSection" v-if="codeToFix.trim()">
            🗑️ Clear
          </button>
        </div>
      </div>

      <div class="quick-actions">
        <h4>🚀 Quick Actions</h4>
        <div class="action-buttons">
          <button class="btn btn-outline" @click="askQuickQuestion('How can I optimize my Garuda Linux system for better performance? Provide specific commands I can run.')">
            ⚡ System Optimization
          </button>
          <button class="btn btn-outline" @click="askQuickQuestion('What are common Linux security best practices? Include terminal commands.')">
            🔒 Security Tips
          </button>
          <button class="btn btn-outline" @click="askQuickQuestion('How do I troubleshoot high CPU/memory usage? Show me the commands.')">
            🔧 Troubleshooting
          </button>
          <button class="btn btn-outline" @click="askQuickQuestion('What are useful Linux terminal commands for system management?')">
            💻 Terminal Commands
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'AIAssistant',
  setup() {
    const aiQuery = ref('')
    const conversation = ref([])
    const isLoading = ref(false)
    
    // Fix code functionality
    const codeToFix = ref('')
    
    // Terminal functionality
    const terminalHistory = ref([])
    const currentCommand = ref('')
    const isExecuting = ref(false)

    const askAI = async () => {
      if (!aiQuery.value.trim() || isLoading.value) return
      
      const question = aiQuery.value.trim()
      isLoading.value = true
      
      try {
        const result = await invoke('general_ai_query', { query: question })
        
        conversation.value.push({
          question: question,
          response: result,
          timestamp: new Date()
        })
        
        aiQuery.value = '' // Clear the input
      } catch (error) {
        conversation.value.push({
          question: question,
          response: 'Error: ' + error,
          timestamp: new Date()
        })
      } finally {
        isLoading.value = false
      }
    }

    const askQuickQuestion = (question) => {
      aiQuery.value = question
      askAI()
    }

    const clearConversation = () => {
      conversation.value = []
      aiQuery.value = ''
    }

    // Terminal methods
    const executeCommand = async () => {
      if (!currentCommand.value.trim() || isExecuting.value) return
      
      const command = currentCommand.value.trim()
      isExecuting.value = true
      
      // Add command to history
      terminalHistory.value.push({
        type: 'command',
        content: command,
        timestamp: new Date()
      })
      
      try {
        const result = await invoke('execute_command', { command: command })
        
        // Add output to history
        terminalHistory.value.push({
          type: 'output',
          content: result,
          timestamp: new Date()
        })
      } catch (error) {
        // Add error to history
        terminalHistory.value.push({
          type: 'error',
          content: 'Error: ' + error,
          timestamp: new Date()
        })
      } finally {
        isExecuting.value = false
        currentCommand.value = ''
      }
    }

    const clearTerminal = () => {
      terminalHistory.value = []
    }

    // New method for asking AI to execute commands
    const askWithExecution = async (question) => {
      if (isLoading.value) return
      
      isLoading.value = true
      
      try {
        // First, ask the AI what commands to run
        const aiResponse = await invoke('general_ai_query', { 
          query: `${question}. Provide specific commands I should run and explain what each does. If you recommend commands, list them clearly so I can execute them.`
        })
        
        // Add to conversation
        conversation.value.push({
          question: question,
          response: aiResponse,
          timestamp: new Date()
        })
        
        // Try to extract and execute commands if the AI suggests any
        const commandMatches = aiResponse.match(/`([^`]+)`/g)
        if (commandMatches) {
          for (const match of commandMatches.slice(0, 3)) { // Limit to first 3 commands
            const command = match.replace(/`/g, '')
            
            // Only execute safe diagnostic commands
            const safeCommands = ['free', 'df', 'ps', 'top', 'lscpu', 'lsblk', 'uname', 'uptime', 'who', 'w', 'systemctl status', 'journalctl']
            const isCommandSafe = safeCommands.some(safe => command.toLowerCase().startsWith(safe.toLowerCase()))
            
            if (isCommandSafe) {
              try {
                const output = await invoke('execute_command', { command: command })
                
                // Add command execution result to conversation
                conversation.value.push({
                  question: `Executed: ${command}`,
                  response: `Command Output:\n${output}`,
                  timestamp: new Date()
                })
              } catch (cmdError) {
                conversation.value.push({
                  question: `Executed: ${command}`,
                  response: `Command Error: ${cmdError}`,
                  timestamp: new Date()
                })
              }
            }
          }
        }
        
      } catch (error) {
        conversation.value.push({
          question: question,
          response: 'Error: ' + error,
          timestamp: new Date()
        })
      } finally {
        isLoading.value = false
      }
    }

    // Fix code functionality
    const fixCode = async () => {
      if (!codeToFix.value.trim() || isLoading.value) return
      
      const code = codeToFix.value.trim()
      isLoading.value = true
      
      try {
        const fixQuery = `Please analyze and fix the following code/text/error message. Provide:
1. What the issue is
2. The fixed version
3. Explanation of what was wrong and how you fixed it

Code/Text to fix:

${code}

Please be specific and provide working solutions.`
        
        const result = await invoke('general_ai_query', { query: fixQuery })
        
        conversation.value.push({
          question: `Fix Code/Text: ${code.substring(0, 100)}${code.length > 100 ? '...' : ''}`,
          response: result,
          timestamp: new Date()
        })
        
        // Don't clear codeToFix automatically so user can see what they submitted
      } catch (error) {
        conversation.value.push({
          question: `Fix Code/Text: ${code.substring(0, 100)}${code.length > 100 ? '...' : ''}`,
          response: 'Error: ' + error,
          timestamp: new Date()
        })
      } finally {
        isLoading.value = false
      }
    }

    const clearFixSection = () => {
      codeToFix.value = ''
    }

    return {
      aiQuery,
      conversation,
      isLoading,
      codeToFix,
      askAI,
      askQuickQuestion,
      askWithExecution,
      fixCode,
      clearFixSection,
      clearConversation
    }
  }
}
</script>

<style scoped>
.ai-assistant {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.assistant-header {
  padding: 1.5rem;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.assistant-header h3 {
  margin: 0 0 0.5rem 0;
  color: var(--primary-color);
  font-size: 1.5rem;
}

.assistant-header p {
  margin: 0;
  color: var(--text-secondary);
  font-style: italic;
}

.assistant-body {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
}

.query-section {
  margin-bottom: 2rem;
  padding: 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-secondary);
}

.form-group {
  margin-bottom: 1rem;
}

.form-label {
  display: block;
  margin-bottom: 0.5rem;
  color: var(--text-primary);
  font-weight: 600;
}

.form-input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.25rem;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

.ai-textarea {
  min-height: 120px;
  resize: vertical;
  font-family: 'Courier New', monospace;
  line-height: 1.4;
}

.button-group {
  display: flex;
  gap: 0.5rem;
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

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--bg-primary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--bg-secondary);
}

.conversation-section {
  margin-bottom: 2rem;
}

.conversation-section h4 {
  margin: 0 0 1rem 0;
  color: var(--primary-color);
}

.conversation-history {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-secondary);
}

.conversation-item {
  padding: 1rem;
  border-bottom: 1px solid var(--border-color);
}

.conversation-item:last-child {
  border-bottom: none;
}

.question {
  margin-bottom: 0.5rem;
  color: var(--text-primary);
}

.response {
  color: var(--text-secondary);
}

.response pre {
  margin: 0.5rem 0 0 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  line-height: 1.4;
  background: var(--bg-primary);
  padding: 0.75rem;
  border-radius: 0.25rem;
  border: 1px solid var(--border-color);
}

.quick-actions h4 {
  margin: 0 0 1rem 0;
  color: var(--primary-color);
}

.action-buttons {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 0.5rem;
}

.btn-outline {
  background: transparent;
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  padding: 0.75rem;
  text-align: left;
}

.btn-outline:hover {
  background: var(--bg-secondary);
  border-color: var(--primary-color);
}

/* AI Actions Section */
.ai-actions-section {
  margin-bottom: 2rem;
}

.ai-actions-section h4 {
  margin: 0 0 1rem 0;
  color: var(--primary-color);
}

.action-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 0.75rem;
}

.btn-action {
  background: linear-gradient(135deg, var(--primary-color), #4CAF50);
  color: white;
  padding: 1rem;
  border: none;
  border-radius: 0.5rem;
  font-weight: 600;
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.3s ease;
  text-align: center;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.btn-action:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0,0,0,0.2);
  opacity: 0.9;
}

.btn-action:active {
  transform: translateY(0);
}

/* Fix Code Section */
.fix-code-section {
  margin-bottom: 2rem;
  padding: 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-secondary);
}

.fix-code-section h4 {
  margin: 0 0 0.5rem 0;
  color: var(--primary-color);
}

.section-description {
  margin: 0 0 1rem 0;
  color: var(--text-secondary);
  font-size: 0.875rem;
  font-style: italic;
}

.code-textarea {
  min-height: 200px;
  resize: vertical;
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
  background: var(--bg-primary);
  border: 2px solid var(--border-color);
}

.code-textarea:focus {
  border-color: var(--primary-color);
  outline: none;
  box-shadow: 0 0 0 3px rgba(74, 144, 226, 0.1);
}

.btn-fix {
  background: linear-gradient(135deg, #ff6b6b, #ee5a6f);
  color: white;
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 0.25rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 4px rgba(255, 107, 107, 0.2);
}

.btn-fix:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(255, 107, 107, 0.3);
  opacity: 0.9;
}

.btn-fix:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
  box-shadow: 0 2px 4px rgba(255, 107, 107, 0.1);
}

/* Terminal Styles */
.terminal-section {
  margin-bottom: 2rem;
}

.terminal-section h4 {
  margin: 0 0 1rem 0;
  color: var(--primary-color);
}

.terminal-container {
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: #1e1e1e;
  font-family: 'Courier New', monospace;
  color: #f0f0f0;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 1rem;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
  border-radius: 0.5rem 0.5rem 0 0;
}

.terminal-title {
  font-size: 0.875rem;
  color: #f0f0f0;
}

.btn-small {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: #444;
  color: #f0f0f0;
  border: 1px solid #666;
}

.btn-small:hover {
  background: #555;
}

.terminal-body {
  min-height: 200px;
  max-height: 300px;
  overflow-y: auto;
  padding: 1rem;
  background: #1e1e1e;
}

.terminal-entry {
  margin-bottom: 0.5rem;
}

.terminal-command {
  color: #f0f0f0;
}

.prompt {
  color: #4CAF50;
  font-weight: bold;
  margin-right: 0.5rem;
}

.terminal-output pre {
  margin: 0;
  color: #f0f0f0;
  background: transparent;
  border: none;
  padding: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.terminal-error pre {
  margin: 0;
  color: #ff6b6b;
  background: transparent;
  border: none;
  padding: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.terminal-input {
  display: flex;
  align-items: center;
  padding: 0.75rem 1rem;
  background: #1e1e1e;
  border-top: 1px solid #444;
  border-radius: 0 0 0.5rem 0.5rem;
}

.command-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #f0f0f0;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  outline: none;
  margin: 0 0.5rem;
}

.command-input::placeholder {
  color: #888;
}
</style>
