<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { Pencil, Send } from 'lucide-vue-next';

import { useSubscriptionDetail } from './useSubscriptionQueries';
import { useLogListBySubscription } from '../logs/useLogQueries';
import { useLogColumns } from '../logs/useLogColumns';
import { useSubscriptionHealthEvents } from './useSubscriptionHealthQueries';
import type { RequestAttemptExtended } from '../logs/LogService';
import { routes } from '@/routes';

import EventSidePanel from '../events/EventSidePanel.vue';
import SubscriptionHealthTimeline from './SubscriptionHealthTimeline.vue';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Switch from '@/components/Hook0Switch.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0HealthBadge from '@/components/Hook0HealthBadge.vue';
import Hook0TableCellTarget from '@/components/Hook0TableCellTarget.vue';
import Hook0CardSkeleton from '@/components/Hook0CardSkeleton.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { organizationId, applicationId } = useRouteIds();
const subscriptionId = computed(() => route.params.subscription_id as string);

// --- Data queries ---
const {
  data: subscription,
  isLoading: subLoading,
  error: subError,
  refetch: subRefetch,
} = useSubscriptionDetail(subscriptionId);

const {
  data: deliveries,
  isLoading: deliveriesLoading,
  error: deliveriesError,
  refetch: deliveriesRefetch,
} = useLogListBySubscription(applicationId, subscriptionId);

const {
  data: healthEvents,
  isLoading: healthLoading,
} = useSubscriptionHealthEvents(subscriptionId, organizationId);

// Remove the "subscription" column — redundant on a page scoped to a single subscription
const allLogColumns = useLogColumns();
const logColumns = computed(() =>
  allLogColumns.filter((col) => col.id !== 'subscription')
);

// Side panel for delivery detail
const sidePanelOpen = ref(false);
const selectedEventId = ref('');

function handleRowClick(row: RequestAttemptExtended) {
  selectedEventId.value = row.event_id;
  sidePanelOpen.value = true;
}

// True when both deliveries AND health events are loaded but both empty — show a single merged empty state
const showMergedEmptyState = computed(() => {
  if (deliveriesLoading.value || healthLoading.value) return false;
  const noDeliveries = !deliveries.value || deliveries.value.length === 0;
  const noHealth = !healthEvents.value || healthEvents.value.length === 0;
  return noDeliveries && noHealth;
});

const httpTarget = computed(() => {
  const target = subscription.value?.target;
  if (!target || !('type' in target) || target.type !== 'http') return null;
  return target as unknown as { type: string; method: string; url: string };
});
</script>

<template>
  <Hook0PageLayout :title="t('subscriptionDetail.title')">
    <!-- Error state -->
    <Hook0ErrorCard
      v-if="subError && !subLoading"
      :error="subError"
      @retry="subRefetch()"
    />

    <!-- Loading skeleton -->
    <template v-else-if="subLoading || !subscription">
      <Hook0CardSkeleton />
      <Hook0CardSkeleton />
    </template>

    <!-- Content -->
    <template v-else>
      <!-- Section 1: Header card — name, target, enabled, health, edit link -->
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>
            {{ subscription.description || t('subscriptions.noDescription') }}
          </template>
          <template #actions>
            <Hook0HealthBadge
              :failure-percent="(subscription as Record<string, unknown>).failure_percent as number | null ?? null"
            />
            <Hook0Button
              variant="secondary"
              type="button"
              @click="
                void router.push({
                  name: routes.SubscriptionsEdit,
                  params: {
                    organization_id: route.params.organization_id,
                    application_id: route.params.application_id,
                    subscription_id: route.params.subscription_id,
                  },
                })
              "
            >
              <Pencil :size="14" aria-hidden="true" />
              {{ t('subscriptionDetail.edit') }}
            </Hook0Button>
          </template>
        </Hook0CardHeader>

        <Hook0CardContent>
          <div class="detail-header__row">
            <div class="detail-header__target">
              <Hook0TableCellTarget
                v-if="httpTarget"
                :method="httpTarget.method"
                :url="httpTarget.url"
              />
              <code v-else class="detail-header__target-raw">
                {{ JSON.stringify(subscription.target) }}
              </code>
            </div>
            <Hook0Switch
              :model-value="subscription.is_enabled"
              disabled
              :aria-label="t('subscriptions.enabledColumn')"
            />
          </div>
        </Hook0CardContent>
      </Hook0Card>

      <!-- Merged empty state when no activity at all -->
      <Hook0Card v-if="showMergedEmptyState">
        <Hook0CardContent>
          <Hook0EmptyState
            :title="t('subscriptionDetail.noActivityTitle')"
            :description="t('subscriptionDetail.noActivityDescription')"
            :icon="Send"
          />
        </Hook0CardContent>
      </Hook0Card>

      <!-- Section 2: Deliveries card -->
      <template v-if="!showMergedEmptyState">
        <Hook0ErrorCard
          v-if="deliveriesError && !deliveriesLoading"
          :error="deliveriesError"
          @retry="deliveriesRefetch()"
        />

        <Hook0Card v-else-if="deliveriesLoading || !deliveries">
          <Hook0CardHeader>
            <template #header>{{ t('subscriptionDetail.deliveries') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0SkeletonGroup :count="3" />
          </Hook0CardContent>
        </Hook0Card>

        <Hook0Card v-else-if="deliveries.length > 0">
          <Hook0CardHeader>
            <template #header>{{ t('subscriptionDetail.deliveries') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0Table
              :columns="logColumns"
              :data="deliveries"
              row-id-field="event_id"
              clickable-rows
              @row-click="handleRowClick"
            />
          </Hook0CardContent>
        </Hook0Card>

        <!-- Section 3: Health timeline card -->
        <Hook0Card v-if="healthEvents && healthEvents.length > 0">
          <Hook0CardHeader>
            <template #header>{{ t('subscriptionDetail.healthTimeline') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <SubscriptionHealthTimeline :events="healthEvents" />
          </Hook0CardContent>
        </Hook0Card>
      </template>
    </template>

    <!-- Side panel for delivery event detail -->
    <EventSidePanel
      :open="sidePanelOpen"
      :event-id="selectedEventId"
      :application-id="applicationId"
      @close="sidePanelOpen = false"
    />
  </Hook0PageLayout>
</template>

<style scoped>
.detail-header__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.detail-header__target {
  flex: 1;
  min-width: 0;
}

.detail-header__target-raw {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  word-break: break-all;
}

/* Log column styles from LogList — needed because h() renders outside this SFC's scope */
:deep(.log-status) {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.1875rem 0.625rem;
  border-radius: var(--radius-full);
  font-size: 0.8125rem;
  font-weight: 600;
  white-space: nowrap;
  cursor: default;
}

:deep(.log-status__dot) {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: currentColor;
  flex-shrink: 0;
}

:deep(.log-status--success) {
  background-color: var(--color-success-light);
  color: var(--color-success);
}

:deep(.log-status--error) {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

:deep(.log-status--warning) {
  background-color: var(--color-warning-light);
  color: var(--color-warning);
}

:deep(.log-status--info) {
  background-color: var(--color-info-light);
  color: var(--color-info);
}

:deep(.log-status--muted) {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-muted);
}

:deep(.log-event-cell) {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

:deep(.log-event-link) {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  text-decoration: none;
}

:deep(.log-event-link:hover) {
  text-decoration: underline;
  text-underline-offset: 2px;
}

:deep(.log-event-type) {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

:deep(.log-duration) {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  cursor: default;
}
</style>
