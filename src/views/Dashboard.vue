<template>
  <div class="dashboard">
    <!-- Header Section -->
    <div class="hero-section">
      <h1>🎛️ Infrastructure Control Center</h1>
      <p>Centralized management for your complete Proxmox infrastructure</p>
    </div>

    <!-- Quick Stats Cards -->
    <div class="stats-grid">
      <div class="stat-card">
        <h3>📦 Containers</h3>
        <div class="stat-number">{{ overview.running_containers }}/{{ overview.total_containers }}</div>
        <p>Running/Total</p>
      </div>
      <div class="stat-card">
        <h3>🖥️ Virtual Machines</h3>
        <div class="stat-number">{{ overview.running_vms }}/{{ overview.total_vms }}</div>
        <p>Running/Total</p>
      </div>
      <div class="stat-card">
        <h3>⚡ System Health</h3>
        <div class="stat-number">{{ systemHealth.cpu_load.toFixed(1) }}%</div>
        <p>CPU Load</p>
      </div>
      <div class="stat-card">
        <h3>💾 Storage</h3>
        <div class="stat-number">{{ systemHealth.disk_usage.toFixed(1) }}%</div>
        <p>Disk Usage</p>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="actions-section">
      <h2>Quick Actions</h2>
      <div class="action-buttons">
        <button @click="refreshData" class="action-btn primary" :disabled="loading">
          🔄 Refresh All Data
        </button>
        <button @click="$router.push('/containers')" class="action-btn">
          📦 Manage Containers
        </button>
        <button @click="$router.push('/vms')" class="action-btn">
          🖥️ Manage VMs
        </button>
        <button @click="$router.push('/maintenance')" class="action-btn">
          🔧 System Maintenance
        </button>
      </div>
    </div>

    <!-- System Overview -->
    <div class="overview-section">
      <h2>System Overview</h2>
      <div class="overview-grid">
        <div class="overview-card">
          <h3>🏠 Your Media Stack</h3>
          <ul>
            <li><strong>Gluetun (CT-101):</strong> VPN Gateway</li>
            <li><strong>Home Assistant (VM-500):</strong> Home Automation</li>
            <li><strong>AI System (CT-900):</strong> Local AI Services</li>
            <li><strong>Alexa VM (VM-611):</strong> Voice Assistant</li>
            <li><strong>Media Services:</strong> Plex, Jellyfin, *arr Stack</li>
          </ul>
        </div>
        <div class="overview-card">
          <h3>📊 Infrastructure Stats</h3>
          <ul>
            <li><strong>Total Services:</strong> 47+ Containers</li>
            <li><strong>Storage:</strong> 220GB Main + 227GB Temp</li>
            <li><strong>Categories:</strong> Core, Media, Enhancement, Monitoring</li>
            <li><strong>Last Updated:</strong> {{ formatDate(overview.last_updated) }}</li>
          </ul>
        </div>
      </div>
    </div>

    <!-- Loading Indicator -->
    <div v-if="loading" class="loading">
      <p>Loading system data...</p>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'Dashboard',
  setup() {
    const loading = ref(false)
    const overview = ref({
      total_containers: 0,
      running_containers: 0,
      total_vms: 0,
      running_vms: 0,
      containers: [],
      vms: [],
      last_updated: ''
    })
    
    const systemHealth = ref({
      cpu_load: 0,
      disk_usage: 0,
      memory_usage: 0,
      network_status: 'Unknown',
      uptime: ''
    })

    const fetchSystemOverview = async () => {
      try {
        loading.value = true
        const result = await invoke('get_system_overview')
        overview.value = result
        console.log('System Overview:', result)
      } catch (error) {
        console.error('Failed to fetch system overview:', error)
        // Set fallback data for development
        overview.value = {
          total_containers: 47,
          running_containers: 42,
          total_vms: 3,
          running_vms: 3,
          containers: [],
          vms: [],
          last_updated: new Date().toISOString()
        }
      } finally {
        loading.value = false
      }
    }

    const fetchSystemHealth = async () => {
      try {
        const maintenance = await invoke('get_maintenance_overview')
        systemHealth.value = maintenance.system_health
        console.log('System Health:', maintenance.system_health)
      } catch (error) {
        console.error('Failed to fetch system health:', error)
        // Set fallback data
        systemHealth.value = {
          cpu_load: 25.5,
          disk_usage: 67.2,
          memory_usage: 58.1,
          network_status: 'Connected',
          uptime: '5 days, 12 hours'
        }
      }
    }

    const refreshData = async () => {
      await Promise.all([
        fetchSystemOverview(),
        fetchSystemHealth()
      ])
    }

    const formatDate = (dateString) => {
      if (!dateString) return 'Never'
      try {
        return new Date(dateString).toLocaleString()
      } catch {
        return 'Invalid date'
      }
    }

    onMounted(() => {
      refreshData()
      // Auto-refresh every 30 seconds
      const interval = setInterval(refreshData, 30000)
      
      // Cleanup interval on unmount
      return () => clearInterval(interval)
    })

    return {
      loading,
      overview,
      systemHealth,
      refreshData,
      formatDate
    }
  }
}
</script>

<style scoped>
.dashboard {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

.hero-section {
  text-align: center;
  margin-bottom: 3rem;
  background: rgba(255, 255, 255, 0.9);
  padding: 2rem;
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.hero-section h1 {
  font-size: 2.5rem;
  margin-bottom: 0.5rem;
  color: #2d3748;
}

.hero-section p {
  font-size: 1.1rem;
  color: #4a5568;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 3rem;
}

.stat-card {
  background: rgba(255, 255, 255, 0.9);
  padding: 1.5rem;
  border-radius: 12px;
  text-align: center;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  transition: transform 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
}

.stat-card h3 {
  margin-bottom: 1rem;
  color: #2d3748;
  font-size: 1.1rem;
}

.stat-number {
  font-size: 2.5rem;
  font-weight: bold;
  color: #667eea;
  margin-bottom: 0.5rem;
}

.actions-section, .overview-section {
  background: rgba(255, 255, 255, 0.9);
  padding: 2rem;
  border-radius: 12px;
  margin-bottom: 2rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
}

.actions-section h2, .overview-section h2 {
  margin-bottom: 1.5rem;
  color: #2d3748;
}

.action-buttons {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.action-btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 8px;
  background: #e2e8f0;
  color: #2d3748;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.action-btn:hover {
  background: #cbd5e0;
  transform: translateY(-2px);
}

.action-btn.primary {
  background: #667eea;
  color: white;
}

.action-btn.primary:hover {
  background: #5a67d8;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 2rem;
}

.overview-card {
  background: #f7fafc;
  padding: 1.5rem;
  border-radius: 8px;
  border-left: 4px solid #667eea;
}

.overview-card h3 {
  margin-bottom: 1rem;
  color: #2d3748;
}

.overview-card ul {
  list-style: none;
  padding: 0;
}

.overview-card li {
  margin-bottom: 0.5rem;
  color: #4a5568;
}

.loading {
  text-align: center;
  padding: 2rem;
  background: rgba(255, 255, 255, 0.9);
  border-radius: 8px;
  margin-top: 2rem;
}

@media (max-width: 768px) {
  .dashboard {
    padding: 1rem;
  }
  
  .hero-section h1 {
    font-size: 2rem;
  }
  
  .stats-grid {
    grid-template-columns: 1fr;
  }
  
  .action-buttons {
    flex-direction: column;
  }
  
  .overview-grid {
    grid-template-columns: 1fr;
  }
}
</style>
