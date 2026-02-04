<script setup lang="ts">
import { h, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';

import {
  useSubscriptionList,
  useToggleSubscription,
  useRemoveSubscription,
} from './useSubscriptionQueries';
import type { Subscription } from './SubscriptionService';
import { routes } from '@/routes';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';
import { push } from 'notivue';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Switch from '@/components/Hook0Switch.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const applicationId = ref(route.params.application_id as string);
const { data: subscriptions, isLoading, error, refetch } = useSubscriptionList(applicationId);

const toggleMutation = useToggleSubscription();
const removeMutation = useRemoveSubscription();

function targetIsHttp(target: object): target is { type: string; method: string; url: string } {
  return 'type' in target && target.type === 'http';
}

function handleToggle(row: Subscription) {
  if (row.is_enabled) {
    const subscriptionName = row.description || t('subscriptions.title');
    if (!confirm(t('subscriptions.disableConfirm', { name: subscriptionName }))) {
      return;
    }
  }

  toggleMutation.mutate(
    { subscriptionId: row.subscription_id, subscription: row },
    {
      onSuccess: () => {
        push.success({
          title: t('common.success'),
          message: row.is_enabled ? t('subscriptions.disabled') : t('subscriptions.enabled'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

function handleDelete(row: Subscription) {
  if (!confirm(t('subscriptions.deleteConfirm'))) return;

  removeMutation.mutate(
    { applicationId: applicationId.value, subscriptionId: row.subscription_id },
    {
      onSuccess: () => {
        push.success({
          title: t('common.success'),
          message: t('subscriptions.deleted'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

const columns: ColumnDef<Subscription, unknown>[] = [
  {
    accessorKey: 'is_enabled',
    header: t('subscriptions.enabledColumn'),
    enableSorting: true,
    cell: (info) =>
      h(Hook0Switch, {
        modelValue: info.row.original.is_enabled,
        'onUpdate:modelValue': () => handleToggle(info.row.original),
      }),
  },
  {
    accessorKey: 'description',
    header: t('common.description'),
    enableSorting: true,
    cell: (info) =>
      h(
        Hook0Button,
        {
          variant: 'link',
          to: {
            name: routes.SubscriptionsDetail,
            params: {
              application_id: route.params.application_id,
              organization_id: route.params.organization_id,
              subscription_id: info.row.original.subscription_id,
            },
          },
          'data-test': 'subscription-description-link',
        },
        () => String(info.getValue() ?? t('subscriptions.noDescription'))
      ),
  },
  {
    accessorKey: 'event_types',
    header: t('subscriptions.eventTypesColumn'),
    enableSorting: true,
    cell: (info) => {
      const val = info.getValue() as string[] | undefined;
      return val ? val.join(', ') : '';
    },
  },
  {
    accessorKey: 'labels',
    header: t('subscriptions.labelsColumn'),
    enableSorting: true,
    cell: (info) =>
      h(Hook0TableCellCode, {
        value: Object.entries((info.row.original.labels ?? {}) as Record<string, string>)
          .map(([key, value]) => `${key}=${value}`)
          .join(' '),
      }),
  },
  {
    accessorKey: 'target',
    header: t('subscriptions.targetColumn'),
    enableSorting: true,
    cell: (info) => {
      const target = (info.row.original.target ?? {}) as object;
      const text = targetIsHttp(target)
        ? `${target.method} ${target.url}`
        : JSON.stringify(info.row.original.target);
      return h(Hook0TableCellCode, { value: text });
    },
  },
  {
    id: 'options',
    header: t('common.actions'),
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: t('common.delete'),
        icon: 'trash',
        onClick: () => handleDelete(info.row.original),
      }),
  },
];
</script>

<template>
  <Hook0PageLayout :title="t('subscriptions.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !subscriptions" data-test="subscriptions-card">
      <Hook0CardHeader>
        <template #header>{{ t('subscriptions.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (subscriptions is guaranteed to be defined here) -->
    <template v-else>
      <Hook0Card data-test="subscriptions-card">
        <Hook0CardHeader>
          <template #header>{{ t('subscriptions.title') }}</template>
          <template #subtitle>
            {{ t('subscriptions.subtitle') }}
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="subscriptions.length > 0">
          <Hook0Table
            data-test="subscriptions-table"
            :columns="columns"
            :data="subscriptions"
            row-id-field="subscription_id"
          />
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0EmptyState
            :title="t('subscriptions.empty.title')"
            :description="t('subscriptions.empty.description')"
          >
            <template #action>
              <Hook0Button
                variant="primary"
                type="button"
                data-test="subscriptions-create-button"
                @click="void router.push({ name: routes.SubscriptionsNew })"
              >
                {{ t('subscriptions.create') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>

        <Hook0CardFooter v-if="subscriptions.length > 0">
          <Hook0Button
            variant="primary"
            type="button"
            data-test="subscriptions-create-button"
            @click="void router.push({ name: routes.SubscriptionsNew })"
          >
            {{ t('subscriptions.create') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>
  </Hook0PageLayout>
</template>

<style scoped></style>
