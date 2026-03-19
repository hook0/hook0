<script setup lang="ts">
import { computed } from 'vue';
import { use } from 'echarts/core';
import { BarChart } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  MarkLineComponent,
  TitleComponent,
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
  TitleComponent,
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

const chartOption = computed(() => {
  const dates = generateDateRange(props.from, props.to);

  const provisionalDates = new Set<string>();
  for (const entry of props.entries) {
    if (entry.is_provisional) {
      provisionalDates.add(entry.date);
    }
  }

  if (!props.stacked) {
    // Application mode: single series
    const amountByDate = new Map<string, number>();
    for (const entry of props.entries) {
      amountByDate.set(entry.date, (amountByDate.get(entry.date) ?? 0) + entry.amount);
    }

    return {
      tooltip: {
        trigger: 'axis' as const,
      },
      grid: {
        left: 50,
        right: 20,
        top: 20,
        bottom: 30,
      },
      xAxis: {
        type: 'category' as const,
        data: dates.map((d) => format(parseISO(d), 'MMM dd')),
      },
      yAxis: {
        type: 'value' as const,
        minInterval: 1,
      },
      series: [
        {
          type: 'bar' as const,
          data: dates.map((d) => ({
            value: amountByDate.get(d) ?? 0,
            itemStyle: provisionalDates.has(d)
              ? { color: '#a5b4fc', opacity: 0.7 }
              : { color: '#6366f1' },
          })),
        },
      ],
    };
  }

  // Organization mode: stacked bar chart, one series per application
  const appMap = new Map<string, { name: string; data: Map<string, number> }>();
  for (const entry of props.entries) {
    if (!appMap.has(entry.application_id)) {
      appMap.set(entry.application_id, {
        name: entry.application_name,
        data: new Map(),
      });
    }
    const app = appMap.get(entry.application_id)!;
    app.data.set(entry.date, (app.data.get(entry.date) ?? 0) + entry.amount);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const series: any[] = Array.from(appMap.values()).map((app) => ({
    name: app.name,
    type: 'bar' as const,
    stack: 'total',
    data: dates.map((d) => ({
      value: app.data.get(d) ?? 0,
      itemStyle: provisionalDates.has(d) ? { opacity: 0.7 } : undefined,
    })),
  }));

  if (props.quotaLimit && props.quotaLimit > 0 && series.length > 0) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    series[0].markLine = {
      silent: true,
      symbol: 'none',
      lineStyle: {
        type: 'dashed',
        color: '#ef4444',
        width: 2,
      },
      label: {
        formatter: `Included in plan: ${props.quotaLimit}/day`,
        position: 'insideEndTop',
      },
      data: [{ yAxis: props.quotaLimit }],
    };
  }

  return {
    tooltip: {
      trigger: 'axis' as const,
    },
    legend: {
      type: 'scroll' as const,
      bottom: 0,
    },
    grid: {
      left: 50,
      right: 20,
      top: 20,
      bottom: 40,
    },
    xAxis: {
      type: 'category' as const,
      data: dates.map((d) => format(parseISO(d), 'MMM dd')),
    },
    yAxis: {
      type: 'value' as const,
      minInterval: 1,
    },
    series,
  };
});
</script>

<template>
  <div class="chart">
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
    <div v-if="entries.length === 0" class="chart__empty">
      <BarChart3 :size="32" aria-hidden="true" />
      <p>No events recorded yet</p>
    </div>
    <VChart v-else :option="chartOption" autoresize class="chart__canvas" />
  </div>
</template>

<style scoped>
.chart__toolbar {
  display: flex;
  justify-content: flex-end;
  gap: 0.25rem;
  margin-bottom: 0.75rem;
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
</style>
