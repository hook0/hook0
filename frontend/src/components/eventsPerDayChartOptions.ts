import { format, parseISO, eachDayOfInterval } from 'date-fns';

import { escapeHtml } from '@/utils/escapeHtml';
import type { EventsPerDayEntry } from '@/pages/organizations/applications/EventsPerDayService';

/** Color palette resolved from CSS custom properties. */
export type ThemeColors = {
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
  tooltipBg: string;
  tooltipText: string;
  tooltipBorder: string;
};

/** Single parameter passed to ECharts tooltip formatter callback. */
export type EChartsTooltipParam = {
  name: string;
  value: number;
  marker: string;
  seriesName: string;
};

/** Bar series configuration for ECharts. */
export type BarSeriesConfig = {
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

/** Resolve design-system color palette from CSS custom properties. */
export function getThemeColors(): ThemeColors {
  const style = getComputedStyle(document.documentElement);
  const css = (name: string, fallback: string): string =>
    style.getPropertyValue(name).trim() || fallback;
  return {
    primary: css('--color-primary', '#6366f1'),
    primaryLight: css('--color-primary-light', '#a5b4fc'),
    success: css('--color-success', '#22c55e'),
    successLight: css('--color-success-light', '#86efac'),
    warning: css('--color-warning', '#f59e0b'),
    warningLight: css('--color-warning-light', '#fcd34d'),
    error: css('--color-error', '#ef4444'),
    errorLight: css('--color-error-light', '#fca5a5'),
    info: css('--color-info', '#3b82f6'),
    infoLight: css('--color-info-light', '#93c5fd'),
    textSecondary: css('--color-text-secondary', '#64748b'),
    border: css('--color-border', '#e2e8f0'),
    tooltipBg: css('--color-text-primary', '#0f172a'),
    tooltipText: css('--color-bg-primary', '#ffffff'),
    tooltipBorder: css('--color-border', '#e2e8f0'),
  };
}

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
export function buildTooltipConfig(colors: ThemeColors): Record<string, unknown> {
  return {
    trigger: 'axis',
    backgroundColor: colors.tooltipBg,
    borderColor: 'transparent',
    textStyle: { color: colors.tooltipText, fontSize: 12 },
    padding: [8, 12],
    extraCssText: `border-radius: 8px; box-shadow: 0 4px 12px ${colors.tooltipBorder};`,
  };
}

/** Build animation config scaled to the number of date points. */
export function buildAnimationConfig(totalDates: number): Record<string, unknown> {
  return {
    animationDuration: 400,
    animationEasing: 'cubicOut',
    animationDelay: (idx: number) => (idx / totalDates) * 300,
  };
}

/** Build shared ECharts axis configuration. */
export function buildAxisConfig(
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
export function buildSimpleChartOption(
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
    tooltip: { ...buildTooltipConfig(colors) },
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
export function formatStackedTooltip(
  params: EChartsTooltipParam[],
  totalLabel: string,
  colors: ThemeColors
): string {
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
    // marker is ECharts-generated trusted HTML (colored circle SVG)
    html += `<td style="padding:1px 0">${String(p.marker)} ${escapeHtml(String(p.seriesName))}</td>`;
    html += `<td style="text-align:right;padding:1px 0 1px 16px;font-weight:600;font-variant-numeric:tabular-nums">${val}</td>`;
    html += `<td style="text-align:right;padding:1px 0 1px 4px;opacity:0.6;font-variant-numeric:tabular-nums">${pct}%</td>`;
    html += `</tr>`;
  }
  html += `</table>`;
  const dividerColor = colors.tooltipText.startsWith('#fff')
    ? 'rgba(255,255,255,0.2)'
    : `${colors.tooltipText}33`;
  html += `<div style="border-top:1px solid ${dividerColor};margin-top:6px;padding-top:6px;font-weight:700;text-align:right">${escapeHtml(totalLabel)}</div>`;
  return html;
}

/** Group entries by application, returning per-app name and date-amount map. */
export function groupEntriesByApp(
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
export function buildStackedChartOption(config: StackedChartConfig): Record<string, unknown> {
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
      formatter: (params: EChartsTooltipParam[]) => {
        const total = params.reduce((s, p) => s + (Number(p.value) || 0), 0);
        return formatStackedTooltip(params, totalLabelFn(total), colors);
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
