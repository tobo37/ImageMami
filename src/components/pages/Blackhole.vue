<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import DestinationSelector from "../ui/DestinationSelector.vue";

interface Device {
  name: string;
  path: string;
  total: number;
}

interface BlackholeFolder {
  path: string;
  files: string[];
}
interface BlackholeProgress {
  progress: number;
}

const destPath = ref<string | null>(null);
const devices = ref<Device[]>([]);
const busyPath = ref<string | null>(null);
const folders = ref<BlackholeFolder[]>([]);
const progress = ref(0);
let unlisten: UnlistenFn | null = null;
const { t } = useI18n();

onMounted(() => {
  destPath.value = localStorage.getItem("importDest");
  loadDevices();
});

async function chooseDest() {
  const selected = await open({ directory: true, multiple: false });
  if (!selected) return;
  const path = Array.isArray(selected) ? selected[0] : selected;
  destPath.value = path;
  localStorage.setItem("importDest", path);
}

async function loadDevices() {
  devices.value = await invoke<Device[]>("list_all_disks");
}

async function scanDisk(path: string) {
  if (!destPath.value) return;
  busyPath.value = path;
  folders.value = [];
  progress.value = 0;
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  unlisten = await listen<BlackholeProgress>("blackhole_progress", (e) => {
    progress.value = e.payload.progress;
  });
  try {
    folders.value = await invoke<BlackholeFolder[]>("scan_blackhole_stream", {
      rootPath: path,
      destPath: destPath.value,
    });
  } finally {
    busyPath.value = null;
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
  }
}

async function importFolder(folder: BlackholeFolder, cut: boolean) {
  if (!destPath.value) return;
  await invoke("import_blackhole", {
    files: folder.files,
    destPath: destPath.value,
    cut,
  });
  folders.value = folders.value.filter((f) => f.path !== folder.path);
}
</script>

<template>
  <div class="view blackhole-view">
    <h1>{{ t("blackhole.title") }}</h1>
    <DestinationSelector
      :path="destPath"
      :label="t('import.destination')"
      :choose-text="t('import.choose')"
      @choose="chooseDest"
    />

    <section>
      <h2>{{ t("blackhole.disks") }}</h2>
      <div v-if="devices.length" class="devices-grid">
        <div v-for="d in devices" :key="d.path" class="card device-card">
          <strong>{{ d.name }}</strong>
          <p class="device-path">{{ d.path }}</p>
          <button class="btn" :disabled="!!busyPath" @click="scanDisk(d.path)">
            {{ t("blackhole.scan") }}
          </button>
        </div>
      </div>
      <p v-else class="placeholder">-</p>
    </section>

    <div v-if="busyPath" class="status">{{ Math.round(progress * 100) }}%</div>

    <section v-if="folders.length" class="folder-list">
      <div v-for="f in folders" :key="f.path" class="card folder-card">
        <p class="folder-path">{{ f.path }}</p>
        <p>{{ f.files.length }} {{ t("blackhole.files") }}</p>
        <div class="actions">
          <button class="btn" @click="importFolder(f, false)">
            {{ t("blackhole.copy") }}
          </button>
          <button class="btn ghost" @click="importFolder(f, true)">
            {{ t("blackhole.cut") }}
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.blackhole-view {
  display: flex;
  flex-direction: column;
  gap: 2rem;
}
.devices-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 1rem;
}
.device-path {
  font-size: 0.8rem;
  opacity: 0.8;
}
.folder-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.folder-path {
  font-size: 0.9rem;
  word-break: break-all;
}
.actions {
  display: flex;
  gap: 0.5rem;
}
</style>
