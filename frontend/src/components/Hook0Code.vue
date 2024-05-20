<script setup lang="ts">
import { Codemirror } from 'vue-codemirror';
import { json } from '@codemirror/lang-json';
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from 'codemirror';
import { computed, ref } from 'vue';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { push } from 'notivue';

defineOptions({
  inheritAttrs: false,
});

interface Props {
  code?: string;
}
const props = defineProps<Props>();
const code = computed(() => props.code ?? '');

const extensions = [json(), oneDark, EditorView.lineWrapping];

// Codemirror EditorView instance ref
const view = ref<EditorView | undefined>(undefined);

function handleReady(payload: Record<string, unknown>) {
  view.value = payload.view as EditorView;
}

async function copyToClipboard() {
  try {
    await navigator.clipboard.writeText(code.value);
    push.success({
      title: 'Copied!',
      message: 'The code has been copied to the clipboard.',
    });
  } catch (err) {
    push.error({
      title: 'Error',
      message: 'An error occurred while copying the code to the clipboard.',
    });
  }
}
</script>

<template>
  <div class="relative">
    <Codemirror
      v-model="code"
      :style="{ minHeight: '100px' }"
      :autofocus="true"
      :indent-with-tab="true"
      :tab-size="2"
      :extensions="extensions"
      @ready="handleReady"
    />
    <button class="absolute top-0 right-0 m-2" @click="copyToClipboard">
      <FontAwesomeIcon :icon="['fas', 'copy']" class="text-gray-300 font-bold" />
    </button>
  </div>
</template>
