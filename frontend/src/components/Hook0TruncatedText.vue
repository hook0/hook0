<script setup lang="ts">
import { ref, onBeforeUnmount, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { Copy, Check } from 'lucide-vue-next';
import { useClipboardCopy } from '@/composables/useClipboardCopy';

type Props = {
  value: string;
  display: string;
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

const TOOLTIP_SHOW_DELAY_MS = 300;
const TOOLTIP_HIDE_DELAY_MS = 150;

const visible = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const tooltipStyle = ref<Record<string, string>>({});
let showTimeout: ReturnType<typeof setTimeout> | null = null;
let hideTimeout: ReturnType<typeof setTimeout> | null = null;

function updatePosition() {
  if (!triggerRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();
  tooltipStyle.value = {
    left: `${rect.left + rect.width / 2}px`,
    top: `${rect.top - 8}px`,
    transform: 'translate(-50%, -100%)',
  };
}

function show() {
  if (hideTimeout) {
    clearTimeout(hideTimeout);
    hideTimeout = null;
  }
  if (showTimeout || visible.value) return;
  showTimeout = setTimeout(() => {
    visible.value = true;
    showTimeout = null;
    void nextTick(updatePosition);
  }, TOOLTIP_SHOW_DELAY_MS);
}

function hide() {
  if (showTimeout) {
    clearTimeout(showTimeout);
    showTimeout = null;
  }
  hideTimeout = setTimeout(() => {
    visible.value = false;
    hideTimeout = null;
  }, TOOLTIP_HIDE_DELAY_MS);
}

function copy() {
  clipboardCopy(props.value, props.copyMessage ?? t('common.idCopied'));
}

onBeforeUnmount(() => {
  if (showTimeout) clearTimeout(showTimeout);
  if (hideTimeout) clearTimeout(hideTimeout);
});
</script>

<template>
  <span
    ref="triggerRef"
    class="hook0-truncated"
    :class="{ 'hook0-truncated--linked': linked, 'hook0-truncated--mono': mono }"
    @mouseenter="show"
    @mouseleave="hide"
    @focusin="show"
    @focusout="hide"
  >
    <slot>{{ display }}</slot>

    <Teleport to="body">
      <Transition name="hook0-truncated-fade">
        <div
          v-if="visible"
          class="hook0-truncated__tooltip"
          :style="tooltipStyle"
          @mouseenter="show"
          @mouseleave="hide"
        >
          <span class="hook0-truncated__tooltip-text">{{ value }}</span>
          <button
            class="hook0-truncated__tooltip-copy"
            type="button"
            :aria-label="t('common.copy')"
            @click.stop.prevent="copy"
          >
            <Check
              v-if="justCopied"
              :size="12"
              aria-hidden="true"
              class="hook0-truncated__icon--success"
            />
            <Copy v-else :size="12" aria-hidden="true" />
          </button>
          <span class="hook0-truncated__tooltip-arrow" aria-hidden="true" />
        </div>
      </Transition>
    </Teleport>
  </span>
</template>

<style scoped>
.hook0-truncated {
  display: inline-flex;
  align-items: center;
  font-size: 0.8125rem;
  line-height: 1.4;
  color: var(--color-text-primary);
  cursor: default;
}

.hook0-truncated--mono {
  font-family: var(--font-mono);
}

.hook0-truncated--linked {
  color: var(--color-primary);
}

.hook0-truncated--linked:hover {
  text-decoration: underline;
}

/* Tooltip */
.hook0-truncated__tooltip {
  position: fixed;
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  background-color: var(--color-text-primary);
  color: var(--color-bg-primary);
  padding: 0.375rem 0.625rem;
  border-radius: var(--radius-md);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  white-space: nowrap;
  z-index: var(--z-tooltip, 9999);
  box-shadow: var(--shadow-lg);
  pointer-events: auto;
}

.hook0-truncated__tooltip-text {
  user-select: all;
}

.hook0-truncated__tooltip-copy {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  background-color: color-mix(in srgb, var(--color-on-dark) 15%, transparent);
  border: none;
  color: var(--color-on-dark);
  cursor: pointer;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  transition: background-color 0.15s ease;
}

.hook0-truncated__tooltip-copy:hover {
  background-color: color-mix(in srgb, var(--color-on-dark) 30%, transparent);
}

.hook0-truncated__icon--success {
  color: var(--color-on-dark);
}

/* Arrow */
.hook0-truncated__tooltip-arrow {
  position: absolute;
  bottom: -4px;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border: 4px solid transparent;
  border-top-color: var(--color-text-primary);
  border-bottom: none;
}

/* Fade */
.hook0-truncated-fade-enter-active {
  transition: opacity 100ms ease;
}

.hook0-truncated-fade-leave-active {
  transition: opacity 75ms ease;
}

.hook0-truncated-fade-enter-from,
.hook0-truncated-fade-leave-to {
  opacity: 0;
}
</style>
