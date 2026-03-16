<script setup lang="ts">
import type { Component } from 'vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  icon: Component;
  title: string;
}

defineProps<Props>();
</script>

<template>
  <component :is="icon" :size="16" aria-hidden="true" :title="title" />
</template>
