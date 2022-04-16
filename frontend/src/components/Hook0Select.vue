<template>
  <select v-bind="{ ...$props, ...$attrs }" class="hook0-select" ref="select" @input="sendEvent()">
    <optgroup :label="group.label" v-for="group in groupedOptions" :key="group.label">
      <option :value="option.value" v-for="option in group.options" :key="option.value">{{ option.label }}</option>
    </optgroup>

    <option :value="option.value" v-for="option in simpleOptions" :key="option.value">{{ option.label }}</option>
  </select>
</template>

<script>
/* eslint-disable no-prototype-builtins */
import {Vue} from "vue-class-component";

function isSimpleOption(option) {
  return option.hasOwnProperty('value') && option.hasOwnProperty('label');
}

function isGroupedOptions(option) {
  return Array.isArray(option.options) && option.options.every(isSimpleOption);
}

export default {
  name: 'hook0-select',
  props: {
    options: {
      required: true,
      type: Array,
      validator: options => {
        // eslint-disable-next-line no-prototype-builtins
        return options.every(option => isSimpleOption(option) || isGroupedOptions(option));
      },
    },
  },
  components: {},
  data() {
    return {
      groupedOptions: [],
      simpleOptions: [],
    };
  },
  watch: {
    options: {
      handler: function (options) {
        this.groupedOptions = options.filter(isGroupedOptions);
        this.simpleOptions = options.filter(isSimpleOption);
      },
      immediate: true,
    },
  },
  methods: {
    sendEvent() {
      this.$emit('input', this.$refs.select.value);
    },
  },
  mounted() {
    this.sendEvent();
  },
  updated() {
    this.sendEvent();
  },
  computed: {},
};
</script>

<style lang="scss" scoped>
.hook0-select {
  @apply block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md;
}
</style>
