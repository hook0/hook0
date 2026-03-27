<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { ExternalLink } from 'lucide-vue-next';
import Hook0Code from '@/components/Hook0Code.vue';

import { useEventDetail } from './useEventQueries';
import { routes } from '@/routes';

import Hook0SidePanel from '@/components/Hook0SidePanel.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

type Props = {
  open: boolean;
  eventId: string;
  applicationId: string;
};

const props = defineProps<Props>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const eventIdRef = computed(() => props.eventId);
const applicationIdRef = computed(() => props.applicationId);

const { data: eventData, isLoading: eventLoading } = useEventDetail(eventIdRef, applicationIdRef);

function openFullPage() {
  emit('close');
  void router.push({
    name: routes.EventsDetail,
    params: {
      application_id: route.params.application_id,
      organization_id: route.params.organization_id,
      event_id: props.eventId,
    },
  });
}
</script>

<template>
  <Hook0SidePanel :open="open" :title="t('events.detail')" width="36rem" @close="emit('close')">
    <template #header>
      <h2 class="event-panel__title">{{ t('events.detail') }}</h2>
      <Hook0Button
        variant="ghost"
        size="sm"
        :aria-label="t('events.openFullPage')"
        data-test="event-panel-full-page"
        @click="openFullPage"
      >
        <ExternalLink :size="16" aria-hidden="true" />
        {{ t('events.openFullPage') }}
      </Hook0Button>
    </template>

    <!-- Loading -->
    <template v-if="eventLoading || !eventData">
      <div class="event-panel__section">
        <Hook0Skeleton size="text-truncated" />
        <Hook0Skeleton size="text" />
        <Hook0Skeleton size="text-truncated" />
      </div>
      <div class="event-panel__section">
        <Hook0Skeleton size="block" />
      </div>
    </template>

    <!-- Event loaded -->
    <template v-else>
      <!-- Metadata -->
      <div class="event-panel__section">
        <h3 class="event-panel__section-title">{{ t('events.metadata') }}</h3>
        <div class="event-panel__meta">
          <div class="event-panel__meta-row">
            <span class="event-panel__meta-label">{{ t('events.id') }}</span>
            <code class="event-panel__meta-value event-panel__meta-value--mono">{{
              eventData.event_id
            }}</code>
          </div>
          <div class="event-panel__meta-row">
            <span class="event-panel__meta-label">{{ t('events.type') }}</span>
            <code class="event-panel__meta-value event-panel__meta-value--mono">{{
              eventData.event_type_name
            }}</code>
          </div>
          <div class="event-panel__meta-row">
            <span class="event-panel__meta-label">{{ t('events.receivedAt') }}</span>
            <span class="event-panel__meta-value">{{ eventData.received_at }}</span>
          </div>
          <div class="event-panel__meta-row">
            <span class="event-panel__meta-label">{{ t('events.occurredAt') }}</span>
            <span class="event-panel__meta-value">{{ eventData.occurred_at }}</span>
          </div>
        </div>
      </div>

      <!-- Labels -->
      <div
        v-if="
          eventData.labels && Object.keys(eventData.labels as Record<string, string>).length > 0
        "
        class="event-panel__section"
      >
        <h3 class="event-panel__section-title">{{ t('events.labels') }}</h3>
        <div class="event-panel__labels">
          <span
            v-for="(val, key) in eventData.labels as Record<string, string>"
            :key="String(key)"
            class="event-panel__label-badge"
          >
            {{ key }}={{ val }}
          </span>
        </div>
      </div>

      <!-- Payload -->
      <div class="event-panel__section">
        <h3 class="event-panel__section-title">{{ t('events.payload') }}</h3>
        <Hook0Code :code="eventData.payload_decoded" language="json" :editable="false" />
      </div>
    </template>
  </Hook0SidePanel>
</template>

<style scoped>
.event-panel__title {
  flex: 1;
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.event-panel__section {
  margin-bottom: 1.25rem;
}

.event-panel__section:last-child {
  margin-bottom: 0;
}

.event-panel__section-title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
  margin: 0 0 0.75rem;
}

.event-panel__meta {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.event-panel__meta-row {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
}

.event-panel__meta-label {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  min-width: 6rem;
  flex-shrink: 0;
}

.event-panel__meta-value {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  word-break: break-all;
}

.event-panel__meta-value--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.event-panel__labels {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.event-panel__label-badge {
  display: inline-flex;
  padding: 0.125rem 0.5rem;
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--color-primary);
  background-color: var(--color-primary-light);
  border-radius: var(--radius-full);
}

.event-panel__code {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
}
</style>
