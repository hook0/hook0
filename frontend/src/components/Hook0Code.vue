<script setup lang="ts">
import { Codemirror } from 'vue-codemirror';
import { json } from '@codemirror/lang-json';
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from 'codemirror';
import { computed, ref } from 'vue';
import { Copy } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import { useClipboardCopy } from '@/composables/useClipboardCopy';

const { t } = useI18n();
const clipboardCopy = useClipboardCopy();

defineOptions({
  inheritAttrs: false,
});

type Props = {
  code: string;
  inline?: boolean;
};
const props = withDefaults(defineProps<Props>(), {
  code: '',
  inline: false,
});
const code = computed(() => props.code);

const extensions = [json(), oneDark, EditorView.lineWrapping];

const view = ref<EditorView | undefined>(undefined);

function handleReady(payload: Record<string, unknown>) {
  view.value = payload.view as EditorView;
}

function copyToClipboard() {
  clipboardCopy(code.value, t('common.codeCopied'));
}
</script>

<template>
  <!-- Inline mode: simple styled <code> element -->
  <code v-if="inline" class="hook0-code-inline" v-bind="$attrs">{{ code }}</code>

  <!-- Full mode: CodeMirror editor -->
  <div v-else class="hook0-code-wrapper" data-test="code-block">
    <Codemirror
      v-model="code"
      :style="{ minHeight: '100px' }"
      :autofocus="true"
      :indent-with-tab="true"
      :tab-size="2"
      :extensions="extensions"
      @ready="handleReady"
    />
    <button class="hook0-code-copy" :aria-label="t('common.copyCode')" @click="copyToClipboard">
      <Copy :size="16" aria-hidden="true" />
    </button>
  </div>
</template>

<style scoped>
/* Inline code styling */
.hook0-code-inline {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-bg-tertiary);
  padding: 0.125rem 0.375rem;
  border-radius: var(--radius-sm);
  white-space: nowrap;
  user-select: text;
  cursor: text;
}

/* Full code wrapper */
.hook0-code-wrapper {
  position: relative;
  border-radius: var(--radius-md);
  overflow: hidden;
}

.hook0-code-copy {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: var(--radius-sm);
  border: none;
  background-color: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.hook0-code-copy:hover {
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
}
</style>
