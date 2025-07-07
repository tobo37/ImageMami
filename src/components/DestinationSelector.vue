<script setup lang="ts">
defineProps<{
  path: string | null
  label: string
  chooseText: string
}>()
const emit = defineEmits<{ choose: [] }>()
function onChoose () { emit('choose') }
</script>

<template>
  <div class="card dest-card">
    <div class="dest-info">
      <strong>{{ label }}</strong>
      <span class="truncate">{{ path || '-' }}</span>
    </div>
    <button class="btn" @click="onChoose">{{ chooseText }}</button>
  </div>
</template>

<style scoped>
.dest-card { justify-content: space-between; gap: 1rem; }
.dest-info { display: flex; flex-direction: column; gap: .25rem; }
.truncate { max-width: 320px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
