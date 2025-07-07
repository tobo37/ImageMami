<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const destPath = ref<string | null>(null)
const devices  = ref<string[]>([])
const { t } = useI18n()

onMounted(() => {
  destPath.value = localStorage.getItem('importDest')
  loadDevices()
})

async function chooseDest () {
  const selected = await open({ directory: true, multiple: false })
  if (!selected) return
  const path = Array.isArray(selected) ? selected[0] : selected
  destPath.value = path
  localStorage.setItem('importDest', path)
}

async function loadDevices () {
  devices.value = await invoke<string[]>('list_external_devices')
}
</script>

<template>
  <div class="view">
    <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 1rem;">
      <strong>{{ t('import.destination') }}</strong>
      <span>{{ destPath || '-' }}</span>
      <button @click="chooseDest">{{ t('import.choose') }}</button>
    </div>
    <div style="margin-bottom: 1rem;">
      <div style="display: flex; align-items: center; gap: 0.5rem;">
        <strong>{{ t('import.devices') }}</strong>
        <button @click="loadDevices">{{ t('import.refresh') }}</button>
      </div>
      <ul v-if="devices.length" style="margin-top: 0.5rem;">
        <li v-for="d in devices" :key="d">{{ d }}</li>
      </ul>
      <span v-else>-</span>
    </div>
    <h1>{{ t('import.title') }}</h1>
  </div>
</template>
