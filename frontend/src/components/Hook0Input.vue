<template>
  <div :class="$attrs.class">
    <input
      ref="ipt"
      v-bind="{ ...omit(['class'], $props), ...$attrs }"
      class="hook0-input"
      :value="$attrs.modelValue"
      @input="$emit('update:modelValue', $event.target.value)"
    />

    <div v-if="hasSlot('helpText')">
      <hook0-text class="helpText">
        <slot name="helpText"></slot>
      </hook0-text>
    </div>
  </div>
</template>

<script lang="ts">
import { omit } from 'ramda';
import { Vue, Options } from 'vue-class-component';

@Options({
  name: 'hook0-input',
  inheritAttrs: false,
  props: {
    helpText: {
      type: String,
      required: false,
    },
  },
})
export default class Hook0Input extends Vue {
  helpText?: string = undefined;

  hasSlot(name = 'default'): boolean {
    return !!this.$slots[name];
  }

  mounted() {
    this._internalState();
  }

  updated() {
    this._internalState();
  }

  _internalState() {
    // checkbox needs special care
    if (this.$attrs.type === 'checkbox' && typeof this.$attrs.value === 'boolean') {
      // @ts-ignore
      this.$refs.ipt.checked = this.$attrs.value;
    }
  }

  omit = omit;
}
</script>

<style lang="scss" scoped>
.hook0-input {
  @apply block w-full shadow-sm focus:ring-indigo-500 focus:border-indigo-500 text-sm border-gray-300 rounded-md;
}

.hook0-input[type='checkbox'] {
  @apply focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded;
}
</style>
