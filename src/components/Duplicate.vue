<script setup lang="ts">
import { ref, onBeforeUnmount } from 'vue'
import HamsterLoader from './HamsterLoader.vue'
import { open } from '@tauri-apps/plugin-dialog'     // v1-API ¹
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'

interface DuplicateGroup {
  hash: string
  paths: string[]
}

interface DuplicateProgress {
  progress: number
  eta_seconds: number
}

const duplicates = ref<DuplicateGroup[]>([])
const busy       = ref(false)
const progress   = ref(0)
const eta        = ref(0)
let unlisten: UnlistenFn | null = null
const { t } = useI18n()

async function chooseAndScan () {
  const selected = await open({ directory: true, multiple: false })
  if (!selected) return          // Dialog abgebrochen

  busy.value = true
  progress.value = 0
  eta.value = 0
  if (unlisten) {
    unlisten()
    unlisten = null
  }
  unlisten = await listen<DuplicateProgress>('duplicate_progress', (e) => {
    progress.value = e.payload.progress
    eta.value = e.payload.eta_seconds
  })
  try {
    duplicates.value = await invoke<DuplicateGroup[]>('scan_folder_stream', {
      path: selected
    })
  } finally {
    busy.value = false
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }
}

onBeforeUnmount(() => {
  if (unlisten) unlisten()
})
</script>

<template>
  <div class="view">
    <button @click="chooseAndScan" :disabled="busy">
      {{ busy ? t('duplicate.scanning') : t('duplicate.choose') }}
    </button>

    <HamsterLoader v-if="busy" />
    <div v-if="busy" style="text-align: center; margin-top: 0.5rem;">
      {{ Math.round(progress * 100) }}% - ETA {{ eta.toFixed(1) }}s
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

