<template>
  <div class="duplicate-card">
    <Thumbnail :path="group.paths[0]" class="preview" />
    <div class="path-list">
      <div
        v-for="p in group.paths"
        :key="p"
        class="path-row"
        :class="{ marked: marked.includes(p) }"
      >
        <span class="path">{{ p }}</span>
        <button @click="toggle(p)">
          {{ marked.includes(p) ? keepText : deleteText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import Thumbnail from './Thumbnail.vue';

const props = defineProps<{
  group: { tag: string; hash: string; paths: string[] };
  marked: string[];
  deleteText: string;
  keepText: string;
}>();

const emit = defineEmits<{ decision: [path: string, value: string] }>();

function toggle(path: string) {
  if (props.marked.includes(path)) emit('decision', path, 'keep');
  else emit('decision', path, 'delete');
}
</script>

<style scoped>
.duplicate-card {
  display: flex;
  gap: 1rem;
  align-items: flex-start;
}
.preview {
  flex-shrink: 0;
}
.path-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.path-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.path-row .path {
  flex: 1;
  word-break: break-word;
  font-size: 0.8rem;
}
.path-row button {
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  cursor: pointer;
  background: red;
  color: white;
}
.path-row.marked .path {
  text-decoration: line-through;
  opacity: 0.7;
}
</style>
