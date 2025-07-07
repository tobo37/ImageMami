<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'

const theme = ref<'light' | 'dark'>('light')

onMounted(() => {
  theme.value = window.matchMedia('(prefers-color-scheme: dark)').matches
    ? 'dark'
    : 'light'
})

watch(
  theme,
  (val) => {
    document.documentElement.setAttribute('data-theme', val)
  },
  { immediate: true }
)

function toggleTheme() {
  theme.value = theme.value === 'light' ? 'dark' : 'light'
}
</script>

<template>
  <div>
    <header class="app-header">
      <button @click="toggleTheme">{{ theme === 'dark' ? '☀️' : '🌙' }}</button>
    </header>
    <router-view />
  </div>
</template>

<style scoped>
.app-header {
  display: flex;
  justify-content: flex-end;
  padding: 0.5rem 1rem;
}
</style>
