<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { Pencil, Send } from 'lucide-vue-next';
import { toast } from 'vue-sonner';
import { handleMutationError } from '@/utils/handleMutationError';

import { useQueryClient } from '@tanstack/vue-query';
import { healthEventKeys } from '@/queries/keys';
import { useSubscriptionDetail, useToggleSubscription } from './useSubscriptionQueries';
import { targetIsHttp } from './SubscriptionService';
import { useLogListBySubscription } from '../logs/useLogQueries';
import { RequestAttemptStatusType } from '../logs/LogService';
import { useSubscriptionHealthEvents } from './useSubscriptionHealthQueries';
import { parseCursorFromQuery } from '@/utils/pagination';
import { useHealthThresholds } from '@/composables/useHealthThresholds';
import { routes } from '@/routes';

import DeliverySplitView from '../logs/DeliverySplitView.vue';
import SubscriptionHealthTimeline from './SubscriptionHealthTimeline.vue';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Switch from '@/components/Hook0Switch.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
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
  isLoading: subscriptionLoading,
  error: subscriptionError,
  refetch: subscriptionRefetch,
} = useSubscriptionDetail(subscriptionId);

const {
  data: deliveries,
  isLoading: deliveriesLoading,
  error: deliveriesError,
  refetch: deliveriesRefetch,
} = useLogListBySubscription(applicationId, subscriptionId);

// Health timeline pagination — cursor is opaque; direction is baked in by the server.
const healthCursor = computed(() => parseCursorFromQuery(route.query.health_cursor));

const {
  data: healthPage,
  isLoading: healthLoading,
  error: healthError,
  refetch: healthRefetch,
} = useSubscriptionHealthEvents(subscriptionId, organizationId, healthCursor);

function navigateHealth(cursor: string) {
  void router.replace({
    query: { ...route.query, health_cursor: cursor },
  });
}

// True when both deliveries AND health events are loaded but both empty
const showMergedEmptyState = computed(() => {
  if (deliveriesLoading.value || healthLoading.value) return false;
  const noDeliveries = !deliveries.value || deliveries.value.length === 0;
  const noHealth = !healthPage.value || healthPage.value.data.length === 0;
  return noDeliveries && noHealth;
});

const httpTarget = computed(() => {
  const target = subscription.value?.target;
  if (!target || !targetIsHttp(target)) return null;
  return target;
});

// Failure rate color based on health thresholds
const { warning: warningThreshold, critical: criticalThreshold } = useHealthThresholds();
const failureRateClass = computed(() => {
  const pct = subscription.value?.failure_percent;
  if (pct == null) return '';
  if (pct >= criticalThreshold.value) return 'failure-rate--critical';
  if (pct >= warningThreshold.value) return 'failure-rate--warning';
  return 'failure-rate--ok';
});

// --- Enable/disable toggle with confirmation for disable ---
const queryClient = useQueryClient();
const toggleMutation = useToggleSubscription();
const showDisableDialog = ref(false);

// Only warn when disabling if there are deliveries in flight.
// A delivery is "in flight" when it hasn't reached a terminal status yet.
const hasPendingDeliveries = computed(() => {
  if (!deliveries.value) return false;
  return deliveries.value.some(
    (d) =>
      d.status.type !== RequestAttemptStatusType.Successful &&
      d.status.type !== RequestAttemptStatusType.Failed
  );
});

function toggleSubscription() {
  const sub = subscription.value;
  if (!sub) return;
  if (sub.is_enabled && hasPendingDeliveries.value) {
    showDisableDialog.value = true;
    return;
  }
  doToggle();
}

function doToggle() {
  const sub = subscription.value;
  if (!sub) return;
  const name = sub.description || t('subscriptions.title');
  const msgKey = sub.is_enabled ? 'subscriptions.disabled' : 'subscriptions.enabled';
  toggleMutation.mutate(
    { subscriptionId: sub.subscription_id, subscription: sub },
    {
      onSuccess: () => {
        toast.success(t('common.success'), {
          description: t(msgKey, { name }),
        });
        // Refetch health timeline — the API inserts a health event on toggle
        void queryClient.invalidateQueries({ queryKey: healthEventKeys.all });
      },
      onError: (err) => handleMutationError(err),
    }
  );
}

function confirmDisable() {
  showDisableDialog.value = false;
  doToggle();
}
</script>

<template>
  <Hook0PageLayout :title="t('subscriptionDetail.title')">
    <!-- Error state -->
    <Hook0ErrorCard
      v-if="subscriptionError && !subscriptionLoading"
      :error="subscriptionError"
      @retry="subscriptionRefetch()"
    />

    <!-- Loading skeleton -->
    <template v-else-if="subscriptionLoading || !subscription">
      <Hook0CardSkeleton />
      <Hook0CardSkeleton />
    </template>

    <!-- Content -->
    <template v-else>
      <!-- Section 1: Header card — name, health, stats, edit -->
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>
            <span class="sub-card__name">{{
              subscription.description || t('subscriptions.noDescription')
            }}</span>
            <Hook0HealthBadge :failure-percent="subscription.failure_percent ?? null" />
          </template>
          <template #actions>
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
          <div class="sub-stats">
            <div class="sub-stats__item">
              <span class="sub-stats__label">{{ t('subscriptions.enabledColumn') }}</span>
              <Hook0Switch
                :model-value="subscription.is_enabled"
                :aria-label="t('subscriptions.enabledColumn')"
                @update:model-value="toggleSubscription()"
              />
            </div>
            <div class="sub-stats__item sub-stats__item--bordered">
              <span class="sub-stats__label">{{ t('subscriptionDetail.targetUrl') }}</span>
              <span class="sub-stats__value">
                <Hook0TableCellTarget
                  v-if="httpTarget"
                  :method="httpTarget.method"
                  :url="httpTarget.url"
                />
                <pre
                  v-else
                  class="detail-header__raw-target"
                ><code>{{ JSON.stringify(subscription.target, null, 2) }}</code></pre>
              </span>
            </div>
            <div
              v-if="subscription.failure_percent != null"
              class="sub-stats__item sub-stats__item--bordered"
            >
              <span class="sub-stats__label">{{ t('subscriptionDetail.failureRate') }}</span>
              <span class="sub-stats__value sub-stats__value--rate" :class="failureRateClass">
                {{ Math.round(subscription.failure_percent) }}%
              </span>
            </div>
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

      <template v-if="!showMergedEmptyState">
        <!-- Health timeline -->
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>{{ t('subscriptionDetail.healthTimeline') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <SubscriptionHealthTimeline
              :page="healthPage"
              :error="healthError"
              :refetch="healthRefetch"
              @navigate="navigateHealth"
            />
          </Hook0CardContent>
        </Hook0Card>

        <!-- Deliveries -->
        <Hook0ErrorCard
          v-if="deliveriesError && !deliveriesLoading"
          :error="deliveriesError"
          @retry="deliveriesRefetch()"
        />

        <Hook0Card v-else-if="deliveriesLoading || !deliveries">
          <Hook0CardHeader>
            <template #header>{{ t('subscriptionDetail.deliveries') }}</template>
            <template #subtitle>{{ t('subscriptionDetail.deliveriesSubtitle') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0SkeletonGroup :count="3" />
          </Hook0CardContent>
        </Hook0Card>

        <div v-else-if="deliveries.length > 0" class="deliveries-section">
          <Hook0Card>
            <Hook0CardHeader>
              <template #header>{{ t('subscriptionDetail.deliveries') }}</template>
              <template #subtitle>{{ t('subscriptionDetail.deliveriesSubtitle') }}</template>
            </Hook0CardHeader>
          </Hook0Card>

          <DeliverySplitView :deliveries="deliveries" :application-id="applicationId" />
        </div>
      </template>
    </template>
    <Hook0Dialog
      :open="showDisableDialog"
      variant="danger"
      :title="t('subscriptions.disableTitle')"
      :confirm-text="t('subscriptions.disable')"
      @close="showDisableDialog = false"
      @confirm="confirmDisable()"
    >
      <i18n-t keypath="subscriptions.disableConfirm" tag="p">
        <template #name>
          &ldquo;<strong>{{ subscription?.description || t('subscriptions.title') }}</strong
          >&rdquo;
        </template>
      </i18n-t>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped>
/* Header card: subscription name in Hook0CardHeader */
.sub-card__name {
  font-weight: 600;
  color: var(--color-text-primary);
}

/* Stats row with divider-separated items */
.sub-stats {
  display: flex;
  align-items: flex-start;
  gap: 0;
}

.sub-stats__item {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  padding-right: 1.5rem;
}

.sub-stats__item--bordered {
  padding-left: 1.5rem;
  border-left: 1px solid var(--color-border);
}

.sub-stats__label {
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--color-text-tertiary);
}

.sub-stats__value {
  font-size: 0.875rem;
  color: var(--color-text-primary);
}

.sub-stats__value--rate {
  font-size: 1.125rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.sub-stats__value--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

/* Failure rate color classes */
.failure-rate--ok {
  color: var(--color-success);
}

.failure-rate--warning {
  color: var(--color-warning);
}

.failure-rate--critical {
  color: var(--color-error);
}

/* Deliveries section */
.deliveries-section {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.deliveries-section :deep(.hook0-split-layout) {
  margin-top: 0;
}

.detail-header__raw-target {
  margin: 0;
  overflow-x: auto;
  font-size: 0.75rem;
  font-family: var(--font-mono);
}
</style>
