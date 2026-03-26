<template>
  <div class="virtual-machines">
    <h1>Virtual Machines</h1>
    <div v-if="loading">Loading...</div>
    <div v-else>
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Status</th>
            <th>Description</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="vm in vms" :key="vm.id">
            <td>{{ vm.name }}</td>
            <td>{{ vm.status }}</td>
            <td>{{ vm.description }}</td>
            <td>
              <button @click="startVM(vm.id)" :disabled="vm.status === 'Running'">Start</button>
              <button @click="stopVM(vm.id)" :disabled="vm.status !== 'Running'">Stop</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'VirtualMachines',
  setup() {
    const vms = ref([])
    const loading = ref(false)

    const fetchVMs = async () => {
      try {
        loading.value = true
        const result = await invoke('get_system_overview')
        vms.value = result.vms
      } catch (error) {
        console.error('Failed to fetch VMs:', error)
      } finally {
        loading.value = false
      }
    }

    const startVM = async (id) => {
      try {
        await invoke('start_vm', { vm_id: id })
        await fetchVMs()
      } catch (error) {
        console.error('Failed to start VM:', error)
      }
    }

    const stopVM = async (id) => {
      try {
        await invoke('stop_vm', { vm_id: id })
        await fetchVMs()
      } catch (error) {
        console.error('Failed to stop VM:', error)
      }
    }

    onMounted(fetchVMs)

    return {
      vms,
      loading,
      startVM,
      stopVM
    }
  }
}
</script>

<style scoped>
.virtual-machines {
  padding: 20px;
}

button {
  margin-right: 5px;
}
table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 20px;
}
th, td {
  border: 1px solid #ddd;
  padding: 8px;
}
th {
  background-color: #f2f2f2;
}
</style>

