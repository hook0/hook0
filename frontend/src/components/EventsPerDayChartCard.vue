<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { FileText } from 'lucide-vue-next';

import type { EventsPerDayEntry } from '@/pages/organizations/applications/EventsPerDayService';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0EventsPerDayChart from '@/components/Hook0EventsPerDayChart.vue';

type Props = {
  title: string;
  entries: EventsPerDayEntry[];
  stacked: boolean;
  from: string;
  to: string;
  days: number;
  quotaLimit?: number;
};

defineProps<Props>();

const emit = defineEmits<{
  'update:days': [value: number];
  refresh: [];
}>();

const { t } = useI18n();
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>
        <Hook0Stack direction="row" align="center" gap="sm">
          <Hook0IconBadge variant="primary" size="sm">
            <FileText :size="14" aria-hidden="true" />
          </Hook0IconBadge>
          <span class="events-chart-card__label">{{ title }}</span>
        </Hook0Stack>
      </template>
      <template #actions>
        <Hook0Button @click="emit('refresh')">
          {{ t('common.refresh') }}
        </Hook0Button>
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0EventsPerDayChart
        :entries="entries"
        :stacked="stacked"
        :from="from"
        :to="to"
        :days="days"
        :quota-limit="quotaLimit"
        @update:days="emit('update:days', $event)"
      />
    </Hook0CardContent>
  </Hook0Card>
</template>

<style scoped>
.events-chart-card__label {
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.5;
}
</style>
