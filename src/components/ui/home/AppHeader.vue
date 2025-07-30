<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { onClickOutside } from '@vueuse/core';

// --- Assets ---
import flagEn from '../../../assets/flag-en.svg?url';
import flagDe from '../../../assets/flag-de.svg?url';
import ThemeToggle from './ThemeToggle.vue';

// --- Refs und State ---
const theme = ref<'light' | 'dark'>('light');
const isLangOpen = ref(false);
const langSelectElement = ref<HTMLDivElement | null>(null);

// --- Vue und i18n Composables ---
const router = useRouter();
const route = useRoute();
const { t, locale } = useI18n();

// --- Computed Properties für die Anzeige ---
const showBackButton = computed(() => route.name !== 'home');

const headerTitle = computed(() => {
  const titleKey = route.meta.titleKey as string | undefined;
  if (!titleKey) return '';
  return t(titleKey);
});

// --- Theme-Logik ---
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

// --- Navigations-Logik ---
function goBack() {
  router.back();
}

// --- Sprachauswahl-Logik ---
const languages = [
  { code: 'en', icon: flagEn, name: 'English' },
  { code: 'de', icon: flagDe, name: 'Deutsch' },
];

const currentLang = computed(
  () => languages.find((l) => l.code === locale.value) ?? languages[0],
);

function setLang(code: string) {
  locale.value = code;
  isLangOpen.value = false; // Dropdown schließen
}

// Schließt den Dropdown, wenn außerhalb geklickt wird
onClickOutside(langSelectElement, () => (isLangOpen.value = false));
</script>

<template>
  <header class="app-header">
    <div class="header-left">
      <transition name="fade">
        <button
          v-if="showBackButton"
          @click="goBack"
          class="icon-button"
          aria-label="Zurück"
        >
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
      </transition>
    </div>

    <div class="header-center">
      <transition name="fade">
        <span v-if="headerTitle" class="app-title">{{ headerTitle }}</span>
      </transition>
    </div>

    <div class="header-right">
      <ThemeToggle :theme="theme" @toggle="toggleTheme" :size="15"></ThemeToggle>
      <div class="lang-select" ref="langSelectElement">
        <button
          @click="isLangOpen = !isLangOpen"
          class="summary-button icon-button"
        >
          <img :src="currentLang.icon" :alt="currentLang.code" class="flag" />
        </button>

        <transition name="fade">
          <div v-if="isLangOpen" class="lang-options">
            <button
              v-for="lang in languages"
              :key="lang.code"
              @click="setLang(lang.code)"
              class="lang-option"
            >
              <img :src="lang.icon" :alt="lang.code" class="flag" />
              <span>{{ lang.name }}</span>
            </button>
          </div>
        </transition>
      </div>
    </div>
  </header>
</template>

<style scoped>
/* Grundstruktur */
.app-header {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  padding: 0.5rem 1rem;
  height: 60px;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.header-left {
  display: flex;
  justify-content: flex-start;
}
.header-center {
  text-align: center;
  font-weight: 600;
  color: var(--text-color, #111);
}
.header-right {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 0.5rem;
}

/* Generischer Button für Icons */
.icon-button {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  padding: 0.5rem;
  cursor: pointer;
  border-radius: 50%;
  color: var(--text-color, #111);
  transition: background-color 0.2s ease;
}
.icon-button:hover {
  background-color: var(--hover-bg, rgba(128, 128, 128, 0.1));
}

/* Sprachauswahl */
.lang-select {
  position: relative;
}
.summary-button .flag {
  border-radius: 50%;
  object-fit: cover;
}
.flag {
  width: 24px;
  height: 24px;
}
.lang-options {
  position: absolute;
  top: calc(100% + 0.5rem);
  right: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  background-color: var(--card-bg, #fff);
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  padding: 0.5rem;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  min-width: 150px;
}
.lang-option {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  background: none;
  border: none;
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  border-radius: 6px;
  text-align: left;
  width: 100%;
  font-size: 0.9rem;
  color: var(--text-color, #111);
  transition: background-color 0.2s ease;
}
.lang-option:hover {
  background-color: var(--hover-bg, rgba(128, 128, 128, 0.1));
}

/* Übergangsanimationen */
.fade-enter-active,
.fade-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-5px);
}
</style>
