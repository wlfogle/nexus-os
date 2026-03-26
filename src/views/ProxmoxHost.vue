<template>
  <div class="proxmox-host">
    <h1>🖥️ Proxmox Host Management</h1>
    
    <div v-if="loading" class="loading">Loading host information...</div>
    
    <div v-else>
      <!-- Host Information Card -->
      <div class="host-info-card">
        <h2>Host Information</h2>
        <div class="info-grid">
          <div class="info-item">
            <label>Hostname:</label>
            <span>{{ hostInfo.hostname }}</span>
          </div>
          <div class="info-item">
            <label>Version:</label>
            <span>{{ hostInfo.version }}</span>
          </div>
          <div class="info-item">
            <label>Status:</label>
            <span class="status-online">{{ hostInfo.node_status }}</span>
          </div>
          <div class="info-item">
            <label>Uptime:</label>
            <span>{{ hostInfo.uptime }}</span>
          </div>
          <div class="info-item">
            <label>CPU Cores:</label>
            <span>{{ hostInfo.cpu_count }}</span>
          </div>
          <div class="info-item">
            <label>Total Memory:</label>
            <span>{{ hostInfo.memory_total }}</span>
          </div>
        </div>
      </div>

      <!-- Host Control Actions -->
      <div class="host-actions">
        <h2>Host Control</h2>
        <div class="action-buttons">
          <button @click="refreshHostInfo" class="btn btn-primary" :disabled="actionLoading">
            🔄 Refresh Info
          </button>
          <button @click="updatePackages" class="btn btn-warning" :disabled="actionLoading">
            📦 Update Packages
          </button>
          <button @click="rebootHost" class="btn btn-danger" :disabled="actionLoading">
            🔄 Reboot Host
          </button>
          <button @click="shutdownHost" class="btn btn-critical" :disabled="actionLoading">
            ⚡ Shutdown Host
          </button>
        </div>
      </div>

      <!-- Storage Information -->
      <div class="storage-section">
        <h2>Storage Information</h2>
        <div class="storage-grid">
          <div v-for="storage in hostInfo.storage_info" :key="storage.name" class="storage-card">
            <h3>{{ storage.name }}</h3>
            <p><strong>Type:</strong> {{ storage.storage_type }}</p>
            <p><strong>Total:</strong> {{ storage.total }}</p>
            <p><strong>Used:</strong> {{ storage.used }}</p>
            <p><strong>Available:</strong> {{ storage.available }}</p>
            <div class="usage-bar">
              <div class="usage-fill" :style="{ width: storage.usage_percent + '%' }"></div>
            </div>
            <p class="usage-text">{{ storage.usage_percent.toFixed(1) }}% used</p>
          </div>
        </div>
      </div>

      <!-- Cluster Status -->
      <div class="cluster-section">
        <h2>Cluster Status</h2>
        <button @click="fetchClusterStatus" class="btn btn-secondary" :disabled="actionLoading">
          📊 Get Cluster Status
        </button>
        <div v-if="clusterStatus" class="cluster-output">
          <pre>{{ clusterStatus }}</pre>
        </div>
      </div>
    </div>

    <!-- Action Result Modal -->
    <div v-if="actionResult" class="action-result" :class="actionResult.type">
      <p>{{ actionResult.message }}</p>
      <button @click="actionResult = null" class="btn btn-small">Close</button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'ProxmoxHost',
  setup() {
    const loading = ref(false)
    const actionLoading = ref(false)
    const hostInfo = ref({
      hostname: '',
      version: '',
      uptime: '',
      cpu_count: 0,
      memory_total: '',
      storage_info: [],
      node_status: ''
    })
    const clusterStatus = ref('')
    const actionResult = ref(null)

    const fetchHostInfo = async () => {
      try {
        loading.value = true
        const result = await invoke('get_proxmox_host_info')
        hostInfo.value = result
        console.log('Host Info:', result)
      } catch (error) {
        console.error('Failed to fetch host info:', error)
        showResult('Failed to fetch host information: ' + error, 'error')
      } finally {
        loading.value = false
      }
    }

    const refreshHostInfo = async () => {
      await fetchHostInfo()
      showResult('Host information refreshed successfully', 'success')
    }

    const updatePackages = async () => {
      if (!confirm('Are you sure you want to update all packages? This may take some time.')) return

      try {
        actionLoading.value = true
        const result = await invoke('update_proxmox_packages')
        showResult(result, 'success')
      } catch (error) {
        console.error('Failed to update packages:', error)
        showResult('Failed to update packages: ' + error, 'error')
      } finally {
        actionLoading.value = false
      }
    }

    const rebootHost = async () => {
      if (!confirm('Are you sure you want to reboot the Proxmox host? This will affect all running containers and VMs.')) return

      try {
        actionLoading.value = true
        const result = await invoke('reboot_proxmox_host')
        showResult(result, 'success')
      } catch (error) {
        console.error('Failed to reboot host:', error)
        showResult('Failed to reboot host: ' + error, 'error')
      } finally {
        actionLoading.value = false
      }
    }

    const shutdownHost = async () => {
      if (!confirm('Are you sure you want to shutdown the Proxmox host? This will turn off all running containers and VMs.')) return

      try {
        actionLoading.value = true
        const result = await invoke('shutdown_proxmox_host')
        showResult(result, 'success')
      } catch (error) {
        console.error('Failed to shutdown host:', error)
        showResult('Failed to shutdown host: ' + error, 'error')
      } finally {
        actionLoading.value = false
      }
    }

    const fetchClusterStatus = async () => {
      try {
        actionLoading.value = true
        const result = await invoke('get_cluster_status')
        clusterStatus.value = result
        showResult('Cluster status retrieved successfully', 'success')
      } catch (error) {
        console.error('Failed to get cluster status:', error)
        showResult('Failed to get cluster status: ' + error, 'error')
      } finally {
        actionLoading.value = false
      }
    }

    const showResult = (message, type) => {
      actionResult.value = { message, type }
      setTimeout(() => {
        actionResult.value = null
      }, 5000)
    }

    onMounted(fetchHostInfo)

    return {
      loading,
      actionLoading,
      hostInfo,
      clusterStatus,
      actionResult,
      refreshHostInfo,
      updatePackages,
      rebootHost,
      shutdownHost,
      fetchClusterStatus
    }
  }
}
</script>

<style scoped>
.proxmox-host {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.loading {
  text-align: center;
  padding: 20px;
  font-size: 18px;
}

.host-info-card {
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  margin-bottom: 20px;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
  margin-top: 15px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 10px;
  background: #f8f9fa;
  border-radius: 4px;
}

.info-item label {
  font-weight: bold;
  color: #495057;
}

.status-online {
  color: #28a745;
  font-weight: bold;
}

.host-actions {
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  margin-bottom: 20px;
}

.action-buttons {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  margin-top: 15px;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #0056b3;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #545b62;
}

.btn-warning {
  background: #ffc107;
  color: #212529;
}

.btn-warning:hover:not(:disabled) {
  background: #e0a800;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #c82333;
}

.btn-critical {
  background: #6f42c1;
  color: white;
}

.btn-critical:hover:not(:disabled) {
  background: #5a32a3;
}

.btn-small {
  padding: 5px 10px;
  font-size: 12px;
}

.storage-section, .cluster-section {
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  margin-bottom: 20px;
}

.storage-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
  margin-top: 15px;
}

.storage-card {
  background: #f8f9fa;
  padding: 15px;
  border-radius: 6px;
  border: 1px solid #dee2e6;
}

.storage-card h3 {
  margin: 0 0 10px 0;
  color: #495057;
}

.storage-card p {
  margin: 5px 0;
  font-size: 14px;
}

.usage-bar {
  width: 100%;
  height: 8px;
  background: #e9ecef;
  border-radius: 4px;
  overflow: hidden;
  margin: 10px 0 5px 0;
}

.usage-fill {
  height: 100%;
  background: linear-gradient(90deg, #28a745, #ffc107, #dc3545);
  transition: width 0.3s ease;
}

.usage-text {
  font-size: 12px;
  color: #6c757d;
  text-align: center;
  margin: 0;
}

.cluster-output {
  margin-top: 15px;
  background: #f8f9fa;
  padding: 15px;
  border-radius: 4px;
  border: 1px solid #dee2e6;
}

.cluster-output pre {
  margin: 0;
  white-space: pre-wrap;
  font-family: 'Courier New', monospace;
  font-size: 12px;
}

.action-result {
  position: fixed;
  top: 20px;
  right: 20px;
  padding: 15px 20px;
  border-radius: 6px;
  box-shadow: 0 4px 6px rgba(0,0,0,0.1);
  z-index: 1000;
  display: flex;
  align-items: center;
  gap: 10px;
}

.action-result.success {
  background: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

.action-result.error {
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

h1 {
  color: #495057;
  margin-bottom: 20px;
}

h2 {
  color: #495057;
  margin: 0 0 15px 0;
  font-size: 1.4em;
}
</style>
