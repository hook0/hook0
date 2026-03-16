<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { RouteLocationRaw, useRouter } from 'vue-router';

import Hook0DropdownOptions from './Hook0DropdownOptions';

interface Props {
  justify?: 'left' | 'right';
}
const props = defineProps<Props>();
const justify = computed(() => props.justify ?? 'right');

const show = ref(false);
const toggler = ref<HTMLElement | null>(null);
const dropdown = ref<HTMLElement | null>(null);

const router = useRouter();

defineSlots<{
  menu(props: Hook0DropdownOptions & { ariaExpanded: boolean; ariaHaspopup: 'true' }): unknown;
  dropdown(props: Hook0DropdownOptions): unknown;
}>();

function toggle(event: Event) {
  event.preventDefault();
  event.stopImmediatePropagation();

  if (show.value) {
    close();
  } else {
    open();
  }
}

function open() {
  show.value = true;
  void nextTick().then(() => {
    const panel = dropdown.value?.querySelector('[role="menu"]') as HTMLElement | null;
    if (panel) {
      panel.focus();
    }
  });
}

function close() {
  show.value = false;
  // Return focus to the trigger element
  const triggerButton = toggler.value?.querySelector(
    'button, [role="button"], a'
  ) as HTMLElement | null;
  if (triggerButton) {
    triggerButton.focus();
  }
}

function route(route: RouteLocationRaw) {
  close();
  return router.push(route);
}

function onClickOutside(event: MouseEvent) {
  if (
    show.value &&
    event.target !== toggler.value &&
    (event.target as HTMLElement).closest('.hook0-toggler') === null &&
    dropdown.value &&
    !dropdown.value.contains(event.target as Node)
  ) {
    close();
  }
}

function onKeydown(event: KeyboardEvent) {
  if (!show.value) {
    return;
  }

  if (event.key === 'Escape') {
    event.preventDefault();
    close();
    return;
  }

  const panel = dropdown.value?.querySelector('[role="menu"]') as HTMLElement | null;
  if (!panel) {
    return;
  }

  const items = Array.from(
    panel.querySelectorAll<HTMLElement>(
      '[role="menuitem"]:not([disabled]), a:not([disabled]), button:not([disabled])'
    )
  );

  if (items.length === 0) {
    return;
  }

  const currentIndex = items.indexOf(document.activeElement as HTMLElement);

  if (event.key === 'ArrowDown') {
    event.preventDefault();
    const nextIndex = currentIndex < items.length - 1 ? currentIndex + 1 : 0;
    items[nextIndex].focus();
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    const prevIndex = currentIndex > 0 ? currentIndex - 1 : items.length - 1;
    items[prevIndex].focus();
  }
}

onMounted(() => {
  document.addEventListener('click', onClickOutside, true);
  document.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  document.removeEventListener('click', onClickOutside, true);
  document.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <div :class="$attrs.class" class="hook0-dropdown">
    <div ref="toggler" class="hook0-toggler">
      <slot
        name="menu"
        :open="open"
        :close="close"
        :route="route"
        :toggle="toggle"
        :aria-expanded="show"
        :aria-haspopup="'true'"
      ></slot>
    </div>

    <div ref="dropdown">
      <transition name="ease">
        <div
          v-if="show"
          class="hook0-dropdown-panel"
          :class="[justify]"
          role="menu"
          aria-orientation="vertical"
          tabindex="-1"
        >
          <slot name="dropdown" :open="open" :close="close" :toggle="toggle" :route="route"></slot>
        </div>
      </transition>
    </div>
  </div>
</template>

<style scoped>
.hook0-dropdown {
  position: relative;
}

.hook0-dropdown .hook0-toggler {
  height: 100%;
}

.hook0-dropdown-panel {
  position: absolute;
  margin-top: 0.5rem;
  width: 20rem;
  border-radius: 0.375rem;
  box-shadow:
    0 10px 15px -3px rgba(0, 0, 0, 0.1),
    0 4px 6px -2px rgba(0, 0, 0, 0.05);
  background-color: var(--color-bg-primary);
  ring: 1px solid rgba(0, 0, 0, 0.05);
  outline: none;
  z-index: 50;
}

.hook0-dropdown-panel :deep(> * + *) {
  border-top: 1px solid var(--color-bg-secondary);
}

.hook0-dropdown-panel.left {
  transform-origin: top left;
  left: 0;
}

.hook0-dropdown-panel.right {
  transform-origin: top right;
  right: 0;
}

.hook0-dropdown.darkmode {
  background-color: var(--color-text-primary);
}

.hook0-dropdown.darkmode .hook0-dropdown-panel {
  background-color: var(--color-text-primary);
}
</style>
