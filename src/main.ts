import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { createI18n } from 'vue-i18n'

const messages = {
  en: {
    duplicate: {
      title: 'Duplicate',
      choose: 'Choose folder & scan',
      scanning: 'Scanning…',
    },
    import: { title: 'Import' },
    sort: { title: 'Sort' },
    blackhole: { title: 'Blackhole' },
  },
  de: {
    duplicate: {
      title: 'Duplikate',
      choose: 'Ordner auswählen & scannen',
      scanning: 'Scanne…',
    },
    import: { title: 'Importieren' },
    sort: { title: 'Sortieren' },
    blackhole: { title: 'Schwarzes Loch' },
  },
}

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages,
})

createApp(App).use(router).use(i18n).mount('#app')
