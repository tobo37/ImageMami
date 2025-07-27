<template>
  <div class="view">
    <!-- SECTION: Settings Card (Visible before scan) -->
    <div v-if="!busy && !duplicates.length" class="settings-card">
      <div class="card-header">
        <h2 class="card-title">{{ t('duplicate.title') }}</h2>
        <p class="card-subtitle">{{ t('duplicate.subtitle') }}</p>
      </div>

      <DestinationSelector
        :path="settings.duplicateDestination"
        :label="t('duplicate.destinationLabel')"
        :choose-text="t('import.choose')"
        @choose="chooseDest"
      />

      <div class="scan-options">
        <h3 class="options-title">{{ t('duplicate.modes.title') }}</h3>
        <div class="mode-picker">
          <label
            class="mode-option"
            :class="{ selected: modes.includes('hash') }"
          >
            <input type="checkbox" value="hash" v-model="modes" />
            <div class="icon-wrapper">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path
                  d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
                ></path>
                <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
                <line x1="12" y1="22.08" x2="12" y2="12"></line>
              </svg>
            </div>
            <div class="mode-text">
              <span class="mode-name">{{ t('duplicate.modes.exact') }}</span>
              <span class="mode-description">{{
                t('duplicate.exactTooltip')
              }}</span>
            </div>
          </label>
          <label
            class="mode-option"
            :class="{ selected: modes.includes('dhash') }"
          >
            <input type="checkbox" value="dhash" v-model="modes" />
            <div class="icon-wrapper">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"></path>
                <circle cx="12" cy="12" r="3"></circle>
              </svg>
            </div>
            <div class="mode-text">
              <span class="mode-name">{{
                t('duplicate.modes.perceptual')
              }}</span>
              <span class="mode-description">{{
                t('duplicate.perceptualTooltip')
              }}</span>
            </div>
          </label>
        </div>
      </div>

      <details class="advanced-options">
        <summary>{{ t('duplicate.advancedOptions') }}</summary>
        <div class="advanced-content">
          <label for="sensitivity">{{ t('duplicate.sensitivityLabel') }}</label>
          <input
            type="range"
            id="sensitivity"
            min="0"
            max="16"
            v-model="perceptualThreshold"
          />
          <span>{{ perceptualThreshold }}</span>
          <small>{{ t('duplicate.sensitivityTooltip') }}</small>
        </div>
      </details>

      <button
        class="btn scan-button"
        @click="handleStartScan"
        :disabled="!settings.duplicateDestination || modes.length === 0"
      >
        {{ t('duplicate.scan') }}
      </button>
    </div>

    <!-- SECTION: Scan Progress -->
    <div v-if="busy" class="scan-status">
      <HamsterLoader />
      <div class="status-text">
        <span v-if="progressInfo">
          {{ progressInfo.processed }} / {{ progressInfo.total }} &mdash;
          {{ formatElapsed(progressInfo.elapsed) }}
          <br />
          <span class="current-file">{{ progressInfo.current }}</span>
        </span>
        <span v-else>{{ Math.round(progress * 100) }}%</span>
        <button class="ghost cancel-button" @click="cancelScan">
          {{ t('common.cancel') }}
        </button>
      </div>
    </div>

    <!-- SECTION: Results -->
    <div v-if="duplicates.length && !busy" class="results-view">
      <div class="results-header">
        <h3>{{ t('duplicate.resultsTitle', { count: duplicates.length }) }}</h3>
        <button class="ghost auto-mark-button" @click="autoMark">
          {{ t('duplicate.autoMark') }}
        </button>
      </div>
      <div class="duplicate-list">
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
            @decision="(path, v) => updateMarked(path, v as 'keep' | 'delete')"
          />
        </div>
      </div>
    </div>

    <!-- SECTION: Floating Action Bars -->
    <div v-if="markedCount > 0" class="delete-bar">
      <button class="delete-button" @click="deleteMarked">
        {{ t('duplicate.deleteMarked', { count: markedCount }) }}
      </button>
    </div>

    <!-- SECTION: Modals -->
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
  preview: string;
  hash?: string;
  dhash?: string;
}

interface DuplicateGroup {
  method: unknown;
  files: FileInfo[];
}

interface DuplicateProgress {
  processed: number;
  total: number;
  elapsed: number;
  current: string;
}

// --- Utility Functions ---
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
        (file) => !marked.value.includes(file.path),
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
const progressInfo = ref<DuplicateProgress | null>(null);
const cancelled = ref(false);
let unlisten: UnlistenFn | null = null;

function formatElapsed(sec: number) {
  if (sec >= 60) {
    const minutes = Math.floor(sec / 60);
    const seconds = Math.round(sec % 60)
      .toString()
      .padStart(2, '0');
    return `${minutes}m ${seconds}s`;
  }
  return `${sec.toFixed(1)}s`;
}

async function startScan(path: string, tags: string[]) {
  if (!path || busy.value) return;

  busy.value = true;
  progress.value = 0;
  progressInfo.value = null;
  cancelled.value = false;
  setDuplicates([]); // Clear previous results

  if (unlisten) unlisten();
  unlisten = await listen<DuplicateProgress>('duplicate_progress', (event) => {
    progressInfo.value = event.payload;
    progress.value =
      event.payload.total > 0
        ? event.payload.processed / event.payload.total
        : 0;
  });

  try {
    // TODO: To make the sensitivity slider work, the `tags` parameter
    // would need to be changed to send an object with the threshold,
    // e.g., `[{ name: 'dhash', threshold: perceptualThreshold.value }]`
    // This requires a corresponding change in the Rust command handler.
    const result = await invoke<{ groups: DuplicateGroup[] }>(
      'scan_folder_stream_multi',
      { path, tags },
    );
    if (!cancelled.value) {
      setDuplicates(result.groups);
    }
  } catch (error) {
    console.error('Scan failed:', error);
    // TODO: Show user-friendly error message
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
const perceptualThreshold = ref(8); // For the advanced options slider

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
  padding-bottom: 6rem; /* Space for delete bar */
}

/* --- Settings Card --- */
.settings-card {
  max-width: 600px;
  margin: 0 auto;
  padding: 2rem;
  background: var(--card-bg);
  border-radius: 1rem;
  border: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}
.card-header {
  text-align: center;
}
.card-title {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 0.25rem;
}
.card-subtitle {
  color: var(--text-color-muted);
}
.options-title {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: var(--text-color-secondary);
}
.scan-button {
  padding: 0.75rem;
  font-size: 1rem;
  font-weight: bold;
}

/* --- Mode Picker --- */
.mode-picker {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.mode-option {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.2s ease;
}
.mode-option:hover {
  border-color: var(--accent-color);
  background: color-mix(in srgb, var(--accent-color), transparent 90%);
}
.mode-option.selected {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-color), transparent 70%);
}
.mode-option input[type='checkbox'] {
  display: none; /* Hide checkbox, we use the label as a button */
}
.icon-wrapper {
  padding: 0.5rem;
  background: var(--bg-color-secondary);
  border-radius: 50%;
  color: var(--accent-color);
  display: grid;
  place-items: center;
}
.mode-text {
  display: flex;
  flex-direction: column;
}
.mode-name {
  font-weight: 600;
}
.mode-description {
  font-size: 0.85rem;
  color: var(--text-color-muted);
}

/* --- Advanced Options --- */
.advanced-options summary {
  cursor: pointer;
  font-weight: 600;
  color: var(--text-color-secondary);
}
.advanced-content {
  padding: 1rem;
  margin-top: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-size: 0.9rem;
}
.advanced-content small {
  color: var(--text-color-muted);
}

/* --- Scan Status --- */
.scan-status {
  text-align: center;
  margin-top: 2rem;
}
.status-text {
  margin-top: 1rem;
  color: var(--text-color-secondary);
}
.current-file {
  font-family: monospace;
  font-size: 0.8rem;
  color: var(--text-color-muted);
  word-break: break-all;
  padding: 0 1rem;
}
.cancel-button {
  margin-left: 0.5rem;
  margin-top: 1rem;
}

/* --- Results View --- */
.results-view {
  margin-top: 1.5rem;
}
.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}
.results-header h3 {
  font-size: 1.25rem;
  margin: 0;
}
.auto-mark-button {
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
}
.duplicate-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
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

/* --- Delete Bar --- */
.delete-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
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

button.ghost {
  background: transparent;
  color: var(--accent-color);
  border: 1px solid color-mix(in srgb, var(--accent-color), transparent 70%);
}
</style>
