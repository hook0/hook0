<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { use } from 'echarts/core';
import { BarChart } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  MarkLineComponent,
} from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import VChart from 'vue-echarts';
import { BarChart3 } from 'lucide-vue-next';

import type { EventsPerDayEntry } from '@/pages/organizations/applications/EventsPerDayService';

import Hook0Button from '@/components/Hook0Button.vue';
import {
  getThemeColors,
  generateDateRange,
  sumByDate,
  buildSimpleChartOption,
  buildStackedChartOption,
} from '@/components/eventsPerDayChartOptions';

use([
  BarChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  MarkLineComponent,
  CanvasRenderer,
]);

type Props = {
  entries: EventsPerDayEntry[];
  stacked: boolean;
  from: string;
  to: string;
  days: number;
  quotaLimit?: number;
};

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:days': [value: number];
}>();

const { t } = useI18n();

const dayPresets = [7, 30, 90] as const;

const themeKey = ref(0);
let observer: MutationObserver | null = null;
onMounted(() => {
  observer = new MutationObserver(() => {
    themeKey.value++;
  });
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['class'],
  });
});
onBeforeUnmount(() => observer?.disconnect());
const colors = computed(() => {
  void themeKey.value;
  return getThemeColors();
});

// KPI stats
const totalEvents = computed(() => props.entries.reduce((sum, e) => sum + e.amount, 0));
const avgPerDay = computed(() => {
  // Ignore leading zero-event days (trimLeft) so avg reflects active usage period
  const firstActiveIndex = props.entries.findIndex((e) => e.amount > 0);
  if (firstActiveIndex === -1) return 0;
  const activeDays = props.entries.length - firstActiveIndex;
  const activeTotal = props.entries.slice(firstActiveIndex).reduce((sum, e) => sum + e.amount, 0);
  return Math.round((activeTotal / activeDays) * 10) / 10;
});

/** Peak daily event count across all entries (M11: use reduce instead of spread). */
const peakDay = computed(() => {
  let max = 0;
  for (const v of sumByDate(props.entries).values()) {
    if (v > max) max = v;
  }
  return max;
});

/** ECharts configuration, dispatching to simple or stacked builder. */
const chartOption = computed(() => {
  const dates = generateDateRange(props.from, props.to);
  const provisionalDates = new Set(
    props.entries.filter((e) => e.is_provisional).map((e) => e.date)
  );

  if (!props.stacked) {
    return buildSimpleChartOption(
      dates,
      props.entries,
      colors.value,
      provisionalDates,
      t('eventsPerDayChart.seriesName')
    );
  }

  return buildStackedChartOption({
    dates,
    entries: props.entries,
    colors: colors.value,
    provisionalDates,
    quotaLimit: props.quotaLimit,
    totalLabelFn: (total) => t('eventsPerDayChart.totalTooltip', { total }),
    quotaLabelText: props.quotaLimit
      ? t('eventsPerDayChart.includedInPlan', { limit: props.quotaLimit })
      : '',
  });
});
</script>

<template>
  <div class="chart">
    <div class="chart__header">
      <!-- KPI stats -->
      <div class="chart__stats" data-test="chart-stats">
        <div class="chart__stat" data-test="chart-stat-total">
          <span class="chart__stat-value">{{ totalEvents.toLocaleString() }}</span>
          <span class="chart__stat-label">{{ t('eventsPerDayChart.totalEvents') }}</span>
        </div>
        <div class="chart__stat" data-test="chart-stat-avg">
          <span class="chart__stat-value">{{ avgPerDay }}</span>
          <span class="chart__stat-label">{{ t('eventsPerDayChart.avgPerDay') }}</span>
        </div>
        <div class="chart__stat" data-test="chart-stat-peak">
          <span class="chart__stat-value">{{ peakDay }}</span>
          <span class="chart__stat-label">{{ t('eventsPerDayChart.peakDay') }}</span>
        </div>
      </div>
      <!-- Day presets -->
      <div class="chart__toolbar" data-test="chart-toolbar">
        <Hook0Button
          v-for="preset in dayPresets"
          :key="preset"
          :variant="days === preset ? 'primary' : 'secondary'"
          size="sm"
          type="button"
          @click="emit('update:days', preset)"
        >
          {{ t('eventsPerDayChart.daysSuffix', { days: preset }) }}
        </Hook0Button>
      </div>
    </div>

    <div v-if="entries.length === 0" class="chart__empty">
      <BarChart3 :size="32" aria-hidden="true" />
      <p>{{ t('eventsPerDayChart.noEvents') }}</p>
    </div>
    <VChart v-else :option="chartOption" autoresize class="chart__canvas" />
  </div>
</template>

<style scoped>
.chart__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.chart__stats {
  display: flex;
  gap: 1.5rem;
}

.chart__stat {
  display: flex;
  flex-direction: column;
}

.chart__stat-value {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1.2;
}

.chart__stat-label {
  font-size: 0.6875rem;
  color: var(--color-text-muted);
}

.chart__toolbar {
  display: flex;
  gap: 0.25rem;
}

.chart__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem 1rem;
  color: var(--color-text-muted);
  gap: 0.75rem;
}

.chart__empty p {
  font-size: 0.875rem;
}

.chart__canvas {
  height: 300px;
}

@media (max-width: 639px) {
  .chart__stats {
    gap: 1rem;
  }

  .chart__stat-value {
    font-size: 1rem;
  }
}
</style>
