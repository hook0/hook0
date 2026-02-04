<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { RouteLocationRaw, useRouter } from 'vue-router';

import Hook0DropdownOptions from './Hook0DropdownOptions';

interface Props {
  justify?: 'left' | 'right';
}
const props = defineProps<Props>();
const justify = computed(() => props.justify ?? 'right');

const show = ref(false);
const toggler = ref(null);
const dropdown = ref<HTMLElement | null>(null);

const router = useRouter();

defineSlots<{
  menu(props: Hook0DropdownOptions): unknown;
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
}

function close() {
  show.value = false;
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

onMounted(() => {
  document.addEventListener('click', onClickOutside, true);
});

onBeforeUnmount(() => {
  document.removeEventListener('click', onClickOutside, true);
});
</script>

<template>
  <div :class="$attrs.class" class="hook0-dropdown">
    <div ref="toggler" class="hook0-toggler">
      <slot name="menu" :open="open" :close="close" :route="route" :toggle="toggle"></slot>
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
  background-color: #ffffff;
  ring: 1px solid rgba(0, 0, 0, 0.05);
  outline: none;
  z-index: 50;
}

.hook0-dropdown-panel :deep(> * + *) {
  border-top: 1px solid #f3f4f6;
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
  background-color: #111827;
}

.hook0-dropdown.darkmode .hook0-dropdown-panel {
  background-color: #111827;
}
</style>
