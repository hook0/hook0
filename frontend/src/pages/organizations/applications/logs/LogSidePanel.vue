<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { ExternalLink } from 'lucide-vue-next';

import { routes } from '@/routes';
import LogDetailContent from './LogDetailContent.vue';

import Hook0SidePanel from '@/components/Hook0SidePanel.vue';
import Hook0Button from '@/components/Hook0Button.vue';

type Props = {
  open: boolean;
  eventId: string;
  applicationId: string;
  responseId: string | null;
  requestAttemptId: string;
  httpResponseStatus: number | null;
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
      request_attempt_id: props.requestAttemptId,
    },
  });
}
</script>

<template>
  <Hook0SidePanel :open="open" :title="t('logs.deliveryDetail')" width="42rem" @close="emit('close')">
    <template #header>
      <h2 class="log-panel__title">{{ t('logs.deliveryDetail') }}</h2>
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

    <LogDetailContent
      :event-id="eventId"
      :application-id="applicationId"
      :response-id="responseId"
      :http-response-status="httpResponseStatus"
    />
  </Hook0SidePanel>
</template>

<style scoped>
.log-panel__title {
  flex: 1;
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-primary);
}
</style>
