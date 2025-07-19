<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import flagEn from '../../assets/flag-en.svg?url';
import flagDe from '../../assets/flag-de.svg?url';

const theme = ref<'light' | 'dark'>('light');

const router = useRouter();
const route = useRoute();
const showBackButton = computed(() => route.name !== 'home');
const headerClass = computed(() => ({ 'with-back': showBackButton.value }));
const { t, locale } = useI18n();
const headerTitle = computed(() => {
  const name = route.name as string | undefined;
  if (!name || name === 'home') return '';
  return t(`${name}.title`).toUpperCase();
});

const languages = [
  { code: 'en', icon: flagEn },
  { code: 'de', icon: flagDe },
];

const langSelect = ref<HTMLDetailsElement | null>(null);

const currentLang = computed(
  () => languages.find((l) => l.code === locale.value) ?? languages[0],
);

function setLang(code: string) {
  locale.value = code;
  langSelect.value?.removeAttribute('open');
}

onMounted(() => {
  theme.value = window.matchMedia('(prefers-color-scheme: dark)').matches
    ? 'dark'
    : 'light';
});

watch(
  theme,
  (val) => {
    document.documentElement.setAttribute('data-theme', val);
  },
  { immediate: true },
);

function toggleTheme() {
  theme.value = theme.value === 'light' ? 'dark' : 'light';
}

function goBack() {
  router.back();
}
</script>

<template>
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
    <span v-if="headerTitle" class="app-title">{{ headerTitle }}</span>
    <button @click="toggleTheme">{{ theme === 'dark' ? '☀️' : '🌙' }}</button>
    <details class="lang-select" ref="langSelect">
      <summary>
        <img :src="currentLang.icon" :alt="currentLang.code" class="flag" />
      </summary>
      <div class="lang-options">
        <button
          v-for="lang in languages"
          :key="lang.code"
          @click.prevent="setLang(lang.code)"
        >
          <img :src="lang.icon" :alt="lang.code" class="flag" />
        </button>
      </div>
    </details>
  </header>
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
.app-title {
  flex-grow: 1;
  text-align: center;
  font-weight: bold;
  text-transform: uppercase;
}
.lang-select {
  position: relative;
  margin-left: 0.5rem;
}

.lang-select summary {
  list-style: none;
  cursor: pointer;
}

.lang-select summary::-webkit-details-marker {
  display: none;
}

.flag {
  width: 24px;
  height: auto;
}

.lang-options {
  position: absolute;
  right: 0;
  display: flex;
  flex-direction: column;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  margin-top: 0.25rem;
  padding: 0.25rem;
  z-index: 10;
}

.lang-options button {
  background: transparent;
  border: none;
  padding: 0.25rem;
  cursor: pointer;
}

.lang-options button:hover {
  background-color: color-mix(in srgb, var(--accent-color), transparent 80%);
}
</style>
