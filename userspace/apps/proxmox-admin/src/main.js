import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'

// Import views
import Dashboard from './views/Dashboard.vue'
import Containers from './views/Containers.vue'
import VirtualMachines from './views/VirtualMachines.vue'
import Maintenance from './views/Maintenance.vue'
import SystemHealth from './views/SystemHealth.vue'
import ProxmoxHost from './views/ProxmoxHost.vue'

// Create router
const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'Dashboard', component: Dashboard },
    { path: '/containers', name: 'Containers', component: Containers },
    { path: '/vms', name: 'VirtualMachines', component: VirtualMachines },
    { path: '/maintenance', name: 'Maintenance', component: Maintenance },
    { path: '/system', name: 'SystemHealth', component: SystemHealth },
    { path: '/proxmox-host', name: 'ProxmoxHost', component: ProxmoxHost },
  ]
})

// Create app
const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
