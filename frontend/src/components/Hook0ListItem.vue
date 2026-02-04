<script setup lang="ts">
import { computed } from 'vue';
import { RouteLocationRaw, useRouter } from 'vue-router';
import { ChevronRight, Check } from 'lucide-vue-next';
import Hook0Spinner from '@/components/Hook0Spinner.vue';

type ItemVariant = 'default' | 'stat' | 'action' | 'selectable';

interface Props {
  to?: RouteLocationRaw;
  href?: string;
  showChevron?: boolean;
  variant?: ItemVariant;
  value?: string | number;
  label?: string;
  separated?: boolean;
  // Selectable variant props
  selected?: boolean;
  disabled?: boolean;
  loading?: boolean;
  error?: boolean;
  name?: string; // For radio group behavior
}

const props = withDefaults(defineProps<Props>(), {
  to: undefined,
  href: undefined,
  showChevron: false,
  variant: 'default',
  value: undefined,
  label: undefined,
  separated: false,
  selected: false,
  disabled: false,
  loading: false,
  error: false,
  name: undefined,
});

const emit = defineEmits<{
  click: [e: MouseEvent];
  select: [];
}>();

defineSlots<{
  icon(): unknown;
  left(): unknown;
  right(): unknown;
}>();

const router = useRouter();

const isClickable = computed(() => props.to || props.href || true);
const showChevronIcon = computed(() => props.showChevron || props.to || props.href);
const isStat = computed(() => props.variant === 'stat');
const isAction = computed(() => props.variant === 'action');
const isSelectable = computed(() => props.variant === 'selectable');

function handleClick(e: MouseEvent) {
  if (props.disabled || props.loading) return;

  emit('click', e);

  if (isSelectable.value) {
    emit('select');
    return;
  }

  if (props.href) {
    window.open(props.href, '_blank');
    return;
  }

  if (props.to) {
    if (e.metaKey || e.ctrlKey) {
      const resolved = router.resolve(props.to);
      window.open(resolved.href, '_blank');
    } else {
      router.push(props.to).catch(console.error);
    }
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    // Create a synthetic mouse event for consistency
    const syntheticEvent = new MouseEvent('click', {
      metaKey: e.metaKey,
      ctrlKey: e.ctrlKey,
    });
    handleClick(syntheticEvent);
  }
}
</script>

<template>
  <!-- Selectable variant (radio/checkbox card-style) -->
  <label
    v-if="isSelectable"
    class="hook0-list-item hook0-list-item--selectable"
    :class="{
      'hook0-list-item--selected': selected,
      'hook0-list-item--disabled': disabled,
      'hook0-list-item--loading': loading,
      'hook0-list-item--error': error,
    }"
    @click="handleClick"
    @keydown="handleKeydown"
  >
    <input
      v-if="name"
      type="radio"
      :name="name"
      :checked="selected"
      :disabled="disabled"
      class="hook0-list-item__radio"
    />
    <div v-if="$slots.icon" class="hook0-list-item__icon hook0-list-item__icon--selectable">
      <slot name="icon" />
    </div>
    <div class="hook0-list-item__left">
      <slot name="left" />
    </div>
    <div v-if="$slots.right" class="hook0-list-item__right">
      <slot name="right" />
    </div>
    <div class="hook0-list-item__indicator">
      <Hook0Spinner v-if="loading" :size="16" />
      <Check v-else-if="selected" :size="16" aria-hidden="true" />
    </div>
  </label>

  <!-- Stat variant -->
  <div v-else-if="isStat" class="hook0-list-item hook0-list-item--stat">
    <div v-if="$slots.icon" class="hook0-list-item__icon hook0-list-item__icon--stat">
      <slot name="icon" />
    </div>
    <div class="hook0-list-item__stat-content">
      <span class="hook0-list-item__stat-value">{{ value }}</span>
      <span class="hook0-list-item__stat-label">{{ label }}</span>
    </div>
  </div>

  <!-- Action variant (for "create new" buttons) -->
  <button
    v-else-if="isAction"
    class="hook0-list-item hook0-list-item--action"
    :class="{
      'hook0-list-item--separated': separated,
    }"
    type="button"
    @click="handleClick"
    @keydown="handleKeydown"
  >
    <div v-if="$slots.icon" class="hook0-list-item__icon hook0-list-item__icon--action">
      <slot name="icon" />
    </div>
    <div class="hook0-list-item__left">
      <slot name="left" />
    </div>
    <div v-if="$slots.right" class="hook0-list-item__right">
      <slot name="right" />
    </div>
  </button>

  <!-- Default variant -->
  <li
    v-else
    class="hook0-list-item"
    :class="{
      'hook0-list-item--clickable': isClickable,
    }"
    role="listitem"
    :tabindex="isClickable ? 0 : undefined"
    @click="handleClick"
    @keydown="handleKeydown"
  >
    <div v-if="$slots.icon" class="hook0-list-item__icon">
      <slot name="icon" />
    </div>
    <div class="hook0-list-item__left">
      <slot name="left" />
    </div>
    <div v-if="$slots.right" class="hook0-list-item__right">
      <slot name="right" />
    </div>
    <ChevronRight
      v-if="showChevronIcon"
      :size="14"
      class="hook0-list-item__chevron"
      aria-hidden="true"
    />
  </li>
</template>

<style scoped>
/* Default variant */
.hook0-list-item {
  padding: 0.5rem 0.75rem;
  display: flex;
  align-items: center;
  font-size: 0.875rem;
  border-radius: var(--radius-md);
  transition: background-color 0.15s ease;
}

.hook0-list-item--clickable {
  cursor: pointer;
}

.hook0-list-item--clickable:hover {
  background-color: var(--color-bg-secondary);
}

.hook0-list-item--clickable:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.hook0-list-item__icon {
  flex-shrink: 0;
  width: 1.75rem;
  height: 1.75rem;
  margin-right: 0.625rem;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
  transition: all 0.15s ease;
}

.hook0-list-item--clickable:hover .hook0-list-item__icon {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.hook0-list-item__left {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
}

.hook0-list-item__right {
  margin-left: 1rem;
  flex-shrink: 0;
}

.hook0-list-item__chevron {
  flex-shrink: 0;
  margin-left: 0.5rem;
  color: var(--color-text-tertiary);
  opacity: 0;
  transition: all 0.15s ease;
}

.hook0-list-item--clickable:hover .hook0-list-item__chevron {
  opacity: 1;
  color: var(--color-text-secondary);
}

/* Stat variant */
.hook0-list-item--stat {
  padding: 0.75rem 1rem;
  border: 1px solid var(--color-border);
  background-color: var(--color-bg-secondary);
  gap: 0.75rem;
  transition: border-color 0.15s ease;
}

.hook0-list-item--stat:hover {
  border-color: var(--color-border-strong);
}

.hook0-list-item__icon--stat {
  width: 2rem;
  height: 2rem;
  margin-right: 0;
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.hook0-list-item__stat-content {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.hook0-list-item__stat-value {
  font-size: 1.125rem;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1.2;
}

.hook0-list-item__stat-label {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
  line-height: 1.3;
}

/* Action variant (for "create new" buttons) */
.hook0-list-item--action {
  width: 100%;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
}

.hook0-list-item--action:hover {
  background-color: var(--color-primary-light);
}

.hook0-list-item--action:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.hook0-list-item--separated {
  margin-top: 0.125rem;
  border-top: 1px solid var(--color-border);
  border-radius: 0;
  padding-top: 0.625rem;
}

.hook0-list-item__icon--action {
  background-color: transparent;
  border: 1.5px dashed var(--color-border-strong);
  color: var(--color-text-tertiary);
}

.hook0-list-item--action:hover .hook0-list-item__icon--action {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.hook0-list-item--action .hook0-list-item__left {
  color: var(--color-text-secondary);
  font-weight: 500;
  font-size: 0.8125rem;
}

.hook0-list-item--action:hover .hook0-list-item__left {
  color: var(--color-primary);
}

/* Selectable variant (radio/checkbox card-style) */
.hook0-list-item--selectable {
  display: flex;
  align-items: center;
  padding: 1rem 1.25rem;
  border: 2px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
  cursor: pointer;
  transition: all 0.15s ease;
  gap: 0.75rem;
}

.hook0-list-item--selectable:hover {
  border-color: var(--color-border-strong);
  background-color: var(--color-bg-secondary);
}

.hook0-list-item--selectable:focus-within {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.hook0-list-item--selected {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.hook0-list-item--selected:hover {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.hook0-list-item--disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.hook0-list-item--loading {
  cursor: wait;
}

.hook0-list-item--error {
  border-color: var(--color-danger);
}

.hook0-list-item--error:hover {
  border-color: var(--color-danger);
}

.hook0-list-item__radio {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.hook0-list-item__icon--selectable {
  width: 2.5rem;
  height: 2.5rem;
  margin-right: 0;
  border-radius: var(--radius-md);
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}

.hook0-list-item--selected .hook0-list-item__icon--selectable {
  background-color: var(--color-primary);
  color: #ffffff;
}

.hook0-list-item__indicator {
  flex-shrink: 0;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-left: auto;
  color: var(--color-primary);
}

.hook0-list-item--selectable .hook0-list-item__left {
  flex-direction: column;
  align-items: flex-start;
  gap: 0.25rem;
}
</style>
