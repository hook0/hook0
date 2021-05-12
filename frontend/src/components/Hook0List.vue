<template>
  <ul class="list" v-bind="{ ...$props, ...$attrs }">
    <slot></slot>
  </ul>
</template>

<script>
export default {
  name: 'hook0-list',
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
.list {
  @apply border border-gray-200 rounded-md divide-y divide-gray-200;
}
</style>
