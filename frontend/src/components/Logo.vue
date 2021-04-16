<template>
  <div>Hook0</div>
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component'

@Options({})
export default class Logo extends Vue {};
</script>

<style scoped>
div{
  @apply text-white;
}
</style>
