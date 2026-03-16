<script setup lang="ts">
import type { Component } from 'vue';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0SimpleProgressBar from '@/components/Hook0SimpleProgressBar.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

export interface ComsumptionQuota {
  icon?: Component;
  name: string;
  comsumption: number;
  quota: number;
}

interface Props {
  title: string;
  entityType: string;
  consomptions: ComsumptionQuota[];
}

const props = defineProps<Props>();
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
      <Hook0CardContentLine v-for="quota in props.consomptions" :key="quota.name" type="full-width">
        <template #content>
          <div class="consumption__row">
            <div class="consumption__info">
              <component
                :is="quota.icon"
                v-if="quota.icon"
                :size="16"
                aria-hidden="true"
                class="consumption__icon"
              />
              <span class="consumption__text">
                <strong>{{ quota.name }}</strong
                >: {{ quota.comsumption }} / {{ quota.quota }} ({{
                  Math.round((quota.comsumption / quota.quota) * 100)
                }}%)
              </span>
            </div>
            <div class="consumption__bar">
              <Hook0SimpleProgressBar
                :percentage="Math.floor((quota.comsumption / quota.quota) * 100)"
              />
            </div>
          </div>
        </template>
      </Hook0CardContentLine>
    </Hook0CardContent>
  </Hook0Card>
</template>

<style scoped>
.consumption__row {
  display: flex;
  align-items: center;
  width: 100%;
  flex-direction: column;
  gap: 0.5rem;
}

@media (min-width: 640px) {
  .consumption__row {
    flex-direction: row;
  }
}

.consumption__info {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

@media (min-width: 640px) {
  .consumption__info {
    width: 33.333%;
  }
}

.consumption__icon {
  flex-shrink: 0;
}

.consumption__text {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}

.consumption__bar {
  width: 100%;
}

@media (min-width: 640px) {
  .consumption__bar {
    width: 66.667%;
  }
}
</style>
