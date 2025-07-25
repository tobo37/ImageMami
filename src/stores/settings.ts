import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const useSettingsStore = defineStore('settings', () => {
  const importDestination = ref<string | null>(
    localStorage.getItem('importDest'),
  );

  const duplicateDestination = ref<string | null>(
    localStorage.getItem('duplicateDest'),
  );

  const allExtensions = [
    'jpg',
    'jpeg',
    'png',
    'gif',
    'bmp',
    'tif',
    'tiff',
    'webp',
    'heic',
    'heif',
    'raw',
    'arw',
    'dng',
    'cr2',
    'nef',
    'pef',
    'rw2',
    'sr2',
  ];

  const allowedExtensions = ref<string[]>(
    JSON.parse(localStorage.getItem('allowedExtensions') ?? 'null') ?? [
      ...allExtensions,
    ],
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

  watch(
    allowedExtensions,
    (val) => {
      localStorage.setItem('allowedExtensions', JSON.stringify(val));
    },
    { deep: true },
  );

  function setImportDestination(path: string | null) {
    importDestination.value = path;
  }

  function setDuplicateDestination(path: string | null) {
    duplicateDestination.value = path;
  }

  function setAllowedExtensions(exts: string[]) {
    allowedExtensions.value = exts;
  }

  return {
    importDestination,
    setImportDestination,
    duplicateDestination,
    setDuplicateDestination,
    allExtensions,
    allowedExtensions,
    setAllowedExtensions,
  };
});
