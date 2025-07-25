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
      @click="handleStartScan"
      :disabled="!settings.duplicateDestination"
    >
      {{ busy ? t('duplicate.scanning') : t('blackhole.scan') }}
    </button>

    <div v-if="busy" class="scan-status">
      <HamsterLoader />
      <div class="status-text">
        {{ Math.round(progress * 100) }}% - ETA {{ etaDisplay }}
        <button class="ghost cancel-button" @click="cancelScan">
          {{ t('common.cancel') }}
        </button>
      </div>
    </div>

    <div v-if="duplicates.length" class="duplicate-list">
      <div
        v-for="d in duplicates"
        :key="d.files[0].path"
        class="duplicate-group"
      >
        <h3>
          {{ tagText(d.method) }}
          <small>{{ formatSize(d.files[0].size) }}</small>
        </h3>
        <DuplicateGroupCard
          :group="d"
          :marked="marked"
          :delete-text="t('common.delete')"
          :keep-text="t('common.keep')"
          @decision="(path, v) => updateMarked(path, v as 'keep' | 'delete')"
        />
      </div>
    </div>

    <div v-if="duplicates.length && !busy" class="auto-mark-bar">
      <button class="ghost auto-mark-button" @click="autoMark">
        {{ t('duplicate.autoMark') }}
      </button>
    </div>

    <div v-if="markedCount > 0" class="delete-bar">
      <button class="delete-button" @click="deleteMarked">
        {{ t('duplicate.deleteMarked', { count: markedCount }) }}
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
import { useSettingsStore } from '../stores/settings';

// Import UI components
import HamsterLoader from '../components/ui/HamsterLoader.vue';
import DuplicateGroupCard from '../components/ui/DuplicateGroupCard.vue';
import DeleteConfirmModal from '../components/ui/DeleteConfirmModal.vue';
import DestinationSelector from '../components/ui/DestinationSelector.vue';

// --- Type Definitions ---
interface FileInfo {
  path: string;
  age: number;
  size: number;
  preview: string; // The thumbnail is now directly included in the data
  hash?: string;
  dhash?: string;
}

interface DuplicateGroup {
  method: unknown;
  files: FileInfo[];
}

type DuplicateProgress = [number, number];

// --- Utility Functions ---
/**
 * Formats a size in bytes into a human-readable string (KB, MB, GB).
 */
function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

// --- State and Logic for Duplicate Management ---
const duplicates = ref<DuplicateGroup[]>([]);
const marked = ref<string[]>([]);
const showConfirm = ref(false);
const markedCount = computed(() => marked.value.length);

function updateMarked(path: string, value: 'keep' | 'delete') {
  if (value === 'delete') {
    if (!marked.value.includes(path)) {
      marked.value.push(path);
    }
  } else {
    const idx = marked.value.indexOf(path);
    if (idx !== -1) {
      marked.value.splice(idx, 1);
    }
  }
}

function autoMark() {
  duplicates.value.forEach((group) => {
    if (group.files.length <= 1) return;
    const newestFile = group.files.reduce((a, b) => (a.age < b.age ? a : b));
    group.files.forEach((file) => {
      if (file.path === newestFile.path) {
        const index = marked.value.indexOf(file.path);
        if (index !== -1) {
          marked.value.splice(index, 1);
        }
      } else {
        if (!marked.value.includes(file.path)) {
          marked.value.push(file.path);
        }
      }
    });
  });
}

function deleteMarked() {
  if (marked.value.length > 0) {
    showConfirm.value = true;
  }
}

async function confirmDelete() {
  if (marked.value.length === 0) return;
  await invoke('delete_files', { paths: marked.value });
  showConfirm.value = false;
  duplicates.value = duplicates.value
    .map((group) => {
      const remainingFiles = group.files.filter(
        (file) => !marked.value.includes(file.path)
      );
      return { ...group, files: remainingFiles };
    })
    .filter((group) => group.files.length > 1);
  marked.value = [];
}

function cancelDelete() {
  showConfirm.value = false;
}

function setDuplicates(newDuplicates: DuplicateGroup[]) {
  duplicates.value = newDuplicates;
  marked.value = [];
}

// --- State and Logic for Scan Handling ---
const busy = ref(false);
const progress = ref(0);
const eta = ref(0);
const cancelled = ref(false);
let unlisten: UnlistenFn | null = null;

const etaDisplay = computed(() => {
  if (eta.value < 1) return '...';
  if (eta.value >= 60) {
    const minutes = Math.floor(eta.value / 60);
    const seconds = Math.round(eta.value % 60).toString().padStart(2, '0');
    return `${minutes}m ${seconds}s`;
  }
  return `${eta.value.toFixed(1)}s`;
});

async function startScan(path: string, tags: string[]) {
  if (!path || busy.value) return;

  busy.value = true;
  progress.value = 0;
  eta.value = 0;
  cancelled.value = false;

  if (unlisten) unlisten();
  unlisten = await listen<DuplicateProgress>('duplicate_progress', (event) => {
    progress.value = event.payload[0];
    eta.value = event.payload[1];
  });

  try {
    const result = await invoke<{ groups: DuplicateGroup[] }>(
      'scan_folder_stream_multi',
      { path, tags }
    );
    if (!cancelled.value) {
      setDuplicates(result.groups);
    }
  } catch (error) {
    console.error('Scan failed:', error);
  } finally {
    busy.value = false;
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
  }
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
  invoke('cancel_scan');
  if (unlisten) {
    unlisten();
  }
});

// --- Component-specific Setup ---
const { t } = useI18n();
const settings = useSettingsStore();
const modes = ref<string[]>(['hash', 'dhash']);

async function chooseDest() {
  const selected = await open({ directory: true, multiple: false });
  if (!selected) return;
  const path = Array.isArray(selected) ? selected[0] : selected;
  settings.setDuplicateDestination(path);
}

function handleStartScan() {
  if (settings.duplicateDestination) {
    startScan(settings.duplicateDestination, modes.value);
  }
}

function tagText(tag: unknown): string {
  let name: string;
  if (typeof tag === 'object' && tag !== null) {
    name = Object.keys(tag)[0];
  } else {
    name = String(tag);
  }
  const tagMap: Record<string, string> = {
    ByteHash: 'hash',
    PerceptualDHash: 'dhash',
  };
  const translationKey = `duplicate.tags.${tagMap[name] ?? name}`;
  const translatedText = t(translationKey);
  return translatedText === translationKey ? name : translatedText;
}
</script>

<style scoped>
.view {
  padding: 1rem;
  padding-bottom: 6rem; /* Add padding to avoid overlap with delete bar */
}

.scan-status {
  text-align: center;
  margin-top: 1rem;
}

.status-text {
  margin-top: 0.5rem;
}

.duplicate-list {
  margin-top: 1.5rem;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.5rem;
}

.duplicate-group h3 {
  font-size: 1rem;
  margin-bottom: 0.5rem;
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

h3 small {
  font-size: 0.8rem;
  color: var(--text-color-muted);
}

.auto-mark-bar {
  margin-top: 1.5rem;
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
  right: 0; /* Use right: 0 instead of width: 100% for better responsiveness */
  display: flex;
  justify-content: center;
  align-items: center;
  background: var(--card-bg);
  border-top: 1px solid var(--border-color);
  padding: 0.75rem;
  z-index: 10;
}

.delete-button {
  background: hsl(0, 70%, 50%);
  color: white;
  padding: 0.75rem 1.5rem;
  border-radius: 0.5rem;
  font-weight: bold;
  border: none;
  cursor: pointer;
  transition: background-color 0.2s;
}

.delete-button:hover {
    background: hsl(0, 80%, 60%);
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
