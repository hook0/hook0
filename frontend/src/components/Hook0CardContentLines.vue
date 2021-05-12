<template>
  <dl class="hook0-card-content-lines">
    <slot></slot>
  </dl>
</template>

<script>
export default {
  name: 'hook0-card-content-lines',
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
.hook0-card-content-lines {
  @apply sm:divide-y sm:divide-gray-200;
}
</style>
