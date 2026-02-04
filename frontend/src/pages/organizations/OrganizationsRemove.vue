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
import Hook0Text from '@/components/Hook0Text.vue';
import { push } from 'notivue';
import router from '@/router.ts';
import { routes } from '@/routes.ts';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// Analytics tracking
const { trackEvent } = useTracking();

interface Props {
  organizationId: string;
  organizationName: string;
}

const props = defineProps<Props>();

const loading = ref(false);

function remove(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (!confirm(`Are you sure to delete "${props.organizationName}" organization?`)) {
    return;
  }

  loading.value = true;

  OrganizationService.remove(props.organizationId)
    .then(() => {
      trackEvent('organization', 'delete', 'success');
      push.success({
        title: 'Organization deleted',
        message: `Organization "${props.organizationName}" has been deleted.`,
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
  <Hook0Card data-test="organization-delete-card">
    <Hook0CardHeader>
      <template #header>
        <Hook0Stack direction="row" align="center" gap="sm">
          <Hook0IconBadge variant="danger">
            <AlertTriangle :size="18" aria-hidden="true" />
          </Hook0IconBadge>
          <Hook0Text>{{ t('remove.deleteOrganization') }}</Hook0Text>
        </Hook0Stack>
      </template>
      <template #subtitle>
        {{ t('remove.deleteOrganizationWarning', { name: organizationName }) || '' }}
        <Hook0Text variant="primary" weight="semibold">{{ organizationName }}</Hook0Text>
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
  </Hook0Card>
</template>

<style scoped>
/* Hook0Text handles all text styling */
</style>
