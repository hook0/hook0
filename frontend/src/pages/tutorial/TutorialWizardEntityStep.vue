<script setup lang="ts">
import type { Component } from 'vue';
import { ref, computed } from 'vue';

import type { UUID } from '@/http';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';

import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import SelectableCard from '@/components/SelectableCard.vue';
import TutorialStepProgress from '@/pages/tutorial/TutorialStepProgress.vue';
import WizardStepLayout from '@/pages/tutorial/WizardStepLayout.vue';

import { Plus, List } from 'lucide-vue-next';

type ProgressStep = {
  icon: Component;
  label: string;
};

type Props = {
  stepNumber: number;
  stepTitle: string;
  stepDescription: string;
  chooseLabel: string;
  createLabel: string;
  selectExistingLabel: string;
  selectLabel: string;
  continueLabel: string;
  progressSteps: ProgressStep[];
  progressCurrent: number;
  entityIcon: Component;
  entityOptions: { label: string; value: string }[];
  entitiesLoading?: boolean;
  entitiesError?: Error | null;
  requireOptions?: boolean;
  skipLabel: string;
  selectionName?: string;
  createDataTest?: string;
  selectDataTest?: string;
};

const props = withDefaults(defineProps<Props>(), {
  entitiesLoading: false,
  entitiesError: undefined,
  requireOptions: false,
  selectionName: 'entity_selection',
  createDataTest: undefined,
  selectDataTest: undefined,
});

const emit = defineEmits<{
  advance: [entityId: UUID];
  skip: [];
  retry: [];
}>();

enum EntitySection {
  Create = 'create',
  SelectExisting = 'select_existing',
}

const entityId = ref<UUID | null>(null);
const selectedEntityId = ref<UUID | null>(null);
const userEntitySection = ref<EntitySection | null>(null);

/**
 * Effective section: user choice takes priority, otherwise auto-select Create
 * when there are no existing entities to pick from (options <= 1 means only the
 * placeholder entry). Using a computed removes any watcher-timing edge case that
 * could leave the section null while the template has already left the skeleton.
 */
const entitySection = computed<EntitySection | null>(() => {
  if (userEntitySection.value !== null) return userEntitySection.value;
  if (props.entityOptions.length <= 1) return EntitySection.Create;
  return null;
});

function handleCreated(id: UUID) {
  entityId.value = id;
  emit('advance', id);
}

function handleAdvance() {
  const id = entityId.value ?? selectedEntityId.value;
  if (id) {
    emit('advance', id);
  }
}

const showCard = computed(() => {
  if (props.requireOptions) {
    return !entityId.value && props.entityOptions.length > 1;
  }
  return !entityId.value;
});
</script>

<template>
  <WizardStepLayout
    :step-number="stepNumber"
    :title="stepTitle"
    :show-skip="true"
    :continue-label="!entityId && selectedEntityId ? continueLabel : undefined"
    :continue-disabled="!selectedEntityId || selectedEntityId === ''"
    @skip="$emit('skip')"
    @continue="handleAdvance"
  >
    <!-- Loading -->
    <Hook0Stack v-if="entitiesLoading" direction="column" gap="md">
      <Hook0Skeleton size="hero" />
      <Hook0Skeleton size="heading" />
      <Hook0Skeleton size="heading" />
    </Hook0Stack>

    <!-- Error -->
    <Hook0ErrorCard v-else-if="entitiesError" :error="entitiesError" @retry="$emit('retry')" />

    <!-- Main content -->
    <Hook0Stack v-else direction="column" gap="lg">
      <span class="entity-step__subtitle">{{ stepDescription }}</span>

      <TutorialStepProgress :steps="progressSteps" :current="progressCurrent" />

      <Hook0Card v-if="showCard">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <component :is="entityIcon" :size="18" aria-hidden="true" />
              {{ chooseLabel }}
            </Hook0Stack>
          </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0Stack layout="grid" gap="md" grid-size="cards" role="radiogroup">
            <SelectableCard
              :model-value="entitySection === EntitySection.Create"
              :label="createLabel"
              :icon="Plus"
              :name="selectionName"
              :data-test="createDataTest"
              @update:model-value="userEntitySection = EntitySection.Create"
            />
            <SelectableCard
              :model-value="entitySection === EntitySection.SelectExisting"
              :label="selectExistingLabel"
              :icon="List"
              :name="selectionName"
              :data-test="selectDataTest"
              :disabled="props.entityOptions.length <= 1"
              @update:model-value="userEntitySection = EntitySection.SelectExisting"
            />
          </Hook0Stack>
        </Hook0CardContent>
      </Hook0Card>

      <slot
        v-if="!entityId && entitySection === EntitySection.Create"
        name="edit"
        :on-created="handleCreated"
      />

      <template v-if="entitySection === EntitySection.SelectExisting">
        <Hook0Card>
          <Hook0CardContent>
            <Hook0CardContentLine type="full-width">
              <template #label>{{ selectLabel }}</template>
              <template #content>
                <Hook0Select v-model="selectedEntityId" :options="entityOptions" />
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
        </Hook0Card>
      </template>
    </Hook0Stack>
  </WizardStepLayout>
</template>

<style scoped>
.entity-step__subtitle {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}
</style>
