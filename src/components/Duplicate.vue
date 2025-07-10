<template>
  <div class="view" @drop.prevent="handleDrop" @dragover.prevent>
    <div
      class="dropzone"
      v-if="!duplicates.length && !busy"
      @click="openDialog"
    >
      <p>{{ t("duplicate.dragDropInstruction") }}</p>
      <small>{{ t("duplicate.orClickToSelect") }}</small>
    </div>

    <HamsterLoader v-if="busy" />
    <div v-if="busy" class="status">
      {{ Math.round(progress * 100) }}% - ETA {{ eta.toFixed(1) }}s
    </div>

    <div v-if="duplicates.length" class="duplicate-list">
      <div v-for="d in duplicates" :key="d.hash" class="duplicate-group">
        <h3>{{ d.tag }}</h3>
        <div class="image-pair">
          <div
            v-for="p in d.paths"
            :key="p"
            class="image-card"
            :class="{ marked: marked.includes(p) }"
          >
            <img :src="'file://' + p" alt="duplicate" />
            <p class="path">{{ p }}</p>
            <div class="actions">
              <button @click="recordDecision(d.tag, p, 'keep')" class="keep">
                {{ t("common.keep") }}
              </button>
              <button
                @click="recordDecision(d.tag, p, 'delete')"
                class="delete"
              >
                {{ t("common.delete") }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <button v-if="markedCount" class="delete-button" @click="deleteMarked">
      {{ t("duplicate.deleteMarked") }}
    </button>

    <div v-if="showConfirm" class="modal-backdrop">
      <div class="modal">
        <p>{{ t("duplicate.confirmDelete", { count: markedCount }) }}</p>
        <ul class="file-list">
          <li v-for="p in marked" :key="p">{{ p }}</li>
        </ul>
        <div class="actions">
          <button @click="confirmDelete">{{ t("common.yes") }}</button>
          <button class="ghost" @click="cancelDelete">
            {{ t("common.no") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import HamsterLoader from "./HamsterLoader.vue";

interface DuplicateGroup {
  tag: string;
  hash: string;
  paths: string[];
}

interface DuplicateProgress {
  progress: number;
  eta_seconds: number;
}

const duplicates = ref<DuplicateGroup[]>([]);
const busy = ref(false);
const progress = ref(0);
const eta = ref(0);
const marked = ref<string[]>([]);
const showConfirm = ref(false);
const markedCount = computed(() => marked.value.length);
let unlisten: UnlistenFn | null = null;
const { t } = useI18n();

function recordDecision(tag: string, path: string, value: string) {
  let del: boolean | null;
  if (value === "keep") del = false;
  else if (value === "delete") del = true;
  else del = null;
  invoke("record_decision", { tag, path, delete: del });
  if (value === "delete") {
    if (!marked.value.includes(path)) marked.value.push(path);
  } else {
    const idx = marked.value.indexOf(path);
    if (idx !== -1) marked.value.splice(idx, 1);
  }
}

async function scanFolder(path: string) {
  busy.value = true;
  progress.value = 0;
  eta.value = 0;
  marked.value = [];
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  unlisten = await listen<DuplicateProgress>("duplicate_progress", (e) => {
    progress.value = e.payload.progress;
    eta.value = e.payload.eta_seconds;
  });
  try {
    duplicates.value = await invoke<DuplicateGroup[]>("scan_folder_stream", {
      path,
    });
  } finally {
    busy.value = false;
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
  }
}

function handleDrop(event: DragEvent) {
  const files = event.dataTransfer?.files;
  if (files && files.length) {
    const path = files[0].webkitRelativePath;
    scanFolder(path);
  }
}

async function openDialog() {
  const selected = await open({ directory: true, multiple: false });
  if (selected) {
    scanFolder(selected as string);
  }
}

function deleteMarked() {
  showConfirm.value = true;
}

async function confirmDelete() {
  await invoke("delete_files", { paths: marked.value });
  showConfirm.value = false;
  duplicates.value = duplicates.value
    .map((g) => ({
      ...g,
      paths: g.paths.filter((p) => !marked.value.includes(p)),
    }))
    .filter((g) => g.paths.length > 0);
  marked.value = [];
}

function cancelDelete() {
  showConfirm.value = false;
}

onBeforeUnmount(() => {
  if (unlisten) unlisten();
});
</script>

<style scoped>
.view {
  padding: 1rem;
}

.dropzone {
  border: 2px dashed var(--border-color);
  padding: 2rem;
  text-align: center;
  border-radius: 1rem;
  background: var(--card-bg);
  color: var(--text-muted);
  margin-top: 2rem;
  cursor: pointer;
  transition: background 0.2s;
}
.dropzone:hover {
  background: var(--hover-bg, rgba(0, 0, 0, 0.05));
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

.image-pair {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.image-card {
  border: 1px solid var(--border-color);
  padding: 0.5rem;
  border-radius: 0.5rem;
  background: var(--card-bg);
  width: 200px;
  text-align: center;
  position: relative;
}

.image-card img {
  width: 100%;
  height: auto;
  object-fit: cover;
  border-radius: 0.25rem;
}

.image-card .path {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin: 0.5rem 0;
  word-break: break-word;
}

.image-card .actions {
  display: flex;
  justify-content: center;
  gap: 0.5rem;
}

.image-card button {
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  cursor: pointer;
}

.image-card button.keep {
  background: var(--accent-color, green);
  color: white;
}

.image-card button.delete {
  background: red;
  color: white;
}

.image-card.marked {
  outline: 2px solid red;
}

.delete-button {
  margin-top: 2rem;
  background: red;
  color: white;
  padding: 0.75rem 1.5rem;
  border-radius: 0.5rem;
  font-weight: bold;
}

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
