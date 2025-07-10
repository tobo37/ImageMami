import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { createI18n } from "vue-i18n";
import "./style.css";

const messages = {
  en: {
    duplicate: {
      title: "Duplicate",
      choose: "Choose folder & scan",
      scanning: "Scanning…",
      deleteMarked: "Delete marked",
      confirmDelete: "Are you sure you want to delete {count} files?",
      dragDropInstruction: "Drag and drop",
      orClickToSelect: "or click",
    },
    common: {
      keep: "keep",
      delete: "delte",
      yes: "Yes",
      no: "No",
    },
    import: {
      title: "Import",
      choose: "Choose destination",
      destination: "Destination:",
      devices: "External devices",
      refresh: "Refresh",
      copy: "Copy images",
    },
    training: {
      title: "Training Data",
      export: "Export file",
    },
    sort: { title: "Sort" },
    blackhole: { title: "Blackhole" },
  },
  de: {
    duplicate: {
      title: "Duplikate",
      choose: "Ordner auswählen & scannen",
      scanning: "Scanne…",
      deleteMarked: "Markierte löschen",
      confirmDelete: "Möchtest du wirklich {count} Dateien löschen?",
      dragDropInstruction: "Ziehe einen Ordner hierher",
      orClickToSelect: "oder klicke hier, um einen Ordner auszuwählen",
    },
    common: {
      keep: "Behalten",
      delete: "Löschen",
      yes: "Ja",
      no: "Nein",
    },
    import: {
      title: "Importieren",
      choose: "Ziel wählen",
      destination: "Ziel:",
      devices: "Externe Geräte",
      refresh: "Aktualisieren",
      copy: "Bilder kopieren",
    },
    training: {
      title: "Trainingsdaten",
      export: "Datei exportieren",
    },
    sort: { title: "Sortieren" },
    blackhole: { title: "Schwarzes Loch" },
  },
};

const i18n = createI18n({
  legacy: false,
  locale: "en",
  fallbackLocale: "en",
  messages,
});

createApp(App).use(router).use(i18n).mount("#app");
