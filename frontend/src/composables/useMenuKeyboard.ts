import type { Ref } from 'vue';

export function useMenuKeyboard(containerRef: Ref<HTMLElement | null>, onClose: () => void) {
  function getMenuItems(): HTMLElement[] {
    if (!containerRef.value) return [];
    return Array.from(containerRef.value.querySelectorAll<HTMLElement>('[role="menuitem"]'));
  }

  function focusItem(items: HTMLElement[], index: number) {
    const clamped = Math.max(0, Math.min(index, items.length - 1));
    items[clamped]?.focus();
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    const items = getMenuItems();
    if (items.length === 0) return;

    const currentIndex = items.indexOf(document.activeElement as HTMLElement);

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        focusItem(items, currentIndex < 0 ? 0 : currentIndex + 1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        focusItem(items, currentIndex < 0 ? items.length - 1 : currentIndex - 1);
        break;
      case 'Home':
        event.preventDefault();
        focusItem(items, 0);
        break;
      case 'End':
        event.preventDefault();
        focusItem(items, items.length - 1);
        break;
      case 'Escape':
        event.preventDefault();
        onClose();
        break;
    }
  }

  return { handleMenuKeydown };
}
