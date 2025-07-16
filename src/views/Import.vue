<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useSettingsStore } from '../stores/settings';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

import DestinationSelector from '../components/ui/DestinationSelector.vue';
import DeviceCard from '../components/ui/DeviceCard.vue';

interface Device {
  name: string;
  path: string;
  total: number;
}

const settings = useSettingsStore();
const devices = ref<Device[]>([]);
const busyPath = ref<string | null>(null);
let pollTimer: number | null = null;

onMounted(() => {
  loadDevices();
});

onUnmounted(() => {
  if (pollTimer !== null) clearTimeout(pollTimer);
});

async function chooseDest() {
  const selected = await open({ directory: true, multiple: false });
  if (!selected) return;
  const path = Array.isArray(selected) ? selected[0] : selected;
  settings.setImportDestination(path);
}

async function loadDevices() {
  devices.value = await invoke<Device[]>('list_external_devices');
  scheduleNext();
}

function scheduleNext() {
  if (pollTimer !== null) clearTimeout(pollTimer);
  const delay = devices.value.length === 0 ? 1000 : 5000;
  pollTimer = window.setTimeout(loadDevices, delay);
}

async function copyDevice(path: string) {
  if (!settings.importDestination) return;
  busyPath.value = path;
  try {
    await invoke('import_device', {
      devicePath: path,
      destPath: settings.importDestination,
    });
  } finally {
    busyPath.value = null;
  }
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
</script>

<template>
  <div class="view import-view">
    <!-- Zielordner auswählen -->
    <DestinationSelector
      :path="settings.importDestination"
      :label="$t('import.destination')"
      :choose-text="$t('import.choose')"
      @choose="chooseDest"
    />

    <!-- Geräte-Liste -->
    <section>
      <header class="section-header">
        <h2>{{ $t('import.devices') }}</h2>
        <button class="btn ghost" @click="loadDevices">
          <span class="icon-rotate-cw" /> {{ $t('import.refresh') }}
        </button>
      </header>

      <div v-if="devices.length" class="devices-grid">
        <DeviceCard
          v-for="d in devices"
          :key="d.path"
          :device="d"
          :disabled="!settings.importDestination"
          :busy="busyPath === d.path"
          :copy-text="$t('import.copy')"
          :format-size="formatSize"
          @import="copyDevice"
        />
      </div>

      <p v-else class="placeholder">-</p>
    </section>
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
.btn.ghost {
  background: transparent;
  color: var(--accent-color);
  border: 1px solid color-mix(in srgb, var(--accent-color), transparent 70%);
}
.icon-rotate-cw::before {
  content: '⟳';
} /* simple icon-stub */
</style>
