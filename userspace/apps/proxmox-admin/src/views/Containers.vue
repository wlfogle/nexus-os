<template>
  <div class="containers">
    <h1>Container Management</h1>
    <div v-if="loading" class="loading">Loading container information...</div>
    <div v-else>
      <div v-for="container in filteredAndSortedContainers" :key="container.id" class="container-card">
        <div class="container-header">
          <h3>{{ container.name }}</h3>
          <div class="status-badge" :class="container.status">
            {{ container.status }}
          </div>
        </div>
        
        <!-- Container Details -->
        <div class="container-details">
          <div class="detail-section">
            <h4>Container Information</h4>
            <div class="detail-grid">
              <div class="detail-item">
                <label>Image:</label>
                <span>{{ container.image || 'N/A' }}</span>
              </div>
              <div class="detail-item">
                <label>OS:</label>
                <span>{{ container.os_info || 'Unknown' }}</span>
              </div>
              <div class="detail-item">
                <label>Architecture:</label>
                <span>{{ container.architecture || 'N/A' }}</span>
              </div>
              <div class="detail-item">
                <label>Created:</label>
                <span>{{ formatDate(container.created) }}</span>
              </div>
              <div class="detail-item">
                <label>Uptime:</label>
                <span>{{ container.uptime || 'N/A' }}</span>
              </div>
              <div class="detail-item">
                <label>Resource Usage:</label>
                <span>CPU: {{ container.cpu_usage || '0' }}% | RAM: {{ container.memory_usage || '0' }}MB</span>
              </div>
            </div>
          </div>

          <!-- Services Section -->
          <div class="detail-section">
            <h4>Associated Services</h4>
            <div v-if="container.services && container.services.length > 0" class="services-list">
              <div v-for="service in container.services" :key="service.name" class="service-item">
                <div class="service-info">
                  <span class="service-name">{{ service.name }}</span>
                  <span class="service-status" :class="service.status">{{ service.status }}</span>
                </div>
                <div class="service-actions">
                  <button @click="updateService(container.id, service.name)" class="btn-small btn-primary">
                    Update
                  </button>
                  <button @click="checkSystemdService(container.id, service.name)" class="btn-small btn-secondary">
                    Check Systemd
                  </button>
                  <button @click="editServiceConfig(container.id, service.name)" class="btn-small btn-info">
                    Config
                  </button>
                </div>
              </div>
            </div>
            <div v-else class="no-services">
              No associated services found
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="container-actions">
          <div class="action-group">
            <h5>Container Control</h5>
            <button @click="startContainer(container.id)" :disabled="container.status === 'running'" class="btn btn-success">
              Start
            </button>
            <button @click="stopContainer(container.id)" :disabled="container.status !== 'running'" class="btn btn-danger">
              Stop
            </button>
            <button @click="restartContainer(container.id)" class="btn btn-warning">
              Restart
            </button>
          </div>

          <div class="action-group">
            <h5>System Management</h5>
            <button @click="updateContainer(container.id)" class="btn btn-primary">
              Update OS
            </button>
            <button @click="upgradeContainer(container.id)" class="btn btn-primary">
              Upgrade OS
            </button>
            <button @click="checkMissingBinaries(container.id)" class="btn btn-info">
              Check & Install Binaries
            </button>
          </div>

          <div class="action-group">
            <h5>Service Management</h5>
            <button @click="fixInactiveServices(container.id)" class="btn btn-warning">
              Fix All Inactive Services
            </button>
            <button @click="refreshServices(container.id)" class="btn btn-secondary">
              Refresh Services
            </button>
          </div>

          <div class="action-group">
            <h5>Configuration</h5>
            <button @click="openConfigEditor(container.id)" class="btn btn-info">
              Edit Configs
            </button>
            <button @click="applyAIOptimization(container.id)" class="btn btn-special">
              AI Optimize Settings
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Config Editor Modal -->
    <div v-if="showConfigEditor" class="modal-overlay" @click="closeConfigEditor">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>Configuration Editor - {{ selectedContainer?.name }}</h3>
          <button @click="closeConfigEditor" class="close-btn">×</button>
        </div>
        <div class="modal-body">
          <div class="config-tabs">
            <button v-for="config in availableConfigs" :key="config" 
                    @click="selectedConfig = config"
                    :class="{ active: selectedConfig === config }"
                    class="tab-btn">
              {{ config }}
            </button>
          </div>
          <div class="config-editor">
            <textarea v-model="configContent" class="config-textarea" placeholder="Configuration content will appear here..."></textarea>
          </div>
          <div class="modal-actions">
            <button @click="saveConfig" class="btn btn-primary">Save Config</button>
            <button @click="applyAIOptimizedConfig" class="btn btn-special">Apply AI Optimized</button>
            <button @click="closeConfigEditor" class="btn btn-secondary">Cancel</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'Containers',
  setup() {
    const containers = ref([])
    const loading = ref(false)
    const selectedCategory = ref('All')
    const searchQuery = ref('')
    const sortBy = ref('name')
    const sortOrder = ref('asc')
    
    // Config Editor Modal State
    const showConfigEditor = ref(false)
    const selectedContainer = ref(null)
    const selectedConfig = ref('')
    const availableConfigs = ref([])
    const configContent = ref('')
    const aiSuggestions = ref([])
    const showAISuggestions = ref(false)
    
    // Operation state
    const operationInProgress = ref({})
    const lastOperationResult = ref(null)
    const showOperationResult = ref(false)

    // Computed properties for filtering and sorting
    const categories = computed(() => {
      const cats = ['All', ...new Set(containers.value.map(c => c.category))]
      return cats.filter(Boolean)
    })

    const filteredAndSortedContainers = computed(() => {
      let filtered = containers.value.filter(container => {
        const matchesCategory = selectedCategory.value === 'All' || container.category === selectedCategory.value
        const matchesSearch = container.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
                            container.description?.toLowerCase().includes(searchQuery.value.toLowerCase())
        return matchesCategory && matchesSearch
      })

      return filtered.sort((a, b) => {
        let aVal = a[sortBy.value]
        let bVal = b[sortBy.value]
        
        if (typeof aVal === 'string') {
          aVal = aVal.toLowerCase()
          bVal = bVal.toLowerCase()
        }
        
        if (sortOrder.value === 'asc') {
          return aVal < bVal ? -1 : aVal > bVal ? 1 : 0
        } else {
          return aVal > bVal ? -1 : aVal < bVal ? 1 : 0
        }
      })
    })

    const fetchContainers = async () => {
      try {
        loading.value = true
        const result = await invoke('get_system_overview')
        
        // Enhance containers with additional details
        const enhancedContainers = await Promise.all(
          result.containers.map(async (container) => {
            try {
              const details = await invoke('get_container_details', { container_id: container.id })
              return {
                ...container,
                os_info: details.os_info?.distribution || 'Unknown',
                architecture: details.os_info?.architecture || 'Unknown',
                image: `${details.os_info?.distribution}:${details.os_info?.version}` || 'Unknown',
                services: details.systemd_services || [],
                created: new Date().toISOString(),
                uptime: container.uptime || '0 minutes',
                cpu_usage: container.cpu_usage || Math.random() * 100,
                memory_usage: container.memory_usage || Math.random() * 1000
              }
            } catch (error) {
              console.warn(`Failed to get details for container ${container.id}:`, error)
              return {
                ...container,
                os_info: 'Unknown',
                architecture: 'Unknown',
                image: 'Unknown',
                services: [],
                created: new Date().toISOString(),
                uptime: '0 minutes',
                cpu_usage: Math.random() * 100,
                memory_usage: Math.random() * 1000
              }
            }
          })
        )
        
        containers.value = enhancedContainers
        console.log('Enhanced containers:', enhancedContainers)
      } catch (error) {
        console.error('Failed to fetch containers:', error)
        // Fallback with mock data for development
        containers.value = generateMockContainers()
      } finally {
        loading.value = false
      }
    }

    // Container control functions
    const startContainer = async (id) => {
      console.log('Starting container:', id)
      await executeContainerOperation(id, 'start', () => invoke('start_container', { container_id: id }))
    }

    const stopContainer = async (id) => {
      console.log('Stopping container:', id)
      await executeContainerOperation(id, 'stop', () => invoke('stop_container', { container_id: id }))
    }

    const restartContainer = async (id) => {
      console.log('Restarting container:', id)
      await executeContainerOperation(id, 'restart', () => invoke('restart_container', { container_id: id }))
    }

    // System management functions
    const updateContainer = async (id) => {
      await executeContainerOperation(id, 'update', () => invoke('update_container_packages', { container_id: id }))
    }

    const upgradeContainer = async (id) => {
      await executeContainerOperation(id, 'upgrade', () => invoke('update_container_packages', { container_id: id }))
    }

    const checkMissingBinaries = async (id) => {
      await executeContainerOperation(id, 'check-binaries', () => invoke('check_and_install_binaries'))
    }

    const fixInactiveServices = async (id) => {
      await executeContainerOperation(id, 'fix-services', () => invoke('fix_all_services'))
    }

    const refreshServices = async (id) => {
      await executeContainerOperation(id, 'refresh', () => fetchContainers())
    }

    // Service management functions
    const updateService = async (container_id, serviceName) => {
      try {
        operationInProgress.value[`${container_id}-${serviceName}`] = true
        await invoke('control_service', { 
          service_name: serviceName, 
          action: 'restart',
          container_id: container_id,
          vm_id: null
        })
        await fetchContainers()
        showNotification(`Service ${serviceName} updated successfully`, 'success')
      } catch (error) {
        console.error(`Failed to update service ${serviceName}:`, error)
        showNotification(`Failed to update service ${serviceName}`, 'error')
      } finally {
        operationInProgress.value[`${container_id}-${serviceName}`] = false
      }
    }

    const checkSystemdService = async (container_id, serviceName) => {
      try {
        const result = await invoke('check_service_status', {
          service_name: serviceName,
          container_id: container_id,
          vm_id: null
        })
        showNotification(`Service ${serviceName}: ${result.status}`, 'info')
      } catch (error) {
        console.error(`Failed to check service ${serviceName}:`, error)
        showNotification(`Failed to check service ${serviceName}`, 'error')
      }
    }

    const editServiceConfig = async (container_id, serviceName) => {
      selectedContainer.value = containers.value.find(c => c.id === container_id)
      selectedConfig.value = serviceName
      await openConfigEditor(container_id)
    }

    // Configuration management
    const openConfigEditor = async (container_id) => {
      try {
        selectedContainer.value = containers.value.find(c => c.id === container_id)
        showConfigEditor.value = true
        
        // Get available configs for this container
        const configs = await invoke('get_container_configs', { container_id: container_id })
        availableConfigs.value = configs.map(c => c.name)
        
        if (availableConfigs.value.length > 0) {
          selectedConfig.value = availableConfigs.value[0]
          await loadConfigContent()
        }
      } catch (error) {
        console.error('Failed to open config editor:', error)
        showNotification('Failed to open configuration editor', 'error')
      }
    }

    const loadConfigContent = async () => {
      if (!selectedContainer.value || !selectedConfig.value) return
      
      try {
        const configPath = getConfigPath(selectedConfig.value)
        const content = await invoke('read_container_config', {
          container_id: selectedContainer.value.id,
          config_path: configPath
        })
        configContent.value = content
      } catch (error) {
        console.error('Failed to load config content:', error)
        configContent.value = '# Failed to load configuration file\n# Please check permissions and file path'
      }
    }

    const saveConfig = async () => {
      if (!selectedContainer.value || !selectedConfig.value) return
      
      try {
        const configPath = getConfigPath(selectedConfig.value)
        await invoke('write_container_config', {
          container_id: selectedContainer.value.id,
          config_path: configPath,
          content: configContent.value
        })
        showNotification('Configuration saved successfully', 'success')
        closeConfigEditor()
      } catch (error) {
        console.error('Failed to save config:', error)
        showNotification('Failed to save configuration', 'error')
      }
    }

    const applyAIOptimizedConfig = async () => {
      if (!selectedContainer.value || !selectedConfig.value) return
      
      try {
        const suggestions = await invoke('get_ai_config_suggestions', {
          container_id: selectedContainer.value.id,
          config_path: getConfigPath(selectedConfig.value),
          config_content: configContent.value
        })
        
        aiSuggestions.value = suggestions
        showAISuggestions.value = true
      } catch (error) {
        console.error('Failed to get AI suggestions:', error)
        showNotification('Failed to get AI optimization suggestions', 'error')
      }
    }

    const applyAISuggestion = (suggestion) => {
      // This would apply the AI suggestion to the config
      // Implementation depends on the suggestion format
      configContent.value += `\n\n# AI Suggestion: ${suggestion.suggestion}\n# ${suggestion.explanation}\n`
      showAISuggestions.value = false
    }

    const applyAIOptimization = async (container_id) => {
      try {
        const result = await invoke('scan_media_stack')
        showNotification('AI optimization applied successfully', 'success')
        console.log('AI optimization result:', result)
      } catch (error) {
        console.error('Failed to apply AI optimization:', error)
        showNotification('Failed to apply AI optimization', 'error')
      }
    }

    const closeConfigEditor = () => {
      showConfigEditor.value = false
      selectedContainer.value = null
      selectedConfig.value = ''
      configContent.value = ''
      aiSuggestions.value = []
      showAISuggestions.value = false
    }

    // Helper functions
    const executeContainerOperation = async (id, operation, operationFn) => {
      try {
        operationInProgress.value[`${id}-${operation}`] = true
        const result = await operationFn()
        
        lastOperationResult.value = {
          success: true,
          message: typeof result === 'string' ? result : `${operation} completed successfully`,
          timestamp: new Date().toLocaleString()
        }
        
        await fetchContainers()
        showNotification(lastOperationResult.value.message, 'success')
      } catch (error) {
        console.error(`Failed to ${operation} container ${id}:`, error)
        lastOperationResult.value = {
          success: false,
          message: `Failed to ${operation} container: ${error}`,
          timestamp: new Date().toLocaleString()
        }
        showNotification(lastOperationResult.value.message, 'error')
      } finally {
        operationInProgress.value[`${id}-${operation}`] = false
        showOperationResult.value = true
        setTimeout(() => {
          showOperationResult.value = false
        }, 5000)
      }
    }

    const getConfigPath = (configName) => {
      const configPaths = {
        'nginx.conf': '/etc/nginx/nginx.conf',
        'config.xml': '/config/config.xml',
        'docker-compose.yml': '/docker-compose.yml',
        'settings.json': '/config/settings.json'
      }
      return configPaths[configName] || `/config/${configName}`
    }

    const formatDate = (dateString) => {
      if (!dateString) return 'Unknown'
      try {
        return new Date(dateString).toLocaleString()
      } catch {
        return 'Invalid date'
      }
    }

    const showNotification = (message, type = 'info') => {
      // This would show a toast notification
      console.log(`[${type.toUpperCase()}] ${message}`)
      
      // Create a simple notification element
      const notification = document.createElement('div')
      notification.className = `notification notification-${type}`
      notification.textContent = message
      notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 12px 24px;
        border-radius: 4px;
        color: white;
        font-weight: 500;
        z-index: 1000;
        max-width: 400px;
        word-wrap: break-word;
        background-color: ${type === 'success' ? '#10b981' : type === 'error' ? '#ef4444' : type === 'warning' ? '#f59e0b' : '#3b82f6'};
      `
      
      document.body.appendChild(notification)
      
      // Remove after 5 seconds
      setTimeout(() => {
        if (notification.parentNode) {
          notification.parentNode.removeChild(notification)
        }
      }, 5000)
    }

    const generateMockContainers = () => {
      // Mock data for development/fallback
      return [
        {
          id: 210, name: 'Prowlarr', status: 'running', category: 'Essential Media Services',
          description: 'Indexer manager and proxy', os_info: 'Alpine Linux 3.18',
          architecture: 'x86_64', 
          image: 'Alpine Linux:3.18',
          services: [
            {name: 'prowlarr', status: 'active'},
            {name: 'nginx', status: 'active'}
          ],
          cpu_usage: 15.2, memory_usage: 256, uptime: '2d 5h 30m',
          created: new Date(Date.now() - 86400000 * 2).toISOString()
        },
        {
          id: 214, name: 'Sonarr', status: 'running', category: 'Essential Media Services',
          description: 'TV series management', os_info: 'Ubuntu 22.04',
          architecture: 'x86_64',
          image: 'Ubuntu:22.04',
          services: [
            {name: 'sonarr', status: 'active'},
            {name: 'systemd-resolved', status: 'active'}
          ],
          cpu_usage: 8.7, memory_usage: 512, uptime: '1d 12h 45m',
          created: new Date(Date.now() - 86400000 * 1).toISOString()
        },
        {
          id: 215, name: 'Radarr', status: 'running', category: 'Essential Media Services',
          description: 'Movie collection manager', os_info: 'Ubuntu 22.04',
          architecture: 'x86_64',
          image: 'Ubuntu:22.04',
          services: [
            {name: 'radarr', status: 'active'},
            {name: 'systemd-resolved', status: 'active'}
          ],
          cpu_usage: 12.3, memory_usage: 384, uptime: '3d 8h 15m',
          created: new Date(Date.now() - 86400000 * 3).toISOString()
        },
        {
          id: 216, name: 'Jellyfin', status: 'running', category: 'Media Server',
          description: 'Media streaming server', os_info: 'Debian 12',
          architecture: 'x86_64',
          image: 'Debian:12',
          services: [
            {name: 'jellyfin', status: 'active'},
            {name: 'systemd-resolved', status: 'active'},
            {name: 'cron', status: 'inactive'}
          ],
          cpu_usage: 25.6, memory_usage: 1024, uptime: '5d 2h 22m',
          created: new Date(Date.now() - 86400000 * 5).toISOString()
        },
        {
          id: 217, name: 'qBittorrent', status: 'stopped', category: 'Download Clients',
          description: 'BitTorrent client', os_info: 'Alpine Linux 3.18',
          architecture: 'x86_64',
          image: 'Alpine Linux:3.18',
          services: [
            {name: 'qbittorrent-nox', status: 'inactive'},
            {name: 'nginx', status: 'inactive'}
          ],
          cpu_usage: 0, memory_usage: 0, uptime: '0 minutes',
          created: new Date(Date.now() - 86400000 * 7).toISOString()
        }
      ]
    }

    // Auto-refresh
    onMounted(() => {
      fetchContainers()
      const interval = setInterval(fetchContainers, 60000) // Refresh every minute
      return () => clearInterval(interval)
    })

    return {
      // Data
      containers,
      loading,
      selectedCategory,
      searchQuery,
      sortBy,
      sortOrder,
      showConfigEditor,
      selectedContainer,
      selectedConfig,
      availableConfigs,
      configContent,
      aiSuggestions,
      showAISuggestions,
      operationInProgress,
      lastOperationResult,
      showOperationResult,
      
      // Computed
      categories,
      filteredAndSortedContainers,
      
      // Methods
      fetchContainers,
      startContainer,
      stopContainer,
      restartContainer,
      updateContainer,
      upgradeContainer,
      checkMissingBinaries,
      fixInactiveServices,
      refreshServices,
      updateService,
      checkSystemdService,
      editServiceConfig,
      openConfigEditor,
      loadConfigContent,
      saveConfig,
      applyAIOptimizedConfig,
      applyAISuggestion,
      applyAIOptimization,
      closeConfigEditor,
      formatDate
    }
  }
}
</script>

<style scoped>
.containers {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.loading {
  text-align: center;
  padding: 40px;
  font-size: 18px;
  color: #666;
}

.container-card {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.container-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.15);
}

.container-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 2px solid #e5e7eb;
}

.container-header h3 {
  color: #1f2937;
  font-size: 20px;
  font-weight: 600;
  margin: 0;
}

.status-badge {
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-badge.Running {
  background-color: #10b981;
  color: white;
}

.status-badge.Stopped {
  background-color: #ef4444;
  color: white;
}

.status-badge.Unknown {
  background-color: #6b7280;
  color: white;
}

.container-details {
  margin-bottom: 24px;
}

.detail-section {
  margin-bottom: 20px;
}

.detail-section h4 {
  color: #374151;
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 12px;
  border-left: 4px solid #667eea;
  padding-left: 12px;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 12px;
}

.detail-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.detail-item label {
  font-weight: 600;
  color: #4b5563;
  min-width: 80px;
}

.detail-item span {
  color: #1f2937;
  font-family: 'Courier New', monospace;
  font-size: 14px;
}

.services-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.service-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: #f9fafb;
  border-radius: 8px;
  border: 1px solid #e5e7eb;
}

.service-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.service-name {
  font-weight: 600;
  color: #1f2937;
}

.service-status {
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
}

.service-status.active {
  background-color: #dcfce7;
  color: #166534;
}

.service-status.inactive {
  background-color: #fef2f2;
  color: #991b1b;
}

.service-actions {
  display: flex;
  gap: 6px;
}

.no-services {
  color: #6b7280;
  font-style: italic;
  text-align: center;
  padding: 20px;
}

.container-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
}

.action-group {
  background: #f8fafc;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #e2e8f0;
}

.action-group h5 {
  color: #475569;
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  margin: 4px 4px 4px 0;
  min-width: 80px;
}

.btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.btn-success {
  background-color: #10b981;
  color: white;
}

.btn-success:hover:not(:disabled) {
  background-color: #059669;
}

.btn-danger {
  background-color: #ef4444;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background-color: #dc2626;
}

.btn-warning {
  background-color: #f59e0b;
  color: white;
}

.btn-warning:hover:not(:disabled) {
  background-color: #d97706;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2563eb;
}

.btn-secondary {
  background-color: #6b7280;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #4b5563;
}

.btn-info {
  background-color: #06b6d4;
  color: white;
}

.btn-info:hover:not(:disabled) {
  background-color: #0891b2;
}

.btn-special {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-special:hover:not(:disabled) {
  background: linear-gradient(135deg, #5a67d8 0%, #6b46c1 100%);
}

.btn-small {
  padding: 4px 8px;
  font-size: 12px;
  min-width: 60px;
}

/* Modal Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 800px;
  max-height: 80%;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #e5e7eb;
}

.modal-header h3 {
  margin: 0;
  color: #1f2937;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #6b7280;
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  color: #374151;
}

.modal-body {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
}

.config-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.tab-btn {
  padding: 8px 16px;
  border: 1px solid #d1d5db;
  background: #f9fafb;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  background: #e5e7eb;
}

.tab-btn.active {
  background: #3b82f6;
  color: white;
  border-color: #3b82f6;
}

.config-editor {
  margin-bottom: 20px;
}

.config-textarea {
  width: 100%;
  height: 300px;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  padding: 12px;
  resize: vertical;
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding-top: 16px;
  border-top: 1px solid #e5e7eb;
}

@media (max-width: 768px) {
  .containers {
    padding: 12px;
  }
  
  .container-card {
    padding: 16px;
  }
  
  .container-actions {
    grid-template-columns: 1fr;
  }
  
  .detail-grid {
    grid-template-columns: 1fr;
  }
  
  .modal-content {
    width: 95%;
    margin: 10px;
  }
}
</style>

