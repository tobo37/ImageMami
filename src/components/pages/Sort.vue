<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";

import DestinationSelector from "../ui/DestinationSelector.vue";
import Thumbnail from "../ui/Thumbnail.vue";

const { t } = useI18n();
const srcPath = ref<string | null>(null);
const images = ref<string[]>([]);
const busy = ref(false);

async function chooseSource() {
  const selected = await open({ directory: true, multiple: false });
  if (!selected) return;
  const path = Array.isArray(selected) ? selected[0] : selected;
  srcPath.value = path;
  await loadImages();
}

async function loadImages() {
  if (!srcPath.value) return;
  images.value = await invoke<string[]>("find_images", { path: srcPath.value });
}

async function startSort() {
  if (!srcPath.value) return;
  busy.value = true;
  try {
    await invoke("sort_images", { path: srcPath.value });
    await loadImages();
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="view sort-view">
    <DestinationSelector
      :path="srcPath"
      :label="t('sort.source')"
      :choose-text="t('sort.choose')"
      @choose="chooseSource"
    />

    <div v-if="images.length" class="thumb-grid">
      <Thumbnail v-for="p in images" :key="p" :path="p" />
    </div>
    <p v-else class="placeholder">-</p>

    <button class="btn" @click="startSort" :disabled="busy || !srcPath">
      {{ busy ? "…" : t("sort.start") }}
    </button>
  </div>
</template>

<style scoped>
.sort-view {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.thumb-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, 120px);
  gap: 0.5rem;
}

.placeholder {
  opacity: 0.6;
}
</style>
