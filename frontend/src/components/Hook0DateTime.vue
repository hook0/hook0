<script setup lang="ts">
import Hook0Text from './Hook0Text.vue';
import { formatISO9075, parseISO } from 'date-fns';
import { computed } from 'vue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  value: string;
}
const props = defineProps<Props>();
const valueHumanized = computed(() => formatISO9075(parseISO(props.value)));
</script>

<template>
  <Hook0Text :title="value">{{ valueHumanized }}</Hook0Text>
</template>

<style lang="scss" scoped></style>
