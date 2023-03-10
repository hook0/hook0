<template>
  <hook0-button
    class="link"
    :class="$attrs.class"
    v-bind="{ ...$props, ...$attrs }"
    role="menuitem"
    tabindex="-1"
    id="menu-item-0"
    @click="$emit('click', $event)"
  >
    <slot></slot>
  </hook0-button>
</template>

<script lang="ts">
import { Options, Vue } from 'vue-class-component';

@Options({
  components: {},
  props: {},
})
export default class Hook0DropdownMenuItemLink extends Vue {}
</script>

<style lang="scss" scoped></style>
