<template>
  <a v-bind="{ ...$props, ...$attrs }" @click="onClick($event)">
    <slot></slot>
  </a>
</template>

<script>
export default {
  name: 'hook0-link',
  components: {},
  data() {
    return {};
  },
  methods: {
    onClick(e) {
      this.$emit('click', e);
    },
  },
  mounted() {},
  computed: {},
};
</script>

<style lang="scss" scoped>
a {
  @apply font-medium text-indigo-600 hover:text-indigo-500;

  &.danger {
    @apply text-red-700 hover:text-red-600;
  }
}
</style>
