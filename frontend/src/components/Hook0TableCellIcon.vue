<script setup lang="ts">
import Hook0Icon from '@/components/Hook0Icon.vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  icon: string;
  title: string;
}

defineProps<Props>();
</script>

<template>
  <Hook0Icon :title="title" :name="icon" />
</template>
