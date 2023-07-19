<script setup lang="ts">
import { Codemirror } from 'vue-codemirror';
import { json } from '@codemirror/lang-json';
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from 'codemirror';
import { computed, ref } from 'vue';

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
</script>

<template>
  <Codemirror
    v-model="code"
    :style="{ minHeight: '100px' }"
    :autofocus="true"
    :indent-with-tab="true"
    :tab-size="2"
    :extensions="extensions"
    @ready="handleReady"
  />
</template>
