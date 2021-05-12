<template>
  <div class="card">
    <slot></slot>
  </div>
</template>

<script>
export default {
  name: 'hook0-card',
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
.card {
  @apply bg-white shadow overflow-hidden sm:rounded-lg;
}
</style>
