<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Pencil } from 'lucide-vue-next';

import { useRouteIds } from '@/composables/useRouteIds';
import { useSubscriptionDetail } from './useSubscriptionQueries';
import { useRetryScheduleDetail } from '../../retry_schedules/useRetryScheduleQueries';
import { useLogList } from '../logs/useLogQueries';
import { routes } from '@/routes';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0HealthBadge from '@/components/Hook0HealthBadge.vue';
import Hook0DateTime from '@/components/Hook0DateTime.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';

const { t } = useI18n();
const { organizationId, applicationId, subscriptionId } = useRouteIds();

const { data: subscription, isLoading, error, refetch } = useSubscriptionDetail(subscriptionId);

const retryScheduleId = computed(() => subscription.value?.retry_schedule_id ?? '');
const { data: retrySchedule } = useRetryScheduleDetail(retryScheduleId, organizationId);

const { data: logs } = useLogList(applicationId);

// TODO: add subscription_id filter to request_attempts.list backend to avoid fetching all app logs
const recentDeliveries = computed(() =>
  (logs.value ?? [])
    .filter((l) => l.subscription.subscription_id === subscriptionId.value)
    .slice(0, 10)
);
</script>

<template>
  <Hook0PageLayout :title="t('subscriptionDetail.title')">
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <template v-else-if="isLoading || !subscription">
      <Hook0Card data-test="subscription-detail-card">
        <Hook0SkeletonGroup :count="5" />
      </Hook0Card>
    </template>

    <template v-else-if="subscription">
      <!-- Header -->
      <Hook0Card data-test="subscription-detail-card">
        <Hook0CardHeader>
          <template #header>
            {{ subscription.description || subscriptionId }}
            <Hook0Badge
              :variant="subscription.is_enabled ? 'success' : 'default'"
              size="sm"
              class="detail-header__badge"
            >
              {{
                subscription.is_enabled ? t('subscriptions.enabled') : t('subscriptions.disabled')
              }}
            </Hook0Badge>
          </template>
          <template #actions>
            <Hook0Button
              variant="secondary"
              :to="{
                name: routes.SubscriptionsEdit,
                params: {
                  organization_id: organizationId,
                  application_id: applicationId,
                  subscription_id: subscriptionId,
                },
              }"
            >
              <template #left>
                <Pencil :size="16" aria-hidden="true" />
              </template>
              {{ t('subscriptionDetail.editAction') }}
            </Hook0Button>
          </template>
        </Hook0CardHeader>
      </Hook0Card>

      <!-- Health -->
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>{{ t('subscriptionDetail.sectionHealth') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine type="split">
            <template #label>{{ t('health.failureRate') }}</template>
            <template #value>
              <Hook0HealthBadge :failure-percent="subscription.failure_percent ?? null" />
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
      </Hook0Card>

      <!-- Configuration -->
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>{{ t('subscriptionDetail.sectionConfig') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine type="split">
            <template #label>{{ t('subscriptionDetail.targetUrl') }}</template>
            <template #value>
              <code class="detail-code">{{ subscription.target.url }}</code>
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine type="split">
            <template #label>{{ t('subscriptionDetail.httpMethod') }}</template>
            <template #value>
              <Hook0Badge variant="info" size="sm">{{ subscription.target.method }}</Hook0Badge>
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine type="split">
            <template #label>{{ t('subscriptionDetail.eventTypes') }}</template>
            <template #value>
              <div class="detail-tags">
                <Hook0Badge
                  v-for="et in subscription.event_types"
                  :key="et"
                  variant="default"
                  size="sm"
                >
                  {{ et }}
                </Hook0Badge>
              </div>
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine v-if="Object.keys(subscription.labels).length > 0" type="split">
            <template #label>{{ t('subscriptionDetail.labels') }}</template>
            <template #value>
              <div class="detail-tags">
                <Hook0Badge
                  v-for="(val, key) in subscription.labels"
                  :key="key"
                  variant="default"
                  size="sm"
                >
                  {{ key }}={{ val }}
                </Hook0Badge>
              </div>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
      </Hook0Card>

      <!-- Retry Schedule -->
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>{{ t('subscriptionDetail.sectionRetrySchedule') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine type="split">
            <template #label>{{ t('subscriptionDetail.retrySchedule') }}</template>
            <template #value>
              {{ retrySchedule ? retrySchedule.name : t('retrySchedules.defaultSchedule') }}
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
      </Hook0Card>

      <!-- Recent Deliveries -->
      <Hook0Card>
        <Hook0CardHeader>
          <template #header>{{ t('subscriptionDetail.sectionRecentDeliveries') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <table v-if="recentDeliveries.length > 0" class="deliveries-table">
            <thead>
              <tr>
                <th>{{ t('subscriptionDetail.statusColumn') }}</th>
                <th>{{ t('subscriptionDetail.dateColumn') }}</th>
                <th>{{ t('subscriptionDetail.retryCountColumn') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="delivery in recentDeliveries" :key="delivery.request_attempt_id">
                <td>
                  <Hook0Badge
                    :variant="delivery.status.type === 'successful' ? 'success' : 'danger'"
                    size="sm"
                  >
                    {{
                      delivery.status.type === 'successful'
                        ? t('subscriptionDetail.statusOk')
                        : t('subscriptionDetail.statusFailed')
                    }}
                  </Hook0Badge>
                </td>
                <td><Hook0DateTime :value="delivery.created_at" /></td>
                <td>{{ delivery.retry_count }}</td>
              </tr>
            </tbody>
          </table>
          <p v-else class="detail-empty">{{ t('subscriptionDetail.noDeliveries') }}</p>
        </Hook0CardContent>
      </Hook0Card>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.detail-header__badge {
  margin-left: 0.5rem;
}

.detail-code {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.detail-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.detail-empty {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  padding: 1rem 0;
}

.deliveries-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.deliveries-table th {
  text-align: left;
  padding: 0.5rem 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  border-bottom: 1px solid var(--color-border);
}

.deliveries-table td {
  padding: 0.5rem 0.75rem;
  color: var(--color-text-primary);
  border-bottom: 1px solid var(--color-border);
}

.deliveries-table tr:last-child td {
  border-bottom: none;
}
</style>
