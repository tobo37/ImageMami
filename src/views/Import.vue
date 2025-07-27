<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useSettingsStore } from '../stores/settings';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import DestinationSelector from '../components/ui/DestinationSelector.vue';
import DeviceCard from '../components/ui/DeviceCard.vue';

interface Device {
  name: string;
  path: string;
  // NEU: Diese Werte sollten idealerweise direkt vom Backend kommen.
  // Falls nicht, müssten sie via 'invoke' separat abgefragt werden.
  used: number; 
  total: number;
  icon: 'usb' | 'sd' | 'hdd' | 'camera' | 'unknown';
}

interface ImportProgress {
  total: number;
  copied: number;
  current: string;
}

const settings = useSettingsStore();
const devices = ref<Device[]>([]);
const busyPath = ref<string | null>(null);
const progressInfo = ref<ImportProgress | null>(null);
let unlisten: UnlistenFn | null = null;
let pollTimer: number | null = null;

onMounted(() => {
  loadDevices();
});

onUnmounted(() => {
  if (pollTimer !== null) clearTimeout(pollTimer);
  if (unlisten) unlisten();
});

// NEU: Benutzerfreundlicher Pfad für die Anzeige
const userFriendlyDestination = computed(() => {
  if (!settings.importDestination) return null;
  // Zeigt nur die letzten beiden Teile des Pfades an, z.B. ".../data/bilder"
  const parts = settings.importDestination.split(/[/\\]/);
  if (parts.length > 2) {
    return `.../${parts[parts.length - 2]}/${parts[parts.length - 1]}`;
  }
  return settings.importDestination;
});

async function chooseDest() {
  const selected = await open({ directory: true, multiple: false });
  if (!selected) return;
  const path = Array.isArray(selected) ? selected[0] : selected;
  settings.setImportDestination(path);
}

async function loadDevices() {
  // Annahme: Dein Rust-Backend liefert jetzt auch 'used' und einen 'icon'-Typ.
  devices.value = await invoke<Device[]>('list_external_devices');
  scheduleNext();
}

function scheduleNext() {
  if (pollTimer !== null) clearTimeout(pollTimer);
  // Wenn kein Gerät gefunden wird, öfter nachsehen (z.B. wenn der User gerade eins einsteckt)
  const delay = devices.value.length === 0 ? 1000 : 5000;
  pollTimer = window.setTimeout(loadDevices, delay);
}

async function copyDevice(path: string) {
  if (!settings.importDestination) return;
  busyPath.value = path;
  progressInfo.value = null;

  if (unlisten) unlisten();
  unlisten = await listen<ImportProgress>('import_progress', (e) => {
    progressInfo.value = e.payload;
  });

  try {
    await invoke('import_device_stream', {
      devicePath: path,
      destPath: settings.importDestination,
    });
  } catch(error) {
    console.error('Import failed:', error);
    // Hier könntest du eine Fehlermeldung anzeigen
  } finally {
    busyPath.value = null;
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
    progressInfo.value = null;
  }
}

// Die `formatSize`-Funktion bleibt wie sie war.
function formatSize(bytes: number) {
  if (bytes === 0) return '0 B';
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
    <DestinationSelector
      :path="settings.importDestination"
      :friendly-path="userFriendlyDestination"
      :label="$t('import.destination')"
      :choose-text="$t('import.choose')"
      @choose="chooseDest"
    />

    <section>
      <header class="section-header">
        <h2>{{ $t('import.devices') }}</h2>
        <button class="btn ghost" @click="loadDevices" :disabled="!!busyPath">
          <span class="icon-rotate-cw" /> {{ $t('import.refresh') }}
        </button>
      </header>
      
      <div v-if="!devices.length && !busyPath" class="placeholder-container">
        <p>{{ $t('import.no_devices_found') }}</p>
        <span>{{ $t('import.connect_device_prompt') }}</span>
      </div>

      <div v-if="devices.length" class="devices-grid">
        <DeviceCard
          v-for="d in devices"
          :key="d.path"
          :device="d"
          :disabled="!settings.importDestination || !!busyPath"
          :busy="busyPath === d.path"
          :copy-text="$t('import.copy')"
          :format-size="formatSize"
          
          :progress="busyPath === d.path ? progressInfo : null"

          @import="copyDevice"
        />
      </div>
      
      </section>
  </div>
</template>

<style scoped>
.import-view {
  display: flex;
  flex-direction: column;
  gap: 2rem;
  padding: 1.5rem; /* Etwas mehr Luft */
}
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem; /* Etwas mehr Abstand */
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-color); /* Visuelle Trennung */
}
.devices-grid {
  display: grid;
  /* Stellt sicher, dass die Karten nicht zu breit werden auf großen Bildschirmen */
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); 
  gap: 1.5rem; /* Mehr Abstand zwischen den Karten */
}

/* NEU: Styling für den Platzhalter, wenn keine Geräte da sind */
.placeholder-container {
  text-align: center;
  padding: 3rem 1rem;
  background-color: var(--bg-color-secondary);
  border-radius: var(--border-radius-large);
  border: 1px dashed var(--border-color);
}
.placeholder-container p {
  font-size: 1.1rem;
  font-weight: 500;
  margin: 0;
}
.placeholder-container span {
  font-size: 0.9rem;
  opacity: 0.7;
}

.btn.ghost {
  background: transparent;
  color: var(--accent-color);
  border: 1px solid var(--accent-color-muted); /* Etwas weicherer Rand */
  transition: all 0.2s ease;
}
.btn.ghost:hover:not(:disabled) {
  background-color: var(--accent-color-muted);
  border-color: var(--accent-color);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.icon-rotate-cw::before {
  content: '⟳';
  display: inline-block;
  transition: transform 0.5s ease;
}
.btn:hover .icon-rotate-cw {
  transform: rotate(180deg);
}
</style>