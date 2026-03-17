<script setup lang="ts">
import { computed } from 'vue';

type LayoutVariant = 'default' | 'fullscreen';

type Props = {
  title?: string;
  description?: string;
  variant?: LayoutVariant;
}

const props = withDefaults(defineProps<Props>(), {
  title: undefined,
  description: undefined,
  variant: 'default',
});

defineSlots<{
  default(): unknown;
  actions(): unknown;
  logo(): unknown;
  footer(): unknown;
  background(): unknown;
  icon(): unknown;
}>();

const isFullscreen = computed(() => props.variant === 'fullscreen');
</script>

<template>
  <!-- Fullscreen variant (for auth pages) -->
  <div v-if="isFullscreen" class="hook0-page-fullscreen">
    <!-- Custom background or default effects -->
    <div v-if="$slots.background" class="hook0-page-fullscreen__background">
      <slot name="background" />
    </div>
    <div v-else class="hook0-page-fullscreen__background">
      <div class="hook0-page-fullscreen__grid-pattern" />
      <div class="hook0-page-fullscreen__blur-circle hook0-page-fullscreen__blur-circle--primary" />
      <div class="hook0-page-fullscreen__blur-circle hook0-page-fullscreen__blur-circle--accent" />
    </div>

    <!-- Main content -->
    <div class="hook0-page-fullscreen__content">
      <!-- Logo slot -->
      <div v-if="$slots.logo" class="hook0-page-fullscreen__logo">
        <slot name="logo" />
      </div>

      <!-- Main content slot -->
      <slot />

      <!-- Footer slot (for trust indicators) -->
      <div v-if="$slots.footer" class="hook0-page-fullscreen__footer">
        <slot name="footer" />
      </div>
    </div>
  </div>

  <!-- Default variant -->
  <div v-else class="hook0-page-layout">
    <div class="hook0-page-header">
      <div class="hook0-page-header-left">
        <div v-if="title || $slots.icon" class="hook0-page-title-row">
          <slot name="icon" />
          <h1 v-if="title" class="hook0-page-title">{{ title }}</h1>
        </div>
        <p v-if="description" class="hook0-page-description">{{ description }}</p>
      </div>
      <div v-if="$slots.actions" class="hook0-page-header-actions">
        <slot name="actions" />
      </div>
    </div>

    <div class="hook0-page-content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
/* Default layout styles */
.hook0-page-layout {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.hook0-page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 1rem;
  flex-wrap: wrap;
}

.hook0-page-title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.hook0-page-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: -0.025em;
}

.hook0-page-description {
  margin-top: 0.25rem;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.hook0-page-header-actions {
  display: flex;
  gap: 0.75rem;
  flex-shrink: 0;
}

.hook0-page-content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

/* Fullscreen layout styles */
.hook0-page-fullscreen {
  min-height: 100vh;
  width: 100%;
  position: relative;
  overflow: hidden;
  background: linear-gradient(180deg, var(--color-bg-primary) 0%, var(--color-bg-secondary) 100%);
  font-family: var(--font-sans);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.hook0-page-fullscreen__background {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.hook0-page-fullscreen__grid-pattern {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(99, 102, 241, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(99, 102, 241, 0.03) 1px, transparent 1px);
  background-size: 60px 60px;
  opacity: 0.5;
}

.hook0-page-fullscreen__blur-circle {
  position: absolute;
  border-radius: 9999px;
  pointer-events: none;
  filter: blur(100px);
}

.hook0-page-fullscreen__blur-circle--primary {
  background-color: var(--color-primary);
  width: 400px;
  height: 400px;
  top: -200px;
  left: -200px;
  opacity: 0.1;
}

.hook0-page-fullscreen__blur-circle--accent {
  background-color: var(--color-success);
  width: 350px;
  height: 350px;
  bottom: -150px;
  right: -150px;
  opacity: 0.08;
}

.hook0-page-fullscreen__content {
  position: relative;
  z-index: 10;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem 1rem;
}

.hook0-page-fullscreen__logo {
  margin-bottom: 2rem;
}

.hook0-page-fullscreen__footer {
  margin-top: 2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2rem;
  width: 100%;
  max-width: 36rem;
}

/* Constraint for main content in fullscreen mode */
.hook0-page-fullscreen__content
  > :deep(:not(.hook0-page-fullscreen__logo):not(.hook0-page-fullscreen__footer)) {
  width: 100%;
  max-width: 28rem;
}

@media (max-width: 640px) {
  .hook0-page-fullscreen__content {
    padding: 2rem 1rem;
  }
}
</style>
