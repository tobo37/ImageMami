<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

const props = defineProps<{ path: string }>();
const src = ref("");

onMounted(async () => {
  const ext = props.path.split(".").pop()?.toLowerCase();
  const rawExts = ["raw", "arw", "dng", "cr2", "nef", "pef", "rw2", "sr2"];
  if (ext && rawExts.includes(ext)) {
    src.value = await invoke<string>("generate_thumbnail", {
      path: props.path,
    });
  } else {
    src.value = convertFileSrc(props.path);
  }
});
</script>

<template>
  <div class="thumb"><img :src="src" /></div>
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
