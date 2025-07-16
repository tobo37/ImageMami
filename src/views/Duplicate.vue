<template>
  <div class="view">
    <DestinationSelector
      :path="settings.duplicateDestination"
      :label="t('import.destination')"
      :choose-text="t('import.choose')"
      @choose="chooseDest"
    />
    <div class="mode-picker">
      <label :title="t('duplicate.exactTooltip')">
        <input type="checkbox" value="hash" v-model="modes" />
        {{ t('duplicate.modes.exact') }}
      </label>
      <label :title="t('duplicate.perceptualTooltip')">
        <input type="checkbox" value="dhash" v-model="modes" />
        {{ t('duplicate.modes.perceptual') }}
      </label>
    </div>
    <button
      v-if="!busy"
      class="btn"
      @click="startScan"
      :disabled="!settings.duplicateDestination"
    >
      {{ busy ? t('duplicate.scanning') : t('blackhole.scan') }}
    </button>

    <HamsterLoader v-if="busy" />
    <div v-if="busy" class="status">
      {{ Math.round(progress * 100) }}% - ETA {{ etaDisplay }}
      <button class="ghost cancel-button" @click="cancelScan">
        {{ t('common.cancel') }}
      </button>
    </div>

    <div v-if="duplicates.length" class="duplicate-list">
      <div v-for="d in duplicates" :key="d.hash" class="duplicate-group">
        <h3>
          {{ tagText(d.tag) }}
          <small>{{ formatSize(d.size) }}</small>
        </h3>
        <DuplicateGroupCard
          :group="d"
          :marked="marked"
          :delete-text="t('common.delete')"
          :keep-text="t('common.keep')"
          @decision="(path: string, v: string) => updateMarked(path, v)"
        />
      </div>
    </div>

    <div v-if="duplicates.length" class="auto-mark-bar">
      <button class="ghost auto-mark-button" @click="autoMark">
        {{ t('duplicate.autoMark') }}
      </button>
    </div>

    <div v-if="markedCount" class="delete-bar">
      <button class="delete-button" @click="deleteMarked">
        {{ t('duplicate.deleteMarked') }}
      </button>
    </div>

    <DeleteConfirmModal
      :visible="showConfirm"
      :files="marked"
      :message="t('duplicate.confirmDelete', { count: markedCount })"
      :yes-text="t('common.yes')"
      :no-text="t('common.no')"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { useI18n } from 'vue-i18n';
import HamsterLoader from '../components/ui/HamsterLoader.vue';
import DuplicateGroupCard from '../components/ui/DuplicateGroupCard.vue';
import DeleteConfirmModal from '../components/ui/DeleteConfirmModal.vue';
import DestinationSelector from '../components/ui/DestinationSelector.vue';
import { useSettingsStore } from '../stores/settings';

interface DuplicateGroup {
  tag: string;
  hash: string;
  size: number;
  paths: string[];
  ages: number[];
}

interface DuplicateProgress {
  progress: number;
  eta_seconds: number;
}

const duplicates = ref<DuplicateGroup[]>([]);
const busy = ref(false);
const progress = ref(0);
const eta = ref(0);
const etaDisplay = computed(() => {
  if (eta.value >= 60) {
    const minutes = Math.floor(eta.value / 60);
    const seconds = Math.round(eta.value % 60)
      .toString()
      .padStart(2, '0');
    return `${minutes}m ${seconds}s`;
  }
  return `${eta.value.toFixed(1)}s`;
});
const marked = ref<string[]>([]);
const modes = ref<string[]>(['hash', 'dhash']);
const showConfirm = ref(false);
const cancelled = ref(false);
const markedCount = computed(() => marked.value.length);
let unlisten: UnlistenFn | null = null;
const { t } = useI18n();
const settings = useSettingsStore();

function tagText(tag: string) {
  const key = `duplicate.tags.${tag}`;
  const result = t(key);
  return result === key ? tag : result;
}

function formatSize(bytes: number) {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes,
    i = 0;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${size.toFixed(1)} ${units[i]}`;
}

function updateMarked(path: string, value: string) {
  if (value === 'delete') {
    if (!marked.value.includes(path)) marked.value.push(path);
  } else {
    const idx = marked.value.indexOf(path);
    if (idx !== -1) marked.value.splice(idx, 1);
  }
}

function autoMark() {
  duplicates.value.forEach((g) => {
    if (g.paths.length <= 1) return;
    const maxAge = Math.max(...g.ages);
    const keepIdx = g.ages.indexOf(maxAge);
    g.paths.forEach((p, idx) => {
      if (idx === keepIdx) {
        const i = marked.value.indexOf(p);
        if (i !== -1) marked.value.splice(i, 1);
      } else if (!marked.value.includes(p)) {
        marked.value.push(p);
      }
    });
  });
}

async function scanFolder(path: string) {
  busy.value = true;
  progress.value = 0;
  eta.value = 0;
  marked.value = [];
  cancelled.value = false;
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  unlisten = await listen<DuplicateProgress>('duplicate_progress', (e) => {
    progress.value = e.payload.progress;
    eta.value = e.payload.eta_seconds;
  });
  try {
    const results = await invoke<DuplicateGroup[]>('scan_folder_stream_multi', {
      path,
      tags: modes.value,
    });
    if (!cancelled.value) {
      duplicates.value = results;
    }
  } finally {
    busy.value = false;
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
  }
}

function startScan() {
  if (settings.duplicateDestination) {
    scanFolder(settings.duplicateDestination);
  }
}

async function chooseDest() {
  const selected = await open({ directory: true, multiple: false });
  if (!selected) return;
  const path = Array.isArray(selected) ? selected[0] : selected;
  settings.setDuplicateDestination(path);
}

function deleteMarked() {
  showConfirm.value = true;
}

async function confirmDelete() {
  await invoke('delete_files', { paths: marked.value });
  showConfirm.value = false;
  duplicates.value = duplicates.value
    .map((g) => {
      const newPaths: string[] = [];
      const newAges: number[] = [];
      g.paths.forEach((p, idx) => {
        if (!marked.value.includes(p)) {
          newPaths.push(p);
          newAges.push(g.ages[idx]);
        }
      });
      return { ...g, paths: newPaths, ages: newAges };
    })
    .filter((g) => g.paths.length > 0);
  marked.value = [];
}

function cancelDelete() {
  showConfirm.value = false;
}

function cancelScan() {
  cancelled.value = true;
  invoke('cancel_scan');
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  busy.value = false;
}

onBeforeUnmount(() => {
  if (unlisten) unlisten();
  invoke('cancel_scan');
});
</script>

<style scoped>
.view {
  padding: 1rem;
}

.status {
  text-align: center;
  margin-top: 0.5rem;
}

.duplicate-list {
  margin-top: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.duplicate-group h3 {
  font-size: 1rem;
  margin-bottom: 0.5rem;
}

.auto-mark-bar {
  margin-top: 1rem;
  text-align: center;
}

.auto-mark-button {
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
}

.delete-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  width: 100%;
  display: flex;
  justify-content: center;
  background: var(--card-bg);
  border-top: 1px solid var(--border-color);
  padding: 0.5rem;
}

.delete-button {
  background: red;
  color: white;
  padding: 0.75rem 1.5rem;
  border-radius: 0.5rem;
  font-weight: bold;
}

.mode-picker {
  margin-bottom: 1rem;
  display: flex;
  gap: 1rem;
  align-items: center;
}
button.ghost {
  background: transparent;
  color: var(--accent-color);
  border: 1px solid color-mix(in srgb, var(--accent-color), transparent 70%);
}

.cancel-button {
  margin-left: 0.5rem;
}
</style>
