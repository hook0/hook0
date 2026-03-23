import { computed } from 'vue';
import { useUiStore, type ColorMode } from '@/stores/ui';
import { useMediaQuery } from '@vueuse/core';

export function useColorMode() {
  const uiStore = useUiStore();
  const prefersDark = useMediaQuery('(prefers-color-scheme: dark)');

  const isDark = computed(() => {
    return uiStore.colorMode === 'dark' || (uiStore.colorMode === 'system' && prefersDark.value);
  });

  function toggle(): void {
    if (isDark.value) {
      uiStore.setColorMode('light');
    } else {
      uiStore.setColorMode('dark');
    }
  }

  function set(mode: ColorMode): void {
    uiStore.setColorMode(mode);
  }

  return {
    colorMode: computed(() => uiStore.colorMode),
    isDark,
    toggle,
    set,
  };
}
