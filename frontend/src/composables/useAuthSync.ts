import { onBeforeUnmount } from 'vue';

/**
 * Sync auth state across browser tabs and handle visibility changes.
 *
 * @param storageKey - localStorage key to watch
 * @param onCleared - called when another tab clears the auth state
 * @param onUpdated - called when another tab writes new tokens
 * @param onVisible - called when this tab becomes visible again
 */
export function useAuthSync(
  storageKey: string,
  onCleared: () => void,
  onUpdated: () => void,
  onVisible: () => void
) {
  function handleStorage(e: StorageEvent) {
    if (e.key !== storageKey) return;
    if (e.newValue === null) {
      onCleared();
    } else {
      onUpdated();
    }
  }

  function handleVisibility() {
    if (document.visibilityState === 'visible') {
      onVisible();
    }
  }

  window.addEventListener('storage', handleStorage);
  document.addEventListener('visibilitychange', handleVisibility);

  onBeforeUnmount(() => {
    window.removeEventListener('storage', handleStorage);
    document.removeEventListener('visibilitychange', handleVisibility);
  });
}
