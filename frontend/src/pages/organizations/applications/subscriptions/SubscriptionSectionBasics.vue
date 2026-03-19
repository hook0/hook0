<script setup lang="ts">
import { useI18n } from 'vue-i18n';

import type { Hook0SelectSingleOption } from '@/components/Hook0Select';

import SubscriptionTestEndpoint from './SubscriptionTestEndpoint.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Button from '@/components/Hook0Button.vue';

const { t } = useI18n();

type Props = {
  description: string | undefined;
  descriptionAttrs: Record<string, unknown>;
  descriptionError: string | undefined;
  targetMethod: string | undefined;
  targetMethodAttrs: Record<string, unknown>;
  targetMethodError: string | undefined;
  targetUrl: string | undefined;
  targetUrlAttrs: Record<string, unknown>;
  targetUrlError: string | undefined;
  httpMethods: Hook0SelectSingleOption[];
};

defineProps<Props>();

const emit = defineEmits<{
  'update:description': [value: string];
  'update:targetMethod': [value: string];
  'update:targetUrl': [value: string];
}>();
</script>

<template>
  <div class="sub-section">
    <!-- Description -->
    <div class="sub-row">
      <div class="sub-row__label">
        <span class="sub-row__title">{{ t('subscriptions.descriptionLabel') }}</span>
        <span class="sub-row__hint">{{ t('subscriptions.descriptionHelpText') }}</span>
      </div>
      <div class="sub-row__content">
        <Hook0Input
          :model-value="description"
          v-bind="descriptionAttrs"
          type="text"
          :placeholder="t('subscriptions.descriptionPlaceholder')"
          :error="descriptionError"
          data-test="subscription-description-input"
          @update:model-value="emit('update:description', $event as string)"
        />
      </div>
    </div>

    <!-- Endpoint -->
    <div class="sub-row">
      <div class="sub-row__label">
        <span class="sub-row__title">{{ t('subscriptions.httpEndpoint') }}</span>
        <span class="sub-row__hint">
          <i18n-t keypath="subscriptions.webhookSiteHint" tag="span">
            <template #link>
              <Hook0Button variant="link" href="https://webhook.site" target="_blank"
                >webhook.site</Hook0Button
              >
            </template>
          </i18n-t>
        </span>
      </div>
      <div class="sub-row__content">
        <div class="sub-row__endpoint">
          <Hook0Select
            :model-value="targetMethod"
            v-bind="targetMethodAttrs"
            :options="httpMethods"
            :error="targetMethodError"
            data-test="subscription-method-select"
            @update:model-value="emit('update:targetMethod', $event as string)"
          />
          <Hook0Input
            :model-value="targetUrl"
            v-bind="targetUrlAttrs"
            type="text"
            placeholder="https://api.example.com/webhooks"
            :error="targetUrlError"
            data-test="subscription-url-input"
            @update:model-value="emit('update:targetUrl', $event as string)"
          />
          <SubscriptionTestEndpoint :target-url="targetUrl || ''" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sub-section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

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

/* Endpoint: [GET] [URL flex:1] [Test Endpoint] */
.sub-row__endpoint {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: flex-start;
}

.sub-row__endpoint > :first-child {
  flex-shrink: 0;
  width: 6rem;
}

.sub-row__endpoint > :nth-child(2) {
  flex: 1;
  min-width: 0;
}

.sub-row__endpoint :deep(.test-endpoint) {
  flex-shrink: 0;
  margin: 0;
}

@media (max-width: 639px) {
  .sub-row__endpoint :deep(.hook0-button) {
    flex-basis: 100%;
    justify-content: center;
  }
}

.sub-row__endpoint :deep(.test-endpoint .hook0-button) {
  height: 2.375rem;
}
</style>
