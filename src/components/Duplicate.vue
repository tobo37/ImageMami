<script setup lang="ts">
import { ref, onBeforeUnmount, computed } from 'vue'
import HamsterLoader from './HamsterLoader.vue'
import { open } from '@tauri-apps/plugin-dialog'     // v1-API ¹
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'

interface DuplicateGroup {
  tag: string
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
const marked     = ref<string[]>([])
const showConfirm = ref(false)
let unlisten: UnlistenFn | null = null
const { t } = useI18n()
const markedCount = computed(() => marked.value.length)

function recordDecision (tag: string, path: string, value: string) {
  let del: boolean | null
  if (value === 'keep') del = false
  else if (value === 'delete') del = true
  else del = null
  invoke('record_decision', { tag, path, delete: del })
  if (value === 'delete') {
    if (!marked.value.includes(path)) marked.value.push(path)
  } else {
    const idx = marked.value.indexOf(path)
    if (idx !== -1) marked.value.splice(idx, 1)
  }
}

async function chooseAndScan () {
  const selected = await open({ directory: true, multiple: false })
  if (!selected) return          // Dialog abgebrochen

  busy.value = true
  progress.value = 0
  eta.value = 0
  marked.value = []
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

function deleteMarked () {
  showConfirm.value = true
}

async function confirmDelete () {
  await invoke('delete_files', { paths: marked.value })
  showConfirm.value = false
  // remove deleted paths from duplicate list
  duplicates.value = duplicates.value
    .map(g => ({
      ...g,
      paths: g.paths.filter(p => !marked.value.includes(p))
    }))
    .filter(g => g.paths.length > 0)
  marked.value = []
}

function cancelDelete () {
  showConfirm.value = false
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
        <strong style="font-size: 0.875rem;">{{ d.tag }}: {{ d.hash }}</strong>
        <ul style="margin-left: 1rem; list-style: disc;">
          <li v-for="(p, idx) in d.paths" :key="p">
            {{ p }}
            <label style="margin-left: 0.5rem;">
              <input type="radio" :name="d.hash + '-' + idx" value="unknown" checked
                @change="recordDecision(d.tag, p, 'unknown')" />
              Unknown
            </label>
            <label style="margin-left: 0.5rem;">
              <input type="radio" :name="d.hash + '-' + idx" value="keep"
                @change="recordDecision(d.tag, p, 'keep')" />
              Keep
            </label>
            <label style="margin-left: 0.5rem;">
              <input type="radio" :name="d.hash + '-' + idx" value="delete"
                @change="recordDecision(d.tag, p, 'delete')" />
              Delete
            </label>
          </li>
        </ul>
      </li>
    </ul>
    <button v-if="markedCount" style="margin-top: 1rem;" @click="deleteMarked">
      {{ t('duplicate.deleteMarked') }}
    </button>

    <div v-if="showConfirm" class="modal-backdrop">
      <div class="modal">
        <p>{{ t('duplicate.confirmDelete', { count: markedCount }) }}</p>
        <ul class="file-list">
          <li v-for="p in marked" :key="p">{{ p }}</li>
        </ul>
        <div class="actions">
          <button @click="confirmDelete">{{ t('common.yes') }}</button>
          <button class="ghost" @click="cancelDelete">{{ t('common.no') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}
.modal {
  background: var(--card-bg);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 1rem;
  max-width: 90%;
}
.file-list {
  max-height: 200px;
  overflow-y: auto;
  margin: 1rem 0;
  padding-left: 1rem;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
button.ghost {
  background: transparent;
  color: var(--accent-color);
  border: 1px solid color-mix(in srgb, var(--accent-color), transparent 70%);
}
</style>

