<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  path: string | null;
  label: string;
  chooseText: string;
}>();
const emit = defineEmits<{ choose: [] }>();

const displayPath = computed(() => {
  if (!props.path) return "-";
  const limit = 50;
  if (props.path.length <= limit) return props.path;
  const start = props.path.slice(0, 25);
  const end = props.path.slice(props.path.length - 25);
  return `${start}...${end}`;
});
function onChoose() {
  emit("choose");
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
  justify-content: space-between;
  gap: 1rem;
}
.dest-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.dest-path {
  display: block;
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
