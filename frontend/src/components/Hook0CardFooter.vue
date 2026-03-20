<script setup lang="ts">
defineSlots<{
  default(): unknown;
}>();
</script>

<template>
  <div class="hook0-card-footer">
    <slot />
  </div>
</template>

<style scoped>
.hook0-card-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  background-color: var(--color-bg-secondary);
  border-top: 1px solid var(--color-border);
}

@media (max-width: 639px) {
  .hook0-card-footer {
    flex-direction: column-reverse;
  }

  .hook0-card-footer :deep(.hook0-button) {
    width: 100%;
    justify-content: center;
  }
}
</style>
