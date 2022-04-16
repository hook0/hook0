<template>
  <div>
    <hook0-icon name="link"></hook0-icon>
    <hook0-text class="ml-1">Hook0</hook0-text>
  </div>
</template>

<script lang="ts">
import {Vue, Options} from 'vue-class-component'

@Options({})
export default class Logo extends Vue {
};
</script>

<style scoped>
div {
  @apply text-white;
}
</style>
