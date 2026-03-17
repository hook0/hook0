import { type Ref, onMounted, onBeforeUnmount } from 'vue';

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
  '[contenteditable]',
  'details > summary',
].join(', ');

export function useFocusTrap(
  containerRef: Ref<HTMLElement | null>,
  options?: { onEscape?: () => void }
) {
  let previouslyFocusedElement: HTMLElement | null = null;

  function getFocusableElements(): HTMLElement[] {
    if (!containerRef.value) return [];
    return Array.from(containerRef.value.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape' && options?.onEscape) {
      event.preventDefault();
      event.stopPropagation();
      options.onEscape();
      return;
    }

    if (event.key === 'Tab') {
      const focusableElements = getFocusableElements();
      if (focusableElements.length === 0) {
        event.preventDefault();
        return;
      }

      const firstElement = focusableElements[0];
      const lastElement = focusableElements[focusableElements.length - 1];

      if (event.shiftKey) {
        if (document.activeElement === firstElement) {
          event.preventDefault();
          lastElement.focus();
        }
      } else {
        if (document.activeElement === lastElement) {
          event.preventDefault();
          firstElement.focus();
        }
      }
    }
  }

  function activate(): void {
    previouslyFocusedElement = document.activeElement as HTMLElement | null;
    const focusableElements = getFocusableElements();
    if (focusableElements.length > 0) {
      focusableElements[0].focus();
    }
  }

  function deactivate(): void {
    previouslyFocusedElement?.focus();
    previouslyFocusedElement = null;
  }

  if (options?.onEscape) {
    onMounted(() => {
      document.addEventListener('keydown', handleKeydown);
    });

    onBeforeUnmount(() => {
      document.removeEventListener('keydown', handleKeydown);
    });
  }

  return { activate, deactivate, handleKeydown };
}
