<template>
  <hook0-alert type="alert">
    <template #title>{{ error.title }} ({{ error.status }} {{ error.id }})</template>
    <template #description>
      <hook0-text>{{ error.detail }}</hook0-text>
      <br/>
      <hook0-text class="def">{{ error.type }}</hook0-text>
    </template>
  </hook0-alert>
</template>

<script lang="ts">
import {Options, Vue} from "vue-class-component";

const hasOwn = (obj: any, attr: string) => Object.prototype.hasOwnProperty.call(obj, attr);

@Options({
  components: {},
  props: {
    error: {
      type: Object,
      required: true,
      validator(problem: any) {
        return hasOwn(problem, 'title') && hasOwn(problem, 'detail');
      }
    },
  },
})
export default class Hook0Error extends Vue {
};
</script>

<style lang="scss" scoped>
</style>
