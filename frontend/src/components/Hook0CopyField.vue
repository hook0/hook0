<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { Copy, Check, Eye, EyeOff } from 'lucide-vue-next';
import { useClipboardCopy } from '@/composables/useClipboardCopy';

type Props = {
  value: string;
  maskable?: boolean;
  copyMessage?: string;
};

const props = withDefaults(defineProps<Props>(), {
  maskable: false,
  copyMessage: undefined,
});

const { t } = useI18n();
const clipboardCopy = useClipboardCopy();

const revealed = ref(!props.maskable);

watch(
  () => props.maskable,
  (isMaskable) => {
    revealed.value = !isMaskable;
  }
);

const justCopied = ref(false);
let resetTimer: ReturnType<typeof setTimeout> | null = null;

onBeforeUnmount(() => {
  if (resetTimer) clearTimeout(resetTimer);
});

const displayValue = computed(() => {
  if (!props.maskable || revealed.value) return props.value;
  return '\u2022'.repeat(Math.min(props.value.length, 32));
});

function toggleReveal() {
  revealed.value = !revealed.value;
}

function copyToClipboard() {
  clipboardCopy(props.value, props.copyMessage ?? t('common.idCopied'));
  justCopied.value = true;
  if (resetTimer) clearTimeout(resetTimer);
  resetTimer = setTimeout(() => {
    justCopied.value = false;
    resetTimer = null;
  }, 1500);
}
</script>

<template>
  <div class="copy-field">
    <input
      class="copy-field__input"
      type="text"
      :value="displayValue"
      readonly
      aria-readonly="true"
      tabindex="-1"
      :aria-label="maskable && !revealed ? t('common.maskedValue') : t('common.fieldValue')"
    />
    <div class="copy-field__actions">
      <button
        v-if="maskable"
        class="copy-field__btn"
        type="button"
        :aria-label="revealed ? t('common.hide') : t('common.show')"
        @click="toggleReveal"
      >
        <EyeOff v-if="revealed" :size="14" aria-hidden="true" />
        <Eye v-else :size="14" aria-hidden="true" />
      </button>
      <button
        class="copy-field__btn"
        type="button"
        :aria-label="t('common.copy')"
        @click="copyToClipboard"
      >
        <Check v-if="justCopied" :size="14" aria-hidden="true" class="copy-field__icon--success" />
        <Copy v-else :size="14" aria-hidden="true" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.copy-field {
  display: flex;
  align-items: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-bg-secondary);
  overflow: hidden;
  max-width: 100%;
}

.copy-field__input {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
  padding: 0.375rem 0.625rem;
  min-width: 0;
  flex: 1;
  cursor: default;
}

.copy-field__actions {
  display: flex;
  align-items: center;
  border-left: 1px solid var(--color-border);
  flex-shrink: 0;
}

.copy-field__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  background: transparent;
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition:
    color 0.15s ease,
    background-color 0.15s ease;
}

.copy-field__btn:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-tertiary);
}

.copy-field__btn:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.copy-field__btn + .copy-field__btn {
  border-left: 1px solid var(--color-border);
}

.copy-field__icon--success {
  color: var(--color-success);
}
</style>
