<template>
  <div>
    <input v-bind="{ ...$props, ...$attrs }" class="hook0-input" @input="$emit('update:modelValue', $event.target.value)" />
    <p class="hook0-input--help-text" v-if="hasSlot('helpText')">
      <slot name="helpText"></slot>
    </p>
  </div>
</template>

<script>
export default {
  name: 'hook0-input',
  inheritAttrs: false,
  props: {
    helpText: {
      type: String,
      required: false,
    },
  },
  components: {},
  data() {
    return {};
  },
  methods: {
    hasSlot(name = 'default') {
      return !!this.$slots[name];
    },
  },
  mounted() {},
  computed: {},
};
</script>

<style lang="scss" scoped>
.hook0-input {
  @apply max-w-lg block w-full shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:max-w-xs sm:text-sm border-gray-300 rounded-md;
}

.hook0-input--help-text {
  @apply mt-2 text-sm text-gray-500;
}
</style>
