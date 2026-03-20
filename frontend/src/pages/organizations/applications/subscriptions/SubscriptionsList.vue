<script setup lang="ts">
import { computed, h, markRaw, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Trash2, Link, BookOpen } from 'lucide-vue-next';
import { DOCS_SUBSCRIPTIONS_URL } from '@/constants/externalLinks';
import Hook0TableCellEventTypes from '@/components/Hook0TableCellEventTypes.vue';
import Hook0TableCellLabels from '@/components/Hook0TableCellLabels.vue';
import Hook0TableCellTarget from '@/components/Hook0TableCellTarget.vue';

import {
  useSubscriptionList,
  useToggleSubscription,
  useRemoveSubscription,
} from './useSubscriptionQueries';
import type { Subscription } from './SubscriptionService';
import { routes } from '@/routes';
import { handleMutationError } from '@/utils/handleMutationError';
import { toast } from 'vue-sonner';
import { usePermissions } from '@/composables/usePermissions';
import { useEntityDelete } from '@/composables/useEntityDelete';

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
import Hook0Dialog from '@/components/Hook0Dialog.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

// Permissions
const { canCreate, canDelete } = usePermissions();

const { applicationId } = useRouteIds();
const { data: subscriptions, isLoading, error, refetch } = useSubscriptionList(applicationId);

const toggleMutation = useToggleSubscription();
const removeMutation = useRemoveSubscription();

function targetIsHttp(target: object): target is { type: string; method: string; url: string } {
  return 'type' in target && target.type === 'http';
}

const showDisableDialog = ref(false);
const subscriptionToDisable = ref<Subscription | null>(null);

const {
  showDeleteDialog,
  entityToDelete: subscriptionToDelete,
  requestDelete: handleDelete,
  confirmDelete,
} = useEntityDelete<Subscription>({
  deleteFn: (row) =>
    removeMutation.mutateAsync({
      applicationId: applicationId.value,
      subscriptionId: row.subscription_id,
    }),
  successTitle: t('common.success'),
  successMessage: t('subscriptions.deleted'),
});

const disableDialogName = computed(() => {
  if (!subscriptionToDisable.value) return '';
  return subscriptionToDisable.value.description || t('subscriptions.title');
});

function handleToggle(row: Subscription) {
  if (row.is_enabled) {
    subscriptionToDisable.value = row;
    showDisableDialog.value = true;
    return;
  }

  const name = row.description || t('subscriptions.title');
  toggleMutation.mutate(
    { subscriptionId: row.subscription_id, subscription: row },
    {
      onSuccess: () => {
        toast.success(t('common.success'), {
          description: t('subscriptions.enabled', { name }),
          duration: 3000,
        });
      },
      onError: (err) => {
        handleMutationError(err);
      },
    }
  );
}

function confirmDisable() {
  const row = subscriptionToDisable.value;
  showDisableDialog.value = false;
  subscriptionToDisable.value = null;
  if (!row) return;

  const name = row.description || t('subscriptions.title');
  toggleMutation.mutate(
    { subscriptionId: row.subscription_id, subscription: row },
    {
      onSuccess: () => {
        toast.success(t('common.success'), {
          description: t('subscriptions.disabled', { name }),
          duration: 3000,
        });
      },
      onError: (err) => {
        handleMutationError(err);
      },
    }
  );
}

const columns: ColumnDef<Subscription, unknown>[] = [
  {
    accessorKey: 'description',
    header: t('common.name'),
    enableSorting: true,
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: String(info.getValue() ?? t('subscriptions.noDescription')),
        to: {
          name: routes.SubscriptionsDetail,
          params: {
            application_id: route.params.application_id,
            organization_id: route.params.organization_id,
            subscription_id: info.row.original.subscription_id,
          },
        },
        'data-test': 'subscription-description-link',
      }),
  },
  {
    accessorKey: 'event_types',
    header: t('subscriptions.eventTypesColumn'),
    enableSorting: true,
    cell: (info) => {
      const val = info.getValue() as string[] | undefined;
      if (!val || val.length === 0) return '';
      return h(Hook0TableCellEventTypes, {
        value: val,
        to: {
          name: routes.EventTypesList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        },
      });
    },
  },
  {
    accessorKey: 'labels',
    header: t('subscriptions.labelsColumn'),
    enableSorting: true,
    cell: (info) => {
      const labels = (info.row.original.labels ?? {}) as Record<string, string>;
      if (Object.keys(labels).length === 0) return '';
      return h(Hook0TableCellLabels, { value: labels });
    },
  },
  {
    accessorKey: 'target',
    header: t('subscriptions.targetColumn'),
    enableSorting: true,
    cell: (info) => {
      const target = (info.row.original.target ?? {}) as object;
      if (targetIsHttp(target)) {
        return h(Hook0TableCellTarget, { method: target.method, url: target.url });
      }
      return h(Hook0TableCellCode, { value: JSON.stringify(info.row.original.target) });
    },
  },
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
  ...(canDelete('subscription')
    ? [
        {
          id: 'options',
          header: t('common.actions'),
          cell: (info: { row: { original: Subscription } }) =>
            h(Hook0TableCellLink, {
              value: t('common.delete'),
              icon: markRaw(Trash2),
              variant: 'danger',
              onClick: () => handleDelete(info.row.original),
            }),
        },
      ]
    : []),
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
          <template #actions>
            <Hook0Button variant="secondary" :href="DOCS_SUBSCRIPTIONS_URL" target="_blank">
              <template #left>
                <BookOpen :size="14" aria-hidden="true" />
              </template>
              {{ t('common.documentation') }}
            </Hook0Button>
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
            :icon="Link"
          >
            <template v-if="canCreate('subscription')" #action>
              <Hook0Button
                variant="primary"
                type="button"
                data-test="subscriptions-create-button"
                @click="
                  void router.push({
                    name: routes.SubscriptionsNew,
                    params: {
                      organization_id: route.params.organization_id,
                      application_id: route.params.application_id,
                    },
                  })
                "
              >
                {{ t('subscriptions.create') }}
              </Hook0Button>
            </template>
          </Hook0EmptyState>
        </Hook0CardContent>

        <Hook0CardFooter v-if="subscriptions.length > 0 && canCreate('subscription')">
          <Hook0Button
            variant="primary"
            type="button"
            data-test="subscriptions-create-button"
            @click="
              void router.push({
                name: routes.SubscriptionsNew,
                params: {
                  organization_id: route.params.organization_id,
                  application_id: route.params.application_id,
                },
              })
            "
          >
            {{ t('subscriptions.create') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>

    <Hook0Dialog
      :open="showDisableDialog"
      variant="danger"
      :title="t('subscriptions.disableTitle')"
      :confirm-text="t('subscriptions.disable')"
      @close="
        showDisableDialog = false;
        subscriptionToDisable = null;
      "
      @confirm="confirmDisable()"
    >
      <i18n-t keypath="subscriptions.disableConfirm" tag="p">
        <template #name>
          &ldquo;<strong>{{ disableDialogName }}</strong
          >&rdquo;
        </template>
      </i18n-t>
    </Hook0Dialog>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="t('subscriptions.delete')"
      @close="
        showDeleteDialog = false;
        subscriptionToDelete = null;
      "
      @confirm="confirmDelete()"
    >
      <p>{{ t('subscriptions.deleteConfirm') }}</p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

