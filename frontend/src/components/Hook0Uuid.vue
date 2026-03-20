<script setup lang="ts">
import { computed, ref, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { Copy, Check } from 'lucide-vue-next';
import { useClipboardCopy } from '@/composables/useClipboardCopy';
import Hook0Tooltip from './Hook0Tooltip.vue';

type Props = {
  value: string;
  linked?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  linked: false,
});

const { t } = useI18n();
const clipboardCopy = useClipboardCopy();

const justCopied = ref(false);
let resetTimer: ReturnType<typeof setTimeout> | null = null;

onBeforeUnmount(() => {
  if (resetTimer) clearTimeout(resetTimer);
});

const truncated = computed(() => {
  if (props.value.length <= 20) return props.value;
  return `${props.value.slice(0, 8)}…${props.value.slice(-8)}`;
});

function copy() {
  clipboardCopy(props.value, t('common.idCopied'));
  justCopied.value = true;
  if (resetTimer) clearTimeout(resetTimer);
  resetTimer = setTimeout(() => {
    justCopied.value = false;
    resetTimer = null;
  }, 1500);
}
</script>

<template>
  <Hook0Tooltip :content="value">
    <span class="hook0-uuid" :class="{ 'hook0-uuid--linked': linked }">
      <span class="hook0-uuid__text">{{ truncated }}</span>
      <button
        class="hook0-uuid__copy"
        type="button"
        :aria-label="t('common.copy')"
        @click.stop.prevent="copy"
      >
        <Check v-if="justCopied" :size="12" aria-hidden="true" class="hook0-uuid__icon--success" />
        <Copy v-else :size="12" aria-hidden="true" />
      </button>
    </span>
  </Hook0Tooltip>
</template>

<style scoped>
.hook0-uuid {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  line-height: 1.4;
  color: var(--color-text-primary);
}

.hook0-uuid--linked .hook0-uuid__text {
  color: var(--color-primary);
}

.hook0-uuid--linked:hover .hook0-uuid__text {
  text-decoration: underline;
}

.hook0-uuid__text {
  white-space: nowrap;
}

.hook0-uuid__copy {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.25rem;
  height: 1.25rem;
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  transition:
    color 0.15s ease,
    background-color 0.15s ease;
}

.hook0-uuid__copy:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-tertiary);
}

.hook0-uuid__copy:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 1px;
}

.hook0-uuid__icon--success {
  color: var(--color-success);
}
</style>
