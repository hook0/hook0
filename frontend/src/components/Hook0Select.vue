<template>
  <select class="hook0-select" v-bind="{ ...omitOptions($props), ...$attrs }" ref="select" @input="sendEvent()">
    <optgroup :label="group.label" v-for="group in groupedOptions" :key="group.label">
      <option :value="option.value" v-for="option in group.options" :key="option.value">{{ option.label }}</option>
    </optgroup>

    <option :value="option.value" v-for="option in simpleOptions" :key="option.value">{{ option.label }}</option>
  </select>
</template>

<script lang="ts">
/* eslint-disable no-prototype-builtins */
import {Options, Vue, VueWithProps} from "vue-class-component";
import {Hook0SelectGroupedOption, Hook0SelectSingleOption} from "@/components/Hook0Select";
import {omit} from "ramda";

function isSimpleOption(option: Hook0SelectSingleOption) {
  return option.hasOwnProperty('value') && option.hasOwnProperty('label');
}

function isGroupedOptions(option: Hook0SelectGroupedOption) {
  return Array.isArray(option.options) && option.options.every(isSimpleOption);
}


@Options({
  name: 'hook0-select',
  inheritAttrs: false,
  props: {
    options: {
      required: true,
      type: Array,
      validator: (options: Array<Hook0SelectSingleOption | Hook0SelectGroupedOption>) => {
        // eslint-disable-next-line no-prototype-builtins
        return options.every(option => isSimpleOption(option as Hook0SelectSingleOption) || isGroupedOptions(option as Hook0SelectGroupedOption));
      },
    },
  },
  components: {},
  watch: {
    options: {
      handler: function (options: (Hook0SelectSingleOption | Hook0SelectGroupedOption)[]) {
        // eslint-disable-next-line
        this.groupedOptions = (options as Hook0SelectGroupedOption[]).filter(isGroupedOptions);
        // eslint-disable-next-line
        this.simpleOptions = (options as Hook0SelectSingleOption[]).filter(isSimpleOption);
      },
      immediate: true,
    },
  },
})



export default class Hook0Select extends Vue {
  private groupedOptions: Hook0SelectGroupedOption[] = [];
  private simpleOptions: Hook0SelectSingleOption[] = [];

  omitOptions($props: VueWithProps<any>) {
    return omit(['options'], $props);
  }

  mounted() {
    this.sendEvent();
  }

  updated() {
    this.sendEvent();
  }

  sendEvent() {
    // @ts-ignore
    this.$emit('update:modelValue', this.$refs.select.value);
  }
}

</script>

<style lang="scss" scoped>
.hook0-select {
  @apply block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md;

  &.width-small {
    @apply w-32;
  }
}
</style>
