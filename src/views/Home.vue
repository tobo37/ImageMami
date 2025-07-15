<script setup lang="ts">
import { useI18n } from "vue-i18n";
const { t } = useI18n();
</script>

<template>
  <div class="view">
    <div class="preview-banner">{{ t("home.previewBanner") }}</div>
    <div class="grid-home">
      <router-link class="card" to="/duplicate">{{
        t("duplicate.title")
      }}</router-link>
      <router-link class="card" to="/import">{{
        t("import.title")
      }}</router-link>
      <router-link class="card" to="/sort">{{ t("sort.title") }}</router-link>
      <router-link class="card" to="/blackhole">{{
        t("blackhole.title")
      }}</router-link>
      <router-link class="card" to="/training">{{
        t("training.title")
      }}</router-link>
    </div>
  </div>
</template>
