<template>
  <div class="card-content">
    <slot></slot>
  </div>
</template>

<script>
export default {
  name: 'hook0-card-content',
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
.card-content {
  @apply border-t border-gray-200 px-4 py-5 sm:p-0;
}
</style>
