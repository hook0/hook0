<script setup lang="ts">
import { AlertCircle, RefreshCw } from 'lucide-vue-next';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import type { Problem } from '@/http';

interface Props {
  error: Problem | Error | string;
  retryable?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  retryable: true,
});

const emit = defineEmits<{
  retry: [];
}>();

function getTitle(): string {
  if (typeof props.error === 'string') return 'Error';
  if (props.error instanceof Error) return 'Error';
  return `${props.error.title} (${props.error.status})`;
}

function getDetail(): string {
  if (typeof props.error === 'string') return props.error;
  if (props.error instanceof Error) return props.error.message;
  return props.error.detail;
}
</script>

<template>
  <Hook0Card>
    <div class="hook0-error-card">
      <div class="hook0-error-card-icon">
        <AlertCircle :size="40" aria-hidden="true" />
      </div>
      <h3 class="hook0-error-card-title">{{ getTitle() }}</h3>
      <p class="hook0-error-card-detail">{{ getDetail() }}</p>
      <Hook0Button v-if="retryable" variant="secondary" size="sm" @click="emit('retry')">
        <template #left>
          <RefreshCw :size="14" aria-hidden="true" />
        </template>
        Retry
      </Hook0Button>
    </div>
  </Hook0Card>
</template>

<style scoped>
.hook0-error-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 3rem 1.5rem;
  text-align: center;
  gap: 0.75rem;
}

.hook0-error-card-icon {
  color: var(--color-danger);
  opacity: 0.6;
}

.hook0-error-card-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.hook0-error-card-detail {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  max-width: 24rem;
}
</style>
