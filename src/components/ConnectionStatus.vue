<template>
  <div class="connection-status">
    <div :class="['status-indicator', { connected: isConnected, disconnected: !isConnected }]">
      <div class="status-dot"></div>
      <span>{{ statusText }}</span>
    </div>
    <button class="btn btn-secondary refresh-btn" @click="checkConnection" :disabled="checking">
      {{ checking ? '⟳' : '↻' }}
    </button>
  </div>
</template>

<script>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'ConnectionStatus',
  setup() {
    const isConnected = ref(false)
    const checking = ref(false)
    
    const statusText = computed(() => {
      if (checking.value) return 'Checking...'
      return isConnected.value ? 'Connected to AI' : 'Disconnected'
    })
    
    const checkConnection = async () => {
      checking.value = true
      try {
        const result = await invoke('check_ai_connection')
        isConnected.value = result
      } catch (error) {
        console.error('Failed to check connection:', error)
        isConnected.value = false
      } finally {
        checking.value = false
      }
    }
    
    onMounted(() => {
      checkConnection()
    })
    
    return {
      isConnected,
      checking,
      statusText,
      checkConnection
    }
  }
}
</script>

<style scoped>
.connection-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem;
  background: var(--bg-secondary);
  border-radius: 0.375rem;
  border: 1px solid var(--border-color);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--error-color);
}

.connected .status-dot {
  background: var(--success-color);
  animation: pulse 2s infinite;
}

.refresh-btn {
  padding: 0.25rem;
  font-size: 0.75rem;
  min-width: auto;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}
</style>
