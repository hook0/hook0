<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import i18n from '@/plugins/i18n';
import type { HealthEvent } from './SubscriptionHealthService';
import Hook0Badge from '@/components/Hook0Badge.vue';

defineProps<{
  events: HealthEvent[];
}>();

const { t } = useI18n();

type BadgeVariant = 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info';

const statusVariantMap: Record<string, BadgeVariant> = {
  resolved: 'success',
  warning: 'warning',
  disabled: 'danger',
};

function getRelativeTimeFormat(): Intl.RelativeTimeFormat {
  return new Intl.RelativeTimeFormat(i18n.global.locale.value, { numeric: 'auto' });
}

/** Format a date as a human-readable relative string (e.g. "3 hours ago") */
function relativeDate(iso: string): string {
  const rtf = getRelativeTimeFormat();
  const diffMs = new Date(iso).getTime() - Date.now();
  const diffMin = Math.round(diffMs / 60_000);
  if (Math.abs(diffMin) < 60) return rtf.format(diffMin, 'minute');
  const diffHr = Math.round(diffMin / 60);
  if (Math.abs(diffHr) < 24) return rtf.format(diffHr, 'hour');
  const diffDay = Math.round(diffHr / 24);
  return rtf.format(diffDay, 'day');
}
</script>

<template>
  <div class="health-timeline">
    <div
      v-for="event in events"
      :key="event.health_event_id"
      class="health-timeline__item"
    >
      <span
        class="health-timeline__dot"
        :class="`health-timeline__dot--${event.status}`"
        aria-hidden="true"
      />
      <div class="health-timeline__content">
        <Hook0Badge :variant="statusVariantMap[event.status] ?? 'default'" size="sm">
          {{ t(`subscriptionDetail.healthStatus.${event.status}`) }}
        </Hook0Badge>
        <Hook0Badge variant="default" size="sm">
          {{ t('subscriptionDetail.healthSource.' + event.source) }}
        </Hook0Badge>
        <span class="health-timeline__date">
          {{ relativeDate(event.created_at) }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.health-timeline {
  position: relative;
  padding-left: 1.25rem;
  border-left: 2px solid var(--color-border);
}

.health-timeline__item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0;
}

.health-timeline__dot {
  position: absolute;
  left: -1.5rem;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.health-timeline__dot--resolved {
  background-color: var(--color-success);
}

.health-timeline__dot--warning {
  background-color: var(--color-warning);
}

.health-timeline__dot--disabled {
  background-color: var(--color-error);
}

.health-timeline__content {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.health-timeline__date {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}
</style>
