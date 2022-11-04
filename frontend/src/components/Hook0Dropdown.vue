<template>
  <div :class="$attrs.class" class="hook0-dropdown">
    <div ref="toggler" class="hook0-toggler">
      <slot name="menu" :open="open" :close="close" :toggle="toggle"></slot>
    </div>

    <div ref="dropdown" v-click-outside="vcoConfig">
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

<script lang="ts">
import { Options, Vue } from 'vue-class-component';
import Hook0Icon from '@/components/Hook0Icon.vue';

@Options({
  components: { Hook0Icon },
  props: {
    justify: {
      type: String,
      default: 'right',
      validator(val: string) {
        return ['left', 'right'].includes(val);
      },
    },
  },
})
export default class Hook0Dropdown extends Vue {
  show = false;

  vcoConfig!: {
    handler: any;
    middleware: any;
  };

  data() {
    return {
      vcoConfig: {
        handler: this.onClickOutside.bind(this),
        middleware: this.onClickOutsideCheck.bind(this),
      },
    };
  }

  toggle(event: Event) {
    event.preventDefault();
    event.stopImmediatePropagation();

    if (this.show) {
      this.close();
    } else {
      this.open();
    }
  }

  open() {
    this.show = true;
  }

  close() {
    this.show = false;
  }

  onClickOutsideCheck(event: Event) {
    return (
      this.show &&
      event.target !== this.$refs.toggler &&
      (event.target as HTMLElement).closest('.hook0-toggler') === null
    );
  }

  onClickOutside(event: Event) {
    this.close();
  }
}
</script>

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
}
</style>
