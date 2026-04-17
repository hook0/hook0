import { computed, type ComputedRef } from 'vue';
import { useColorMode } from '@/composables/useColorMode';

// Design-system palette resolved from CSS custom properties.
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

function readThemeColors(): ThemeColors {
  const style = getComputedStyle(document.documentElement);
  const css = (name: string, fallback: string): string =>
    style.getPropertyValue(name).trim() || fallback;
  return {
    primary: css('--color-primary', '#22c55e'),
    primaryLight: css('--color-primary-light', '#86efac'),
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

// CSS custom properties are not reactive in Vue. We depend on isDark
// (the source of truth, not the DOM side-effect) and re-read on toggle.
export function useThemeColors(): ComputedRef<ThemeColors> {
  const { isDark } = useColorMode();
  return computed(() => {
    // Touch isDark so Vue re-runs this on theme toggle; getComputedStyle is opaque to it.
    void isDark.value;
    return readThemeColors();
  });
}
