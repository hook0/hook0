<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { RefreshCw } from 'lucide-vue-next';

import type { SelectableEventType } from './subscription.types';
import { routes } from '@/routes';

import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Checkbox from '@/components/Hook0Checkbox.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';

const { t } = useI18n();

type Props = {
  eventTypes: SelectableEventType[];
  loading: boolean;
  error: Error | null;
};

const props = defineProps<Props>();

const emit = defineEmits<{
  'update:eventTypes': [value: SelectableEventType[]];
  refresh: [];
}>();

const localEventTypes = computed(() => props.eventTypes);

function toggleEventType(index: number, selected: boolean) {
  const updated = [...props.eventTypes];
  updated[index] = { ...updated[index], selected };
  emit('update:eventTypes', updated);
}
</script>

<template>
  <div class="sub-row">
    <div class="sub-row__label">
      <span class="sub-row__title">
        {{ t('eventTypes.title') }}
        <Hook0Button
          variant="ghost"
          size="sm"
          :aria-label="t('subscriptions.refreshEventTypes')"
          :title="t('subscriptions.refreshEventTypes')"
          @click="emit('refresh')"
        >
          <RefreshCw :size="14" aria-hidden="true" />
        </Hook0Button>
      </span>
      <span class="sub-row__hint">
        <i18n-t keypath="subscriptions.eventTypesHint" tag="span">
          <template #link>
            <router-link :to="{ name: routes.EventTypesList }" target="_blank" style="color:var(--color-primary);text-decoration:none"
              >event types</router-link
            >
          </template>
        </i18n-t>
      </span>
    </div>
    <div class="sub-row__content">
      <Hook0Loader v-if="loading" />
      <Hook0ErrorCard v-else-if="error" :error="error" @retry="emit('refresh')" />
      <template v-else>
        <div v-if="localEventTypes.length > 0" class="event-type-wrap" data-test="event-types-list">
          <label
            v-for="(eventType, index) in localEventTypes"
            :key="index"
            class="event-type-chip"
            :class="{ 'event-type-chip--selected': eventType.selected }"
            :data-test="`event-type-item-${index}`"
          >
            <Hook0Checkbox
              :model-value="eventType.selected"
              :data-test="`event-type-checkbox-${index}`"
              @update:model-value="toggleEventType(index, $event as boolean)"
            >
              <span :data-test="`event-type-label-${index}`">
                {{ eventType.event_type_name }}
              </span>
            </Hook0Checkbox>
          </label>
        </div>
        <p v-else class="event-type-empty">
          {{ t('subscriptions.noEventTypes') }}
          <Hook0Button variant="link" :to="{ name: routes.EventTypesList }">{{
            t('eventTypes.title')
          }}</Hook0Button>
        </p>
      </template>
    </div>
  </div>
</template>

<style scoped>
.sub-row {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.5rem;
}

@media (min-width: 640px) {
  .sub-row {
    grid-template-columns: 2fr 3fr;
    gap: 1.5rem;
  }
}

.sub-row__label {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  padding-top: 0.25rem;
}

.sub-row__title {
  font-size: 0.875rem;
  font-weight: 700;
  color: var(--color-text-primary);
  display: flex;
  align-items: center;
  gap: 0.125rem;
}

.sub-row__hint {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.sub-row__hint :deep(.hook0-button) {
  padding-top: 0;
  padding-bottom: 0;
}

.sub-row__content {
  min-width: 0;
}

/* Flex-wrap chips */
.event-type-wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.event-type-chip {
  display: inline-flex;
  align-items: center;
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease;
}

.event-type-chip:hover {
  background-color: var(--color-bg-secondary);
}

.event-type-chip--selected {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light, #ede9fe);
}

.event-type-empty {
  margin: 0;
  padding: 1rem;
  text-align: center;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-md);
}
</style>
