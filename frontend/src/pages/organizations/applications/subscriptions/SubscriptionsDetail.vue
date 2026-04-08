<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { Pencil, Send } from 'lucide-vue-next';
import { toast } from 'vue-sonner';
import { handleMutationError } from '@/utils/handleMutationError';

import { useSubscriptionDetail, useToggleSubscription } from './useSubscriptionQueries';
import { targetIsHttp } from './SubscriptionService';
import { useLogListBySubscription } from '../logs/useLogQueries';
import { useSubscriptionHealthEvents } from './useSubscriptionHealthQueries';
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
  return target;
});

// --- Enable/disable toggle with confirmation for disable ---
const toggleMutation = useToggleSubscription();
const showDisableDialog = ref(false);

// Only warn when disabling if there are pending/waiting deliveries that would be affected
const hasPendingDeliveries = computed(() => {
  if (!deliveries.value) return false;
  return deliveries.value.some(
    (d) => d.status.type === 'waiting' || d.status.type === 'pending' || d.status.type === 'inprogress'
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
        <Hook0CardContent>
          <div class="detail-header">
            <div class="detail-header__info">
              <div class="detail-header__name-row">
                <span class="detail-header__name">
                  {{ subscription.description || t('subscriptions.noDescription') }}
                </span>
                <Hook0HealthBadge :failure-percent="subscription.failure_percent ?? null" />
              </div>
              <div class="detail-header__meta">
                <div class="detail-header__meta-item">
                  <span class="detail-header__meta-label">{{ t('subscriptions.enabledColumn') }}</span>
                  <Hook0Switch
                    :model-value="subscription.is_enabled"
                    :aria-label="t('subscriptions.enabledColumn')"
                    @update:model-value="toggleSubscription()"
                  />
                </div>
                <div class="detail-header__meta-item">
                  <span class="detail-header__meta-label">{{ t('subscriptionDetail.targetUrl') }}</span>
                  <span class="detail-header__meta-value">
                    <Hook0TableCellTarget
                      v-if="httpTarget"
                      :method="httpTarget.method"
                      :url="httpTarget.url"
                    />
                    <code v-else>{{ JSON.stringify(subscription.target) }}</code>
                  </span>
                </div>
                <div v-if="subscription.retry_schedule_id" class="detail-header__meta-item">
                  <span class="detail-header__meta-label">{{ t('subscriptions.retryScheduleLabel') }}</span>
                  <span class="detail-header__meta-value detail-header__meta-value--mono">
                    {{ subscription.retry_schedule_id }}
                  </span>
                </div>
                <div v-if="subscription.failure_percent != null" class="detail-header__meta-item">
                  <span class="detail-header__meta-label">{{ t('subscriptionDetail.failureRate') }}</span>
                  <span class="detail-header__meta-value">
                    {{ Math.round(subscription.failure_percent) }}%
                  </span>
                </div>
              </div>
            </div>
            <div class="detail-header__actions">
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
.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1.5rem;
}

.detail-header__info {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  flex: 1;
  min-width: 0;
}

.detail-header__name-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

/* Health badge: bolder text, use the primary green from the design system */
:deep(.detail-header__name-row .hook0-badge) {
  font-weight: 700;
}

.detail-header__name {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.detail-header__meta {
  display: flex;
  gap: 1.5rem;
}

.detail-header__meta-item {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.detail-header__meta-label {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--color-text-tertiary);
}

.detail-header__meta-value {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.detail-header__meta-value--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.detail-header__actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.75rem;
  flex-shrink: 0;
}

.deliveries-section {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.deliveries-section :deep(.hook0-split-layout) {
  margin-top: 0;
}
</style>
