<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Copy, Check } from 'lucide-vue-next';
import { useClipboardCopy } from '@/composables/useClipboardCopy';
import Hook0Tooltip from './Hook0Tooltip.vue';

type Props = {
  value: string;
  copyMessage?: string;
  linked?: boolean;
  mono?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  copyMessage: undefined,
  linked: false,
  mono: true,
});

const { t } = useI18n();
const { copy: clipboardCopy, justCopied } = useClipboardCopy();

function copy() {
  clipboardCopy(props.value, props.copyMessage ?? t('common.idCopied'));
}
</script>

<template>
  <Hook0Tooltip interactive :delay="300">
    <span
      class="hook0-tooltip-copy"
      :class="{ 'hook0-tooltip-copy--linked': linked, 'hook0-tooltip-copy--mono': mono }"
    >
      <slot />
    </span>

    <template #content>
      <span class="hook0-tooltip-copy__text">{{ value }}</span>
      <button
        class="hook0-tooltip-copy__btn"
        type="button"
        :aria-label="t('common.copy')"
        @click.stop.prevent="copy"
      >
        <Check
          v-if="justCopied"
          :size="12"
          aria-hidden="true"
          class="hook0-tooltip-copy__icon--success"
        />
        <Copy v-else :size="12" aria-hidden="true" />
      </button>
    </template>
  </Hook0Tooltip>
</template>

<style scoped>
.hook0-tooltip-copy {
  display: inline-flex;
  align-items: center;
  font-size: 0.8125rem;
  line-height: 1.4;
  color: var(--color-text-primary);
  cursor: default;
  max-width: 100%;
  min-width: 0;
}

.hook0-tooltip-copy--mono {
  font-family: var(--font-mono);
}

.hook0-tooltip-copy--linked {
  cursor: pointer;
}

.hook0-tooltip-copy--linked:hover {
  text-decoration: underline;
  text-underline-offset: 2px;
}

.hook0-tooltip-copy__text {
  user-select: all;
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.hook0-tooltip-copy__btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  margin-left: 0.375rem;
  background-color: color-mix(in srgb, var(--color-on-dark) 15%, transparent);
  border: none;
  color: var(--color-on-dark);
  cursor: pointer;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  transition: background-color 0.15s ease;
}

.hook0-tooltip-copy__btn:hover {
  background-color: color-mix(in srgb, var(--color-on-dark) 30%, transparent);
}

.hook0-tooltip-copy__icon--success {
  color: var(--color-on-dark);
}
</style>
