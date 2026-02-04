<script setup lang="ts">
import { computed } from 'vue';
import { AlertCircle, AlertTriangle, CheckCircle, Info } from 'lucide-vue-next';

type AlertType = 'alert' | 'warning' | 'success' | 'info';

interface Props {
  title?: string;
  description?: string;
  type?: AlertType;
}

const props = withDefaults(defineProps<Props>(), {
  title: undefined,
  description: undefined,
  type: 'alert',
});

const alertIcon = computed(() => {
  const map = {
    alert: AlertCircle,
    warning: AlertTriangle,
    success: CheckCircle,
    info: Info,
  };
  return map[props.type];
});
</script>

<template>
  <div class="hook0-alert" :class="type" role="alert">
    <div class="hook0-alert-inner">
      <div class="hook0-alert-icon">
        <component :is="alertIcon" :size="20" aria-hidden="true" />
      </div>
      <div class="hook0-alert-content">
        <h3 v-if="title" class="hook0-alert-title">
          {{ title }}
          <slot name="title" />
        </h3>
        <div v-if="description" class="hook0-alert-description">
          <p>
            {{ description }}
            <slot name="description" />
          </p>
        </div>
        <div v-else class="hook0-alert-description">
          <slot name="description" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hook0-alert {
  padding: 1rem;
  border-radius: var(--radius-md);
  border: 1px solid;
}

.hook0-alert-inner {
  display: flex;
  gap: 0.75rem;
}

.hook0-alert-icon {
  flex-shrink: 0;
  margin-top: 0.125rem;
}

.hook0-alert-title {
  font-size: 0.875rem;
  font-weight: 600;
  margin-bottom: 0.25rem;
}

.hook0-alert-description {
  font-size: 0.875rem;
}

/* Alert (error) */
.hook0-alert.alert {
  background-color: color-mix(in srgb, var(--color-danger) 8%, var(--color-bg-primary));
  border-color: color-mix(in srgb, var(--color-danger) 25%, transparent);
  color: var(--color-danger);
}

/* Warning */
.hook0-alert.warning {
  background-color: color-mix(in srgb, var(--color-warning) 8%, var(--color-bg-primary));
  border-color: color-mix(in srgb, var(--color-warning) 25%, transparent);
  color: var(--color-warning);
}

/* Success */
.hook0-alert.success {
  background-color: color-mix(in srgb, var(--color-success) 8%, var(--color-bg-primary));
  border-color: color-mix(in srgb, var(--color-success) 25%, transparent);
  color: var(--color-success);
}

/* Info */
.hook0-alert.info {
  background-color: color-mix(in srgb, var(--color-info) 8%, var(--color-bg-primary));
  border-color: color-mix(in srgb, var(--color-info) 25%, transparent);
  color: var(--color-info);
}
</style>
