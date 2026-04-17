import { format, parseISO, eachDayOfInterval } from 'date-fns';
import type { ComposeOption } from 'echarts/core';
import type { BarSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  TooltipComponentOption,
  LegendComponentOption,
  MarkLineComponentOption,
} from 'echarts/components';

import { escapeHtml } from '@/utils/escapeHtml';
import type { EventsPerDayEntry } from '@/pages/organizations/applications/EventsPerDayService';
import type { ThemeColors } from '@/composables/useThemeColors';

/** Composed ECharts option type matching the registered components. */
export type ECOption = ComposeOption<
  | BarSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | LegendComponentOption
  | MarkLineComponentOption
>;

/** Single parameter passed to ECharts tooltip formatter callback. */
export type EChartsTooltipParam = {
  name: string;
  value: number;
  marker: string;
  seriesName: string;
};

/** Bar series configuration for ECharts. */
type BarSeriesConfig = BarSeriesOption & {
  markLine?: MarkLineComponentOption;
};

/** Options object for buildStackedChartOption (replaces 7 positional params). */
export type StackedChartConfig = {
  dates: string[];
  entries: EventsPerDayEntry[];
  colors: ThemeColors;
  provisionalDates: Set<string>;
  quotaLimit: number | undefined;
  totalLabelFn: (total: number) => string;
  quotaLabelText: string;
};

export const BAR_RADIUS: [number, number, number, number] = [3, 3, 0, 0];

const CHART_GRID = { left: 50, right: 20, top: 10, bottom: 30 };
const CHART_GRID_STACKED_BASE = { left: 50, right: 20, bottom: 50 };
const TOOLTIP_PADDING: [number, number] = [8, 12];
const AXIS_FONT_SIZE = 11;
const ANIMATION_DURATION_MS = 400;

/** Generate array of 'yyyy-MM-dd' strings covering a date range (inclusive). */
export function generateDateRange(from: string, to: string): string[] {
  return eachDayOfInterval({ start: parseISO(from), end: parseISO(to) }).map((d) =>
    format(d, 'yyyy-MM-dd')
  );
}

/** Sum event amounts grouped by date. */
export function sumByDate(entries: EventsPerDayEntry[]): Map<string, number> {
  return entries.reduce(
    (m, e) => m.set(e.date, (m.get(e.date) ?? 0) + e.amount),
    new Map<string, number>()
  );
}

/** Build shared tooltip styling using theme colors. */
export function buildTooltipConfig(colors: ThemeColors): ECOption['tooltip'] {
  return {
    trigger: 'axis',
    backgroundColor: colors.tooltipBg,
    borderColor: 'transparent',
    textStyle: { color: colors.tooltipText, fontSize: 12 },
    padding: TOOLTIP_PADDING,
    extraCssText: `border-radius: 8px; box-shadow: 0 4px 12px ${colors.tooltipBorder};`,
  };
}

/** Build animation config scaled to the number of date points. */
export function buildAnimationConfig(
  totalDates: number
): Pick<ECOption, 'animationDuration' | 'animationEasing' | 'animationDelay'> {
  return {
    animationDuration: ANIMATION_DURATION_MS,
    animationEasing: 'cubicOut',
    animationDelay: (idx: number) => (idx / totalDates) * 300,
  };
}

/** Build shared ECharts axis configuration. */
export function buildAxisConfig(
  colors: ThemeColors,
  dates: string[]
): Pick<ECOption, 'xAxis' | 'yAxis'> {
  return {
    xAxis: {
      type: 'category',
      data: dates.map((d) => format(parseISO(d), 'MMM dd')),
      axisLine: { lineStyle: { color: colors.border } },
      axisLabel: { color: colors.textSecondary, fontSize: AXIS_FONT_SIZE },
    },
    yAxis: {
      type: 'value',
      minInterval: 1,
      axisLine: { show: false },
      splitLine: { lineStyle: { color: colors.border, type: 'dashed' } },
      axisLabel: { color: colors.textSecondary, fontSize: AXIS_FONT_SIZE },
    },
  };
}

/** Build ECharts option for non-stacked (single series) mode. */
export function buildSimpleChartOption(
  dates: string[],
  entries: EventsPerDayEntry[],
  colors: ThemeColors,
  provisionalDates: Set<string>,
  seriesName: string
): ECOption {
  const amountByDate = sumByDate(entries);
  const { xAxis, yAxis } = buildAxisConfig(colors, dates);

  return {
    ...buildAnimationConfig(dates.length),
    tooltip: { ...buildTooltipConfig(colors) },
    grid: CHART_GRID,
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

/** Build a single HTML table row for a tooltip entry. */
function tooltipRow(p: EChartsTooltipParam, total: number): string {
  const val = Number(p.value) || 0;
  const pct = total > 0 ? Math.round((val / total) * 100) : 0;
  // marker is ECharts-generated trusted HTML (colored circle SVG)
  return [
    '<tr>',
    `<td style="padding:1px 0">${String(p.marker)} ${escapeHtml(String(p.seriesName))}</td>`,
    `<td style="text-align:right;padding:1px 0 1px 16px;font-weight:600;font-variant-numeric:tabular-nums">${val}</td>`,
    `<td style="text-align:right;padding:1px 0 1px 4px;opacity:0.6;font-variant-numeric:tabular-nums">${pct}%</td>`,
    '</tr>',
  ].join('');
}

/** Format HTML tooltip content for stacked chart mode. */
export function formatStackedTooltip(
  params: EChartsTooltipParam[],
  totalLabel: string,
  colors: ThemeColors
): string {
  if (!params || params.length === 0) return '';
  const date = escapeHtml(String(params[0].name));
  const total = params.reduce((s, p) => s + (Number(p.value) || 0), 0);

  const rows = params
    .filter((p) => (Number(p.value) || 0) > 0)
    .map((p) => tooltipRow(p, total))
    .join('');

  const dividerColor = colors.tooltipText.startsWith('#fff')
    ? 'rgba(255,255,255,0.2)'
    : `${colors.tooltipText}33`;

  return [
    `<div style="font-weight:700;margin-bottom:6px">${date}</div>`,
    `<table style="width:100%;border-spacing:0">${rows}</table>`,
    `<div style="border-top:1px solid ${dividerColor};margin-top:6px;padding-top:6px;font-weight:700;text-align:right">${escapeHtml(totalLabel)}</div>`,
  ].join('');
}

/** Group entries by application, returning per-app name and date-amount map. */
export function groupEntriesByApp(
  entries: EventsPerDayEntry[]
): Map<string, { name: string; data: Map<string, number> }> {
  const appMap = new Map<string, { name: string; data: Map<string, number> }>();
  for (const entry of entries) {
    let app = appMap.get(entry.application_id);
    if (!app) {
      app = { name: entry.application_name, data: new Map() };
      appMap.set(entry.application_id, app);
    }
    app.data.set(entry.date, (app.data.get(entry.date) ?? 0) + entry.amount);
  }
  return appMap;
}

/** Find the topmost non-zero series index per date (for rounded corners on stacked bars). */
export function findTopSeriesPerDate(
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
export function buildStackedChartOption(config: StackedChartConfig): ECOption {
  const { dates, entries, colors, provisionalDates, quotaLimit, totalLabelFn, quotaLabelText } =
    config;
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
      ...buildTooltipConfig(colors),
      formatter: (params: unknown) => {
        const arr = (Array.isArray(params) ? params : [params]) as EChartsTooltipParam[];
        const total = arr.reduce((s, p) => s + (Number(p.value) || 0), 0);
        return formatStackedTooltip(arr, totalLabelFn(total), colors);
      },
    },
    legend: {
      type: 'scroll',
      bottom: 0,
      textStyle: { color: colors.textSecondary, fontSize: 11 },
    },
    grid: { ...CHART_GRID_STACKED_BASE, top: quotaLimit ? 30 : 10 },
    xAxis,
    yAxis,
    series,
  };
}
