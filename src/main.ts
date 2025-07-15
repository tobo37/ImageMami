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
      tags: {
        hash: "Exact match",
      },
    },
    common: {
      keep: "keep",
      delete: "delete",
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
    home: {
      previewBanner: "preview - wait for release",
    },
    sort: { title: "Sort" },
    blackhole: {
      title: "Blackhole",
      scan: "Scan",
      disks: "Drives",
      folders: "Folders",
      copy: "Copy",
      cut: "Cut",
      files: "files",
    },
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
      tags: {
        hash: "Absolut identisch",
      },
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
    home: {
      previewBanner: "Vorschau - warte auf Release",
    },
    sort: { title: "Sortieren" },
    blackhole: {
      title: "Schwarzes Loch",
      scan: "Scannen",
      disks: "Laufwerke",
      folders: "Ordner",
      copy: "Kopieren",
      cut: "Ausschneiden",
      files: "Dateien",
    },
  },
};

const i18n = createI18n({
  legacy: false,
  locale: "en",
  fallbackLocale: "en",
  messages,
});

createApp(App).use(router).use(i18n).mount("#app");
