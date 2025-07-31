import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import { createI18n } from 'vue-i18n';
import en from './locales/en.json';
import de from './locales/de.json';
import es from './locales/es.json';
import zh from './locales/zh.json';
import fr from './locales/fr.json';
import pt from './locales/pt.json';
import ja from './locales/ja.json';
import hi from './locales/hi.json';
import ru from './locales/ru.json';
import './style.css';

const messages = {
  en,
  de,
  es,
  zh,
  fr,
  pt,
  ja,
  hi,
  ru,
};

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages,
});

const pinia = createPinia();

createApp(App).use(router).use(i18n).use(pinia).mount('#app');
