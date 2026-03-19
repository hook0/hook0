<script setup lang="ts">
import { computed } from 'vue';
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
import { format, parseISO, eachDayOfInterval } from 'date-fns';
import { BarChart3 } from 'lucide-vue-next';

import type { EventsPerDayEntry } from '@/pages/organizations/applications/EventsPerDayService';

import Hook0Button from '@/components/Hook0Button.vue';

use([
  BarChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  MarkLineComponent,
  CanvasRenderer,
]);

/** Color palette resolved from CSS custom properties. */
type ThemeColors = {
  primary: string;
  primaryLight: string;
  success: string;
  successLight: string;
  warning: string;
  warningLight: string;
  error: string;
  errorLight: string;
  info: string;
  infoLight: string;
  textSecondary: string;
  border: string;
};

/** Single parameter passed to ECharts tooltip formatter callback. */
type EChartsTooltipParam = {
  name: string;
  value: number;
  marker: string;
  seriesName: string;
};

/** Bar series configuration for ECharts. */
type BarSeriesConfig = {
  name: string;
  type: 'bar';
  stack?: string;
  color: string;
  data: Array<{
    value: number;
    itemStyle: Record<string, unknown>;
  }>;
  markLine?: Record<string, unknown>;
};

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
const BAR_RADIUS: [number, number, number, number] = [3, 3, 0, 0];

/** Escape HTML special characters to prevent XSS in tooltip content. */
function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/** Resolve design-system color palette from CSS custom properties. */
function getThemeColors(): ThemeColors {
  const style = getComputedStyle(document.documentElement);
  const css = (name: string): string => style.getPropertyValue(name).trim();
  return {
    primary: css('--color-primary'),
    primaryLight: css('--color-primary-light'),
    success: css('--color-success'),
    successLight: css('--color-success-light'),
    warning: css('--color-warning'),
    warningLight: css('--color-warning-light'),
    error: css('--color-error'),
    errorLight: css('--color-error-light'),
    info: css('--color-info'),
    infoLight: css('--color-info-light'),
    textSecondary: css('--color-text-secondary'),
    border: css('--color-border'),
  };
}

/** Generate array of 'yyyy-MM-dd' strings covering a date range (inclusive). */
function generateDateRange(from: string, to: string): string[] {
  return eachDayOfInterval({ start: parseISO(from), end: parseISO(to) }).map((d) =>
    format(d, 'yyyy-MM-dd')
  );
}

/** Sum event amounts grouped by date. */
function sumByDate(entries: EventsPerDayEntry[]): Map<string, number> {
  return entries.reduce(
    (m, e) => m.set(e.date, (m.get(e.date) ?? 0) + e.amount),
    new Map<string, number>()
  );
}

/** Shared tooltip styling used by both simple and stacked chart modes. */
const BASE_TOOLTIP_CONFIG = {
  trigger: 'axis',
  backgroundColor: 'rgba(15, 23, 42, 0.95)',
  borderColor: 'transparent',
  textStyle: { color: '#fff', fontSize: 12 },
  padding: [8, 12],
  extraCssText: 'border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.15);',
};

/** Build animation config scaled to the number of date points. */
function buildAnimationConfig(totalDates: number): Record<string, unknown> {
  return {
    animationDuration: 400,
    animationEasing: 'cubicOut',
    animationDelay: (idx: number) => (idx / totalDates) * 300,
  };
}

/** Build shared ECharts axis configuration. */
function buildAxisConfig(
  colors: ThemeColors,
  dates: string[]
): { xAxis: Record<string, unknown>; yAxis: Record<string, unknown> } {
  return {
    xAxis: {
      type: 'category',
      data: dates.map((d) => format(parseISO(d), 'MMM dd')),
      axisLine: { lineStyle: { color: colors.border } },
      axisLabel: { color: colors.textSecondary, fontSize: 11 },
    },
    yAxis: {
      type: 'value',
      minInterval: 1,
      axisLine: { show: false },
      splitLine: { lineStyle: { color: colors.border, type: 'dashed' } },
      axisLabel: { color: colors.textSecondary, fontSize: 11 },
    },
  };
}

/** Build ECharts option for non-stacked (single series) mode. */
function buildSimpleChartOption(
  dates: string[],
  entries: EventsPerDayEntry[],
  colors: ThemeColors,
  provisionalDates: Set<string>,
  seriesName: string
): Record<string, unknown> {
  const amountByDate = sumByDate(entries);
  const { xAxis, yAxis } = buildAxisConfig(colors, dates);

  return {
    ...buildAnimationConfig(dates.length),
    tooltip: { ...BASE_TOOLTIP_CONFIG },
    grid: { left: 50, right: 20, top: 10, bottom: 30 },
    xAxis,
    yAxis,
    series: [
      {
        type: 'bar',
        name: seriesName,
        itemStyle: { borderRadius: BAR_RADIUS },
        data: dates.map((d) => ({
          value: amountByDate.get(d) ?? 0,
          itemStyle: provisionalDates.has(d)
            ? { color: colors.primaryLight, opacity: 0.7 }
            : { color: colors.primary },
        })),
      },
    ],
  };
}

/** Format HTML tooltip content for stacked chart mode. */
function formatStackedTooltip(params: EChartsTooltipParam[], totalLabel: string): string {
  if (!params || params.length === 0) return '';
  const date = escapeHtml(String(params[0].name));
  const total = params.reduce((s, p) => s + (Number(p.value) || 0), 0);
  let html = `<div style="font-weight:700;margin-bottom:6px">${date}</div>`;
  html += '<table style="width:100%;border-spacing:0">';
  for (const p of params) {
    const val = Number(p.value) || 0;
    if (val === 0) continue;
    const pct = total > 0 ? Math.round((val / total) * 100) : 0;
    html += `<tr>`;
    html += `<td style="padding:1px 0">${String(p.marker)} ${escapeHtml(String(p.seriesName))}</td>`;
    html += `<td style="text-align:right;padding:1px 0 1px 16px;font-weight:600;font-variant-numeric:tabular-nums">${val}</td>`;
    html += `<td style="text-align:right;padding:1px 0 1px 4px;opacity:0.6;font-variant-numeric:tabular-nums">${pct}%</td>`;
    html += `</tr>`;
  }
  html += `</table>`;
  html += `<div style="border-top:1px solid rgba(255,255,255,0.2);margin-top:6px;padding-top:6px;font-weight:700;text-align:right">${escapeHtml(totalLabel)}</div>`;
  return html;
}

/** Group entries by application, returning per-app name and date-amount map. */
function groupEntriesByApp(
  entries: EventsPerDayEntry[]
): Map<string, { name: string; data: Map<string, number> }> {
  const appMap = new Map<string, { name: string; data: Map<string, number> }>();
  for (const entry of entries) {
    if (!appMap.has(entry.application_id)) {
      appMap.set(entry.application_id, { name: entry.application_name, data: new Map() });
    }
    const app = appMap.get(entry.application_id)!;
    app.data.set(entry.date, (app.data.get(entry.date) ?? 0) + entry.amount);
  }
  return appMap;
}

/** Find the topmost non-zero series index per date (for rounded corners on stacked bars). */
function findTopSeriesPerDate(
  dates: string[],
  appList: Array<{ name: string; data: Map<string, number> }>
): Map<string, number> {
  const topSeriesPerDate = new Map<string, number>();
  for (const d of dates) {
    let topIdx = -1;
    for (let i = 0; i < appList.length; i++) {
      if ((appList[i].data.get(d) ?? 0) > 0) topIdx = i;
    }
    topSeriesPerDate.set(d, topIdx);
  }
  return topSeriesPerDate;
}

/** Build ECharts option for stacked (multi-app) mode. */
function buildStackedChartOption(
  dates: string[],
  entries: EventsPerDayEntry[],
  colors: ThemeColors,
  provisionalDates: Set<string>,
  quotaLimit: number | undefined,
  totalLabelFn: (total: number) => string,
  quotaLabelText: string
): Record<string, unknown> {
  const palette = [colors.primary, colors.success, colors.warning, colors.error, colors.info];
  const paletteLighter = [
    colors.primaryLight,
    colors.successLight,
    colors.warningLight,
    colors.errorLight,
    colors.infoLight,
  ];

  const appMap = groupEntriesByApp(entries);
  const appList = Array.from(appMap.values());
  const topSeriesPerDate = findTopSeriesPerDate(dates, appList);

  const series: BarSeriesConfig[] = appList.map((app, i) => ({
    name: app.name,
    type: 'bar',
    stack: 'total',
    color: palette[i % palette.length],
    data: dates.map((d) => {
      const value = app.data.get(d) ?? 0;
      const isTop = topSeriesPerDate.get(d) === i;
      const isProvisional = provisionalDates.has(d);
      return {
        value,
        itemStyle: {
          ...(isTop ? { borderRadius: BAR_RADIUS } : {}),
          ...(isProvisional
            ? { color: paletteLighter[i % paletteLighter.length], opacity: 0.7 }
            : {}),
        },
      };
    }),
  }));

  if (quotaLimit && quotaLimit > 0 && series.length > 0) {
    series[0].markLine = {
      silent: true,
      symbol: 'none',
      animationDuration: 600,
      animationDelay: 500,
      animationEasing: 'cubicOut',
      lineStyle: { type: 'dashed', color: colors.error, width: 2 },
      label: {
        formatter: quotaLabelText,
        position: 'insideEndTop',
        fontSize: 11,
      },
      data: [{ yAxis: quotaLimit }],
    };
  }

  const { xAxis, yAxis } = buildAxisConfig(colors, dates);

  return {
    ...buildAnimationConfig(dates.length),
    tooltip: {
      ...BASE_TOOLTIP_CONFIG,
      formatter: (params: EChartsTooltipParam[]) => {
        const total = params.reduce((s, p) => s + (Number(p.value) || 0), 0);
        return formatStackedTooltip(params, totalLabelFn(total));
      },
    },
    legend: {
      type: 'scroll',
      bottom: 0,
      textStyle: { color: colors.textSecondary, fontSize: 11 },
    },
    grid: { left: 50, right: 20, top: quotaLimit ? 30 : 10, bottom: 50 },
    xAxis,
    yAxis,
    series,
  };
}

// KPI stats
const totalEvents = computed(() => props.entries.reduce((sum, e) => sum + e.amount, 0));
const avgPerDay = computed(() => {
  if (props.days === 0) return 0;
  return Math.round((totalEvents.value / props.days) * 10) / 10;
});

/** Peak daily event count across all entries. */
const peakDay = computed(() => Math.max(0, ...sumByDate(props.entries).values()));

/** ECharts configuration, dispatching to simple or stacked builder. */
const chartOption = computed(() => {
  const colors = getThemeColors();
  const dates = generateDateRange(props.from, props.to);
  const provisionalDates = new Set(
    props.entries.filter((e) => e.is_provisional).map((e) => e.date)
  );

  if (!props.stacked) {
    return buildSimpleChartOption(
      dates,
      props.entries,
      colors,
      provisionalDates,
      t('eventsPerDayChart.seriesName')
    );
  }

  return buildStackedChartOption(
    dates,
    props.entries,
    colors,
    provisionalDates,
    props.quotaLimit,
    (total) => t('eventsPerDayChart.totalTooltip', { total }),
    props.quotaLimit ? t('eventsPerDayChart.includedInPlan', { limit: props.quotaLimit }) : ''
  );
});
</script>

<template>
  <div class="chart">
    <div class="chart__header">
      <!-- KPI stats -->
      <div class="chart__stats">
        <div class="chart__stat">
          <span class="chart__stat-value">{{ totalEvents.toLocaleString() }}</span>
          <span class="chart__stat-label">{{ t('eventsPerDayChart.totalEvents') }}</span>
        </div>
        <div class="chart__stat">
          <span class="chart__stat-value">{{ avgPerDay }}</span>
          <span class="chart__stat-label">{{ t('eventsPerDayChart.avgPerDay') }}</span>
        </div>
        <div class="chart__stat">
          <span class="chart__stat-value">{{ peakDay }}</span>
          <span class="chart__stat-label">{{ t('eventsPerDayChart.peakDay') }}</span>
        </div>
      </div>
      <!-- Day presets -->
      <div class="chart__toolbar">
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
