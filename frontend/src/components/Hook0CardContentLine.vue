<template>
  <div class="hook0-card-content-line">
    <dt class="hook0-card-content-line-label">
      <slot name="label"></slot>
    </dt>
    <dd class="hook0-card-content-line-content">
      <slot name="content"></slot>
    </dd>
  </div>
</template>

<script>
export default {
  name: 'hook0-card-content-line',
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
.hook0-card-content-line {
  @apply py-4 sm:py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6;
}

.hook0-card-content-line-label {
  @apply text-sm font-medium text-gray-500;
}
.hook0-card-content-line-content {
  @apply mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2;
}
</style>
