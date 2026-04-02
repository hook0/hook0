<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { ExternalLink } from 'lucide-vue-next';

import { routes } from '@/routes';
import LogDetailContent from './LogDetailContent.vue';
import type { RequestAttemptExtended } from './LogService';

import Hook0SidePanel from '@/components/Hook0SidePanel.vue';
import Hook0Button from '@/components/Hook0Button.vue';

type Props = {
  open: boolean;
  attempt: RequestAttemptExtended;
  applicationId: string;
};

const props = defineProps<Props>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

function openFullPage() {
  emit('close');
  void router.push({
    name: routes.LogDetail,
    params: {
      organization_id: route.params.organization_id,
      application_id: route.params.application_id,
      request_attempt_id: props.attempt.request_attempt_id,
    },
  });
}
</script>

<template>
  <Hook0SidePanel
    :open="open"
    :title="t('logs.deliveryDetail')"
    width="42rem"
    @close="emit('close')"
  >
    <template #header>
      <h2 style="flex: 1; margin: 0">{{ t('logs.deliveryDetail') }}</h2>
      <Hook0Button
        variant="ghost"
        size="sm"
        :aria-label="t('logs.openFullPage')"
        data-test="log-panel-full-page"
        @click="openFullPage"
      >
        <ExternalLink :size="16" aria-hidden="true" />
        {{ t('logs.openFullPage') }}
      </Hook0Button>
    </template>

    <LogDetailContent :attempt="attempt" :application-id="applicationId" />
  </Hook0SidePanel>
</template>
