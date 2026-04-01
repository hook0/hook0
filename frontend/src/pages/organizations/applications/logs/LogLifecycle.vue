<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { CheckCircle2, XCircle, Clock, Circle, CircleDashed } from 'lucide-vue-next';
import Hook0DateFormatted from '@/components/Hook0DateFormatted.vue';

import type { Event } from '@/pages/organizations/applications/events/EventsService';
import type { RequestAttemptExtended } from './LogService';

type Props = {
  event: Event;
  attempt: RequestAttemptExtended;
};

const props = defineProps<Props>();

const { t } = useI18n();

type LifecycleStep = {
  label: string;
  description: string;
  date: string | null | undefined;
  status: 'done' | 'active' | 'pending' | 'error' | 'next';
  icon: typeof CheckCircle2;
};

const steps = computed<LifecycleStep[]>(() => {
  const { event, attempt } = props;
  const result: LifecycleStep[] = [];

  // occurred_at and received_at only present for externally-ingested events;
  // created_at and picked_at always exist in the lifecycle
  if (event.occurred_at) {
    result.push({
      label: t('logs.lifecycle.occurred'),
      description: t('logs.lifecycle.occurredDesc'),
      date: event.occurred_at,
      status: 'done',
      icon: CheckCircle2,
    });
  }

  if (event.received_at) {
    result.push({
      label: t('logs.lifecycle.received'),
      description: t('logs.lifecycle.receivedDesc'),
      date: event.received_at,
      status: 'done',
      icon: CheckCircle2,
    });
  }

  result.push({
    label: t('logs.lifecycle.created'),
    description: t('logs.lifecycle.createdDesc'),
    date: attempt.created_at,
    status: attempt.created_at ? 'done' : 'pending',
    icon: attempt.created_at ? CheckCircle2 : Circle,
  });

  result.push({
    label: t('logs.lifecycle.picked'),
    description: t('logs.lifecycle.pickedDesc'),
    date: attempt.picked_at,
    status: attempt.picked_at ? 'done' : 'pending',
    icon: attempt.picked_at ? CheckCircle2 : Circle,
  });

  // Terminal state: failed may have scheduled retry (delay_until),
  // succeeded is final, absence of both means still in progress
  if (attempt.failed_at) {
    result.push({
      label: t('logs.lifecycle.failed'),
      description: t('logs.lifecycle.failedDesc'),
      date: attempt.failed_at,
      status: 'error',
      icon: XCircle,
    });
    if (attempt.delay_until) {
      result.push({
        label: t('logs.lifecycle.retryAt'),
        description: t('logs.lifecycle.retryAtDesc'),
        date: attempt.delay_until,
        status: 'active',
        icon: Clock,
      });
    }
  } else if (attempt.succeeded_at) {
    result.push({
      label: t('logs.lifecycle.delivered'),
      description: t('logs.lifecycle.deliveredDesc'),
      date: attempt.succeeded_at,
      status: 'done',
      icon: CheckCircle2,
    });
  } else {
    result.push({
      label: t('logs.lifecycle.delivered'),
      description: t('logs.lifecycle.deliveredDesc'),
      date: null,
      status: 'pending',
      icon: Circle,
    });
  }

  // Mark the first pending step as "next" (the step currently waiting to happen)
  const firstPending = result.find((s) => s.status === 'pending');
  if (firstPending) {
    firstPending.status = 'next';
    firstPending.icon = CircleDashed;
  }

  return result;
});
</script>

<template>
  <div class="log-lifecycle">
    <div
      v-for="(step, index) in steps"
      :key="step.label"
      class="log-lifecycle__step"
      :class="`log-lifecycle__step--${step.status}`"
    >
      <div class="log-lifecycle__indicator">
        <component
          :is="step.icon"
          :size="16"
          aria-hidden="true"
          class="log-lifecycle__icon"
          :class="`log-lifecycle__icon--${step.status}`"
        />
        <div
          v-if="index < steps.length - 1"
          class="log-lifecycle__line"
          :class="{
            'log-lifecycle__line--done':
              step.status === 'done' && steps[index + 1]?.status === 'done',
            'log-lifecycle__line--next':
              step.status === 'done' && steps[index + 1]?.status === 'next',
            'log-lifecycle__line--error': steps[index + 1]?.status === 'error',
          }"
        />
      </div>
      <div class="log-lifecycle__content">
        <div class="log-lifecycle__header">
          <span class="log-lifecycle__label">{{ step.label }}</span>
          <Hook0DateFormatted v-if="step.date" :value="step.date" class="log-lifecycle__date" />
          <span v-else class="log-lifecycle__date log-lifecycle__date--pending">—</span>
        </div>
        <span class="log-lifecycle__description">{{ step.description }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-lifecycle {
  display: flex;
  flex-direction: column;
}

.log-lifecycle__step {
  display: flex;
  gap: 0.625rem;
  min-height: 3.5rem;
}

.log-lifecycle__indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex-shrink: 0;
  width: 1rem;
}

.log-lifecycle__icon {
  flex-shrink: 0;
}

.log-lifecycle__icon--done {
  color: var(--color-primary);
}

.log-lifecycle__icon--done :deep(circle) {
  fill: var(--color-primary);
  stroke: var(--color-primary);
}

.log-lifecycle__icon--done :deep(path) {
  stroke: var(--color-on-dark);
}

.log-lifecycle__icon--active {
  color: var(--color-info);
}

.log-lifecycle__icon--next {
  color: var(--color-warning);
  animation: spin-slow 4s linear infinite;
}

@keyframes spin-slow {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.log-lifecycle__icon--pending {
  color: var(--color-text-tertiary);
}

.log-lifecycle__icon--error {
  color: var(--color-error);
}

.log-lifecycle__icon--error :deep(circle) {
  fill: var(--color-error);
  stroke: var(--color-error);
}

.log-lifecycle__icon--error :deep(path) {
  stroke: var(--color-on-dark);
}

.log-lifecycle__line {
  flex: 1;
  width: 2px;
  min-height: 0.75rem;
  background-color: var(--color-border);
}

.log-lifecycle__line--done {
  background-color: var(--color-primary);
}

.log-lifecycle__line--next {
  background-color: color-mix(in srgb, var(--color-warning) 50%, transparent);
  animation: pulse-line 2s ease-in-out infinite;
}

@keyframes pulse-line {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.4;
  }
}

.log-lifecycle__line--error {
  background-color: var(--color-error);
}

.log-lifecycle__content {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  padding-bottom: 0.5rem;
  flex: 1;
  min-width: 0;
}

.log-lifecycle__header {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  justify-content: space-between;
  margin-top: -0.125rem;
}

.log-lifecycle__label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  min-width: 4.5rem;
}

.log-lifecycle__date {
  font-size: 0.8125rem;
}

.log-lifecycle__date--pending {
  color: var(--color-text-tertiary);
}

.log-lifecycle__description {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

.log-lifecycle__step--done .log-lifecycle__label {
  color: var(--color-primary);
}

.log-lifecycle__step--next .log-lifecycle__label {
  color: var(--color-warning);
}

.log-lifecycle__step--error .log-lifecycle__label {
  color: var(--color-error);
}

.log-lifecycle__step--active .log-lifecycle__label {
  color: var(--color-info);
}

.log-lifecycle__step--pending .log-lifecycle__label,
.log-lifecycle__step--pending .log-lifecycle__description {
  color: var(--color-text-tertiary);
}
</style>
