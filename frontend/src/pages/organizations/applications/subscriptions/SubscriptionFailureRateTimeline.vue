<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { use } from 'echarts/core';
import type { ComposeOption } from 'echarts/core';
import { LineChart } from 'echarts/charts';
import type { LineSeriesOption } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
  MarkAreaComponent,
} from 'echarts/components';
import type {
  GridComponentOption,
  TooltipComponentOption,
  MarkLineComponentOption,
  MarkAreaComponentOption,
} from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import VChart from 'vue-echarts';
import { LineChart as LineIcon } from 'lucide-vue-next';

import {
  HEALTH_WINDOWS,
  type HealthTimeline,
  type HealthWindow,
  type HealthEvent,
  type HealthBucket,
} from './SubscriptionHealthService';
import { buildTooltipConfig } from '@/components/eventsPerDayChartOptions';
import { useHealthThresholds } from '@/composables/useHealthThresholds';
import { useThemeColors } from '@/composables/useThemeColors';

import Hook0Button from '@/components/Hook0Button.vue';

use([
  LineChart,
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
  MarkAreaComponent,
  CanvasRenderer,
]);

type ECOption = ComposeOption<
  | LineSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | MarkLineComponentOption
  | MarkAreaComponentOption
>;

function failurePercent(bucket: HealthBucket): number {
  return (bucket.failed_count / bucket.total_count) * 100;
}

function roundOneDecimal(value: number): number {
  return Math.round(value * 10) / 10;
}

const props = defineProps<{
  timeline: HealthTimeline | undefined;
  window: HealthWindow;
}>();

const emit = defineEmits<{
  'update:window': [value: HealthWindow];
}>();

const { t } = useI18n();
const { warning: warningThreshold, critical: criticalThreshold } = useHealthThresholds();

const colors = useThemeColors();

const buckets = computed(() => props.timeline?.buckets ?? []);
const events = computed(() => props.timeline?.events ?? []);

const peakFailureRate = computed(() =>
  roundOneDecimal(
    buckets.value
      .filter((bucket) => bucket.total_count > 0)
      .reduce((max, bucket) => Math.max(max, failurePercent(bucket)), 0)
  )
);

const totalDeliveries = computed(() =>
  buckets.value.reduce((sum, bucket) => sum + bucket.total_count, 0)
);
const totalFailures = computed(() =>
  buckets.value.reduce((sum, bucket) => sum + bucket.failed_count, 0)
);

function statusColor(status: HealthEvent['status']): string {
  const palette = colors.value;
  switch (status) {
    case 'disabled':
      return palette.error;
    case 'warning':
      return palette.warning;
    case 'resolved':
      return palette.success;
  }
}

function eventLabel(event: HealthEvent): string {
  if (event.cause === 'manual') {
    return event.status === 'disabled'
      ? t('subscriptionDetail.healthChart.manualDisable')
      : t('subscriptionDetail.healthChart.manualEnable');
  }
  return t(`subscriptionDetail.healthStatus.${event.status}`);
}

// Null on zero-traffic buckets so the line breaks rather than dives to 0%.
const seriesData = computed<[number, number | null][]>(() =>
  buckets.value.map((bucket) => [
    new Date(bucket.bucket_start).getTime(),
    bucket.total_count > 0 ? roundOneDecimal(failurePercent(bucket)) : null,
  ])
);

const eventMarkers = computed(() =>
  events.value.map((event) => ({
    xAxis: new Date(event.created_at).getTime(),
    lineStyle: {
      color: statusColor(event.status),
      type: event.cause === 'manual' ? ('solid' as const) : ('dashed' as const),
      width: 1,
    },
    label: { show: false },
    emphasis: {
      label: {
        show: true,
        formatter: eventLabel(event),
        color: statusColor(event.status),
        fontSize: 11,
        fontWeight: 600 as const,
        position: 'end' as const,
        align: 'center' as const,
        verticalAlign: 'bottom' as const,
        rotate: 0,
        backgroundColor: colors.value.tooltipText,
        padding: [4, 8],
        borderRadius: 4,
      },
      lineStyle: { width: 2 },
    },
  }))
);

const chartOption = computed<ECOption>(() => {
  const palette = colors.value;

  return {
    grid: { left: 50, right: 20, top: 24, bottom: 32 },
    tooltip: {
      ...buildTooltipConfig(palette),
      formatter: (params) => {
        const point = Array.isArray(params) ? params[0] : params;
        if (!Array.isArray(point.value)) return '';
        // We control the series shape: [time_ms, percent | null].
        const [time, percent] = point.value as [number, number | null];
        if (percent === null) return '';
        // Both interpolations come from numbers (Date / toFixed) — no XSS surface.
        return `<div style="font-size:11px;opacity:.7;margin-bottom:2px">${new Date(time).toLocaleString()}</div>
                <div style="font-weight:600">${percent.toFixed(1)}%</div>`;
      },
    },
    xAxis: {
      type: 'time',
      axisLine: { lineStyle: { color: palette.border } },
      axisLabel: { color: palette.textSecondary, fontSize: 11 },
    },
    yAxis: {
      type: 'value',
      min: 0,
      max: 100,
      axisLabel: { color: palette.textSecondary, fontSize: 11, formatter: '{value}%' },
      splitLine: { lineStyle: { color: palette.border, type: 'dashed' } },
    },
    series: [
      {
        type: 'line',
        name: t('subscriptionDetail.healthChart.failureRate'),
        smooth: true,
        showSymbol: false,
        connectNulls: false,
        data: seriesData.value,
        lineStyle: { width: 2, color: palette.primary },
        areaStyle: { color: palette.primaryLight, opacity: 0.25 },
        markArea: {
          silent: true,
          data: [
            [
              {
                yAxis: warningThreshold.value,
                itemStyle: { color: palette.warningLight, opacity: 0.18 },
              },
              { yAxis: criticalThreshold.value },
            ],
            [
              {
                yAxis: criticalThreshold.value,
                itemStyle: { color: palette.errorLight, opacity: 0.18 },
              },
              { yAxis: 100 },
            ],
          ],
        },
        markLine: {
          // silent=false so event markers expand on hover.
          silent: false,
          symbol: ['none', 'none'],
          data: [
            {
              yAxis: warningThreshold.value,
              lineStyle: { color: palette.warning, type: 'solid', width: 1 },
              label: { show: false },
            },
            {
              yAxis: criticalThreshold.value,
              lineStyle: { color: palette.error, type: 'solid', width: 1 },
              label: { show: false },
            },
            ...eventMarkers.value,
          ],
        },
      },
    ],
  };
});
</script>

<template>
  <div class="health-chart">
    <div class="health-chart__header">
      <div class="health-chart__stats">
        <div class="health-chart__stat">
          <span class="health-chart__stat-value">{{ peakFailureRate.toFixed(1) }}%</span>
          <span class="health-chart__stat-label">{{
            t('subscriptionDetail.healthChart.peakRate')
          }}</span>
        </div>
        <div class="health-chart__stat">
          <span class="health-chart__stat-value">{{ totalDeliveries.toLocaleString() }}</span>
          <span class="health-chart__stat-label">{{
            t('subscriptionDetail.healthChart.deliveries')
          }}</span>
        </div>
        <div class="health-chart__stat">
          <span class="health-chart__stat-value">{{ totalFailures.toLocaleString() }}</span>
          <span class="health-chart__stat-label">{{
            t('subscriptionDetail.healthChart.failures')
          }}</span>
        </div>
      </div>
      <div class="health-chart__toolbar">
        <Hook0Button
          v-for="value in HEALTH_WINDOWS"
          :key="value"
          :variant="window === value ? 'primary' : 'secondary'"
          size="sm"
          type="button"
          @click="emit('update:window', value)"
        >
          {{ value }}
        </Hook0Button>
      </div>
    </div>

    <div v-if="buckets.length === 0" class="health-chart__empty">
      <LineIcon :size="32" aria-hidden="true" />
      <p>{{ t('subscriptionDetail.healthChart.empty') }}</p>
    </div>
    <VChart v-else :option="chartOption" autoresize class="health-chart__canvas" />
  </div>
</template>

<style scoped>
.health-chart__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.health-chart__stats {
  display: flex;
  gap: 1.5rem;
}

.health-chart__stat {
  display: flex;
  flex-direction: column;
}

.health-chart__stat-value {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1.2;
}

.health-chart__stat-label {
  font-size: 0.6875rem;
  color: var(--color-text-secondary);
}

.health-chart__toolbar {
  display: flex;
  gap: 0.25rem;
}

.health-chart__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem 1rem;
  color: var(--color-text-secondary);
  gap: 0.75rem;
}

.health-chart__empty p {
  font-size: 0.875rem;
}

.health-chart__canvas {
  height: 280px;
}

@media (max-width: 639px) {
  .health-chart__stats {
    gap: 1rem;
  }

  .health-chart__stat-value {
    font-size: 1rem;
  }
}
</style>
