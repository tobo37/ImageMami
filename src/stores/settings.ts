import { defineStore } from "pinia";
import { ref, watch } from "vue";

export const useSettingsStore = defineStore("settings", () => {
  const importDestination = ref<string | null>(
    localStorage.getItem("importDest"),
  );

  watch(importDestination, (val) => {
    if (val) {
      localStorage.setItem("importDest", val);
    } else {
      localStorage.removeItem("importDest");
    }
  });

  function setImportDestination(path: string | null) {
    importDestination.value = path;
  }

  return { importDestination, setImportDestination };
});
