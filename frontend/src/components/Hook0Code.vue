<template>
  <codemirror
    v-model="code"
    :style="{ height: '400px' }"
    :autofocus="true"
    :indent-with-tab="true"
    :tab-size="2"
    :extensions="extensions"
    @ready="handleReady"
  />
</template>

<script lang="ts">
import { Vue, Options } from 'vue-class-component';

// https://github.com/surmon-china/vue-codemirror
import { Codemirror } from 'vue-codemirror';

import { json } from '@codemirror/lang-json';
import { oneDark } from '@codemirror/theme-one-dark';
import { ShallowRef, shallowRef } from 'vue';
import { EditorView } from 'codemirror';

@Options({
  name: 'hook0-code',
  inheritAttrs: false,
  components: { Codemirror },
  props: {
    code: {
      type: String,
      required: false,
    },
  },
})
export default class Hook0Code extends Vue {
  extensions = [json(), oneDark];

  // Codemirror EditorView instance ref
  view: EditorView | undefined;

  handleReady(payload: Record<string, any>) {
    this.view = payload.view as EditorView;
  }
}
</script>
<style></style>
