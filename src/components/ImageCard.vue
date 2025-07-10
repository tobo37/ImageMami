<template>
  <div class="image-card" :class="{ marked }">
    <img :src="'file://' + path" alt="duplicate" />
    <p class="path">{{ path }}</p>
    <div class="actions">
      <button @click="$emit('decision', 'keep')" class="keep">
        {{ keepText }}
      </button>
      <button @click="$emit('decision', 'delete')" class="delete">
        {{ deleteText }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  path: string;
  marked: boolean;
  keepText: string;
  deleteText: string;
}>();
</script>

<style scoped>
.image-card {
  border: 1px solid var(--border-color);
  padding: 0.5rem;
  border-radius: 0.5rem;
  background: var(--card-bg);
  width: 200px;
  text-align: center;
  position: relative;
}
.image-card img {
  width: 100%;
  height: auto;
  object-fit: cover;
  border-radius: 0.25rem;
}
.image-card .path {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin: 0.5rem 0;
  word-break: break-word;
}
.image-card .actions {
  display: flex;
  justify-content: center;
  gap: 0.5rem;
}
.image-card button {
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  cursor: pointer;
}
.image-card button.keep {
  background: var(--accent-color, green);
  color: white;
}
.image-card button.delete {
  background: red;
  color: white;
}
.image-card.marked {
  outline: 2px solid red;
}
</style>
