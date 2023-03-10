<template>
  <a
    class="hook0-button"
    :class="{ loading: loadingStatus, 'hook0-button-split': hasSlot('right') || hasSlot('left') }"
    v-bind="omit({ ...$props, ...$attrs })"
    @click="onClick($event)"
    :disabled="loadingStatus || disabled"
    :href="_href"
  >
    <div class="hook0-button-left" v-if="hasSlot('left') && !loadingStatus">
      <slot name="left"></slot>
    </div>
    <div class="hook0-button-center">
      <slot v-if="!loadingStatus"></slot>
    </div>
    <div class="hook0-button-right" v-if="hasSlot('right') || loadingStatus">
      <hook0-icon name="spinner" spin class="animate-spin" v-if="loadingStatus"></hook0-icon>
      <slot name="right"></slot>
    </div>
  </a>
</template>

<script lang="ts">
import Hook0Icon from '@/components/Hook0Icon.vue';
import { RouteLocationRaw } from 'vue-router';
import { defineComponent, PropType } from 'vue';
import { omit } from 'ramda';

export default defineComponent({
  components: {
    Hook0Icon,
  },
  // type inference enabled
  props: {
    // Loading as a boolean
    loading: {
      default: false,
      validator: (value: any) => value instanceof Promise || typeof value === 'boolean',
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
  data() {
    return {
      loadingStatus: false,
    };
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
      const { href } = this.$router.resolve(this.to);
      return href; // for accessibility
    },
  },
  mounted() {
    this._forwardPromiseState();
  },
  updated() {
    this._forwardPromiseState();
  },
  methods: {
    _forwardPromiseState() {
      if (!((this.loading as any) instanceof Promise)) {
        this.loadingStatus = this.loading;
        return;
      }

      const setStatus = (state: boolean) => () => {
        if (!((this.loading as any) instanceof Promise)) {
          return;
        }

        this.loadingStatus = state;
      };

      setStatus(true)();
      // @ts-ignore
      // eslint-disable-next-line
      this.loading.finally(setStatus(false));
    },
    omit(props: Record<string, any>) {
      return omit(['onClick'], props);
    },
    onClick(e: MouseEvent) {
      if (!this.href) {
        e.preventDefault();
        e.stopImmediatePropagation();
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

      if (e.metaKey && this._href) {
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
    },
  },
});
</script>

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
