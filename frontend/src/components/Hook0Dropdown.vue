<script setup lang="ts">
import { computed, ref } from 'vue';
import { directive as vClickOutsideElement } from 'vue-click-outside-element';

import Hook0DropdownOptions from './Hook0DropdownOptions';

interface Props {
  justify?: 'left' | 'right';
}
const props = defineProps<Props>();
const justify = computed(() => props.justify ?? 'right');

const show = ref(false);
const toggler = ref(null);

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

function onClickOutside(event: Event) {
  if (
    show.value &&
    event.target !== toggler.value &&
    (event.target as HTMLElement).closest('.hook0-toggler') === null
  ) {
    close();
  }
}
</script>

<template>
  <div :class="$attrs.class" class="hook0-dropdown">
    <div ref="toggler" class="hook0-toggler">
      <slot name="menu" :open="open" :close="close" :toggle="toggle"></slot>
    </div>

    <div ref="dropdown" v-click-outside-element="onClickOutside">
      <transition name="ease">
        <div
          v-if="show"
          class="hook0-dropdown-panel"
          :class="[justify]"
          role="menu"
          aria-orientation="vertical"
          tabindex="-1"
        >
          <slot name="dropdown" :open="open" :close="close" :toggle="toggle"></slot>
        </div>
      </transition>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.hook0-dropdown {
  @apply relative;

  .hook0-toggler {
    @apply h-full;
  }

  .hook0-dropdown-panel {
    &.left {
      @apply origin-top-left absolute left-0 mt-2 w-56 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 divide-y divide-gray-100 focus:outline-none w-80 z-50;
    }

    &.right {
      @apply origin-top-right absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 divide-y divide-gray-100 focus:outline-none w-80 z-50;
    }
  }

  &.dropdown-right {
    .hook0-dropdown-panel {
    }
  }

  &.darkmode {
    @apply bg-gray-900;

    .hook0-dropdown-panel {
      @apply bg-gray-900;
    }
  }
}
</style>
