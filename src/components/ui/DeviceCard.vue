<script setup lang="ts">
import { computed } from 'vue';

// Definiere die Interfaces direkt hier, um sie mit import.vue synchron zu halten
interface Device {
  name: string;
  path: string;
  used: number;
  total: number;
  icon: 'usb' | 'sd' | 'hdd' | 'camera' | 'unknown';
}

interface ImportProgress {
  total: number;
  copied: number;
  current: string;
}

const props = defineProps<{
  device: Device;
  disabled: boolean;
  copyText: string;
  busy: boolean;
  formatSize: (bytes: number) => string;
  // NEU: Nimmt das Progress-Objekt entgegen
  progress: ImportProgress | null; 
}>();

const emit = defineEmits<{ import: [path: string] }>();

// NEU: Logik, die den 'icon'-String vom Backend in ein Emoji umwandelt
const displayIcon = computed(() => {
  switch (props.device.icon) {
    case 'usb': return '💾';
    case 'sd': return '🂡';
    case 'camera': return '📷';
    case 'hdd': return '🖴';
    default: return '❓';
  }
});

function onCopy() {
  emit('import', props.device.path);
}
</script>

<template>
  <div class="card device-card" :class="{ disabled: props.disabled, busy: props.busy }">
    <header class="device-header">
      <span class="device-icon">{{ displayIcon }}</span>
      <strong class="device-name" :title="device.name">{{ device.name }}</strong>
    </header>

    <div class="storage-info">
      <div class="storage-text">
        <span>{{ formatSize(device.used) }} / {{ formatSize(device.total) }}</span>
      </div>
      <progress class="storage-bar" :value="device.used" :max="device.total"></progress>
    </div>

    <footer class="device-footer">
      <button v-if="!props.busy" class="btn w-full" :disabled="props.disabled" @click="onCopy">
        {{ copyText }}
      </button>

      <div v-else class="progress-container">
        <div v-if="props.progress" class="progress-details">
          <progress class="import-progress" :value="props.progress.copied" :max="props.progress.total"></progress>
          <div class="progress-text">
            <span>{{ props.progress.copied }} / {{ props.progress.total }}</span>
            <small :title="props.progress.current">{{ props.progress.current }}</small>
          </div>
        </div>
        <div v-else class="starting-text">
          <span>Starte Import...</span>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.device-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  background-color: var(--bg-color-secondary);
  border: 1px solid var(--border-color);
  transition: all 0.2s ease;
}
.device-card.disabled:not(.busy) {
  opacity: 0.6;
  background-color: var(--bg-color);
}

.device-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.device-icon {
  font-size: 1.8rem;
}
.device-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* --- Storage Info --- */
.storage-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.storage-text {
  font-size: 0.85rem;
  color: var(--text-color-secondary);
}
progress.storage-bar {
  width: 100%;
  height: 6px;
  accent-color: var(--text-color-secondary);
}

/* --- Footer & Progress --- */
.device-footer {
  margin-top: auto; /* Drückt den Footer nach unten */
  padding-top: 0.5rem;
  border-top: 1px solid var(--border-color);
}
.w-full {
  width: 100%;
}
.progress-container {
  width: 100%;
}
.starting-text {
  text-align: center;
  font-size: 0.9rem;
  color: var(--text-color-secondary);
}
.progress-details {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.progress-text {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
}
.progress-text small {
  max-width: 50%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: right;
  color: var(--text-color-secondary);
}
progress.import-progress {
  width: 100%;
  height: 8px;
  accent-color: var(--accent-color);
}

/* Allgemeiner Stil für alle Progress-Bars */
progress {
  border-radius: 4px;
  overflow: hidden;
  border: none;
}
progress::-webkit-progress-bar {
  background-color: var(--border-color);
}
progress::-webkit-progress-value {
  background-color: var(--accent-color);
  transition: width 0.3s ease;
}
progress.storage-bar::-webkit-progress-value {
  background-color: var(--text-color-secondary);
}
</style>