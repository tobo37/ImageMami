<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import DestinationSelector from "../ui/DestinationSelector.vue";
import DeviceCard from "../ui/DeviceCard.vue";
import HamsterLoader from "../ui/HamsterLoader.vue";

interface Device {
  name: string;
  path: string;
  total: number;
}

interface ImportProgress {
  copied: number;
  total: number;
}

const destPath = ref<string | null>(null);
const devices = ref<Device[]>([]);
const busyPath = ref<string | null>(null);
const progress = ref(0);
let unlisten: UnlistenFn | null = null;
let pollTimer: number | null = null;

onMounted(() => {
  destPath.value = localStorage.getItem("importDest");
  loadDevices();
});

onUnmounted(() => {
  if (pollTimer !== null) clearTimeout(pollTimer);
  if (unlisten) unlisten();
});

async function chooseDest() {
  const selected = await open({ directory: true, multiple: false });
  if (!selected) return;
  const path = Array.isArray(selected) ? selected[0] : selected;
  destPath.value = path;
  localStorage.setItem("importDest", path);
}

async function loadDevices() {
  devices.value = await invoke<Device[]>("list_external_devices");
  scheduleNext();
}

function scheduleNext() {
  if (pollTimer !== null) clearTimeout(pollTimer);
  const delay = devices.value.length === 0 ? 1000 : 5000;
  pollTimer = window.setTimeout(loadDevices, delay);
}

async function copyDevice(path: string) {
  if (!destPath.value) return;
  busyPath.value = path;
  progress.value = 0;
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  unlisten = await listen<ImportProgress>("import_progress", (e) => {
    const { copied, total } = e.payload;
    progress.value = total ? copied / total : 1;
  });
  try {
    await invoke("import_device", {
      devicePath: path,
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

function formatSize(bytes: number) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes,
    i = 0;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${size.toFixed(1)} ${units[i]}`;
}
</script>

<template>
  <div class="view import-view">
    <!-- Zielordner auswählen -->
    <DestinationSelector
      :path="destPath"
      :label="$t('import.destination')"
      :choose-text="$t('import.choose')"
      @choose="chooseDest"
    />

    <!-- Geräte-Liste -->
    <section>
      <header class="section-header">
        <h2>{{ $t("import.devices") }}</h2>
        <button class="btn ghost" @click="loadDevices">
          <span class="icon-rotate-cw" /> {{ $t("import.refresh") }}
        </button>
      </header>

      <div v-if="devices.length" class="devices-grid">
        <DeviceCard
          v-for="d in devices"
          :key="d.path"
          :device="d"
          :disabled="!destPath"
          :busy="busyPath === d.path"
          :copy-text="$t('import.copy')"
          :format-size="formatSize"
          @import="copyDevice"
        />
      </div>

      <p v-else class="placeholder">-</p>
    </section>

    <HamsterLoader v-if="busyPath" />
    <div v-if="busyPath" class="status">{{ Math.round(progress * 100) }}%</div>
  </div>
</template>

<style scoped>
.import-view {
  display: flex;
  flex-direction: column;
  gap: 2rem;
}
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}
.devices-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 1.2rem;
}
.placeholder {
  opacity: 0.6;
}

.status {
  text-align: center;
  margin-top: 0.5rem;
}
.btn.ghost {
  background: transparent;
  color: var(--accent-color);
  border: 1px solid color-mix(in srgb, var(--accent-color), transparent 70%);
}
.icon-rotate-cw::before {
  content: "⟳";
} /* simple icon-stub */
</style>
