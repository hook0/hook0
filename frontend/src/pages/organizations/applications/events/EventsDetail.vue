<script setup lang="ts">
import { useI18n } from 'vue-i18n';

import { useRouteIds } from '@/composables/useRouteIds';
import { useEventDetail } from './useEventQueries';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0DateTime from '@/components/Hook0DateTime.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';

const { t } = useI18n();
const { eventId, applicationId } = useRouteIds();

const { data: event, isLoading, error, refetch } = useEventDetail(eventId, applicationId);
</script>

<template>
  <Hook0PageLayout :title="t('events.detail')">
    <!-- Loading skeleton -->
    <Hook0Card v-if="isLoading">
      <Hook0CardHeader>
        <template #header>{{ t('events.detail') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="4" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Error state -->
    <Hook0ErrorCard v-else-if="error" :error="error" @retry="refetch()" />

    <!-- Data loaded -->
    <template v-else-if="event">
      <Hook0Stack data-test="event-detail-page" direction="column" gap="md">
        <Hook0Card data-test="event-detail-card">
          <Hook0CardHeader>
            <template #header>
              {{ t('events.eventOfType') }}
              <Hook0Code inline :code="event.event_type_name" data-test="event-detail-type" />
            </template>
            <template #subtitle>
              <Hook0Stack direction="column" gap="xs">
                <Hook0Stack direction="row" align="baseline" gap="xs">
                  <span class="event-detail__label">{{ t('events.id') }}:</span>
                  <Hook0Code inline :code="event.event_id" />
                </Hook0Stack>

                <Hook0Stack direction="row" align="baseline" gap="xs">
                  <span class="event-detail__label">{{ t('events.occurredAt') }}:</span>
                  <Hook0DateTime :value="event.occurred_at" />
                </Hook0Stack>

                <Hook0Stack direction="row" align="baseline" gap="xs">
                  <span class="event-detail__label">{{ t('events.receivedAt') }}:</span>
                  <Hook0DateTime :value="event.received_at" />
                </Hook0Stack>

                <Hook0Stack direction="row" align="baseline" gap="xs">
                  <span class="event-detail__label">{{ t('events.sourceIp') }}:</span>
                  <Hook0Code inline :code="event.ip" />
                </Hook0Stack>
              </Hook0Stack>
            </template>
          </Hook0CardHeader>
        </Hook0Card>

        <Hook0Card>
          <Hook0CardHeader>
            <template #header>{{ t('events.metadata') }}</template>
            <template #subtitle>
              <Hook0Button variant="link" href="https://documentation.hook0.com/docs/metadata">{{
                t('events.metadataLearnMore')
              }}</Hook0Button>
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <template v-if="event.metadata && Object.keys(event.metadata).length > 0">
              <Hook0CardContentLine v-for="(value, key) in event.metadata" :key="key">
                <template #label>{{ key }}</template>
                <template #content>
                  <Hook0Code inline :code="String(value)" />
                </template>
              </Hook0CardContentLine>
            </template>
            <template v-else>
              <Hook0CardContentLine>
                <template #label>{{ t('events.noMetadata') }}</template>
              </Hook0CardContentLine>
            </template>
          </Hook0CardContent>
        </Hook0Card>

        <Hook0Card>
          <Hook0CardHeader>
            <template #header>{{ t('events.labels') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0CardContentLine v-for="(value, key) in event.labels" :key="key">
              <template #label>{{ key }}</template>
              <template #content>
                <Hook0Code inline :code="String(value)" />
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
        </Hook0Card>

        <Hook0Card>
          <Hook0CardHeader>
            <template #header>{{ t('events.payload') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0CardContentLine>
              <template #label>{{ t('events.payloadContentType') }}</template>
              <template #content>
                <Hook0Code inline :code="event.payload_content_type" />
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label>{{ t('events.payloadDecoded') }}</template>
              <template #content>
                <Hook0Code :code="event.payload_decoded" />
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label>{{ t('events.payloadRaw') }}</template>
              <template #content>
                <Hook0Code :code="event.payload" />
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
        </Hook0Card>
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.event-detail__label {
  color: var(--color-text-secondary);
  font-weight: 500;
  font-size: 0.875rem;
  line-height: 1.5;
}
</style>
