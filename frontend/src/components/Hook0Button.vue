<script setup lang="ts">
import { RouteLocationRaw, useRouter } from 'vue-router';
import { ref, computed, onMounted, onUpdated, useSlots } from 'vue';
import { omit } from 'ramda';

import Hook0Icon from '@/components/Hook0Icon.vue';

interface Props {
  loading?: boolean | Promise<unknown>;
  to?: RouteLocationRaw;
  href?: string;
  disabled?: boolean;
}
const router = useRouter();
const props = defineProps<Props>();
const emit = defineEmits(['click']);
defineSlots<{
  default(): unknown;
  left(): unknown;
  right(): unknown;
}>();

const href = computed(() => {
  if (props.href) {
    return props.href;
  }

  if (!props.to) {
    return undefined;
  }

  const { href } = router.resolve(props.to);
  return href; // for accessibility
});
const loading = computed(() => props.loading ?? false);

const loadingStatus = ref(false);

function _forwardPromiseState() {
  if (!(loading.value instanceof Promise)) {
    loadingStatus.value = loading.value;
    return;
  }

  const setStatus = (state: boolean) => () => {
    if (!(loading.value instanceof Promise)) {
      return;
    }

    loadingStatus.value = state;
  };

  setStatus(true)();
  loading.value.finally(setStatus(false));
}

function omitOnClick(props: Record<string, unknown>) {
  return omit(['onClick'], props);
}

function onClick(e: MouseEvent) {
  if (!props.href) {
    e.preventDefault();
    e.stopImmediatePropagation();
  }

  if (props.loading || props.disabled) {
    // do nothing
    return;
  }

  if (!href.value) {
    // no href so bubble-up event
    emit('click', e);
    return;
  }

  if (e.metaKey && href.value) {
    // support for power-user that want to open links in another tab
    window.open(href.value);
    return true;
  }

  if (props.to) {
    router.push(props.to).catch((err) => {
      console.error(err);
    });
  }
}
function hasSlot(name: string): boolean {
  return !!useSlots()[name];
}

onMounted(() => {
  _forwardPromiseState();
});

onUpdated(() => {
  _forwardPromiseState();
});
</script>

<template>
  <a
    class="hook0-button"
    :class="{ loading: loadingStatus, 'hook0-button-split': hasSlot('right') || hasSlot('left') }"
    v-bind="omitOnClick({ ...$props, ...$attrs })"
    :disabled="loadingStatus || disabled"
    :href="href"
    @click="onClick($event)"
  >
    <div v-if="hasSlot('left') && !loadingStatus" class="hook0-button-left">
      <slot name="left"></slot>
    </div>
    <div class="hook0-button-center">
      <slot v-if="!loadingStatus"></slot>
    </div>
    <div v-if="hasSlot('right') || loadingStatus" class="hook0-button-right">
      <Hook0Icon v-if="loadingStatus" name="spinner" spin class="animate-spin"></Hook0Icon>
      <slot name="right"></slot>
    </div>
  </a>
</template>

<style lang="scss" scoped>
.hook0-button {
  @apply select-none cursor-pointer font-medium text-indigo-600 hover:text-indigo-500;

  .hook0-button-left {
    @apply inline-block;
  }

  .hook0-button-center {
    @apply inline-block;
  }

  .hook0-button-right {
    @apply inline-block;
  }

  &.dropdown {
    @apply max-w-lg block w-full sm:max-w-xs cursor-pointer py-4 pl-4 pr-4;
  }

  /** must be after dropdown **/
  &.hook0-button-split {
    @apply flex justify-between items-stretch;

    .hook0-button-left {
      @apply justify-self-start self-center;
    }

    .hook0-button-center {
      @apply justify-self-start self-center;
    }

    .hook0-button-right {
      @apply justify-self-end self-center;
    }
  }

  &.link {
    @apply hover:bg-indigo-100 hover:text-gray-900 text-gray-700 block mb-0 px-4 py-2 text-sm;

    &.darkmode {
      @apply hover:bg-gray-800 hover:text-gray-500;
    }
  }

  &.primary {
    @apply inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 cursor-pointer;
  }

  &.secondary {
    @apply inline-flex items-center px-4 py-2 border border-transparent text-base font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 cursor-pointer;
  }

  &.danger {
    @apply inline-flex items-center px-4 py-2 border border-transparent text-base font-medium rounded-md text-red-700 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 cursor-pointer;
  }

  &.white {
    @apply inline-flex items-center justify-center px-4 py-2 bg-white text-sm font-medium border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 outline-none cursor-pointer;

    &.center {
      @apply rounded-none;
    }

    &.left {
      @apply rounded-none rounded-l-lg;
    }

    &.right {
      @apply rounded-none rounded-r-lg;
    }

    &.active {
      @apply bg-gray-50 border-gray-300;
    }
  }

  &[disabled='disabled'] {
    @apply opacity-20 transition-all;
  }
}
</style>
