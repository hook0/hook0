<script setup lang="ts">
import { computed } from 'vue';
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
import { format, addDays, parseISO } from 'date-fns';
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

interface Props {
  entries: EventsPerDayEntry[];
  stacked: boolean;
  from: string;
  to: string;
  days: number;
  quotaLimit?: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:days': [value: number];
}>();

const dayPresets = [7, 30, 90];
const BAR_RADIUS: [number, number, number, number] = [3, 3, 0, 0];

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function getThemeColors() {
  return {
    primary: cssVar('--color-primary') || '#6366f1',
    primaryLight: cssVar('--color-primary-light') || '#ede9fe',
    success: cssVar('--color-success') || '#16a34a',
    successLight: cssVar('--color-success-light') || '#dcfce7',
    warning: cssVar('--color-warning') || '#d97706',
    warningLight: cssVar('--color-warning-light') || '#fef3c7',
    error: cssVar('--color-error') || '#dc2626',
    errorLight: cssVar('--color-error-light') || '#fef2f2',
    info: cssVar('--color-info') || '#0ea5e9',
    infoLight: cssVar('--color-info-light') || '#e0f2fe',
    textSecondary: cssVar('--color-text-secondary') || '#6b7280',
    border: cssVar('--color-border') || '#e5e7eb',
  };
}

function generateDateRange(from: string, to: string): string[] {
  const dates: string[] = [];
  let current = parseISO(from);
  const end = parseISO(to);
  while (current <= end) {
    dates.push(format(current, 'yyyy-MM-dd'));
    current = addDays(current, 1);
  }
  return dates;
}

// KPI stats
const totalEvents = computed(() => props.entries.reduce((sum, e) => sum + e.amount, 0));
const avgPerDay = computed(() => {
  if (props.days === 0) return 0;
  return Math.round((totalEvents.value / props.days) * 10) / 10;
});
const peakDay = computed(() => {
  const byDate = new Map<string, number>();
  for (const e of props.entries) {
    byDate.set(e.date, (byDate.get(e.date) ?? 0) + e.amount);
  }
  let max = 0;
  for (const v of byDate.values()) {
    if (v > max) max = v;
  }
  return max;
});

const chartOption = computed(() => {
  const colors = getThemeColors();
  const palette = [colors.primary, colors.success, colors.warning, colors.error, colors.info];
  const paletteLighter = [
    colors.primaryLight,
    colors.successLight,
    colors.warningLight,
    colors.errorLight,
    colors.infoLight,
  ];
  const dates = generateDateRange(props.from, props.to);

  const provisionalDates = new Set<string>();
  for (const entry of props.entries) {
    if (entry.is_provisional) {
      provisionalDates.add(entry.date);
    }
  }

  // Animation: all bars grow together from bottom, staggered by date index (not by series)
  const totalDates = dates.length;
  const animationConfig = {
    animationDuration: 400,
    animationEasing: 'cubicOut' as const,
    animationDelay: (idx: number) => (idx / totalDates) * 300,
  };

  // Tooltip formatter for both modes
  const tooltipConfig = {
    trigger: 'axis' as const,
    backgroundColor: 'rgba(15, 23, 42, 0.95)',
    borderColor: 'transparent',
    textStyle: { color: '#fff', fontSize: 12 },
    padding: [8, 12],
    extraCssText: 'border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.15);',
  };

  if (!props.stacked) {
    const amountByDate = new Map<string, number>();
    for (const entry of props.entries) {
      amountByDate.set(entry.date, (amountByDate.get(entry.date) ?? 0) + entry.amount);
    }

    return {
      ...animationConfig,
      tooltip: tooltipConfig,
      grid: { left: 50, right: 20, top: 10, bottom: 30 },
      xAxis: {
        type: 'category' as const,
        data: dates.map((d) => format(parseISO(d), 'MMM dd')),
        axisLine: { lineStyle: { color: colors.border } },
        axisLabel: { color: colors.textSecondary, fontSize: 11 },
      },
      yAxis: {
        type: 'value' as const,
        minInterval: 1,
        axisLine: { show: false },
        splitLine: { lineStyle: { color: colors.border, type: 'dashed' as const } },
        axisLabel: { color: colors.textSecondary, fontSize: 11 },
      },
      series: [
        {
          type: 'bar' as const,
          name: 'Events',
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

  // Stacked mode
  const appMap = new Map<string, { name: string; data: Map<string, number> }>();
  for (const entry of props.entries) {
    if (!appMap.has(entry.application_id)) {
      appMap.set(entry.application_id, { name: entry.application_name, data: new Map() });
    }
    const app = appMap.get(entry.application_id)!;
    app.data.set(entry.date, (app.data.get(entry.date) ?? 0) + entry.amount);
  }

  const appList = Array.from(appMap.values());

  // Top series per date for rounded corners
  const topSeriesPerDate = new Map<string, number>();
  for (const d of dates) {
    let topIdx = -1;
    for (let i = 0; i < appList.length; i++) {
      if ((appList[i].data.get(d) ?? 0) > 0) topIdx = i;
    }
    topSeriesPerDate.set(d, topIdx);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const series: any[] = appList.map((app, i) => ({
    name: app.name,
    type: 'bar' as const,
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

  if (props.quotaLimit && props.quotaLimit > 0 && series.length > 0) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    series[0].markLine = {
      silent: true,
      symbol: 'none',
      animationDuration: 600,
      animationDelay: 500,
      animationEasing: 'cubicOut',
      lineStyle: { type: 'dashed', color: colors.error, width: 2 },
      label: {
        formatter: `Included in plan: ${props.quotaLimit} events/day`,
        position: 'insideEndTop',
        fontSize: 11,
      },
      data: [{ yAxis: props.quotaLimit }],
    };
  }

  return {
    ...animationConfig,
    tooltip: {
      ...tooltipConfig,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      formatter: (params: any[]) => {
        if (!params || params.length === 0) return '';
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        const date = String(params[0].name);
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        const total = params.reduce((s: number, p: any) => s + (Number(p.value) || 0), 0);
        let html = `<div style="font-weight:700;margin-bottom:6px">${date}</div>`;
        html += '<table style="width:100%;border-spacing:0">';
        for (const p of params) {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          const val = Number(p.value) || 0;
          if (val === 0) continue;
          const pct = total > 0 ? Math.round((val / total) * 100) : 0;
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          html += `<tr><td style="padding:1px 0">${String(p.marker)} ${String(p.seriesName)}</td><td style="text-align:right;padding:1px 0 1px 16px;font-weight:600;font-variant-numeric:tabular-nums">${val}</td><td style="text-align:right;padding:1px 0 1px 4px;opacity:0.6;font-variant-numeric:tabular-nums">${pct}%</td></tr>`;
        }
        html += `</table><div style="border-top:1px solid rgba(255,255,255,0.2);margin-top:6px;padding-top:6px;font-weight:700;text-align:right">Total: ${total} events</div>`;
        return html;
      },
    },
    legend: {
      type: 'scroll' as const,
      bottom: 0,
      textStyle: { color: colors.textSecondary, fontSize: 11 },
    },
    grid: { left: 50, right: 20, top: props.quotaLimit ? 30 : 10, bottom: 50 },
    xAxis: {
      type: 'category' as const,
      data: dates.map((d) => format(parseISO(d), 'MMM dd')),
      axisLine: { lineStyle: { color: colors.border } },
      axisLabel: { color: colors.textSecondary, fontSize: 11 },
    },
    yAxis: {
      type: 'value' as const,
      minInterval: 1,
      axisLine: { show: false },
      splitLine: { lineStyle: { color: colors.border, type: 'dashed' as const } },
      axisLabel: { color: colors.textSecondary, fontSize: 11 },
    },
    series,
  };
});
</script>

<template>
  <div class="chart">
    <div class="chart__header">
      <!-- KPI stats -->
      <div class="chart__stats">
        <div class="chart__stat">
          <span class="chart__stat-value">{{ totalEvents.toLocaleString() }}</span>
          <span class="chart__stat-label">Total events</span>
        </div>
        <div class="chart__stat">
          <span class="chart__stat-value">{{ avgPerDay }}</span>
          <span class="chart__stat-label">Avg / day</span>
        </div>
        <div class="chart__stat">
          <span class="chart__stat-value">{{ peakDay }}</span>
          <span class="chart__stat-label">Peak day</span>
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
          {{ preset }}d
        </Hook0Button>
      </div>
    </div>

    <div v-if="entries.length === 0" class="chart__empty">
      <BarChart3 :size="32" aria-hidden="true" />
      <p>No events recorded yet</p>
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
