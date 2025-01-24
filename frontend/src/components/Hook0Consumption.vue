<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';

export interface ComsumptionQuota {
  icon?: string;
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
        Here is the consumption of your <strong>{{ props.entityType }}</strong
        >.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine
          v-for="quota in props.consomptions"
          :key="quota.name"
          type="full-width"
        >
          <template #content>
            <Hook0Icon v-if="quota.icon" :name="quota.icon" class="mr-1" />
            <Hook0Text class="text-md">
              <strong>{{ quota.name }}</strong
              >: {{ quota.comsumption }} / {{ quota.quota }} ({{
                Math.round((quota.comsumption / quota.quota) * 100)
              }}%)
            </Hook0Text>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
  </Hook0Card>
</template>
