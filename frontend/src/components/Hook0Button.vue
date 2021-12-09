<template>
  <button
    :class="{ 'hook0-button': true, loading: loading }"
    @click="onClick($event)"
    :disabled="loading"
  >
    <div class="hook0-button-left" v-if="hasSlot('left') && !loading">
      <slot name="left"></slot>
    </div>
    <slot v-if="!loading"></slot>
    <hook0-icon name="spinner" class="animate-spin" v-if="loading"></hook0-icon>
    <div class="hook0-button-right" v-if="hasSlot('right') && !loading">
      <slot name="right"></slot>
    </div>
  </button>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component';
import Hook0Icon from '@/components/Hook0Icon.vue';

@Options({
  name: 'hook0-button',
  props: {
    loading: {
      type: Boolean,
      default: false,
    },
  },
  components: {
    Hook0Icon,
  },
})
export default class Hook0Button extends Vue {
  loading = false;

  onClick(e: MouseEvent) {
    if (this.loading) {
      return;
    }
    this.$emit('click', e);
  }

  hasSlot(name = 'default'): boolean {
    return !!this.$slots[name];
  }
};
</script>

<style lang="scss" scoped>
.hook0-button {
  &.primary {
    @apply inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500;
  }

  &.secondary {
    @apply inline-flex items-center px-4 py-2 border border-transparent text-base font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500;
  }

  &.danger {
    @apply inline-flex items-center justify-center px-4 py-2 border border-transparent font-medium rounded-md text-red-700 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:text-sm;
  }

  &.white {
    @apply inline-flex items-center justify-center px-4 py-2 bg-white text-sm font-medium border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 outline-none;

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

.hook0-button-left {
  @apply -ml-1 mr-3 h-5 w-5;
}

.hook0-button-right {
  @apply ml-3 -mr-1 h-5 w-5;
}
</style>
