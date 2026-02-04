<script setup lang="ts">
/**
 * Hook0EntityIcon - Single source of truth for entity type icons
 *
 * Provides consistent icons for all Hook0 entity types throughout the UI.
 * Use this component instead of importing icons directly for entity representation.
 *
 * @example
 * <Hook0EntityIcon entity="organization" :size="16" />
 * <Hook0EntityIcon entity="event" :size="20" />
 */
import { computed, type Component } from 'vue';
import { Building2, Box, Tag, Bell, Zap } from 'lucide-vue-next';

export type EntityType = 'organization' | 'application' | 'event-type' | 'subscription' | 'event';
export type EntityIconSize = 14 | 16 | 18 | 20 | 24;

interface Props {
  entity: EntityType;
  size?: EntityIconSize;
}

const props = withDefaults(defineProps<Props>(), {
  size: 16,
});

const entityIconMap: Record<EntityType, Component> = {
  organization: Building2,
  application: Box,
  'event-type': Tag,
  subscription: Bell,
  event: Zap,
};

const iconComponent = computed(() => entityIconMap[props.entity]);
</script>

<template>
  <component :is="iconComponent" :size="size" aria-hidden="true" v-bind="$attrs" />
</template>
