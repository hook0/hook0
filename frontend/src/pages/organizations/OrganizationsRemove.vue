<script setup lang="ts">
import { ref } from 'vue';
import { AlertTriangle, Trash2 } from 'lucide-vue-next';

import * as OrganizationService from './OrganizationService';
import { Problem } from '@/http';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import { push } from 'notivue';
import router from '@/router.ts';
import { routes } from '@/routes.ts';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// Analytics tracking
const { trackEvent } = useTracking();

// Permissions
const { canDelete } = usePermissions();

interface Props {
  organizationId: string;
  organizationName: string;
}

const props = defineProps<Props>();

const loading = ref(false);
const showDeleteDialog = ref(false);

function remove(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();
  showDeleteDialog.value = true;
}

function confirmRemove() {
  showDeleteDialog.value = false;
  loading.value = true;

  OrganizationService.remove(props.organizationId)
    .then(() => {
      trackEvent('organization', 'delete', 'success');
      push.success({
        title: t('remove.organizationDeleted'),
        message: t('remove.organizationDeletedMessage', { name: props.organizationName }),
        duration: 5000,
      });
      return router.push({ name: routes.Home });
    })
    .catch(displayError)
    // finally
    .finally(() => (loading.value = false));
}

function displayError(err: Problem) {
  console.error(err);
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}
</script>

<template>
  <Hook0Card v-if="canDelete('organization')" data-test="organization-delete-card">
    <Hook0CardHeader>
      <template #header>
        <Hook0Stack direction="row" align="center" gap="sm">
          <Hook0IconBadge variant="danger">
            <AlertTriangle :size="18" aria-hidden="true" />
          </Hook0IconBadge>
          <span class="org-remove__title">{{ t('remove.deleteOrganization') }}</span>
        </Hook0Stack>
      </template>
      <template #subtitle>
        {{ t('remove.deleteOrganizationWarning', { name: organizationName }) || '' }}
        <span class="org-remove__name">{{ organizationName }}</span>
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0Alert type="alert">
        <template #description>
          {{
            t('remove.irreversibleWarning') ||
            'This action deletes the organization and everything it contains. There is no going back.'
          }}
        </template>
      </Hook0Alert>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button
        variant="danger"
        type="button"
        :loading="loading"
        data-test="organization-delete-button"
        @click="remove($event)"
      >
        <Trash2 :size="16" aria-hidden="true" />
        {{ t('common.delete') }}
      </Hook0Button>
    </Hook0CardFooter>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="t('remove.deleteOrganization')"
      @close="showDeleteDialog = false"
      @confirm="confirmRemove()"
    >
      <p>{{ t('remove.confirmDeleteOrganization', { name: organizationName }) }}</p>
    </Hook0Dialog>
  </Hook0Card>
</template>

<style scoped>
.org-remove__title {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}

.org-remove__name {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}
</style>
