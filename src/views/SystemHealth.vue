<template>
  <div class="system-health">
    <h1>System Health</h1>
    <div v-if="loading">Loading...</div>
    <div v-else>
      <div class="health-metrics">
        <div class="metric-card">
          <h3>💾 Disk Usage</h3>
          <div class="metric-value">{{ systemHealth.disk_usage.toFixed(1) }}%</div>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: systemHealth.disk_usage + '%' }"></div>
          </div>
        </div>
        <div class="metric-card">
          <h3>🧠 Memory Usage</h3>
          <div class="metric-value">{{ systemHealth.memory_usage.toFixed(1) }}%</div>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: systemHealth.memory_usage + '%' }"></div>
          </div>
        </div>
        <div class="metric-card">
          <h3>⚡ CPU Load</h3>
          <div class="metric-value">{{ systemHealth.cpu_load.toFixed(1) }}%</div>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: systemHealth.cpu_load + '%' }"></div>
          </div>
        </div>
        <div class="metric-card">
          <h3>🌐 Network Status</h3>
          <div class="metric-value">{{ systemHealth.network_status }}</div>
        </div>
        <div class="metric-card">
          <h3>⏰ Uptime</h3>
          <div class="metric-value">{{ systemHealth.uptime }}</div>
        </div>
      </div>
      <div class="refresh-section">
        <button @click="fetchSystemHealth" class="refresh-btn">🔄 Refresh</button>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'SystemHealth',
  setup() {
    const systemHealth = ref({
      disk_usage: 0,
      memory_usage: 0,
      cpu_load: 0,
      network_status: 'Unknown',
      uptime: 'Unknown'
    })
    const loading = ref(false)

    const fetchSystemHealth = async () => {
      try {
        loading.value = true
        const result = await invoke('get_maintenance_overview')
        systemHealth.value = result.system_health
      } catch (error) {
        console.error('Failed to fetch system health:', error)
      } finally {
        loading.value = false
      }
    }

    onMounted(fetchSystemHealth)

    return {
      systemHealth,
      loading,
      fetchSystemHealth
    }
  }
}
</script>

<style scoped>
.system-health {
  padding: 20px;
}

.health-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-bottom: 20px;
}

.metric-card {
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  text-align: center;
}

.metric-card h3 {
  margin: 0 0 10px 0;
  color: #333;
}

.metric-value {
  font-size: 24px;
  font-weight: bold;
  color: #667eea;
  margin-bottom: 10px;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background-color: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #667eea, #764ba2);
  transition: width 0.3s ease;
}

.refresh-section {
  text-align: center;
}

.refresh-btn {
  background: #667eea;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
}

.refresh-btn:hover {
  background: #5a6fd8;
}
</style>

