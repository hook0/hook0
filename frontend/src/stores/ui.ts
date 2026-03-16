import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { useMediaQuery } from '@vueuse/core';

const LOCAL_STORAGE_KEY_THEME = 'hook0-theme';
const LOCAL_STORAGE_KEY_RECENT_WORKSPACES = 'hook0-recent-workspaces';
const MAX_RECENT_PAGES = 5;
const MAX_RECENT_WORKSPACES = 5;

export type ColorMode = 'light' | 'dark' | 'system';

export interface RecentPage {
  path: string;
  name: string;
  timestamp: number;
}

export interface RecentWorkspace {
  organizationId: string;
  organizationName: string;
  applicationId: string | null;
  applicationName: string | null;
  timestamp: number;
}

export const useUiStore = defineStore('ui', () => {
  // Dark mode
  const prefersDark = useMediaQuery('(prefers-color-scheme: dark)');
  const storedTheme = window.localStorage.getItem(LOCAL_STORAGE_KEY_THEME) as ColorMode | null;
  const colorMode = ref<ColorMode>(storedTheme ?? 'system');

  function applyColorMode(): void {
    const isDark =
      colorMode.value === 'dark' || (colorMode.value === 'system' && prefersDark.value);
    document.documentElement.classList.toggle('dark', isDark);
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
    const existing = recentPages.value.findIndex((p) => p.path === path);
    if (existing >= 0) {
      recentPages.value.splice(existing, 1);
    }
    recentPages.value.unshift({ path, name, timestamp: Date.now() });
    if (recentPages.value.length > MAX_RECENT_PAGES) {
      recentPages.value = recentPages.value.slice(0, MAX_RECENT_PAGES);
    }
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

    const existing = recentWorkspaces.value.findIndex((w) => {
      const existingKey = w.applicationId
        ? `${w.organizationId}:${w.applicationId}`
        : w.organizationId;
      return existingKey === key;
    });

    if (existing >= 0) {
      recentWorkspaces.value.splice(existing, 1);
    }

    recentWorkspaces.value.unshift({
      organizationId,
      organizationName,
      applicationId,
      applicationName,
      timestamp: Date.now(),
    });

    if (recentWorkspaces.value.length > MAX_RECENT_WORKSPACES) {
      recentWorkspaces.value = recentWorkspaces.value.slice(0, MAX_RECENT_WORKSPACES);
    }

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
