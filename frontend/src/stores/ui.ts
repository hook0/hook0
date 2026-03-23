import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { useMediaQuery } from '@vueuse/core';

import { LOCAL_STORAGE_KEY_THEME, resolveIsDark } from '@/constants/theme';
export type { ColorMode } from '@/constants/theme';
import type { ColorMode } from '@/constants/theme';

const LOCAL_STORAGE_KEY_RECENT_WORKSPACES = 'hook0-recent-workspaces';
const MAX_RECENT_PAGES = 5;
const MAX_RECENT_WORKSPACES = 5;

export type RecentPage = {
  path: string;
  name: string;
  timestamp: number;
};

export type RecentWorkspace = {
  organizationId: string;
  organizationName: string;
  applicationId: string | null;
  applicationName: string | null;
  timestamp: number;
};

export const useUiStore = defineStore('ui', () => {
  // Dark mode
  const prefersDark = useMediaQuery('(prefers-color-scheme: dark)');
  const storedTheme = window.localStorage.getItem(LOCAL_STORAGE_KEY_THEME) as ColorMode | null;
  const colorMode = ref<ColorMode>(storedTheme ?? 'system');

  function applyColorMode(): void {
    document.documentElement.classList.toggle(
      'dark',
      resolveIsDark(colorMode.value, prefersDark.value)
    );
  }

  function setColorMode(mode: ColorMode): void {
    colorMode.value = mode;
    window.localStorage.setItem(LOCAL_STORAGE_KEY_THEME, mode);
    applyColorMode();
  }

  // Apply on init and watch system preference changes
  applyColorMode();
  watch(prefersDark, () => {
    if (colorMode.value === 'system') {
      applyColorMode();
    }
  });

  // Sidebar
  const sidebarCollapsed = ref(false);
  const mobileSidebarOpen = ref(false);
  const mobileDrawerOpen = ref(false);

  function toggleSidebar(): void {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function toggleMobileSidebar(): void {
    mobileSidebarOpen.value = !mobileSidebarOpen.value;
  }

  function toggleMobileDrawer(): void {
    mobileDrawerOpen.value = !mobileDrawerOpen.value;
  }

  function closeMobileNav(): void {
    mobileSidebarOpen.value = false;
    mobileDrawerOpen.value = false;
  }

  function closeMobileDrawer(): void {
    mobileDrawerOpen.value = false;
  }

  // Command palette
  const commandPaletteOpen = ref(false);

  function openCommandPalette(): void {
    commandPaletteOpen.value = true;
  }

  function closeCommandPalette(): void {
    commandPaletteOpen.value = false;
  }

  function toggleCommandPalette(): void {
    commandPaletteOpen.value = !commandPaletteOpen.value;
  }

  // Keyboard shortcuts cheat sheet
  const shortcutsCheatSheetOpen = ref(false);

  function openShortcutsCheatSheet(): void {
    shortcutsCheatSheetOpen.value = true;
  }

  function closeShortcutsCheatSheet(): void {
    shortcutsCheatSheetOpen.value = false;
  }

  function toggleShortcutsCheatSheet(): void {
    shortcutsCheatSheetOpen.value = !shortcutsCheatSheetOpen.value;
  }

  // Recent pages
  const recentPages = ref<RecentPage[]>(
    JSON.parse(window.localStorage.getItem('hook0-recent-pages') ?? '[]') as RecentPage[]
  );

  function addRecentPage(path: string, name: string): void {
    recentPages.value = [
      { path, name, timestamp: Date.now() },
      ...recentPages.value.filter((p) => p.path !== path),
    ].slice(0, MAX_RECENT_PAGES);
    window.localStorage.setItem('hook0-recent-pages', JSON.stringify(recentPages.value));
  }

  // Recent workspaces (org/app combinations)
  const recentWorkspaces = ref<RecentWorkspace[]>(
    JSON.parse(
      window.localStorage.getItem(LOCAL_STORAGE_KEY_RECENT_WORKSPACES) ?? '[]'
    ) as RecentWorkspace[]
  );

  function addRecentWorkspace(
    organizationId: string,
    organizationName: string,
    applicationId: string | null,
    applicationName: string | null
  ): void {
    // Create a unique key for comparison (org + app combo)
    const key = applicationId ? `${organizationId}:${applicationId}` : organizationId;

    recentWorkspaces.value = [
      {
        organizationId,
        organizationName,
        applicationId,
        applicationName,
        timestamp: Date.now(),
      },
      ...recentWorkspaces.value.filter((w) => {
        const existingKey = w.applicationId
          ? `${w.organizationId}:${w.applicationId}`
          : w.organizationId;
        return existingKey !== key;
      }),
    ].slice(0, MAX_RECENT_WORKSPACES);

    window.localStorage.setItem(
      LOCAL_STORAGE_KEY_RECENT_WORKSPACES,
      JSON.stringify(recentWorkspaces.value)
    );
  }

  function clearRecentWorkspaces(): void {
    recentWorkspaces.value = [];
    window.localStorage.removeItem(LOCAL_STORAGE_KEY_RECENT_WORKSPACES);
  }

  const effectiveColorMode = computed<'light' | 'dark'>(() => {
    if (colorMode.value === 'system') {
      return prefersDark.value ? 'dark' : 'light';
    }
    return colorMode.value;
  });

  function toggleColorMode(): void {
    setColorMode(effectiveColorMode.value === 'dark' ? 'light' : 'dark');
  }

  return {
    // Dark mode
    colorMode,
    setColorMode,
    effectiveColorMode,
    toggleColorMode,
    // Sidebar
    sidebarCollapsed,
    mobileSidebarOpen,
    mobileDrawerOpen,
    toggleSidebar,
    toggleMobileSidebar,
    toggleMobileDrawer,
    closeMobileNav,
    closeMobileDrawer,
    // Command palette
    commandPaletteOpen,
    openCommandPalette,
    closeCommandPalette,
    toggleCommandPalette,
    // Keyboard shortcuts cheat sheet
    shortcutsCheatSheetOpen,
    openShortcutsCheatSheet,
    closeShortcutsCheatSheet,
    toggleShortcutsCheatSheet,
    // Recent pages
    recentPages,
    addRecentPage,
    // Recent workspaces
    recentWorkspaces,
    addRecentWorkspace,
    clearRecentWorkspaces,
  };
});
