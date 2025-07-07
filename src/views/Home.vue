<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>

<template>
  <div class="grid grid-cols-2 gap-4">
    <router-link class="p-4 border rounded" to="/duplicate">{{ t('duplicate.title') }}</router-link>
    <router-link class="p-4 border rounded" to="/import">{{ t('import.title') }}</router-link>
    <router-link class="p-4 border rounded" to="/sort">{{ t('sort.title') }}</router-link>
    <router-link class="p-4 border rounded" to="/blackhole">{{ t('blackhole.title') }}</router-link>
  </div>
</template>
