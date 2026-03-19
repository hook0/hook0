<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import type { ConsumptionQuota } from '@/components/consumption.types';
import { UNLIMITED_QUOTA } from '@/constants';

const { t } = useI18n();

type Props = {
  title: string;
  entityType: string;
  consumptions: ConsumptionQuota[];
};

const props = defineProps<Props>();

/** Severity variant for quota consumption progress bars. */
type BarVariant = 'ok' | 'warning' | 'danger';

/** Pre-calculated enriched consumption data for template rendering. */
type EnrichedConsumption = ConsumptionQuota & {
  pct: number;
  variant: BarVariant;
  formattedValue: string;
  formattedLimit: string;
};

const enrichedConsumptions = computed<EnrichedConsumption[]>(() => {
  return props.consumptions.map((quota) => {
    const isUnlimited = quota.quota >= UNLIMITED_QUOTA || quota.quota <= 0;
    const pct = isUnlimited ? 0 : Math.round((quota.consumption / quota.quota) * 100);
    const variant: BarVariant = pct >= 90 ? 'danger' : pct >= 70 ? 'warning' : 'ok';
    const formattedValue = quota.displayValue ?? String(quota.consumption);
    let formattedLimit = '';
    if (!quota.displayValue) {
      if (quota.quota >= UNLIMITED_QUOTA) {
        formattedLimit = t('common.unlimited');
      } else {
        formattedLimit = String(quota.quota);
      }
    }
    return { ...quota, pct, variant, formattedValue, formattedLimit };
  });
});
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>{{ props.title }}</template>
      <template #subtitle>
        {{ t('common.consumption', { entityType: props.entityType }) }}
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <div class="consumption">
        <div
          v-for="(quota, index) in enrichedConsumptions"
          :key="quota.name"
          class="consumption__row"
          :class="{ 'consumption__row--bordered': index > 0 }"
        >
          <div class="consumption__label">
            <span class="consumption__name">
              <component
                :is="quota.icon"
                v-if="quota.icon"
                :size="16"
                aria-hidden="true"
                class="consumption__icon"
              />
              {{ quota.name }}
            </span>
            <span v-if="quota.description" class="consumption__desc">{{ quota.description }}</span>
          </div>
          <div class="consumption__meter">
            <div class="consumption__values">
              <span v-if="quota.displayValue" class="consumption__display">
                <strong class="consumption__num">{{ quota.displayValue }}</strong>
                <span v-if="quota.displayUnit" class="consumption__unit">{{
                  quota.displayUnit
                }}</span>
              </span>
              <span v-else class="consumption__used">
                <strong class="consumption__num">{{ quota.formattedValue }}</strong>
                <span v-if="quota.formattedLimit" class="consumption__of">
                  {{ t('common.consumptionOf', { limit: quota.formattedLimit }) }}
                </span>
              </span>
              <span
                v-if="!quota.displayValue"
                class="consumption__pct"
                :class="`consumption__pct--${quota.variant}`"
              >
                {{ quota.pct }}%
              </span>
            </div>
            <div v-if="!quota.displayValue" class="consumption__track">
              <div
                class="consumption__fill"
                :class="`consumption__fill--${quota.variant}`"
                :style="{ width: `${Math.min(quota.pct, 100)}%` }"
              />
            </div>
          </div>
        </div>
      </div>
    </Hook0CardContent>
  </Hook0Card>
</template>

<style scoped>
.consumption {
  display: flex;
  flex-direction: column;
}

.consumption__row {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.5rem;
  padding: 0.875rem 0;
}

.consumption__row--bordered {
  border-top: 1px solid var(--color-border);
}

@media (min-width: 640px) {
  .consumption__row {
    grid-template-columns: 2fr 3fr;
    gap: 1.5rem;
    align-items: center;
  }
}

.consumption__label {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.consumption__name {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.consumption__icon {
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.consumption__desc {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.consumption__meter {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.consumption__values {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

.consumption__used {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.consumption__num {
  color: var(--color-text-primary);
}

.consumption__display {
  display: inline-flex;
  align-items: baseline;
  gap: 0.25rem;
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.consumption__unit {
  color: var(--color-text-secondary);
  font-weight: 400;
}

.consumption__of {
  color: var(--color-text-muted);
}

.consumption__pct {
  font-size: 0.75rem;
  font-weight: 600;
}

.consumption__pct--ok {
  color: var(--color-success);
}

.consumption__pct--warning {
  color: var(--color-warning);
}

.consumption__pct--danger {
  color: var(--color-error);
}

.consumption__track {
  height: 6px;
  border-radius: 3px;
  background-color: var(--color-bg-tertiary);
  overflow: hidden;
}

.consumption__fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.5s cubic-bezier(0.22, 1, 0.36, 1);
}

.consumption__fill--ok {
  background-color: var(--color-success);
}

.consumption__fill--warning {
  background-color: var(--color-warning);
}

.consumption__fill--danger {
  background-color: var(--color-error);
}
</style>
