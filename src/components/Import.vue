<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'

const destPath = ref<string | null>(null)
const { t } = useI18n()

onMounted(() => {
  destPath.value = localStorage.getItem('importDest')
})

async function chooseDest () {
  const selected = await open({ directory: true, multiple: false })
  if (!selected) return
  const path = Array.isArray(selected) ? selected[0] : selected
  destPath.value = path
  localStorage.setItem('importDest', path)
}
</script>

<template>
  <div class="view">
    <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 1rem;">
      <strong>{{ t('import.destination') }}</strong>
      <span>{{ destPath || '-' }}</span>
      <button @click="chooseDest">{{ t('import.choose') }}</button>
    </div>
    <h1>{{ t('import.title') }}</h1>
  </div>
</template>
