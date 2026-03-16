<script setup lang="ts">
import { computed, h, markRaw, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Plus, Bot, BookOpen, Check, Key, Eye, Pencil, Trash2 } from 'lucide-vue-next';

import {
  useServiceTokenList,
  useCreateServiceToken,
  useUpdateServiceToken,
  useRemoveServiceToken,
} from './useServiceTokenQueries';
import type { ServiceToken } from './ServicesTokenService';
import { routes } from '@/routes';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';
import { push } from 'notivue';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import Hook0Input from '@/components/Hook0Input.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { trackEvent } = useTracking();

// Permissions
const { canCreate, canEdit, canDelete } = usePermissions();

const organizationId = computed(() => route.params.organization_id as string);
const { data: serviceTokens, isLoading, error, refetch } = useServiceTokenList(organizationId);

const createMutation = useCreateServiceToken();
const updateMutation = useUpdateServiceToken();
const removeMutation = useRemoveServiceToken();

const showCreateDialog = ref(false);
const newTokenName = ref('');

const showEditDialog = ref(false);
const editTokenName = ref('');
const tokenToEdit = ref<ServiceToken | null>(null);

const showDeleteDialog = ref(false);
const tokenToDelete = ref<ServiceToken | null>(null);

function createNew(event: MouseEvent) {
  event.stopImmediatePropagation();
  event.preventDefault();
  newTokenName.value = '';
  showCreateDialog.value = true;
}

function confirmCreate() {
  const name = newTokenName.value.trim();
  showCreateDialog.value = false;
  newTokenName.value = '';
  if (!name) return;

  createMutation.mutate(
    { name, organization_id: organizationId.value },
    {
      onSuccess: () => {
        trackEvent('service-token', 'create', 'success');
        push.success({
          title: t('common.success'),
          message: t('serviceTokens.created'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

function handleEdit(row: ServiceToken) {
  tokenToEdit.value = row;
  editTokenName.value = row.name;
  showEditDialog.value = true;
}

function confirmEdit() {
  const row = tokenToEdit.value;
  const name = editTokenName.value.trim();
  showEditDialog.value = false;
  tokenToEdit.value = null;
  editTokenName.value = '';
  if (!row || !name) return;

  updateMutation.mutate(
    { tokenId: row.token_id, token: { name, organization_id: organizationId.value } },
    {
      onSuccess: () => {
        trackEvent('service-token', 'update', 'success');
        push.success({
          title: t('common.success'),
          message: t('serviceTokens.updated'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

function handleDelete(row: ServiceToken) {
  tokenToDelete.value = row;
  showDeleteDialog.value = true;
}

function confirmDelete() {
  const row = tokenToDelete.value;
  showDeleteDialog.value = false;
  tokenToDelete.value = null;
  if (!row) return;

  removeMutation.mutate(
    { tokenId: row.token_id, organizationId: organizationId.value },
    {
      onSuccess: () => {
        trackEvent('service-token', 'delete', 'success');
        push.success({
          title: t('common.success'),
          message: t('serviceTokens.deleted'),
          duration: 3000,
        });
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}

function handleShow(row: ServiceToken) {
  void router.push({
    name: routes.ServiceTokenView,
    params: {
      organization_id: organizationId.value,
      service_token_id: row.token_id,
    },
  });
}

const columns: ColumnDef<ServiceToken, unknown>[] = [
  {
    accessorKey: 'name',
    header: t('common.name'),
    enableSorting: true,
  },
  {
    accessorKey: 'created_at',
    header: t('common.createdAt'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellDate, { value: info.getValue() as string | null }),
  },
  {
    id: 'show',
    header: '',
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: t('serviceTokens.show'),
        icon: markRaw(Eye),
        onClick: () => handleShow(info.row.original),
      }),
  },
  ...(canEdit('service_token')
    ? [
        {
          id: 'edit',
          header: '',
          cell: (info: { row: { original: ServiceToken } }) =>
            h(Hook0TableCellLink, {
              value: t('serviceTokens.editAction'),
              icon: markRaw(Pencil),
              onClick: () => handleEdit(info.row.original),
            }),
        },
      ]
    : []),
  ...(canDelete('service_token')
    ? [
        {
          id: 'delete',
          header: '',
          cell: (info: { row: { original: ServiceToken } }) =>
            h(Hook0TableCellLink, {
              value: t('common.delete'),
              icon: markRaw(Trash2),
              onClick: () => handleDelete(info.row.original),
            }),
        },
      ]
    : []),
];
</script>

<template>
  <Hook0PageLayout :title="t('serviceTokens.title')">
    <!-- Error state (check FIRST - errors take priority) -->
    <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

    <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
    <Hook0Card v-else-if="isLoading || !serviceTokens" data-test="service-tokens-card">
      <Hook0CardHeader>
        <template #header>{{ t('serviceTokens.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Data loaded (serviceTokens is guaranteed to be defined here) -->
    <template v-else>
      <!-- Service Tokens List -->
      <Hook0Stack direction="column" gap="lg">
        <Hook0Card data-test="service-tokens-card">
          <Hook0CardHeader>
            <template #header>{{ t('serviceTokens.title') }}</template>
            <template #subtitle>
              {{ t('serviceTokens.subtitle') }}
            </template>
          </Hook0CardHeader>

          <Hook0CardContent v-if="serviceTokens.length > 0">
            <Hook0Table
              data-test="service-tokens-table"
              :columns="columns"
              :data="serviceTokens"
              row-id-field="token_id"
            />
          </Hook0CardContent>

          <Hook0CardContent v-else>
            <Hook0EmptyState
              :title="t('serviceTokens.empty.title')"
              :description="t('serviceTokens.empty.description')"
            >
              <template v-if="canCreate('service_token')" #action>
                <Hook0Button
                  variant="primary"
                  type="button"
                  data-test="service-tokens-create-button"
                  @click="createNew"
                >
                  <template #left>
                    <Plus :size="16" aria-hidden="true" />
                  </template>
                  {{ t('serviceTokens.create') }}
                </Hook0Button>
              </template>
            </Hook0EmptyState>
          </Hook0CardContent>

          <Hook0CardFooter v-if="serviceTokens.length > 0 && canCreate('service_token')">
            <Hook0Button
              variant="primary"
              type="button"
              data-test="service-tokens-create-button"
              @click="createNew"
            >
              <template #left>
                <Plus :size="16" aria-hidden="true" />
              </template>
              {{ t('serviceTokens.create') }}
            </Hook0Button>
          </Hook0CardFooter>
        </Hook0Card>

        <!-- AI Integration Banner -->
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>
              <Hook0Stack direction="row" align="center" gap="sm">
                <Hook0IconBadge variant="primary">
                  <Bot :size="18" aria-hidden="true" />
                </Hook0IconBadge>
                <Hook0Stack direction="column" gap="none">
                  <Hook0Stack direction="row" align="center" gap="sm">
                    {{ t('serviceTokens.aiTitle') }}
                    <Hook0Badge variant="primary" size="sm">MCP</Hook0Badge>
                  </Hook0Stack>
                </Hook0Stack>
              </Hook0Stack>
            </template>
            <template #subtitle>
              {{ t('serviceTokens.aiSubtitle') }}
            </template>
            <template #actions>
              <Hook0Button
                variant="primary"
                href="https://documentation.hook0.com/reference/mcp-for-ia-assistant"
                target="_blank"
              >
                <template #left>
                  <BookOpen :size="16" aria-hidden="true" />
                </template>
                {{ t('serviceTokens.aiSetupGuide') }}
              </Hook0Button>
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0Stack direction="column" gap="lg">
              <Hook0Stack direction="column" gap="md">
                {{ t('serviceTokens.aiDescription') }}
                <Hook0Stack layout="grid" gap="md" grid-size="compact">
                  <Hook0Badge variant="success" display="trust">
                    <template #icon>
                      <Check :size="16" aria-hidden="true" />
                    </template>
                    {{ t('serviceTokens.aiFeature1') }}
                  </Hook0Badge>
                  <Hook0Badge variant="success" display="trust">
                    <template #icon>
                      <Check :size="16" aria-hidden="true" />
                    </template>
                    {{ t('serviceTokens.aiFeature2') }}
                  </Hook0Badge>
                  <Hook0Badge variant="success" display="trust">
                    <template #icon>
                      <Check :size="16" aria-hidden="true" />
                    </template>
                    {{ t('serviceTokens.aiFeature3') }}
                  </Hook0Badge>
                </Hook0Stack>
              </Hook0Stack>
              <Hook0Alert type="warning">
                <template #title>
                  <Hook0Stack direction="row" align="center" gap="xs">
                    <Key :size="16" aria-hidden="true" />
                    {{ t('serviceTokens.aiGetStarted') }}
                  </Hook0Stack>
                </template>
              </Hook0Alert>
            </Hook0Stack>
          </Hook0CardContent>
        </Hook0Card>
      </Hook0Stack>
    </template>

    <Hook0Dialog
      :open="showCreateDialog"
      variant="default"
      :title="t('serviceTokens.create')"
      @close="
        showCreateDialog = false;
        newTokenName = '';
      "
      @confirm="confirmCreate()"
    >
      <Hook0Input
        v-model="newTokenName"
        :placeholder="t('common.name')"
        :label="t('common.name')"
        data-test="service-token-name-input"
        @keydown.enter="confirmCreate()"
      />
    </Hook0Dialog>

    <Hook0Dialog
      :open="showEditDialog"
      variant="default"
      :title="t('serviceTokens.editAction')"
      @close="
        showEditDialog = false;
        tokenToEdit = null;
        editTokenName = '';
      "
      @confirm="confirmEdit()"
    >
      <Hook0Input
        v-model="editTokenName"
        :placeholder="t('common.name')"
        :label="t('common.name')"
        data-test="service-token-edit-name-input"
        @keydown.enter="confirmEdit()"
      />
    </Hook0Dialog>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="t('common.delete')"
      @close="
        showDeleteDialog = false;
        tokenToDelete = null;
      "
      @confirm="confirmDelete()"
    >
      <p>{{ t('serviceTokens.deleteConfirm') }}</p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped>
/* Ensure MCP feature badges stack properly at narrow widths */
:deep(.hook0-badge-trust) {
  display: flex;
  align-items: flex-start;
  white-space: normal;
  min-width: 0;
}
</style>
