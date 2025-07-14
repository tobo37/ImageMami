<template>
  <div v-if="visible" class="modal-backdrop">
    <div class="modal">
      <p>{{ message }}</p>
      <ul class="file-list">
        <li v-for="p in files" :key="p">{{ p }}</li>
      </ul>
      <div class="actions">
        <button @click="$emit('confirm')">{{ yesText }}</button>
        <button class="ghost" @click="$emit('cancel')">{{ noText }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  files: string[];
  message: string;
  yesText: string;
  noText: string;
}>();
</script>

<style scoped>
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}
.modal {
  background: var(--card-bg);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 1rem;
  max-width: 90%;
}
.file-list {
  max-height: 200px;
  overflow-y: auto;
  margin: 1rem 0;
  padding-left: 1rem;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
button.ghost {
  background: transparent;
  color: var(--accent-color);
  border: 1px solid color-mix(in srgb, var(--accent-color), transparent 70%);
}
</style>
