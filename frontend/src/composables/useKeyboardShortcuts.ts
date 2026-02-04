import { onMounted, onUnmounted } from 'vue';
import { useUiStore } from '@/stores/ui';

export function useKeyboardShortcuts() {
  const uiStore = useUiStore();

  function handleKeydown(e: KeyboardEvent): void {
    // cmd+k / ctrl+k - Command palette
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      uiStore.toggleCommandPalette();
      return;
    }

    // Escape - Close overlays
    if (e.key === 'Escape') {
      if (uiStore.commandPaletteOpen) {
        uiStore.closeCommandPalette();
        return;
      }
      if (uiStore.mobileDrawerOpen) {
        uiStore.toggleMobileDrawer();
        return;
      }
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown);
  });
}
