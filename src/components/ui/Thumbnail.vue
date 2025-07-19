<script setup lang="ts">
import { ref, onMounted, watch, onBeforeUnmount } from 'vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';

const props = defineProps<{ path: string }>();
const src = ref('');
const imgRef = ref<HTMLImageElement | null>(null);
let observer: IntersectionObserver | null = null;

async function loadImage() {
  const ext = props.path.split('.').pop()?.toLowerCase();
  const rawExts = ['raw', 'arw', 'dng', 'cr2', 'nef', 'pef', 'rw2', 'sr2'];
  if (ext && rawExts.includes(ext)) {
    src.value = await invoke<string>('generate_thumbnail', { path: props.path });
  } else {
    src.value = convertFileSrc(props.path);
  }
}

function onError() {
  invoke<string>('generate_thumbnail', { path: props.path }).then((s) => {
    src.value = s;
  });
}

function observe() {
  if (!observer && imgRef.value) {
    observer = new IntersectionObserver(([e]) => {
      if (e.isIntersecting) {
        loadImage();
        observer?.disconnect();
        observer = null;
      }
    });
    observer.observe(imgRef.value);
  }
}

onMounted(observe);
watch(
  () => props.path,
  () => {
    src.value = '';
    observer?.disconnect();
    observer = null;
    observe();
  },
);
onBeforeUnmount(() => {
  observer?.disconnect();
});
</script>

<template>
  <div class="thumb">
    <img ref="imgRef" :src="src" loading="lazy" @error="onError" />
  </div>
</template>

<style scoped>
.thumb {
  width: 120px;
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-color);
  padding: 0.25rem;
  border-radius: 0.25rem;
  background: var(--card-bg);
}
.thumb img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
</style>
