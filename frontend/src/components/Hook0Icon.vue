<template>
  <div v-html="svg" />
</template>

<script>
import { icon } from '@fortawesome/fontawesome-svg-core';

export default {
  props: {
    name: {
      type: String,
      required: true,
    },
  },
  name: 'hook0-icon',
  components: {},
  data() {
    return {
      svg: icon({ prefix: 'fas', iconName: this.name }).html[0],
    };
  },
  methods: {},
  mounted() {},
  computed: {},
};
</script>

<style lang="scss" scoped></style>
