<script setup lang="ts">
import { computed } from 'vue';
import DotsLoader from './DotsLoader.vue';

interface Device {
  name: string;
  path: string;
  total: number;
}

const props = defineProps<{
  device: Device;
  disabled: boolean;
  copyText: string;
  busy: boolean;
  formatSize: (bytes: number) => string;
}>();

const emit = defineEmits<{ import: [path: string] }>();
const icon = computed(() => {
  const n = props.device.name.toLowerCase();
  if (n.includes('sd') || n.includes('usb')) return '💾';
  return '🗄️';
});
function onCopy() {
  emit('import', props.device.path);
}
</script>

<template>
  <div class="card device-card">
    <div class="device-header">
      <span class="device-icon">{{ icon }}</span>
      <strong class="device-name">{{ props.device.name }}</strong>
    </div>

    <p class="device-path">{{ props.device.path }}</p>
    <p class="device-size">{{ props.formatSize(props.device.total) }}</p>

    <button
      v-if="!props.busy"
      class="btn w-full"
      :disabled="disabled || props.busy"
      @click="onCopy"
    >
      {{ copyText }}
    </button>
    <DotsLoader v-else />
  </div>
</template>

<style scoped>
.device-card {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.device-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.device-icon {
  font-size: 1.4rem;
}
.device-name {
  flex: 1;
  text-transform: none;
}
.device-path {
  font-size: 0.8rem;
  opacity: 0.8;
  word-break: break-all;
}
.device-size {
  font-size: 0.9rem;
}
.w-full {
  width: 100%;
}
</style>
