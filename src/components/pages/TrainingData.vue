<script setup lang="ts">
import { ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { exportTraining } from "../../services/tauriApi";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const busy = ref(false);

async function exportFile() {
  const selected = await save({
    filters: [{ name: "JSON", extensions: ["json"] }],
  });
  if (!selected) return;
  busy.value = true;
  try {
    await exportTraining(selected);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="view">
    <button @click="exportFile" :disabled="busy">
      {{ busy ? "…" : t("training.export") }}
    </button>
  </div>
</template>
