<template>
  <div class="image-card" :class="{ marked }">
    <img :src="src" alt="duplicate" />
    <p class="path">{{ path }}</p>
    <div class="actions">
      <button v-if="marked" @click="$emit('decision', 'keep')" class="keep">
        {{ keepText }}
      </button>
      <button @click="$emit('decision', 'delete')" class="delete">
        {{ deleteText }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';

const props = defineProps<{
  path: string;
  marked: boolean;
  keepText: string;
  deleteText: string;
}>();

const src = ref<string>('');

onMounted(async () => {
  const ext = props.path.split('.').pop()?.toLowerCase();
  const rawExts = ['raw', 'arw', 'dng', 'cr2', 'nef', 'pef', 'rw2', 'sr2'];
  if (ext && rawExts.includes(ext)) {
    src.value = await invoke<string>('generate_thumbnail', {
      path: props.path,
    });
  } else {
    src.value = convertFileSrc(props.path);
  }
});
</script>

<style scoped>
.image-card {
  border: 1px solid var(--border-color);
  padding: 0.5rem;
  border-radius: 0.5rem;
  background: var(--card-bg);
  width: 256px;
  text-align: center;
  position: relative;
}
.image-card img {
  max-width: 256px;
  max-height: 256px;
  width: auto;
  height: auto;
  object-fit: contain;
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
  outline: 2px solid var(--md-sys-color-primary, var(--accent-color));
  box-shadow: 0 2px 6px
    color-mix(
      in srgb,
      var(--md-sys-color-primary, var(--accent-color)),
      transparent 50%
    );
}
</style>
