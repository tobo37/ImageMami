<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const theme = ref<'light' | 'dark'>('light')

const router = useRouter()
const route = useRoute()
const showBackButton = computed(() => route.name !== 'home')
const headerClass = computed(() => ({ 'with-back': showBackButton.value }))

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

function goBack() {
  router.back()
}
</script>

<template>
  <div>
    <header :class="['app-header', headerClass]">
      <button v-if="showBackButton" @click="goBack">🔙</button>
      <button @click="toggleTheme">{{ theme === 'dark' ? '☀️' : '🌙' }}</button>
    </header>
    <router-view />
  </div>
</template>

<style scoped>
.app-header {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding: 0.5rem 1rem;
}
.app-header.with-back {
  justify-content: space-between;
}
</style>
