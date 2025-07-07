<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'     // v1-API ¹
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'

interface DuplicateGroup {
  hash: string
  paths: string[]
}

const duplicates = ref<DuplicateGroup[]>([])
const progress   = ref(0)
const busy       = ref(false)
const { t } = useI18n()

async function chooseAndScan () {
  const selected = await open({ directory: true, multiple: false })
  if (!selected) return          // Dialog abgebrochen

  busy.value = true
  duplicates.value = []
  progress.value = 0

  const unlistenProgress = await listen<number>('scan_progress', (e) => {
    progress.value = e.payload
  })
  const unlistenDuplicate = await listen<DuplicateGroup>('duplicate_found', (e) => {
    const existing = duplicates.value.find(d => d.hash === e.payload.hash)
    if (existing) {
      existing.paths = e.payload.paths
    } else {
      duplicates.value.push(e.payload)
    }
  })

  try {
    duplicates.value = await invoke<DuplicateGroup[]>('scan_folder', {
      path: selected
    })
  } finally {
    unlistenProgress()
    unlistenDuplicate()
    busy.value = false
    progress.value = 100
  }
}
</script>

<template>
  <div class="view">
    <button @click="chooseAndScan" :disabled="busy">
      {{ busy ? t('duplicate.scanning') : t('duplicate.choose') }}
    </button>

    <div v-if="busy" style="margin-top: 0.5rem;">
      <progress :value="progress" max="100" style="width: 100%;"></progress>
      <span>{{ Math.round(progress) }}%</span>
    </div>

    <ul v-if="duplicates.length" style="margin-top: 1rem;">
      <li v-for="d in duplicates" :key="d.hash" style="margin-top: 0.5rem;">
        <strong style="font-size: 0.875rem;">{{ d.hash }}</strong>
        <ul style="margin-left: 1rem; list-style: disc;">
          <li v-for="p in d.paths" :key="p">{{ p }}</li>
        </ul>
      </li>
    </ul>
  </div>
</template>
