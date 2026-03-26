<template>
  <div class="maintenance">
    <h1>Maintenance</h1>
    
    <!-- Global Actions -->
    <div class="global-actions">
      <button @click="checkAndInstallBinaries" :disabled="installing" class="install-btn">
        {{ installing ? 'Installing...' : 'Check & Install Missing Binaries' }}
      </button>
      <button @click="fixAllServices" :disabled="fixing" class="fix-btn">
        {{ fixing ? 'Fixing...' : 'Fix All Inactive Services' }}
      </button>
      <button @click="runContainerFixScript" :disabled="scriptRunning" class="script-btn">
        {{ scriptRunning ? 'Running...' : 'Fix All Containers' }}
      </button>
      <button @click="runMediaServicesFixScript" :disabled="scriptRunning" class="script-btn">
        {{ scriptRunning ? 'Running...' : 'Fix Media Services' }}
      </button>
      <button @click="runHardwareOptimization" :disabled="scriptRunning" class="script-btn">
        {{ scriptRunning ? 'Running...' : 'Optimize Hardware' }}
      </button>
      <button @click="updateDuckDNS" :disabled="scriptRunning" class="script-btn">
        {{ scriptRunning ? 'Running...' : 'Update DuckDNS' }}
      </button>
    </div>
    
    <!-- Results Section -->
    <div v-if="installResult" class="result-section" :class="{ success: installResult.success, error: !installResult.success }">
      <h3>Installation Results</h3>
      <p>{{ installResult.message }}</p>
      <div v-if="installResult.installed_items.length > 0">
        <strong>Installed:</strong> {{ installResult.installed_items.join(', ') }}
      </div>
      <div v-if="installResult.failed_items.length > 0">
        <strong>Failed:</strong> {{ installResult.failed_items.join(', ') }}
      </div>
    </div>
    
    <div v-if="fixResult" class="result-section" :class="{ success: fixResult.success, error: !fixResult.success }">
      <h3>Service Fix Results</h3>
      <p>{{ fixResult.message }}</p>
      <div v-if="fixResult.actions_taken.length > 0">
        <strong>Actions Taken:</strong>
        <ul>
          <li v-for="action in fixResult.actions_taken" :key="action">{{ action }}</li>
        </ul>
      </div>
    </div>
    
    <div v-if="scriptResult" class="result-section" :class="{ success: scriptResult.success, error: !scriptResult.success }">
      <h3>Script Execution Results</h3>
      <p><strong>Duration:</strong> {{ scriptResult.duration }}</p>
      <pre>{{ scriptResult.output }}</pre>
    </div>
    
    <div v-if="loading">Loading...</div>
    <div v-else>
      <div class="maintenance-section">
        <h2>Services</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Status</th>
              <th>Health</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="service in maintenance.services" :key="service.name">
              <td>{{ service.name }}</td>
              <td>
                <span :class="getStatusClass(service.status)">{{ service.status }}</span>
              </td>
              <td>
                <span v-if="service.active" class="health-good">✓ Running</span>
                <span v-else class="health-bad">✗ Inactive</span>
              </td>
              <td>
                <button @click="controlService(service.name, 'start')" :disabled="service.active">Start</button>
                <button @click="controlService(service.name, 'stop')" :disabled="!service.active">Stop</button>
                <button @click="controlService(service.name, 'restart')">Restart</button>
                <button v-if="!service.active" @click="fixService(service.name)" class="fix-btn">Fix</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      
      <div class="maintenance-section">
        <h2>Binaries</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Path</th>
              <th>Version</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="binary in maintenance.binaries" :key="binary.name">
              <td>{{ binary.name }}</td>
              <td>{{ binary.exists ? binary.path : 'Not found' }}</td>
              <td>{{ binary.version || 'N/A' }}</td>
              <td>
                <span v-if="binary.exists" class="health-good">✓ Installed</span>
                <span v-else class="health-bad">✗ Missing</span>
              </td>
              <td>
                <button v-if="!binary.exists" @click="installBinary(binary.name)" class="install-btn">
                  Install
                </button>
                <span v-else class="status-ok">OK</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      
      <div class="maintenance-section">
        <h2>Config Files</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Path</th>
              <th>Size (KB)</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="config in maintenance.configs" :key="config.name">
              <td>{{ config.name }}</td>
              <td>{{ config.path }}</td>
              <td>{{ config.exists ? (config.size / 1024).toFixed(2) : 'N/A' }}</td>
              <td>
                <span v-if="config.exists" class="health-good">✓ Exists</span>
                <span v-else class="health-bad">✗ Missing</span>
                <span v-if="config.exists && !config.readable" class="health-warning">⚠ Not readable</span>
              </td>
              <td>
                <button v-if="config.exists" @click="editConfig(config)" class="edit-btn">Edit</button>
                <button v-if="!config.exists" @click="createConfig(config)" class="create-btn">Create</button>
                <button v-if="config.exists && !config.readable" @click="fixConfigPermissions(config)" class="fix-btn">Fix Permissions</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'Maintenance',
  setup() {
    const maintenance = ref({ services: [], binaries: [], configs: [] })
    const loading = ref(false)
    const installing = ref(false)
    const fixing = ref(false)
    const scriptRunning = ref(false)
    const installResult = ref(null)
    const fixResult = ref(null)
    const scriptResult = ref(null)

    const fetchMaintenanceOverview = async () => {
      try {
        loading.value = true
        const result = await invoke('get_maintenance_overview')
        maintenance.value = result
      } catch (error) {
        console.error('Failed to fetch maintenance overview:', error)
      } finally {
        loading.value = false
      }
    }

    const controlService = async (name, action) => {
      try {
        await invoke('control_service', { service_name: name, action })
        await fetchMaintenanceOverview()
      } catch (error) {
        console.error(`Failed to ${action} service:`, error)
      }
    }
    
    const checkAndInstallBinaries = async () => {
      try {
        installing.value = true
        installResult.value = null
        const result = await invoke('check_and_install_binaries')
        installResult.value = result
        await fetchMaintenanceOverview()
      } catch (error) {
        console.error('Failed to check and install binaries:', error)
        installResult.value = { success: false, message: error.toString(), installed_items: [], failed_items: [] }
      } finally {
        installing.value = false
      }
    }
    
    const fixAllServices = async () => {
      try {
        fixing.value = true
        fixResult.value = null
        const result = await invoke('fix_all_services')
        fixResult.value = result
        await fetchMaintenanceOverview()
      } catch (error) {
        console.error('Failed to fix all services:', error)
        fixResult.value = { success: false, message: error.toString(), actions_taken: [] }
      } finally {
        fixing.value = false
      }
    }
    
    const runContainerFixScript = async () => {
      try {
        scriptRunning.value = true
        scriptResult.value = null
        const result = await invoke('run_container_fix_script')
        scriptResult.value = result
        await fetchMaintenanceOverview()
      } catch (error) {
        console.error('Failed to run container fix script:', error)
        scriptResult.value = { success: false, output: error.toString(), duration: '0s' }
      } finally {
        scriptRunning.value = false
      }
    }
    
    const runMediaServicesFixScript = async () => {
      try {
        scriptRunning.value = true
        scriptResult.value = null
        const result = await invoke('run_media_services_fix')
        scriptResult.value = result
        await fetchMaintenanceOverview()
      } catch (error) {
        console.error('Failed to run media services fix script:', error)
        scriptResult.value = { success: false, output: error.toString(), duration: '0s' }
      } finally {
        scriptRunning.value = false
      }
    }
    
    const runHardwareOptimization = async () => {
      try {
        scriptRunning.value = true
        scriptResult.value = null
        const result = await invoke('run_hardware_optimization')
        scriptResult.value = result
      } catch (error) {
        console.error('Failed to run hardware optimization:', error)
        scriptResult.value = { success: false, output: error.toString(), duration: '0s' }
      } finally {
        scriptRunning.value = false
      }
    }
    
    const updateDuckDNS = async () => {
      try {
        scriptRunning.value = true
        scriptResult.value = null
        const result = await invoke('update_duckdns')
        scriptResult.value = result
      } catch (error) {
        console.error('Failed to update DuckDNS:', error)
        scriptResult.value = { success: false, output: error.toString(), duration: '0s' }
      } finally {
        scriptRunning.value = false
      }
    }
    
    const getStatusClass = (status) => {
      if (status === 'Active' || status === 'Running') return 'status-active'
      if (status === 'Inactive' || status === 'Stopped') return 'status-inactive'
      return 'status-unknown'
    }
    
    const fixService = async (serviceName) => {
      try {
        await invoke('control_service', { service_name: serviceName, action: 'restart' })
        await fetchMaintenanceOverview()
      } catch (error) {
        console.error(`Failed to fix service ${serviceName}:`, error)
      }
    }
    
    const installBinary = async (binaryName) => {
      try {
        installing.value = true
        const result = await invoke('check_and_install_binaries')
        installResult.value = result
        await fetchMaintenanceOverview()
      } catch (error) {
        console.error(`Failed to install binary ${binaryName}:`, error)
      } finally {
        installing.value = false
      }
    }
    
    const editConfig = async (config) => {
      // Placeholder for config editing functionality
      console.log('Edit config:', config)
    }
    
    const createConfig = async (config) => {
      // Placeholder for config creation functionality
      console.log('Create config:', config)
    }
    
    const fixConfigPermissions = async (config) => {
      // Placeholder for fixing config permissions
      console.log('Fix config permissions:', config)
    }

    onMounted(fetchMaintenanceOverview)

    return {
      maintenance,
      loading,
      installing,
      fixing,
      scriptRunning,
      installResult,
      fixResult,
      scriptResult,
      controlService,
      checkAndInstallBinaries,
      fixAllServices,
      runContainerFixScript,
      runMediaServicesFixScript,
      runHardwareOptimization,
      updateDuckDNS,
      getStatusClass,
      fixService,
      installBinary,
      editConfig,
      createConfig,
      fixConfigPermissions
    }
  }
}
</script>

<style scoped>
.maintenance {
  padding: 20px;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 12px;
  backdrop-filter: blur(10px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.global-actions {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
  margin-bottom: 2rem;
  padding: 1.5rem;
  background: rgba(102, 126, 234, 0.1);
  border-radius: 8px;
  border: 1px solid rgba(102, 126, 234, 0.2);
}

button {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  margin-right: 0;
}

.install-btn {
  background: linear-gradient(135deg, #4CAF50, #45a049);
  color: white;
}

.install-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #45a049, #3d8b40);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(76, 175, 80, 0.3);
}

.fix-btn {
  background: linear-gradient(135deg, #FF9800, #F57C00);
  color: white;
}

.fix-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #F57C00, #E65100);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(255, 152, 0, 0.3);
}

.script-btn {
  background: linear-gradient(135deg, #2196F3, #1976D2);
  color: white;
}

.script-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #1976D2, #1565C0);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(33, 150, 243, 0.3);
}

.edit-btn {
  background: linear-gradient(135deg, #9C27B0, #7B1FA2);
  color: white;
}

.create-btn {
  background: linear-gradient(135deg, #607D8B, #455A64);
  color: white;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.result-section {
  margin: 1.5rem 0;
  padding: 1.5rem;
  border-radius: 8px;
  border-left: 4px solid;
}

.result-section.success {
  background: rgba(76, 175, 80, 0.1);
  border-left-color: #4CAF50;
  color: #2E7D32;
}

.result-section.error {
  background: rgba(244, 67, 54, 0.1);
  border-left-color: #F44336;
  color: #C62828;
}

.result-section h3 {
  margin-bottom: 1rem;
  font-size: 1.2rem;
}

.result-section pre {
  background: rgba(0, 0, 0, 0.05);
  padding: 1rem;
  border-radius: 4px;
  white-space: pre-wrap;
  word-wrap: break-word;
  max-height: 300px;
  overflow-y: auto;
}

.result-section ul {
  margin: 0.5rem 0;
  padding-left: 1.5rem;
}

.maintenance-section {
  margin-top: 2rem;
  background: white;
  border-radius: 8px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.maintenance-section h2 {
  color: #4a5568;
  margin-bottom: 1rem;
  font-size: 1.5rem;
}

table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 1rem;
}

th, td {
  border: 1px solid #e2e8f0;
  padding: 0.75rem;
  text-align: left;
}

th {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  font-weight: 600;
}

tr:nth-child(even) {
  background: rgba(102, 126, 234, 0.05);
}

.status-active {
  color: #4CAF50;
  font-weight: 600;
}

.status-inactive {
  color: #F44336;
  font-weight: 600;
}

.status-unknown {
  color: #FF9800;
  font-weight: 600;
}

.health-good {
  color: #4CAF50;
  font-weight: 600;
}

.health-bad {
  color: #F44336;
  font-weight: 600;
}

.health-warning {
  color: #FF9800;
  font-weight: 600;
  margin-left: 0.5rem;
}

.status-ok {
  color: #4CAF50;
  font-weight: 600;
}

td button {
  padding: 0.5rem 1rem;
  margin-right: 0.5rem;
  font-size: 0.875rem;
}

@media (max-width: 768px) {
  .global-actions {
    flex-direction: column;
  }
  
  .global-actions button {
    width: 100%;
  }
  
  table {
    font-size: 0.875rem;
  }
  
  th, td {
    padding: 0.5rem;
  }
}
</style>

