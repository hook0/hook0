<script setup lang="ts">
import { computed } from 'vue';
import Hook0TruncatedText from './Hook0TruncatedText.vue';

type Props = {
  value: string;
  linked?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  linked: false,
});

const display = computed(() => {
  if (props.value.length <= 20) return props.value;
  return `${props.value.slice(0, 8)}\u2026${props.value.slice(-8)}`;
});
</script>

<template>
  <Hook0TruncatedText :value="value" :display="display" :linked="linked" />
</template>
