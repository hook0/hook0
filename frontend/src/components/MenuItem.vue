<template>
  <a class="default" :class="{ active: active, inactive: !active }">
    <slot></slot>
    {{name}}
  </a>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component'

@Options({
  props:{
    name: {
      type: String,
      required: true
    },
    active: {
      type: Boolean,
      required: true
    },
  }
})
export default class MenuItem extends Vue {
};
</script>

<style scoped>
.default{
  @apply text-gray-300 text-white group flex items-center px-2 py-2 text-sm font-medium rounded-md;
}

.active{
  @apply bg-gray-900;
}

.inactive{
  @apply hover:text-white hover:bg-gray-700;
}
</style>
