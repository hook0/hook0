<script setup lang="ts">
import { Codemirror } from 'vue-codemirror';
import { json } from '@codemirror/lang-json';
import { EditorView } from 'codemirror';
import { EditorState, type Extension } from '@codemirror/state';
import { StreamLanguage } from '@codemirror/language';
import { computed, ref, shallowRef, watch } from 'vue';
import { Copy } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import { useClipboardCopy } from '@/composables/useClipboardCopy';
import type { Hook0CodeLanguage } from '@/components/Hook0Code';
import type { ColouredLanguage, PlainTextLanguage } from '@/components/Hook0CodeColouring';

const { t } = useI18n();
const { copy: clipboardCopy } = useClipboardCopy();

defineOptions({
  inheritAttrs: false,
});

const emit = defineEmits<{
  /** The reader took the block away. Only the parent knows what the block was worth counting as. */
  copy: [];
}>();

type Props = {
  code: string;
  inline?: boolean;
  language?: Hook0CodeLanguage;
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

type GrammarLoader = () => Promise<Extension>;

/**
 * Where each language's grammar comes from: a loader for every language meant to colour, and `null`
 * for each one `Hook0CodeColouring` declares plain. The type is what holds that split — a coloured
 * language owes a loader and a plain one owes `null`, so a block dropping to plain text by setting an
 * entry to `null` (or by a language arriving with no entry) is a compile error here rather than a
 * colourless panel noticed later on screen.
 */
type GrammarRegistry = { readonly [L in ColouredLanguage]: GrammarLoader } & {
  readonly [L in PlainTextLanguage]: null;
};

const GRAMMARS: GrammarRegistry = {
  json: () => Promise.resolve(json()),
  bash: () =>
    import('@codemirror/legacy-modes/mode/shell').then((mod) => StreamLanguage.define(mod.shell)),
  javascript: () => import('@codemirror/lang-javascript').then((mod) => mod.javascript()),
  typescript: () =>
    import('@codemirror/lang-javascript').then((mod) => mod.javascript({ typescript: true })),
  python: () =>
    import('@codemirror/legacy-modes/mode/python').then((mod) => StreamLanguage.define(mod.python)),
  go: () => import('@codemirror/legacy-modes/mode/go').then((mod) => StreamLanguage.define(mod.go)),
  ruby: () =>
    import('@codemirror/legacy-modes/mode/ruby').then((mod) => StreamLanguage.define(mod.ruby)),
  lua: () =>
    import('@codemirror/legacy-modes/mode/lua').then((mod) => StreamLanguage.define(mod.lua)),
  java: () =>
    import('@codemirror/legacy-modes/mode/clike').then((mod) => StreamLanguage.define(mod.java)),
  csharp: () =>
    import('@codemirror/legacy-modes/mode/clike').then((mod) => StreamLanguage.define(mod.csharp)),
  kotlin: () =>
    import('@codemirror/legacy-modes/mode/clike').then((mod) => StreamLanguage.define(mod.kotlin)),
  // `plain: true` because the snippets are bare PHP: the grammar defaults to an HTML document that
  // turns PHP on only after a `<?php`, and with no such tag it would colour none of what is shown.
  php: () => import('@codemirror/lang-php').then((mod) => mod.php({ plain: true })),
  rust: () => import('@codemirror/lang-rust').then((mod) => mod.rust()),
  // Zig is the only one nobody publishes a grammar for. Writing one here would mean maintaining a
  // grammar for the least used of the languages this block shows, and one whose syntax is still
  // moving; plain text is the honest fallback and the block already renders it correctly.
  zig: null,
};

function loadLanguage(lang: Hook0CodeLanguage) {
  const grammar = GRAMMARS[lang];
  if (typeof grammar !== 'function') {
    langExtension.value = null;
    return;
  }
  void grammar()
    .then((extension) => {
      // A reader flipping through the language picker starts several loads, and they do not
      // necessarily land in the order they were asked for; only the one still being looked at counts.
      if (props.language === lang) {
        langExtension.value = extension;
      }
    })
    // A grammar chunk that fails to load — a renamed or broken CodeMirror dependency — leaves the
    // block as plain text, which is indistinguishable on screen from a language declared plain. The
    // failure is surfaced rather than swallowed, so it does not pass for a decision nobody made.
    .catch(console.error);
}

watch(() => props.language, loadLanguage, { immediate: true });

const extensions = computed(() => {
  const base: Extension[] = [lightTheme, EditorView.lineWrapping];
  if (!props.editable) {
    base.push(EditorState.readOnly.of(true), EditorView.editable.of(false));
  }
  return langExtension.value ? [langExtension.value, ...base] : base;
});

const view = ref<EditorView | undefined>(undefined);

function handleReady(payload: Record<string, unknown>) {
  view.value = payload.view as EditorView;
}

function copyToClipboard() {
  clipboardCopy(code.value, t('common.codeCopied'));
  emit('copy');
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
      :extensions="extensions"
      @ready="handleReady"
    />
    <button
      class="hook0-code-copy"
      :aria-label="t('common.copyCode')"
      data-test="code-copy"
      @click="copyToClipboard"
    >
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
