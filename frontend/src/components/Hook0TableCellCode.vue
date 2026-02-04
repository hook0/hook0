<script setup lang="ts">
import Hook0Text from '@/components/Hook0Text.vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  value: string;
}

defineProps<Props>();
</script>

<template>
  <Hook0Text class="code" style="width: fit-content">{{ value }}</Hook0Text>
</template>
