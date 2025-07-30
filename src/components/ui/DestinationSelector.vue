<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  path: string | null;
  label: string;
  chooseText: string;
}>();
const emit = defineEmits<{ choose: [] }>();

const displayPath = computed(() => {
  if (!props.path) return '-';
  const limit = 50;
  if (props.path.length <= limit) return props.path;
  const start = props.path.slice(0, 25);
  const end = props.path.slice(props.path.length - 25);
  return `${start}...${end}`;
});
function onChoose() {
  emit('choose');
}
</script>

<template>
  <div class="card dest-card">
    <div class="dest-info">
      <strong>{{ props.label }}</strong>
      <span class="dest-path" :title="props.path || '-'">{{
        displayPath
      }}</span>
    </div>
    <button class="btn" @click="onChoose">{{ props.chooseText }}</button>
  </div>
</template>

<style scoped>
.dest-card {
  display: flex; /* NEU: Stellt sicher, dass die Ausrichtung stimmt */
  align-items: center; /* NEU: Vertikal zentrieren */
  justify-content: space-between;
  gap: 1rem;
}
.dest-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  /* NEU: Verhindert, dass der Text den Button verdrängt */
  min-width: 0; 
}
.dest-path {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  /* NEU: Etwas hellere Farbe für bessere Lesbarkeit */
  color: var(--text-color-secondary); 
  font-size: 0.9rem;
}
</style>