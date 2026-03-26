<template>
  <div class="model-selector">
    <div class="form-group">
      <label class="form-label">AI Model</label>
      <select v-model="selectedModel" class="form-input" @change="updateModel">
        <option v-for="model in availableModels" :key="model" :value="model">
          {{ model }}
        </option>
      </select>
    </div>
    <button class="btn btn-secondary refresh-models-btn" @click="loadModels" :disabled="loading">
      {{ loading ? '⟳' : '↻' }} Refresh
    </button>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'ModelSelector',
  setup() {
    const selectedModel = ref('codellama:7b')
    const availableModels = ref(['codellama:7b'])
    const loading = ref(false)

    const loadModels = async () => {
      loading.value = true
      try {
        const models = await invoke('get_available_models')
        availableModels.value = models.length > 0 ? models : ['codellama:7b']
        if (!availableModels.value.includes(selectedModel.value)) {
          selectedModel.value = availableModels.value[0]
        }
      } catch (error) {
        console.error('Failed to load models:', error)
        availableModels.value = ['codellama:7b']
      } finally {
        loading.value = false
      }
    }

    const updateModel = () => {
      // Model selection logic can be handled by parent component
      console.log('Selected model:', selectedModel.value)
    }

    onMounted(() => {
      loadModels()
    })

    return {
      selectedModel,
      availableModels,
      loading,
      loadModels,
      updateModel
    }
  }
}
</script>

<style scoped>
.model-selector {
  padding: 1rem;
  border-top: 1px solid var(--border-color);
}

.refresh-models-btn {
  width: 100%;
  margin-top: 0.5rem;
  font-size: 0.75rem;
}
</style>
