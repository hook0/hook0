<template>
  <span :title="title">
    <font-awesome-icon
      fixed-width
      v-bind="{ ...$props, ...$attrs }"
      :icon="['fas'].concat(names_std)"
    ></font-awesome-icon>
  </span>
</template>

<script lang="ts">
// Search for icons here: https://fontawesome.com/search?m=free&s=solid
// and don't forgot to import in main.ts (see "library.add(..."), the icon you want in order to keep the js bundle size small
import { Options, Vue, VueWithProps } from 'vue-class-component';

@Options({
  props: {
    name: {
      type: String,
      required: true,
      validator(val: string) {
        return val && val.length > 0;
      },
    },
    title: {
      type: String,
      required: false,
    },
  },
  computed: {
    names_std: function () {
      return (this as VueWithProps<{ name: string }>).name
        .split(' ')
        .map((x) => x.replace(/^fa-/, ''));
    },
  },
})
export default class Hook0Icon extends Vue {}
</script>

<style lang="scss" scoped>
.darkmode {
  @apply text-gray-200;
}
</style>
