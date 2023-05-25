<template>
  <hook0-text :title="value">{{ value_humanized }}</hook0-text>
</template>

<script lang="ts">
import { Options, Vue } from 'vue-class-component';
import Hook0Text from './Hook0Text.vue';
import { Codemirror } from 'vue-codemirror';
import { formatRFC7231, parseISO } from 'date-fns';

@Options({
  inheritAttrs: false,
  components: {
    Hook0Text,
  },
  props: {
    value: {
      type: String,
      required: true,
    },
  },
})
export default class Hook0DateTime extends Vue {
  private value_humanized = '';
  private value = '';

  mounted() {
    this.refresh();
  }

  updated() {
    this.refresh();
  }

  refresh() {
    // @ts-ignore
    this.value_humanized = formatRFC7231(parseISO(this.$props.value as string));
  }
}
</script>

<style lang="scss" scoped></style>
