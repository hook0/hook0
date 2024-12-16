<script setup lang="ts">
defineSlots<{
  header(): unknown;
  subtitle(): unknown;
  actions(): unknown;
}>();
</script>

<template>
  <div class="hook0-card-header">
    <div class="hook0-card-header-container">
      <div class="hook0-card-header-left">
        <h3 class="hook0-card-header-title">
          <slot name="header"></slot>
        </h3>
        <p class="hook0-card-header-subtitle">
          <slot name="subtitle"></slot>
        </p>
      </div>
      <div class="hook0-card-header-right">
        <slot name="actions"></slot>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.hook0-card-header {
  @apply bg-white px-4 py-5 border-b border-gray-200 sm:px-6;

  .hook0-card-header-container {
    @apply -ml-4 -mt-4 flex justify-between items-center flex-wrap sm:flex-nowrap;
  }

  .hook0-card-header-left {
    @apply ml-4 mt-4 w-full;

    .hook0-card-header-title {
      @apply text-lg leading-6 font-medium text-gray-900;
    }

    .hook0-card-header-subtitle {
      @apply mt-1 max-w-2xl text-sm text-gray-500;
    }
  }

  .hook0-card-header-right {
    @apply ml-4 mt-4 flex-shrink-0 flex;

    & > * {
      @apply ml-4;
    }
  }
}
</style>
