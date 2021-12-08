<template>
  <div v-html="svg" />
</template>

<script lang="ts">
import { icon, IconName } from '@fortawesome/fontawesome-svg-core';

export default {
  props: {
    name: {
      type: String,
      required: true,
    },
  },
  name: 'hook0-icon',
  data() {
    return {
      svg: icon({ prefix: 'fas', iconName: this.name as IconName }).html[0],
    };
  },
};
</script>

<style lang="scss" scoped></style>
