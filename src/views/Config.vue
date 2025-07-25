<script setup lang="ts">
import { useSettingsStore } from '../stores/settings';
import { useI18n } from 'vue-i18n';

const settings = useSettingsStore();
const { t } = useI18n();

function toggle(ext: string) {
  const current = settings.allowedExtensions.slice();
  if (current.includes(ext)) {
    settings.setAllowedExtensions(current.filter((e) => e !== ext));
  } else {
    current.push(ext);
    settings.setAllowedExtensions(current);
  }
}
</script>

<template>
  <div class="view config-view">
    <h2>{{ t('config.allowedExtensions') }}</h2>
    <div class="ext-grid">
      <label v-for="ext in settings.allExtensions" :key="ext">
        <input
          type="checkbox"
          :checked="settings.allowedExtensions.includes(ext)"
          @change="toggle(ext)"
        />
        {{ ext }}
      </label>
    </div>
  </div>
</template>

<style scoped>
.config-view {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.ext-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
  gap: 0.5rem;
}

label {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}
</style>
