<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { Pencil, Send } from 'lucide-vue-next';

import { useSubscriptionDetail } from './useSubscriptionQueries';
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
              </div>
            </div>
            <div class="detail-header__actions">
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
              <Hook0Switch
                :model-value="subscription.is_enabled"
                disabled
                :aria-label="t('subscriptions.enabledColumn')"
              />
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

      <!-- Section 2: Deliveries -->
      <template v-if="!showMergedEmptyState">
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

        <template v-else-if="deliveries.length > 0">
          <Hook0Card>
            <Hook0CardHeader>
              <template #header>{{ t('subscriptionDetail.deliveries') }}</template>
              <template #subtitle>{{ t('subscriptionDetail.deliveriesSubtitle') }}</template>
            </Hook0CardHeader>
          </Hook0Card>

          <DeliverySplitView :deliveries="deliveries" :application-id="applicationId" />
        </template>

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
  gap: 0.75rem;
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

/* Reduce gap between Delivery Logs card and the split layout table */
:deep(.hook0-split-layout) {
  margin-top: 0.25rem;
}
</style>
