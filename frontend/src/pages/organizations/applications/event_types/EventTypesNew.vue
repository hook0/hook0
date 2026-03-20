<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';

import { useCreateEventType } from './useEventTypeQueries';
import { createEventTypeSchema } from './eventType.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { routes } from '@/routes';
import { handleMutationError } from '@/utils/handleMutationError';
import type { UUID } from '@/http';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import { BookOpen } from 'lucide-vue-next';
import { DOCS_EVENT_TYPES_URL } from '@/constants/externalLinks';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0InputRow from '@/components/Hook0InputRow.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0PageLayout from '@/components/Hook0PageLayout.vue';

const router = useRouter();
const route = useRoute();
const { t } = useI18n();
const { trackEvent } = useTracking();

// Permissions
const { canCreate } = usePermissions();

type Props = {
  tutorialMode?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  tutorialMode: false,
});

const emit = defineEmits(['tutorial-event-type-created']);

// VeeValidate form with Zod schema
const { errors, defineField, handleSubmit } = useForm({
  validationSchema: toTypedSchema(createEventTypeSchema()),
});

const [service, serviceAttrs] = defineField('service');
const [resourceType, resourceTypeAttrs] = defineField('resource_type');
const [verb, verbAttrs] = defineField('verb');

// Mutation
const createMutation = useCreateEventType();

const onSubmit = handleSubmit((values) => {
  createMutation.mutate(
    {
      application_id: route.params.application_id as UUID,
      service: values.service,
      resource_type: values.resource_type,
      verb: values.verb,
    },
    {
      onSuccess: () => {
        const eventTypeName = `${values.service}.${values.resource_type}.${values.verb}`;
        trackEvent('event-type', 'create', eventTypeName);
        if (props.tutorialMode) {
          emit('tutorial-event-type-created');
        } else {
          void router.push({
            name: routes.EventTypesList,
          });
        }
      },
      onError: (err) => {
        handleMutationError(err);
      },
    }
  );
});
</script>

<template>
  <Hook0PageLayout :title="t('eventTypes.createTitle')">
    <Hook0Form data-test="event-type-form" @submit="onSubmit">
      <Hook0Card data-test="event-type-card">
        <Hook0CardHeader>
          <template #header>{{ t('eventTypes.createTitle') }}</template>
          <template #subtitle>{{ t('eventTypes.createSubtitle') }}</template>
          <template #actions>
            <Hook0Button variant="secondary" :href="DOCS_EVENT_TYPES_URL" target="_blank">
              <template #left>
                <BookOpen :size="14" aria-hidden="true" />
              </template>
              {{ t('common.documentation') }}
            </Hook0Button>
          </template>
        </Hook0CardHeader>

        <Hook0CardContentLine>
          <template #label>
            <span class="event-type-new__field-label">{{ t('eventTypes.eventTypeLabel') }}</span>
          </template>
          <template #content>
            <Hook0InputRow gap="sm" class="hook0-input-row--no-collapse">
              <Hook0Input
                v-model="service"
                v-bind="serviceAttrs"
                type="text"
                :placeholder="t('eventTypes.servicePlaceholder')"
                :error="errors.service"
                data-test="event-type-service-input"
              />
              <span class="event-type-new__separator hook0-input-row__separator">.</span>
              <Hook0Input
                v-model="resourceType"
                v-bind="resourceTypeAttrs"
                type="text"
                :placeholder="t('eventTypes.resourceTypePlaceholder')"
                :error="errors.resource_type"
                data-test="event-type-resource-input"
              />
              <span class="event-type-new__separator hook0-input-row__separator">.</span>
              <Hook0Input
                v-model="verb"
                v-bind="verbAttrs"
                type="text"
                :placeholder="t('eventTypes.verbPlaceholder')"
                :error="errors.verb"
                data-test="event-type-verb-input"
              />
            </Hook0InputRow>
          </template>
        </Hook0CardContentLine>
        <Hook0CardFooter>
          <Hook0Button
            v-if="!props.tutorialMode"
            variant="secondary"
            type="button"
            data-test="event-type-cancel-button"
            @click="$router.back()"
          >
            {{ t('common.cancel') }}
          </Hook0Button>
          <Hook0Button
            v-if="!tutorialMode && canCreate('event_type')"
            variant="primary"
            type="button"
            :loading="createMutation.isPending.value"
            :disabled="!service || !resourceType || !verb"
            data-test="event-type-submit-button"
            @click="onSubmit"
          >
            {{ t('eventTypes.create') }}
          </Hook0Button>

          <Hook0Button
            v-else
            variant="primary"
            type="submit"
            :loading="createMutation.isPending.value"
            :disabled="!service || !resourceType || !verb"
            data-test="event-type-submit-button"
            @click="onSubmit"
          >
            {{ t('eventTypes.createFirstEventType') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </Hook0Form>
  </Hook0PageLayout>
</template>

<style scoped>
.event-type-new__field-label {
  color: var(--color-text-primary);
  font-weight: 500;
  font-size: 0.875rem;
  line-height: 1.5;
}

.event-type-new__separator {
  color: var(--color-text-primary);
  font-weight: 700;
  font-size: 1rem;
  flex-shrink: 0;
  width: 0.75rem;
  text-align: center;
  align-self: flex-start;
  margin-top: 0.625rem;
}

@media (max-width: 767px) {
  .event-type-new__separator {
    width: 0.5rem;
  }

  :deep(.hook0-card-footer) {
    flex-direction: column;
  }

  :deep(.hook0-card-footer .hook0-button) {
    width: 100%;
    justify-content: center;
  }
}
</style>
