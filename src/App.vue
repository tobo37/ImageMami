<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'     // v1-API ¹
import { invoke } from '@tauri-apps/api/core'

interface DuplicateGroup {
  hash: string
  paths: string[]
}

const duplicates = ref<DuplicateGroup[]>([])
const busy       = ref(false)

async function chooseAndScan () {
  const selected = await open({ directory: true, multiple: false })
  if (!selected) return          // Dialog abgebrochen

  busy.value = true
  try {
    duplicates.value = await invoke<DuplicateGroup[]>('scan_folder', {
      path: selected
    })
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <button @click="chooseAndScan" :disabled="busy">
    {{ busy ? 'Scanne…' : 'Ordner auswählen & scannen' }}
  </button>

  <ul v-if="duplicates.length">
    <li v-for="d in duplicates" :key="d.hash" class="mt-4">
      <strong class="text-sm">{{ d.hash }}</strong>
      <ul class="ml-4 list-disc">
        <li v-for="p in d.paths" :key="p">{{ p }}</li>
      </ul>
    </li>
  </ul>
</template>
