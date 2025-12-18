import { ref, onBeforeUnmount } from 'vue';

/**
 * Composable for mouse-tracking card glow effect.
 * Uses requestAnimationFrame throttling and proper cleanup on unmount.
 */
export function useCardGlow() {
  const cardRef = ref<HTMLElement | null>(null);
  const mouseX = ref<string>('50%');
  const mouseY = ref<string>('50%');
  let rafId: number | null = null;

  function handleMouseMove(event: MouseEvent) {
    if (!cardRef.value) return;
    if (rafId !== null) return;

    rafId = requestAnimationFrame(() => {
      if (!cardRef.value) {
        rafId = null;
        return;
      }
      const rect = cardRef.value.getBoundingClientRect();
      mouseX.value = `${event.clientX - rect.left}px`;
      mouseY.value = `${event.clientY - rect.top}px`;
      rafId = null;
    });
  }

  onBeforeUnmount(() => {
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
  });

  return {
    cardRef,
    mouseX,
    mouseY,
    handleMouseMove,
  };
}
