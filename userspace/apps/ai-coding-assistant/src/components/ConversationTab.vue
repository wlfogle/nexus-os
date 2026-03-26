<template>
  <div class="conversation-container">
    <div class="conversation-header">
      <div class="conversation-title">
        <input 
          v-if="isEditing"
          v-model="editableName"
          @blur="finishEditing"
          @keyup.enter="finishEditing"
          @keyup.escape="cancelEditing"
          class="title-input"
          ref="titleInput"
        />
        <h3 v-else @dblclick="startEditing">{{ conversationName }}</h3>
      </div>
      <div class="conversation-actions">
        <button @click="clearMessages" title="Clear conversation" class="action-btn">
          🗑️
        </button>
        <button @click="exportConversation" title="Export conversation" class="action-btn">
          💾
        </button>
      </div>
    </div>

    <div class="messages-container" ref="messagesContainer">
      <div v-if="messages.length === 0" class="empty-state">
        <div class="empty-icon">💬</div>
        <h3>Start a new conversation</h3>
        <p>Ask me anything about code, files, or system tasks!</p>
      </div>

      <div v-for="message in messages" :key="message.id" class="message" :class="message.role">
        <div class="message-avatar">
          <span v-if="message.role === 'user'">👤</span>
          <span v-else>🤖</span>
        </div>
        <div class="message-content">
          <div class="message-text" v-html="formatMessage(message.content)"></div>
          <div class="message-meta">
            <span class="timestamp">{{ formatTime(message.timestamp) }}</span>
            <button @click="copyMessage(message.content)" class="copy-btn" title="Copy message">
              📋
            </button>
          </div>
        </div>
      </div>

      <div v-if="isLoading" class="message assistant">
        <div class="message-avatar">
          <span>🤖</span>
        </div>
        <div class="message-content">
          <div class="typing-indicator">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      </div>
    </div>

    <div class="input-container">
      <div class="input-wrapper">
        <textarea
          v-model="currentMessage"
          @keydown="handleKeyDown"
          @input="adjustTextareaHeight"
          placeholder="Ask me anything..."
          class="message-input"
          ref="messageInput"
          rows="1"
        ></textarea>
        <button 
          @click="sendMessage" 
          :disabled="!currentMessage.trim() || isLoading"
          class="send-btn"
        >
          <span v-if="isLoading">⏳</span>
          <span v-else>🚀</span>
        </button>
      </div>
      <div class="input-actions">
        <button @click="attachFile" class="attach-btn" title="Attach file">
          📎
        </button>
        <span class="char-count">{{ currentMessage.length }}/4000</span>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, nextTick, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'ConversationTab',
  props: {
    conversationId: {
      type: String,
      required: true
    },
    conversationName: {
      type: String,
      required: true
    }
  },
  emits: ['rename'],
  setup(props, { emit }) {
    const messages = ref([])
    const currentMessage = ref('')
    const isLoading = ref(false)
    const messagesContainer = ref(null)
    const messageInput = ref(null)
    const titleInput = ref(null)
    
    // Title editing
    const isEditing = ref(false)
    const editableName = ref(props.conversationName)
    
    // Load messages for this conversation from localStorage
    const loadMessages = () => {
      const saved = localStorage.getItem(`conversation-${props.conversationId}`)
      if (saved) {
        messages.value = JSON.parse(saved)
      }
    }
    
    // Save messages to localStorage
    const saveMessages = () => {
      localStorage.setItem(`conversation-${props.conversationId}`, JSON.stringify(messages.value))
    }
    
    // Auto-scroll to bottom when new messages arrive
    const scrollToBottom = () => {
      nextTick(() => {
        if (messagesContainer.value) {
          messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
        }
      })
    }
    
    // Send message to AI
    const sendMessage = async () => {
      if (!currentMessage.value.trim() || isLoading.value) return
      
      const userMessage = {
        id: Date.now() + Math.random(),
        role: 'user',
        content: currentMessage.value.trim(),
        timestamp: new Date()
      }
      
      messages.value.push(userMessage)
      const messageToSend = currentMessage.value.trim()
      currentMessage.value = ''
      adjustTextareaHeight()
      saveMessages()
      scrollToBottom()
      
      isLoading.value = true
      
      try {
        const response = await invoke('general_ai_query', { query: messageToSend })
        
        const assistantMessage = {
          id: Date.now() + Math.random(),
          role: 'assistant',
          content: response,
          timestamp: new Date()
        }
        
        messages.value.push(assistantMessage)
        saveMessages()
        scrollToBottom()
      } catch (error) {
        console.error('AI request failed:', error)
        
        const errorMessage = {
          id: Date.now() + Math.random(),
          role: 'assistant',
          content: `Sorry, I encountered an error: ${error}`,
          timestamp: new Date()
        }
        
        messages.value.push(errorMessage)
        saveMessages()
        scrollToBottom()
      } finally {
        isLoading.value = false
      }
    }
    
    // Handle keyboard shortcuts
    const handleKeyDown = (event) => {
      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault()
        sendMessage()
      }
    }
    
    // Auto-resize textarea
    const adjustTextareaHeight = () => {
      nextTick(() => {
        const textarea = messageInput.value
        if (textarea) {
          textarea.style.height = 'auto'
          textarea.style.height = Math.min(textarea.scrollHeight, 150) + 'px'
        }
      })
    }
    
    // Format message content (basic markdown-like formatting)
    const formatMessage = (content) => {
      return content
        .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
        .replace(/\*(.*?)\*/g, '<em>$1</em>')
        .replace(/`(.*?)`/g, '<code>$1</code>')
        .replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>')
        .replace(/\n/g, '<br>')
    }
    
    // Format timestamp
    const formatTime = (timestamp) => {
      return new Date(timestamp).toLocaleTimeString([], { 
        hour: '2-digit', 
        minute: '2-digit' 
      })
    }
    
    // Copy message to clipboard
    const copyMessage = async (content) => {
      try {
        await navigator.clipboard.writeText(content)
      } catch (error) {
        console.error('Failed to copy message:', error)
      }
    }
    
    // Clear all messages
    const clearMessages = () => {
      if (confirm('Are you sure you want to clear this conversation?')) {
        messages.value = []
        saveMessages()
      }
    }
    
    // Export conversation
    const exportConversation = () => {
      const exportData = {
        conversationId: props.conversationId,
        conversationName: props.conversationName,
        messages: messages.value,
        exportedAt: new Date().toISOString()
      }
      
      const dataStr = JSON.stringify(exportData, null, 2)
      const blob = new Blob([dataStr], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      
      const a = document.createElement('a')
      a.href = url
      a.download = `conversation-${props.conversationName}-${Date.now()}.json`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
    }
    
    // Attach file (placeholder)
    const attachFile = () => {
      // TODO: Implement file attachment
      alert('File attachment coming soon!')
    }
    
    // Title editing functions
    const startEditing = () => {
      editableName.value = props.conversationName
      isEditing.value = true
      nextTick(() => {
        if (titleInput.value) {
          titleInput.value.focus()
          titleInput.value.select()
        }
      })
    }
    
    const finishEditing = () => {
      if (editableName.value.trim() && editableName.value !== props.conversationName) {
        emit('rename', props.conversationId, editableName.value.trim())
      }
      isEditing.value = false
    }
    
    const cancelEditing = () => {
      editableName.value = props.conversationName
      isEditing.value = false
    }
    
    // Watch for conversation name changes
    watch(() => props.conversationName, (newName) => {
      editableName.value = newName
    })
    
    // Load messages when component mounts
    onMounted(() => {
      loadMessages()
      scrollToBottom()
    })
    
    return {
      messages,
      currentMessage,
      isLoading,
      messagesContainer,
      messageInput,
      titleInput,
      isEditing,
      editableName,
      sendMessage,
      handleKeyDown,
      adjustTextareaHeight,
      formatMessage,
      formatTime,
      copyMessage,
      clearMessages,
      exportConversation,
      attachFile,
      startEditing,
      finishEditing,
      cancelEditing
    }
  }
}
</script>

<style scoped>
.conversation-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.conversation-header {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-secondary);
}

.conversation-title h3 {
  margin: 0;
  color: var(--text-primary);
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.conversation-title h3:hover {
  background: var(--bg-primary);
}

.title-input {
  background: var(--bg-primary);
  border: 1px solid var(--primary-color);
  border-radius: 4px;
  padding: 0.25rem 0.5rem;
  color: var(--text-primary);
  font-size: 1.1rem;
  font-weight: 600;
}

.conversation-actions {
  display: flex;
  gap: 0.5rem;
}

.action-btn {
  background: none;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 1rem;
}

.action-btn:hover {
  background: var(--bg-primary);
  border-color: var(--primary-color);
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  scroll-behavior: smooth;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.empty-state h3 {
  margin-bottom: 0.5rem;
  color: var(--text-primary);
}

.message {
  display: flex;
  gap: 1rem;
  margin-bottom: 1.5rem;
  align-items: flex-start;
}

.message.user {
  flex-direction: row-reverse;
}

.message-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--bg-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.2rem;
  flex-shrink: 0;
}

.message.user .message-avatar {
  background: var(--primary-color);
}

.message-content {
  flex: 1;
  max-width: calc(100% - 60px);
}

.message.user .message-content {
  text-align: right;
}

.message-text {
  background: var(--bg-secondary);
  padding: 1rem;
  border-radius: 12px;
  color: var(--text-primary);
  line-height: 1.5;
  word-wrap: break-word;
}

.message.user .message-text {
  background: var(--primary-color);
  color: white;
}

.message-text :deep(code) {
  background: rgba(0, 0, 0, 0.1);
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
  font-family: 'Fira Code', monospace;
}

.message-text :deep(pre) {
  background: rgba(0, 0, 0, 0.1);
  padding: 1rem;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5rem 0;
}

.message-text :deep(pre code) {
  background: none;
  padding: 0;
}

.message-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.5rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.message.user .message-meta {
  justify-content: flex-end;
}

.copy-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 3px;
  transition: all 0.2s;
}

.copy-btn:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.typing-indicator {
  display: flex;
  gap: 0.3rem;
  padding: 1rem;
  background: var(--bg-secondary);
  border-radius: 12px;
}

.typing-indicator span {
  width: 8px;
  height: 8px;
  background: var(--text-secondary);
  border-radius: 50%;
  animation: typing 1.4s infinite ease-in-out;
}

.typing-indicator span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-indicator span:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes typing {
  0%, 60%, 100% {
    transform: translateY(0);
    opacity: 0.5;
  }
  30% {
    transform: translateY(-10px);
    opacity: 1;
  }
}

.input-container {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.input-wrapper {
  display: flex;
  gap: 0.5rem;
  align-items: flex-end;
}

.message-input {
  flex: 1;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 0.75rem 1rem;
  color: var(--text-primary);
  font-family: inherit;
  font-size: 0.9rem;
  line-height: 1.4;
  resize: none;
  min-height: 44px;
  max-height: 150px;
  overflow-y: auto;
}

.message-input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.send-btn {
  background: var(--primary-color);
  border: none;
  border-radius: 12px;
  padding: 0.75rem 1rem;
  color: white;
  cursor: pointer;
  font-size: 1.1rem;
  transition: all 0.2s;
  min-width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.send-btn:hover:not(:disabled) {
  background: var(--primary-dark);
  transform: translateY(-1px);
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.input-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 0.5rem;
}

.attach-btn {
  background: none;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 0.5rem;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.attach-btn:hover {
  background: var(--bg-primary);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.char-count {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

/* Scrollbar Styling */
.messages-container::-webkit-scrollbar {
  width: 6px;
}

.messages-container::-webkit-scrollbar-track {
  background: var(--bg-secondary);
}

.messages-container::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.messages-container::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}
</style>
