<script setup lang="ts">
import { computed, h, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { Pencil, Send, RotateCcw, ArrowLeft } from 'lucide-vue-next';
import { toast } from 'vue-sonner';
import { handleMutationError } from '@/utils/handleMutationError';

import { useSubscriptionDetail } from './useSubscriptionQueries';
import { targetIsHttp } from './SubscriptionService';
import { useLogListBySubscription, useRetryDelivery } from '../logs/useLogQueries';
import { useLogColumns } from '../logs/useLogColumns';
import { useSubscriptionHealthEvents } from './useSubscriptionHealthQueries';
import type { RequestAttemptExtended } from '../logs/LogService';
import { routes } from '@/routes';

import LogDetailContent from '../logs/LogDetailContent.vue';
import SubscriptionHealthTimeline from './SubscriptionHealthTimeline.vue';

import Hook0SplitLayout from '@/components/Hook0SplitLayout.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

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
const { organizationId, applicationId, subscriptionId } = useRouteIds();

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
  error: healthError,
  refetch: healthRefetch,
} = useSubscriptionHealthEvents(subscriptionId, organizationId);

const retryMutation = useRetryDelivery();

// Remove the "subscription" column (redundant), add retry button column
const allLogColumns = useLogColumns();
const logColumns = computed(() => [
  ...allLogColumns.filter((col) => col.id !== 'subscription'),
  {
    id: 'retry',
    header: '',
    size: 40,
    cell: ({ row }: { row: { original: RequestAttemptExtended } }) =>
      h(
        Hook0Button,
        {
          variant: 'ghost',
          type: 'button',
          disabled: retryMutation.isPending.value,
          'aria-label': t('subscriptionDetail.retryDelivery'),
          onClick: (e: Event) => {
            e.stopPropagation();
            retryMutation.mutate(row.original.request_attempt_id, {
              onSuccess: () => toast.success(t('subscriptionDetail.retryQueued')),
              onError: (err: Error) => handleMutationError(err),
            });
          },
        },
        () => h(RotateCcw, { size: 16, 'aria-hidden': 'true' })
      ),
  },
]);

const selectedRow = ref<RequestAttemptExtended | null>(null);

function handleRowClick(row: RequestAttemptExtended) {
  selectedRow.value = row;
}

function clearSelection() {
  selectedRow.value = null;
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
  if (!target || !targetIsHttp(target)) return null;
  const url = target.url.startsWith('http') ? target.url : `https://${target.url}`;
  return { ...target, url };
});
</script>

<template>
  <Hook0PageLayout :title="t('subscriptionDetail.title')">
    <!-- Error state -->
    <Hook0ErrorCard v-if="subError && !subLoading" :error="subError" @retry="subRefetch()" />

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
            <Hook0HealthBadge :failure-percent="subscription.failure_percent ?? null" />
            <Hook0Button
              variant="ghost"
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
                {{ JSON.stringify(subscription.target, null, 2) }}
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
          >
            <template #action>
              <Hook0Button
                variant="primary"
                :to="{
                  name: routes.EventsSend,
                  params: {
                    organization_id: route.params.organization_id,
                    application_id: route.params.application_id,
                  },
                }"
              >
                {{ t('subscriptionDetail.sendFirstEvent') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
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
            <Hook0SplitLayout
              :show-detail="!!selectedRow"
              :detail-key="selectedRow?.request_attempt_id"
            >
              <template #back>
                <Hook0Button variant="ghost" size="sm" @click="clearSelection">
                  <ArrowLeft :size="16" aria-hidden="true" />
                  {{ t('logs.backToList') }}
                </Hook0Button>
              </template>
              <template #list>
                <Hook0Table
                  :columns="logColumns"
                  :data="deliveries"
                  row-id-field="request_attempt_id"
                  clickable-rows
                  :active-row-id="selectedRow?.request_attempt_id"
                  @row-click="handleRowClick"
                />
              </template>
              <template #detail>
                <LogDetailContent
                  v-if="selectedRow"
                  :attempt="selectedRow"
                  :application-id="applicationId"
                />
                <Hook0Skeleton v-else size="block" />
              </template>
            </Hook0SplitLayout>
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
        <Hook0ErrorCard
          v-else-if="healthError && !healthLoading"
          :error="healthError"
          @retry="healthRefetch()"
        />
      </template>
    </template>
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

@import '../logs/log-cells.css';

/* Split layout overrides for delivery list */
:deep(.hook0-split-layout__list table) {
  table-layout: fixed;
}

:deep(.hook0-split-layout__list .hook0-table-th:first-child),
:deep(.hook0-split-layout__list .hook0-table-td:first-child) {
  width: 12rem;
  white-space: nowrap;
  overflow: visible;
}

:deep(.hook0-split-layout__list .hook0-table-td:not(:first-child)),
:deep(.hook0-split-layout__list .hook0-table-th:not(:first-child)) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
