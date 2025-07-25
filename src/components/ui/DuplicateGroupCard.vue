<template>
  <div class="duplicate-card">
    <!-- Use a standard img tag and bind its src to the thumbnail data URL -->
    <img
      v-if="group.files.length > 0 && group.files[0].preview"
      :src="group.files[0].preview"
      class="preview"
      alt="File preview"
    />
    <div class="path-list">
      <label
        v-for="f in group.files"
        :key="f.path"
        class="path-row"
        :class="{ marked: marked.includes(f.path) }"
      >
        <input
          type="checkbox"
          :checked="marked.includes(f.path)"
          @change="(e) => toggle(f.path, e)"
        />
        <span class="path" v-html="highlight(f.path)"></span>
        <span class="age">{{ formatAge(f.age) }}</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

// --- Type Definitions ---
interface FileInfo {
  path: string;
  age: number;
  size: number;
  preview: string; // The thumbnail is now part of the FileInfo type
  hash?: string;
  dhash?: string;
}

interface DuplicateGroup {
  method: unknown;
  files: FileInfo[];
}

const props = defineProps<{
  group: DuplicateGroup;
  marked: string[];
  deleteText: string;
  keepText: string;
}>();

const emit = defineEmits<{ decision: [path: string, value: string] }>();

function toggle(path: string, e: Event) {
  const checked = (e.target as HTMLInputElement).checked;
  if (checked) emit('decision', path, 'delete');
  else emit('decision', path, 'keep');
}

const highlightedPaths = computed(() => {
  const paths = props.group.files.map((f) => f.path);
  const normalized = paths.map((p) => p.replace(/\\/g, '/').split('/'));
  const maxParts = Math.max(...normalized.map((arr) => arr.length));
  const diffIndices: number[] = [];
  for (let i = 0; i < maxParts; i++) {
    const parts = new Set(normalized.map((arr) => arr[i] ?? ''));
    if (parts.size > 1) diffIndices.push(i);
  }
  return normalized.map((arr) =>
    arr
      .map((seg, idx) => (diffIndices.includes(idx) ? `<b>${seg}</b>` : seg))
      .join('/')
  );
});

function highlight(path: string) {
  const paths = props.group.files.map((f) => f.path);
  const idx = paths.indexOf(path);
  return highlightedPaths.value[idx] ?? path;
}

function formatAge(sec: number) {
  const days = sec / 86400;
  if (days >= 1) return `${days.toFixed(1)}d`;
  const hours = sec / 3600;
  if (hours >= 1) return `${hours.toFixed(1)}h`;
  const minutes = sec / 60;
  if (minutes >= 1) return `${minutes.toFixed(1)}m`;
  return `${sec}s`;
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
  width: 128px; /* Example width */
  height: 128px; /* Example height */
  object-fit: cover;
  border-radius: 0.5rem;
  background-color: var(--bg-color-secondary);
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
  cursor: pointer;
}
.path-row .path {
  flex: 1;
  word-break: break-word;
  font-size: 1rem;
}
.path-row .age {
  font-size: 0.85rem;
  color: var(--text-muted);
  white-space: nowrap;
}
.path-row input[type='checkbox'] {
  accent-color: red;
  width: 1.2rem;
  height: 1.2rem;
}
.path-row b {
  font-weight: 900;
}
.path-row.marked .path {
  text-decoration: line-through;
  opacity: 0.7;
}
</style>
