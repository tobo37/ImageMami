<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>

<template>
  <div class="view">
    <h1>{{ t('blackhole.title') }}</h1>
  </div>
</template>
