<script setup lang="ts">
import { ref } from 'vue';

// Definiert, welche "Props" (Eigenschaften) die Komponente von außen annimmt
const props = defineProps({
  to: { type: String, required: true },
  title: { type: String, required: true },
  description: { type: String, required: true },
  isDroppable: { type: Boolean, default: false }, // Um Drag&Drop zu aktivieren
});

// Definiert, welche "Events" die Komponente auslösen kann
const emit = defineEmits(['files-dropped']);

const isDragOver = ref(false);

function onDrop(event: DragEvent) {
  if (!props.isDroppable) return;
  isDragOver.value = false;
  const files = event.dataTransfer?.files;
  if (files && files.length > 0) {
    emit('files-dropped', files);
  }
}
</script>

<template>
  <router-link
    :to="to"
    class="card"
    :class="{ 'drag-over': isDragOver }"
    @dragover.prevent="isDragOver = props.isDroppable"
    @dragleave.prevent="isDragOver = false"
    @drop.prevent="onDrop"
  >
    <div class="card-icon">
      <slot name="icon"></slot>
    </div>
    <div class="card-content">
      <h2 class="card-title">{{ title }}</h2>
      <p class="card-description">{{ description }}</p>
    </div>
  </router-link>
</template>

<style scoped>
.card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 2rem 2rem 2.5rem 2rem;
  background-color: var(
    --color-background-soft
  ); /* Beispiel: Etwas hellerer Hintergrund */
  border-radius: 12px;
  text-decoration: none;
  color: var(--color-text);
  border: 2px solid transparent;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease,
    border-color 0.2s ease;
  min-height: 250px;
}

.card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.2);
}

.card.drag-over {
  border-color: var(
    --color-primary
  ); /* Hebt die Kachel hervor, wenn eine Datei darüber schwebt */
  transform: scale(1.02);
}

.card-icon {
  font-size: 3rem; /* Macht das SVG größer */
  margin-bottom: 1.5rem;
  color: var(--color-primary); /* Beispiel: Eine Akzentfarbe für das Icon */
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.card-title {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0;
}

.card-description {
  font-size: 1rem;
  color: var(
    --color-text-soft
  ); /* Leicht ausgegrauter Text für die Beschreibung */
  margin: 0;
  line-height: 1.5;
}
</style>
