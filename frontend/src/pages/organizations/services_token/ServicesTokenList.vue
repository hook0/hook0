<script setup lang="ts">
import { h, markRaw, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { Plus, Bot, BookOpen, Check, Key, Pencil, Trash2 } from 'lucide-vue-next';

import {
  useServiceTokenList,
  useCreateServiceToken,
  useUpdateServiceToken,
  useRemoveServiceToken,
} from './useServiceTokenQueries';
import type { ServiceToken } from './ServicesTokenService';
import { routes } from '@/routes';
import {
  DOCS_SERVICE_TOKENS_URL,
  API_DOCS_SERVICE_TOKENS_URL,
  MCP_GUIDE_URL,
} from '@/constants/externalLinks';
import { handleMutationError } from '@/utils/handleMutationError';
import { toast } from 'vue-sonner';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import { useEntityDelete } from '@/composables/useEntityDelete';
import { useRouteIds } from '@/composables/useRouteIds';

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
import Hook0CopyField from '@/components/Hook0CopyField.vue';
import Hook0DocButtons from '@/components/Hook0DocButtons.vue';
import QuickReferenceCard from '@/components/QuickReferenceCard.vue';

const { t } = useI18n();
const { trackEvent } = useTracking();

// Permissions
const { canCreate, canEdit, canDelete } = usePermissions();

const { organizationId } = useRouteIds();
const { data: serviceTokens, isLoading, error, refetch } = useServiceTokenList(organizationId);

const createMutation = useCreateServiceToken();
const updateMutation = useUpdateServiceToken();
const removeMutation = useRemoveServiceToken();

const showCreateDialog = ref(false);
const newTokenName = ref('');

const showEditDialog = ref(false);
const editTokenName = ref('');
const tokenToEdit = ref<ServiceToken | null>(null);

const {
  showDeleteDialog,
  entityToDelete: tokenToDelete,
  requestDelete: handleDelete,
  confirmDelete,
} = useEntityDelete<ServiceToken>({
  deleteFn: (row) =>
    removeMutation.mutateAsync({ tokenId: row.token_id, organizationId: organizationId.value }),
  successTitle: t('common.success'),
  successMessage: t('serviceTokens.deleted'),
  onSuccess: () => trackEvent('service-token', 'delete', 'success'),
});

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
        toast.success(t('common.success'), {
          description: t('serviceTokens.created'),
          duration: 3000,
        });
      },
      onError: (err) => {
        handleMutationError(err);
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
        toast.success(t('common.success'), {
          description: t('serviceTokens.updated'),
          duration: 3000,
        });
      },
      onError: (err) => {
        handleMutationError(err);
      },
    }
  );
}

const columns: ColumnDef<ServiceToken, unknown>[] = [
  {
    accessorKey: 'name',
    header: t('common.name'),
    enableSorting: true,
    cell: (info) => {
      const row = info.row.original;
      return h(Hook0TableCellLink, {
        value: row.name,
        to: {
          name: routes.ServiceTokenView,
          params: {
            organization_id: organizationId.value,
            service_token_id: row.token_id,
          },
        },
        dataTest: 'token-name-link',
      });
    },
  },
  {
    accessorKey: 'biscuit',
    header: t('serviceTokens.tokenLabel'),
    enableSorting: false,
    cell: (info) =>
      h(Hook0CopyField, {
        value: String(info.getValue()),
        maskable: true,
        copyMessage: t('serviceTokens.tokenCopied'),
      }),
  },
  {
    accessorKey: 'created_at',
    header: t('common.createdAt'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellDate, { value: info.getValue<string | null>() }),
  },
  {
    id: 'actions',
    header: t('common.actions'),
    cell: (info) => {
      const row = info.row.original;
      const actions: ReturnType<typeof h>[] = [];
      if (canEdit('service_token')) {
        actions.push(
          h(Hook0TableCellLink, {
            value: t('serviceTokens.editAction'),
            icon: markRaw(Pencil),
            dataTest: 'token-edit-action',
            onClick: () => handleEdit(row),
          })
        );
      }
      if (canDelete('service_token')) {
        actions.push(
          h(Hook0TableCellLink, {
            value: t('common.delete'),
            icon: markRaw(Trash2),
            variant: 'danger',
            dataTest: 'token-delete-action',
            onClick: () => handleDelete(row),
          })
        );
      }
      return h('div', { class: 'service-token__actions' }, actions);
    },
  },
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
            <template #actions>
              <Hook0DocButtons
                :doc-url="DOCS_SERVICE_TOKENS_URL"
                :api-url="API_DOCS_SERVICE_TOKENS_URL"
              />
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
              :icon="Key"
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

        <!-- Quick Reference -->
        <QuickReferenceCard
          :title="t('serviceTokens.quickReference')"
          :subtitle="t('serviceTokens.quickReferenceDescription')"
          :organization-id="organizationId"
        />

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
              <Hook0Button variant="primary" :href="MCP_GUIDE_URL" target="_blank">
                <template #left>
                  <BookOpen :size="16" aria-hidden="true" />
                </template>
                {{ t('serviceTokens.aiSetupGuide') }}
              </Hook0Button>
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0Stack direction="column" gap="md">
              <span class="mcp__description">{{ t('serviceTokens.aiDescription') }}</span>
              <div class="mcp__features">
                <span class="mcp__feature">
                  <span class="mcp__feature-dot"><Check :size="12" aria-hidden="true" /></span>
                  {{ t('serviceTokens.aiFeature1') }}
                </span>
                <span class="mcp__feature">
                  <span class="mcp__feature-dot"><Check :size="12" aria-hidden="true" /></span>
                  {{ t('serviceTokens.aiFeature2') }}
                </span>
                <span class="mcp__feature">
                  <span class="mcp__feature-dot"><Check :size="12" aria-hidden="true" /></span>
                  {{ t('serviceTokens.aiFeature3') }}
                </span>
              </div>
              <Hook0Alert
                type="info"
                :title="t('serviceTokens.aiGetStartedTitle')"
                :description="t('serviceTokens.aiGetStarted')"
              />
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
        autofocus
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
      <p v-if="tokenToDelete">
        <i18n-t keypath="serviceTokens.deleteConfirm" tag="span">
          <template #name>
            <strong>{{ tokenToDelete.name }}</strong>
          </template>
        </i18n-t>
      </p>
    </Hook0Dialog>
  </Hook0PageLayout>
</template>

<style scoped>
/* Ensure MCP feature badges stack properly at narrow widths */
.mcp__description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  line-height: 1.6;
}

.mcp__features {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.mcp__feature {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  font-weight: 400;
  color: var(--color-text-primary);
}

.mcp__feature-dot {
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
  background-color: var(--color-success-light);
  color: var(--color-success);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.service-token__actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
</style>
