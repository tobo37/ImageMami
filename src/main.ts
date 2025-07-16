import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { createI18n } from "vue-i18n";
import en from "./locales/en.json";
import de from "./locales/de.json";
import "./style.css";

const messages = {
  en,
  de,
};

const i18n = createI18n({
  legacy: false,
  locale: "en",
  fallbackLocale: "en",
  messages,
});

createApp(App).use(router).use(i18n).mount("#app");
