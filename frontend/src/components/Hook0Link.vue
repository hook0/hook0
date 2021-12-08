<template>
  <a v-bind="{ ...$props, ...$attrs }" @click="onClick($event)">
    <slot></slot>
  </a>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component';

@Options({
  name: 'hook0-link',
  data() {
    return {};
  },
  methods: {
    onClick(e: MouseEvent) {
      this.$emit('click', e);
    },
  },
})
export default class Hook0Link extends Vue {};
</script>

<style lang="scss" scoped>
a {
  @apply font-medium text-indigo-600 hover:text-indigo-500;

  &.danger {
    @apply text-red-700 hover:text-red-600;
  }
}
</style>
