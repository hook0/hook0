<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { CheckCircle2, XCircle, Clock, Circle } from 'lucide-vue-next';
import Hook0DateFormatted from '@/components/Hook0DateFormatted.vue';

type Props = {
  occurredAt?: string | null;
  receivedAt?: string | null;
  createdAt?: string | null;
  pickedAt?: string | null;
  succeededAt?: string | null;
  failedAt?: string | null;
  delayUntil?: string | null;
};

const props = withDefaults(defineProps<Props>(), {
  occurredAt: undefined,
  receivedAt: undefined,
  createdAt: undefined,
  pickedAt: undefined,
  succeededAt: undefined,
  failedAt: undefined,
  delayUntil: undefined,
});

const { t } = useI18n();

type LifecycleStep = {
  label: string;
  description: string;
  date: string | null | undefined;
  status: 'done' | 'active' | 'pending' | 'error';
  icon: typeof CheckCircle2;
};

const steps = computed<LifecycleStep[]>(() => {
  const result: LifecycleStep[] = [];

  if (props.occurredAt) {
    result.push({
      label: t('logs.lifecycle.occurred'),
      description: t('logs.lifecycle.occurredDesc'),
      date: props.occurredAt,
      status: 'done',
      icon: CheckCircle2,
    });
  }

  if (props.receivedAt) {
    result.push({
      label: t('logs.lifecycle.received'),
      description: t('logs.lifecycle.receivedDesc'),
      date: props.receivedAt,
      status: 'done',
      icon: CheckCircle2,
    });
  }

  result.push({
    label: t('logs.lifecycle.created'),
    description: t('logs.lifecycle.createdDesc'),
    date: props.createdAt,
    status: props.createdAt ? 'done' : 'pending',
    icon: props.createdAt ? CheckCircle2 : Circle,
  });

  result.push({
    label: t('logs.lifecycle.picked'),
    description: t('logs.lifecycle.pickedDesc'),
    date: props.pickedAt,
    status: props.pickedAt ? 'done' : 'pending',
    icon: props.pickedAt ? CheckCircle2 : Circle,
  });

  if (props.failedAt) {
    result.push({
      label: t('logs.lifecycle.failed'),
      description: t('logs.lifecycle.failedDesc'),
      date: props.failedAt,
      status: 'error',
      icon: XCircle,
    });
    if (props.delayUntil) {
      result.push({
        label: t('logs.lifecycle.retryAt'),
        description: t('logs.lifecycle.retryAtDesc'),
        date: props.delayUntil,
        status: 'active',
        icon: Clock,
      });
    }
  } else if (props.succeededAt) {
    result.push({
      label: t('logs.lifecycle.delivered'),
      description: t('logs.lifecycle.deliveredDesc'),
      date: props.succeededAt,
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

  return result;
});
</script>

<template>
  <div class="log-lifecycle">
    <div
      v-for="(step, index) in steps"
      :key="step.label + index"
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
            'log-lifecycle__line--done': step.status === 'done',
            'log-lifecycle__line--error': step.status === 'error',
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
  gap: 0;
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
  stroke: white;
}

.log-lifecycle__icon--active {
  color: var(--color-info);
}

.log-lifecycle__icon--pending {
  color: var(--color-text-tertiary);
}

.log-lifecycle__icon--error {
  color: var(--color-error);
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

.log-lifecycle__step--pending .log-lifecycle__label,
.log-lifecycle__step--pending .log-lifecycle__description {
  color: var(--color-text-tertiary);
}
</style>
