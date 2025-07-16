import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const useSettingsStore = defineStore('settings', () => {
  const importDestination = ref<string | null>(
    localStorage.getItem('importDest'),
  );

  const duplicateDestination = ref<string | null>(
    localStorage.getItem('duplicateDest'),
  );

  watch(importDestination, (val) => {
    if (val) {
      localStorage.setItem('importDest', val);
    } else {
      localStorage.removeItem('importDest');
    }
  });

  watch(duplicateDestination, (val) => {
    if (val) {
      localStorage.setItem('duplicateDest', val);
    } else {
      localStorage.removeItem('duplicateDest');
    }
  });

  function setImportDestination(path: string | null) {
    importDestination.value = path;
  }

  function setDuplicateDestination(path: string | null) {
    duplicateDestination.value = path;
  }

  return {
    importDestination,
    setImportDestination,
    duplicateDestination,
    setDuplicateDestination,
  };
});
