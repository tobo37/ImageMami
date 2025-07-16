<template>
  <div class="duplicate-card">
    <Thumbnail :path="group.paths[0]" class="preview" />
    <div class="path-list">
      <div
        v-for="(p, i) in group.paths"
        :key="p"
        class="path-row"
        :class="{ marked: marked.includes(p) }"
      >
        <span class="path" v-html="highlight(p)"></span>
        <span class="date">{{ formatDate(group.dates[i]) }}</span>
        <button @click="toggle(p)">
          {{ marked.includes(p) ? keepText : deleteText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Thumbnail from './Thumbnail.vue';

const props = defineProps<{
  group: { tag: string; hash: string; paths: string[]; dates: string[] };
  marked: string[];
  deleteText: string;
  keepText: string;
}>();

const emit = defineEmits<{ decision: [path: string, value: string] }>();

function toggle(path: string) {
  if (props.marked.includes(path)) emit('decision', path, 'keep');
  else emit('decision', path, 'delete');
}

const highlightedPaths = computed(() => {
  const normalized = props.group.paths.map((p) =>
    p.replace(/\\/g, '/').split('/'),
  );
  const maxParts = Math.max(...normalized.map((arr) => arr.length));
  const diffIndices: number[] = [];
  for (let i = 0; i < maxParts; i++) {
    const parts = new Set(normalized.map((arr) => arr[i] ?? ''));
    if (parts.size > 1) diffIndices.push(i);
  }
  return normalized.map((arr) =>
    arr
      .map((seg, idx) => (diffIndices.includes(idx) ? `<b>${seg}</b>` : seg))
      .join('/'),
  );
});

function highlight(path: string) {
  const idx = props.group.paths.indexOf(path);
  return highlightedPaths.value[idx] ?? path;
}

function formatDate(iso: string) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return '';
  return date.toLocaleDateString();
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
.path-row .date {
  font-size: 0.75rem;
  color: var(--text-muted);
  white-space: nowrap;
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
