<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

interface Device {
  name: string
  path: string
  total: number
}

const destPath = ref<string | null>(null)
const devices  = ref<Device[]>([])
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
  devices.value = await invoke<Device[]>('list_external_devices')
}

async function copyDevice (path: string) {
  if (!destPath.value) return
  await invoke('import_device', {
    devicePath: path,
    destPath: destPath.value
  })
}

function formatSize (bytes: number) {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let i = 0
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024
    i++
  }
  return `${size.toFixed(1)} ${units[i]}`
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
        <li v-for="d in devices" :key="d.path" style="display: flex; align-items: center; gap: 0.5rem;">
          <span>{{ d.name }} ({{ formatSize(d.total) }}) - {{ d.path }}</span>
          <button @click="copyDevice(d.path)" :disabled="!destPath">{{ t('import.copy') }}</button>
        </li>
      </ul>
      <span v-else>-</span>
    </div>
    <h1>{{ t('import.title') }}</h1>
  </div>
</template>
