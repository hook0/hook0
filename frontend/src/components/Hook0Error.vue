<script setup lang="ts">
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import type { Problem } from '@/http';

interface Props {
  error: Problem;
}

const props = defineProps<Props>();
</script>

<template>
  <Hook0Alert type="alert">
    <template #title>{{ props.error.title }} ({{ props.error.status }})</template>
    <template #description>
      <Hook0Text>{{ props.error.detail }}</Hook0Text>
      <br />
      <Hook0Text class="def">{{ props.error.id }}</Hook0Text>
    </template>
  </Hook0Alert>
</template>
