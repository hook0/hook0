<template>
  <a
    v-bind="{ ...$props, ...$attrs }"
    class="hook0-button"
    :class="{ loading: loading }"
    @click="onClick($event)"
    :disabled="loading || disabled"
    :href="_href"
  >
    <div class="hook0-button-left" v-if="hasSlot('left') && !loading">
      <slot name="left"></slot>
    </div>
    <slot v-if="!loading"></slot>
    <hook0-icon name="spinner" class="animate-spin" v-if="loading"></hook0-icon>
    <div class="hook0-button-right" v-if="hasSlot('right') && !loading">
      <slot name="right"></slot>
    </div>
  </a>
</template>

<script lang="ts">

import Hook0Icon from '@/components/Hook0Icon.vue';
import {RouteLocationRaw} from "vue-router";
import {defineComponent, PropType} from 'vue'

export default defineComponent({
  components: {
    Hook0Icon
  },
  // type inference enabled
  props: {
    loading: {
      type: Boolean,
      default: false,
    },
    // helper to let the button go to a specified route
    to: {
      type: Object as PropType<RouteLocationRaw>,
      required: false,
    },
    href: {
      type: String,
      required: false,
    },
    disabled: {
      type: Boolean,
      default: false,
      required: false,
    },
  },
  computed: {
    _href(): undefined | string {
      if (this.href) {
        return this.href;
      }

      if (!this.to) {
        return undefined;
      }

      // @ts-ignore
      const {href} = this.$router.resolve(this.to);
      return href; // for accessibility
    }
  },
  methods: {
    onClick(e: MouseEvent) {
      if (!this.href) {
        e.preventDefault();
      }

      if (this.loading || this.disabled) {
        // do nothing
        return;
      }

      if (!this._href) {
        // no href so bubble-up event
        this.$emit('click', e);
        return;
      }

      if (e.metaKey) {
        // support for power-user that want to open links in another tab
        window.open(this._href);
        return true;
      }

      // @ts-ignore
      this.$router.push(this.to).catch((err) => {
        console.error(err);
      });
    },
    hasSlot(name = 'default'): boolean {
      return !!this.$slots[name];
    }
  }
})
</script>

<style lang="scss" scoped>
.hook0-button {
  @apply select-none font-medium text-indigo-600 hover:text-indigo-500;

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

.hook0-button-left {
  @apply -ml-1 mb-1 h-5 w-5;
}

.hook0-button-right {
  @apply ml-3 -mr-1 h-5 w-5;
}
</style>
