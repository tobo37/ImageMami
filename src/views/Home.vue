<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import HomeCard from '../components/ui/home/HomeCard.vue'; // Pfad anpassen

const { t } = useI18n();

// Optional: Hier könntest du die Logik für den Datei-Import via Drag&Drop handhaben
function handleFilesDropped(files: FileList) {
  console.log('Dateien für den Import erhalten:', files);
  // Hier die Import-Logik starten und zur Import-Seite weiterleiten...
}
</script>

<template>
  <div class="view">
    <h1 class="welcome-title">{{t('home.title')}}</h1>
    <p class="welcome-subtitle">{{t('home.subtitle')}}</p>

    <div class="grid-home">
      <HomeCard
        to="/duplicate"
        :title="t('home.duplicate.title')"
        :description="t('home.duplicate.description')"
      >
        <template #icon>
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M7.5 3.375C7.5 2.33947 8.33947 1.5 9.375 1.5H9.75C11.8211 1.5 13.5 3.17893 13.5 5.25V7.125C13.5 8.16053 14.3395 9 15.375 9H17.25C19.3211 9 21 10.6789 21 12.75V16.125C21 17.1605 20.1605 18 19.125 18H9.375C8.33947 18 7.5 17.1605 7.5 16.125V3.375Z"
              fill="currentColor"
            />
            <path
              d="M15 5.25C15 3.93695 14.518 2.73648 13.7212 1.8159C17.1201 2.70377 19.7962 5.37988 20.6841 8.77881C19.7635 7.98204 18.5631 7.5 17.25 7.5H15.375C15.1679 7.5 15 7.33211 15 7.125V5.25Z"
              fill="currentColor"
            />
            <path
              d="M4.875 6H6V16.125C6 17.989 7.51104 19.5 9.375 19.5H16.5V20.625C16.5 21.6605 15.6605 22.5 14.625 22.5H4.875C3.83947 22.5 3 21.6605 3 20.625V7.875C3 6.83947 3.83947 6 4.875 6Z"
              fill="currentColor"
            />
          </svg>
        </template>
      </HomeCard>

      <HomeCard
        to="/import"
        :title="t('home.import.title')"
        :description="t('home.import.description')"
        :is-droppable="true"
        @files-dropped="handleFilesDropped"
      >
        <template #icon>
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              fill-rule="evenodd"
              clip-rule="evenodd"
              d="M12 2.25C12.4142 2.25 12.75 2.58579 12.75 3V14.6893L15.9697 11.4697C16.2626 11.1768 16.7374 11.1768 17.0303 11.4697C17.3232 11.7626 17.3232 12.2374 17.0303 12.5303L12.5303 17.0303C12.2374 17.3232 11.7626 17.3232 11.4697 17.0303L6.96967 12.5303C6.67678 12.2374 6.67678 11.7626 6.96967 11.4697C7.26256 11.1768 7.73744 11.1768 8.03033 11.4697L11.25 14.6893V3C11.25 2.58579 11.5858 2.25 12 2.25ZM3 15.75C3.41421 15.75 3.75 16.0858 3.75 16.5V18.75C3.75 19.5784 4.42157 20.25 5.25 20.25H18.75C19.5784 20.25 20.25 19.5784 20.25 18.75V16.5C20.25 16.0858 20.5858 15.75 21 15.75C21.4142 15.75 21.75 16.0858 21.75 16.5V18.75C21.75 20.4069 20.4069 21.75 18.75 21.75H5.25C3.59315 21.75 2.25 20.4069 2.25 18.75V16.5C2.25 16.0858 2.58579 15.75 3 15.75Z"
              fill="currentColor"
            />
          </svg>
        </template>
      </HomeCard>
    </div>
  </div>
</template>

<style scoped>
/* Hauptcontainer, der die ganze Seite strukturiert */
.view {
  display: flex;
  flex-direction: column;
  justify-content: center; /* Zentriert den gesamten Inhaltsblock vertikal */
  align-items: center;     /* Zentriert den Inhaltsblock horizontal */
  box-sizing: border-box;  /* Stellt sicher, dass Padding in der Höhe enthalten ist */
  min-height: 90vh;        /* Nutzt den Großteil der Bildschirmhöhe */
  padding: 2rem;
}

.welcome-title {
  text-align: center;
  font-size: 2.5rem;
  font-weight: 600;
  margin-top: 0; /* Margin wird vom Flex-Container gesteuert */
  margin-bottom: 0.5rem;
}

.welcome-subtitle {
  text-align: center;
  font-size: 1.2rem;
  color: var(--color-text-soft);
  margin-bottom: 3rem;
}

.grid-home {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 2rem;
  width: 100%; /* Grid soll die volle Breite des zentrierten Bereichs nutzen */
  max-width: 720px; /* Verhindert, dass die Karten auf sehr breiten Bildschirmen zu groß werden */
}
</style>
