export const LOCAL_STORAGE_KEY_THEME = 'hook0-theme';

export type ColorMode = 'light' | 'dark' | 'system';

/**
 * Resolve whether dark mode should be active, given a stored theme value
 * and the system preference. Shared between main.ts (pre-mount) and ui store.
 */
export function resolveIsDark(stored: string | null, prefersDark: boolean): boolean {
  return stored === 'dark' || (stored !== 'light' && prefersDark);
}
