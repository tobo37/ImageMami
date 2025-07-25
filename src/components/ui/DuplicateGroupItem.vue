<template>
  <div class="duplicate-group">
    <h3>
      {{ tagText(group.method) }}
      <small>{{ formatSize(group.files[0].size) }}</small>
    </h3>
    <DuplicateGroupCard
      :group="group"
      :marked="marked"
      :delete-text="deleteText"
      :keep-text="keepText"
      @decision="(path: string, v: string) => onDecision(path, v)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import DuplicateGroupCard from './DuplicateGroupCard.vue';

interface FileInfo {
  path: string;
  age: number;
  size: number;
  hash?: string;
  dhash?: string;
}

interface DuplicateGroup {
  method: unknown;
  files: FileInfo[];
}

const props = defineProps<{
  source: DuplicateGroup;
  marked: string[];
  deleteText: string;
  keepText: string;
  onDecision?: (path: string, value: string) => void;
}>();
const group = computed(() => props.source);
const emit = defineEmits<{ decision: [path: string, value: string] }>();
const { t } = useI18n();

function onDecision(path: string, value: string) {
  emit('decision', path, value);
  props.onDecision?.(path, value);
}

function tagText(tag: unknown) {
  let name: string;
  if (typeof tag === 'object' && tag !== null) {
    name = Object.keys(tag)[0];
  } else {
    name = String(tag);
  }
  const map: Record<string, string> = {
    ByteHash: 'hash',
    PerceptualDHash: 'dhash',
  };
  const key = `duplicate.tags.${map[name] ?? name}`;
  const result = t(key);
  return result === key ? name : result;
}

function formatSize(bytes: number) {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes,
    i = 0;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${size.toFixed(1)} ${units[i]}`;
}
</script>

<style scoped>
.duplicate-group {
  margin-bottom: 1rem;
}

.duplicate-group h3 {
  font-size: 1rem;
  margin-bottom: 0.5rem;
}
</style>
