<script setup lang="ts">
import { Codemirror } from 'vue-codemirror';
import { json } from '@codemirror/lang-json';
import { EditorView } from 'codemirror';
import type { Extension } from '@codemirror/state';
import { StreamLanguage } from '@codemirror/language';
import { computed, ref, shallowRef, watch } from 'vue';
import { Copy } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import { useClipboardCopy } from '@/composables/useClipboardCopy';

const { t } = useI18n();
const { copy: clipboardCopy } = useClipboardCopy();

defineOptions({
  inheritAttrs: false,
});

type CodeLanguage = 'json' | 'javascript' | 'bash' | 'rust';

type Props = {
  code: string;
  inline?: boolean;
  language?: CodeLanguage;
  editable?: boolean;
};
const props = withDefaults(defineProps<Props>(), {
  code: '',
  inline: false,
  language: 'json',
  editable: true,
});
const code = computed(() => props.code);

const lightTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--color-bg-secondary)',
    color: 'var(--color-text-primary)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--color-bg-tertiary)',
    color: 'var(--color-text-tertiary)',
    borderRight: '1px solid var(--color-border)',
  },
  '.cm-activeLine': {
    backgroundColor: 'var(--color-bg-tertiary)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--color-bg-tertiary)',
  },
});

// Dynamic language loading to avoid bundling all languages upfront
const langExtension = shallowRef<Extension | null>(null);

function loadLanguage(lang: CodeLanguage) {
  if (lang === 'json') {
    langExtension.value = json();
  } else if (lang === 'javascript') {
    void import('@codemirror/lang-javascript').then((mod) => {
      langExtension.value = mod.javascript({ typescript: true });
    });
  } else if (lang === 'rust') {
    void import('@codemirror/lang-rust').then((mod) => {
      langExtension.value = mod.rust();
    });
  } else if (lang === 'bash') {
    void import('@codemirror/legacy-modes/mode/shell').then((mod) => {
      langExtension.value = StreamLanguage.define(mod.shell);
    });
  } else {
    langExtension.value = null;
  }
}

watch(() => props.language, loadLanguage, { immediate: true });

const extensions = computed(() => {
  const base: Extension[] = [lightTheme, EditorView.lineWrapping];
  return langExtension.value ? [langExtension.value, ...base] : base;
});

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
      :autofocus="false"
      :indent-with-tab="true"
      :tab-size="2"
      :disabled="!editable"
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
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.hook0-code-copy:hover {
  background-color: var(--color-bg-elevated);
  color: var(--color-text-primary);
}
</style>
