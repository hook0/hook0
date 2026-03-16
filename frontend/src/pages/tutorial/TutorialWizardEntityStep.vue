<script setup lang="ts">
import type { Component } from 'vue';
import { ref, watch } from 'vue';

import type { UUID } from '@/http';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import SelectableCard from '@/components/SelectableCard.vue';
import TutorialStepProgress from '@/pages/tutorial/TutorialStepProgress.vue';

import { Plus, List, ArrowRight, X } from 'lucide-vue-next';

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
  /** When true, the card/edit/select sections require options to be present (used by Application step) */
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

const enum EntitySection {
  Create = 'create',
  SelectExisting = 'select_existing',
}

const entityId = ref<UUID | null>(null);
const selectedEntityId = ref<UUID | null>(null);
const entitySection = ref<EntitySection | null>(null);

// Auto-select "create" if no entities exist (options only has the placeholder)
watch(
  () => props.entityOptions,
  (options) => {
    if ((options ?? []).length <= 1 && entitySection.value === null) {
      entitySection.value = EntitySection.Create;
    }
  }
);

function handleCreated(id: UUID) {
  entityId.value = id;
}

function handleAdvance() {
  const id = entityId.value ?? selectedEntityId.value;
  if (id) {
    emit('advance', id);
  }
}

const showCard = () => {
  if (props.requireOptions) {
    return !entityId.value && props.entityOptions.length > 1;
  }
  return !entityId.value;
};
</script>

<template>
  <div class="wizard-modal__header">
    <Hook0Stack direction="row" align="center" gap="sm">
      <Hook0Badge display="step" variant="primary">{{ stepNumber }}</Hook0Badge>
      <span id="wizard-step-title" class="wizard-modal__title">{{ stepTitle }}</span>
    </Hook0Stack>
    <button
      class="wizard-modal__close"
      type="button"
      :aria-label="skipLabel"
      @click="$emit('skip')"
    >
      <X :size="18" aria-hidden="true" />
    </button>
  </div>

  <div class="wizard-modal__content">
    <!-- Top-level loading (when entitiesLoading is provided and true) -->
    <Hook0Stack v-if="entitiesLoading" direction="column" gap="md">
      <Hook0Skeleton size="hero" />
      <Hook0Skeleton size="heading" />
      <Hook0Skeleton size="heading" />
    </Hook0Stack>

    <!-- Top-level error (when entitiesError is provided) -->
    <Hook0ErrorCard v-else-if="entitiesError" :error="entitiesError" @retry="$emit('retry')" />

    <!-- Main content -->
    <Hook0Stack v-else direction="column" gap="lg">
      <span class="wizard-modal__subtitle">{{ stepDescription }}</span>

      <TutorialStepProgress :steps="progressSteps" :current="progressCurrent" />

      <Hook0Card v-if="showCard()">
        <Hook0CardHeader>
          <template #header>
            <Hook0Stack direction="row" align="center" gap="sm">
              <component :is="entityIcon" :size="18" aria-hidden="true" />
              <Hook0Stack direction="row" align="center" gap="none">
                {{ chooseLabel }}
              </Hook0Stack>
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
              @update:model-value="entitySection = EntitySection.Create"
            />
            <SelectableCard
              :model-value="entitySection === EntitySection.SelectExisting"
              :label="selectExistingLabel"
              :icon="List"
              :name="selectionName"
              :data-test="selectDataTest"
              @update:model-value="entitySection = EntitySection.SelectExisting"
            />
          </Hook0Stack>
        </Hook0CardContent>
      </Hook0Card>

      <slot
        v-if="!entityId && entitySection === EntitySection.Create"
        name="edit"
        :on-created="handleCreated"
      />

      <!-- Select existing entity -->
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
  </div>

  <div class="wizard-modal__footer">
    <Hook0Button variant="secondary" type="button" @click="$emit('skip')">
      <X :size="16" aria-hidden="true" />
      {{ skipLabel }}
    </Hook0Button>
    <Hook0Button
      v-if="entityId || (selectedEntityId && selectedEntityId !== '')"
      variant="primary"
      type="button"
      @click="handleAdvance"
    >
      {{ continueLabel }}
      <ArrowRight :size="16" aria-hidden="true" />
    </Hook0Button>
  </div>
</template>
