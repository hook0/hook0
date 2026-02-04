<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';

import { useCreateEventType } from './useEventTypeQueries';
import { eventTypeSchema } from './eventType.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { routes } from '@/routes';
import { displayError } from '@/utils/displayError';
import type { Problem, UUID } from '@/http';
import { useTracking } from '@/composables/useTracking';

import Hook0List from '@/components/Hook0List.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0InputRow from '@/components/Hook0InputRow.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0Form from '@/components/Hook0Form.vue';

const router = useRouter();
const route = useRoute();
const { t } = useI18n();
const { trackEvent } = useTracking();

interface Props {
  tutorialMode?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  tutorialMode: false,
});

const emit = defineEmits(['tutorial-event-type-created']);

// VeeValidate form with Zod schema
const { errors, defineField, handleSubmit } = useForm({
  validationSchema: toTypedSchema(eventTypeSchema),
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
        displayError(err as unknown as Problem);
      },
    }
  );
});
</script>

<template>
  <Hook0Form data-test="event-type-form" @submit="onSubmit">
    <Hook0Card data-test="event-type-card">
      <Hook0CardHeader>
        <template #header>{{ t('eventTypes.createTitle') }}</template>
        <template #subtitle>{{ t('eventTypes.createSubtitle') }}</template>
      </Hook0CardHeader>

      <Hook0CardContentLine>
        <template #label>
          <Hook0Text variant="primary" weight="medium">{{
            t('eventTypes.eventTypeLabel')
          }}</Hook0Text>
        </template>
        <template #content>
          <Hook0InputRow gap="sm">
            <Hook0Input
              v-model="service"
              v-bind="serviceAttrs"
              type="text"
              :placeholder="t('eventTypes.servicePlaceholder')"
              :error="errors.service"
              data-test="event-type-service-input"
            />
            <Hook0Text variant="primary" weight="bold" size="lg">.</Hook0Text>
            <Hook0Input
              v-model="resourceType"
              v-bind="resourceTypeAttrs"
              type="text"
              :placeholder="t('eventTypes.resourceTypePlaceholder')"
              :error="errors.resource_type"
              data-test="event-type-resource-input"
            />
            <Hook0Text variant="primary" weight="bold" size="lg">.</Hook0Text>
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
          v-if="!tutorialMode"
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

    <Hook0Card>
      <Hook0CardContent>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Text variant="primary" block>{{ t('eventTypes.helpDescription') }}</Hook0Text>
            <Hook0HelpText>{{ t('eventTypes.helpFormat') }}</Hook0HelpText>
            <Hook0Code inline :code="'<service>.<resourceType>.<verb>'" />
          </template>
        </Hook0CardContentLine>
        <Hook0CardContentLine type="columns">
          <template #content>
            <Hook0Stack direction="column" gap="sm">
              <Hook0Text variant="primary" weight="bold" block>
                <Hook0Code inline code="<service>" />
                {{ t('eventTypes.serviceExamples') }}
              </Hook0Text>

              <Hook0List>
                <Hook0ListItem>
                  <template #left>billing</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>chat</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>contacts</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>connectors</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>file</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>iam</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>iap</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>integrations</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>logging</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>monitoring</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>storage</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>workflows</template>
                </Hook0ListItem>
              </Hook0List>
            </Hook0Stack>

            <Hook0Stack direction="column" gap="sm">
              <Hook0Text variant="primary" weight="bold" block>
                <Hook0Code inline code="<resourceType>" />
                {{ t('eventTypes.resourceTypeExamples') }}
              </Hook0Text>
              <Hook0List>
                <Hook0ListItem>
                  <template #left>project</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>action</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>comment</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>collaborator</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>teammember</template>
                </Hook0ListItem>
              </Hook0List>
            </Hook0Stack>

            <Hook0Stack direction="column" gap="sm">
              <Hook0Text variant="primary" weight="bold" block>
                <Hook0Code inline code="<verb>" />
                {{ t('eventTypes.verbExamples') }}
              </Hook0Text>
              <Hook0List>
                <Hook0ListItem>
                  <template #left>created</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>updated</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>deleted</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>copied</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>versioned</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>executed</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>completed</template>
                </Hook0ListItem>
              </Hook0List>
            </Hook0Stack>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
      <Hook0CardFooter> </Hook0CardFooter>
    </Hook0Card>
  </Hook0Form>
</template>

<style scoped>
/* No custom styles - Hook0 components handle layout */
</style>
