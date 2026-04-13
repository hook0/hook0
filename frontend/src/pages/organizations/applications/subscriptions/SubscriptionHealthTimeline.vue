<script setup lang="ts">
import { computed, ref, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Tooltip from '@/components/Hook0Tooltip.vue';
import { formatDate } from '@/utils/formatDate';
import { useHealthThresholds } from '@/composables/useHealthThresholds';
import { useSubscriptionHealthEvents } from './useSubscriptionHealthQueries';

const props = defineProps<{
  subscriptionId: string;
  organizationId: string;
}>();

const { t, locale } = useI18n();
const { warning, critical } = useHealthThresholds();

type BadgeVariant = 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info';

const statusVariantMap: Record<string, BadgeVariant> = {
  resolved: 'success',
  warning: 'warning',
  disabled: 'danger',
};

// Cached per locale change — avoids recreating Intl.RelativeTimeFormat on every timeline event
const rtf = computed(() => new Intl.RelativeTimeFormat(locale.value, { numeric: 'auto' }));

/** Format a date as a human-readable relative string (e.g. "3 hours ago") */
function relativeDate(iso: string): string {
  const diffMs = new Date(iso).getTime() - Date.now();
  const diffMin = Math.round(diffMs / 60_000);
  if (Math.abs(diffMin) < 60) return rtf.value.format(diffMin, 'minute');
  const diffHr = Math.round(diffMin / 60);
  if (Math.abs(diffHr) < 24) return rtf.value.format(diffHr, 'hour');
  const diffDay = Math.round(diffHr / 24);
  return rtf.value.format(diffDay, 'day');
}

// Local cursor stack — the backend only gives us `next_cursor`, so we remember
// where each page started to implement a "Previous" button. `null` = first page.
const cursorStack = ref<(string | null)[]>([null]);
const currentCursor = computed(() => cursorStack.value[cursorStack.value.length - 1] ?? null);

const subscriptionIdRef = toRef(props, 'subscriptionId');
const organizationIdRef = toRef(props, 'organizationId');

const { data, isLoading } = useSubscriptionHealthEvents(
  subscriptionIdRef,
  organizationIdRef,
  currentCursor
);

const hasPrevious = computed(() => cursorStack.value.length > 1);
const hasNext = computed(() => data.value?.next_cursor != null);

function goPrevious(): void {
  if (hasPrevious.value) {
    cursorStack.value.pop();
  }
}

function goNext(): void {
  const next = data.value?.next_cursor;
  if (next != null) {
    cursorStack.value.push(next);
  }
}
</script>

<template>
  <div class="health-timeline-wrapper">
    <ol class="health-timeline">
      <li
        v-for="event in data?.items ?? []"
        :key="event.health_event_id"
        class="health-timeline__item"
      >
        <span
          class="health-timeline__dot"
          :class="`health-timeline__dot--${event.status}`"
          aria-hidden="true"
        />
        <div class="health-timeline__content">
          <Hook0Tooltip
            :content="
              t(`subscriptionDetail.healthStatus.${event.status}Tooltip`, {
                threshold: event.status === 'disabled' ? critical : warning,
              })
            "
            position="top"
          >
            <Hook0Badge :variant="statusVariantMap[event.status] ?? 'default'" size="sm">
              {{ t(`subscriptionDetail.healthStatus.${event.status}`) }}
            </Hook0Badge>
          </Hook0Tooltip>
          <Hook0Badge variant="default" size="sm">
            {{ t(`subscriptionDetail.healthCause.${event.cause}`) }}
          </Hook0Badge>
          <Hook0Tooltip :content="formatDate(event.created_at)" position="top">
            <span class="health-timeline__date">
              {{ relativeDate(event.created_at) }}
            </span>
          </Hook0Tooltip>
        </div>
      </li>
    </ol>
    <div class="health-timeline__pagination">
      <Hook0Button
        variant="secondary"
        size="sm"
        :disabled="!hasPrevious || isLoading"
        @click="goPrevious"
      >
        {{ t('common.previous') }}
      </Hook0Button>
      <Hook0Button variant="secondary" size="sm" :disabled="!hasNext || isLoading" @click="goNext">
        {{ t('common.next') }}
      </Hook0Button>
    </div>
  </div>
</template>

<style scoped>
.health-timeline-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.health-timeline {
  position: relative;
  padding-left: 1.25rem;
  border-left: 2px solid var(--color-border);
  list-style: none;
  margin: 0;
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

.health-timeline__pagination {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
