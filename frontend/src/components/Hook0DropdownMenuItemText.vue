<template>
  <div v-bind="{ ...$props, ...$attrs }" class="px-4 py-2 text-sm" role="none">
    <slot></slot>
  </div>
</template>

<script lang="ts">
import {Options, Vue} from "vue-class-component";

@Options({
  components: {},
  props: {},
})
export default class Hook0DropdownMenuItemText extends Vue {
}
</script>

<style lang="scss" scoped></style>
