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
      <button v-if="showBackButton" @click="goBack" aria-label="Back">
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M15 19L8 12L15 5"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
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
