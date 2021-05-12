<template>
  <span v-bind="{ ...$props, ...$attrs }" class="text">
    <slot></slot>
  </span>
</template>

<script>
export default {
  name: 'hook0-loader',
  components: {},
  data() {
    return {};
  },
  methods: {},
  mounted() {},
  computed: {},
};
</script>

<style lang="scss" scoped>
.text {
  @apply font-normal;

  &.medium {
    @apply font-medium;
  }
}
</style>
