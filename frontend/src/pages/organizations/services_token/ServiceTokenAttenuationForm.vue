<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';
import { addDays, addYears, isBefore } from 'date-fns';
import { Biscuit } from '@biscuit-auth/biscuit-wasm';
import { ShieldCheck, Zap } from 'lucide-vue-next';

import {
  attenuateBiscuit,
  attenuateBiscuitWithDatalog,
  parseBiscuitFromBase64,
  getBiscuitBlocks,
} from '@/utils/biscuit_auth';
import type { BiscuitBlockInfo } from '@/utils/biscuit_auth';
import { trySyncCall } from '@/utils/result';
import { useTracking } from '@/composables/useTracking';

import TokenPreviewTabs from './TokenPreviewTabs.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Checkbox from '@/components/Hook0Checkbox.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';

type Props = {
  biscuitToken: string;
  publicKey: string | undefined;
  applicationOptions: Array<{ label: string; value: string }>;
  appsLoading: boolean;
  appsError: Error | null;
};

const props = defineProps<Props>();

const emit = defineEmits<{
  cancel: [];
  retryApps: [];
}>();

const { t } = useI18n();
const { trackEvent } = useTracking();

// Mode toggle
type AttenuationMode = 'simple' | 'advanced';
const attenuationMode = ref<AttenuationMode>('simple');

// Form state (shared)
const selectedApplicationId = ref<string | null>(null);
const attenuatedBiscuit = ref<Biscuit | null>(null);

// Simple mode state
type ExpiryPreset = '7d' | '30d' | '90d' | '1y' | 'custom' | 'none';
const selectedExpiryPreset = ref<ExpiryPreset>('30d');
const customExpiryDate = ref<string | null>(null);

const expiryPresetOptions = computed(() => [
  { label: t('serviceTokens.noExpiry'), value: 'none' },
  { label: t('serviceTokens.expiryOptions.7d'), value: '7d' },
  { label: t('serviceTokens.expiryOptions.30d'), value: '30d' },
  { label: t('serviceTokens.expiryOptions.90d'), value: '90d' },
  { label: t('serviceTokens.expiryOptions.1y'), value: '1y' },
  { label: t('serviceTokens.expiryOptions.custom'), value: 'custom' },
]);

// Advanced mode state
const isDateExpirationAttenuation = ref(false);
const dateAttenuation = ref<string | null>(null);
const customDatalogClaims = ref('');

// Token preview
const tokenPreviewBlocks = ref<Array<BiscuitBlockInfo>>([]);
const tokenPreviewRaw = ref('');

const tokenPreviewTabBlocks = computed(() =>
  tokenPreviewBlocks.value.map((b) => ({
    label: t('serviceTokens.blockNumber', { index: b.index }),
    code: b.source,
  }))
);

function computeExpiryDate(): Date | null {
  if (selectedExpiryPreset.value === 'none') return null;
  const now = new Date();
  switch (selectedExpiryPreset.value) {
    case '7d':
      return addDays(now, 7);
    case '30d':
      return addDays(now, 30);
    case '90d':
      return addDays(now, 90);
    case '1y':
      return addYears(now, 1);
    case 'custom':
      return customExpiryDate.value ? new Date(customExpiryDate.value) : null;
    default:
      return null;
  }
}

watch(attenuatedBiscuit, (biscuit) => {
  if (biscuit) {
    tokenPreviewBlocks.value = getBiscuitBlocks(biscuit);
    tokenPreviewRaw.value = biscuit.toBase64();
  } else {
    tokenPreviewBlocks.value = [];
    tokenPreviewRaw.value = '';
  }
});

watch(attenuationMode, () => {
  attenuatedBiscuit.value = null;
  selectedApplicationId.value = null;
  selectedExpiryPreset.value = '30d';
  customExpiryDate.value = null;
  isDateExpirationAttenuation.value = false;
  dateAttenuation.value = null;
  customDatalogClaims.value = '';
});

function validateAndSubmit(attenuateFn: () => Biscuit, trackLabel: string): void {
  const pubKey = props.publicKey;
  if (!pubKey) {
    toast.error(t('common.somethingWentWrong'), {
      description: t('serviceTokens.publicKeyError'),
      duration: 5000,
    });
    return;
  }

  const result = trySyncCall(attenuateFn);

  if (!result.ok) {
    toast.error(t('common.somethingWentWrong'), {
      description: result.error.message || t('serviceTokens.tokenGenerationError'),
      duration: 5000,
    });
    return;
  }

  attenuatedBiscuit.value = result.value;
  trackEvent('service-token', 'attenuate', trackLabel);
  toast.success(t('common.success'), {
    description: t('serviceTokens.tokenGenerated'),
    duration: 5000,
  });
}

function isExpiryDateValid(date: Date | null): boolean {
  if (date && isBefore(date, new Date())) {
    toast.error(t('common.error'), {
      description: t('serviceTokens.invalidExpirationDate'),
      duration: 5000,
    });
    return false;
  }
  return true;
}

function submitSimple() {
  const pubKey = props.publicKey;
  const token = props.biscuitToken;
  const expiryDate = computeExpiryDate();

  if (!selectedApplicationId.value && !expiryDate) {
    toast.error(t('common.error'), {
      description: t('serviceTokens.invalidForm'),
      duration: 5000,
    });
    return;
  }

  if (!isExpiryDateValid(expiryDate)) return;

  if (!pubKey) return;

  validateAndSubmit(
    () => attenuateBiscuit(token, selectedApplicationId.value, expiryDate, pubKey),
    'simple'
  );
}

function submitAdvanced() {
  const pubKey = props.publicKey;
  const token = props.biscuitToken;
  const expiry =
    isDateExpirationAttenuation.value && dateAttenuation.value
      ? new Date(dateAttenuation.value)
      : null;

  if (!selectedApplicationId.value && !expiry && customDatalogClaims.value.trim().length === 0) {
    toast.error(t('common.error'), {
      description: t('serviceTokens.invalidForm'),
      duration: 5000,
    });
    return;
  }

  if (!isExpiryDateValid(expiry)) return;

  if (!pubKey) return;

  validateAndSubmit(
    () =>
      attenuateBiscuitWithDatalog(token, selectedApplicationId.value, expiry, customDatalogClaims.value, pubKey),
    'advanced'
  );
}

function submit() {
  if (attenuationMode.value === 'simple') {
    submitSimple();
  } else {
    submitAdvanced();
  }
}

function previewToken() {
  const pubKey = props.publicKey;
  if (!pubKey) return;

  const result = trySyncCall(() => parseBiscuitFromBase64(props.biscuitToken, pubKey));
  if (!result.ok) return;

  tokenPreviewBlocks.value = getBiscuitBlocks(result.value);
  tokenPreviewRaw.value = props.biscuitToken;
}
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>{{ t('serviceTokens.attenuateTitle') }}</template>
      <template #subtitle>
        {{ t('serviceTokens.attenuateDescription') }}
      </template>
    </Hook0CardHeader>

    <!-- Mode Toggle -->
    <Hook0CardContent>
      <Hook0CardContentLine>
        <template #label>
          <Hook0Stack direction="column" gap="xs">
            {{ t('serviceTokens.modeToggleLabel') }}
            <Hook0HelpText tone="info">
              {{
                attenuationMode === 'simple'
                  ? t('serviceTokens.simpleDescription')
                  : t('serviceTokens.advancedDescription')
              }}
            </Hook0HelpText>
          </Hook0Stack>
        </template>
        <template #content>
          <div
            class="mode-toggle"
            role="radiogroup"
            :aria-label="t('serviceTokens.modeToggleLabel')"
          >
            <button
              type="button"
              class="mode-toggle__option"
              :class="{ 'mode-toggle__option--active': attenuationMode === 'simple' }"
              role="radio"
              :aria-checked="attenuationMode === 'simple'"
              @click="attenuationMode = 'simple'"
            >
              <Zap :size="16" aria-hidden="true" />
              {{ t('serviceTokens.simpleMode') }}
            </button>
            <button
              type="button"
              class="mode-toggle__option"
              :class="{ 'mode-toggle__option--active': attenuationMode === 'advanced' }"
              role="radio"
              :aria-checked="attenuationMode === 'advanced'"
              @click="attenuationMode = 'advanced'"
            >
              <ShieldCheck :size="16" aria-hidden="true" />
              {{ t('serviceTokens.advancedMode') }}
            </button>
          </div>
        </template>
      </Hook0CardContentLine>
    </Hook0CardContent>

    <!-- Loading applications -->
    <Hook0CardContent v-if="appsLoading">
      <Hook0Stack direction="column" gap="md">
        <Hook0Skeleton size="hero" />
        <Hook0Skeleton size="heading" />
      </Hook0Stack>
    </Hook0CardContent>

    <!-- Error loading applications -->
    <Hook0ErrorCard v-else-if="appsError" :error="appsError" @retry="emit('retryApps')" />

    <!-- Simple Mode Form -->
    <template v-else-if="attenuationMode === 'simple'">
      <Hook0Form @submit="submit">
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label>
              <Hook0Stack direction="column" gap="xs">
                {{ t('serviceTokens.reduceScope') }}
                <Hook0HelpText tone="emphasis">
                  {{ t('serviceTokens.reduceScopeHelp') }}
                </Hook0HelpText>
              </Hook0Stack>
            </template>
            <template #content>
              <Hook0Select
                v-model="selectedApplicationId"
                :options="applicationOptions"
              ></Hook0Select>
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine>
            <template #label>
              <Hook0Stack direction="column" gap="xs">
                {{ t('serviceTokens.expiry') }}
                <Hook0HelpText tone="emphasis">
                  {{ t('serviceTokens.setExpirationHelp') }}
                </Hook0HelpText>
              </Hook0Stack>
            </template>
            <template #content>
              <Hook0Select
                v-model="selectedExpiryPreset"
                :options="expiryPresetOptions"
              ></Hook0Select>
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine v-if="selectedExpiryPreset === 'custom'">
            <template #label>{{ t('serviceTokens.customExpiryDate') }}</template>
            <template #content>
              <Hook0Input v-model="customExpiryDate" type="datetime-local"></Hook0Input>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
        <Hook0CardFooter>
          <Hook0Button
            variant="secondary"
            type="button"
            data-test="service-token-back-button"
            @click="emit('cancel')"
          >
            {{ t('common.cancel') }}
          </Hook0Button>
          <Hook0Button variant="primary" submit>
            {{ t('serviceTokens.generateAttenuated') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Form>
    </template>

    <!-- Advanced Mode Form -->
    <template v-else>
      <Hook0Form @submit="submit">
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label>
              <Hook0Stack direction="column" gap="xs">
                {{ t('serviceTokens.reduceScope') }}
                <Hook0HelpText tone="emphasis">
                  {{ t('serviceTokens.reduceScopeHelp') }}
                </Hook0HelpText>
              </Hook0Stack>
            </template>
            <template #content>
              <Hook0Select
                v-model="selectedApplicationId"
                :options="applicationOptions"
              ></Hook0Select>
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine>
            <template #label>
              <Hook0Stack direction="column" gap="xs">
                {{ t('serviceTokens.setExpiration') }}
                <Hook0HelpText tone="emphasis">
                  {{ t('serviceTokens.setExpirationHelp') }}
                </Hook0HelpText>
              </Hook0Stack>
            </template>
            <template #content>
              <Hook0Checkbox v-model="isDateExpirationAttenuation" />
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine v-if="isDateExpirationAttenuation">
            <template #label>{{ t('serviceTokens.expirationDate') }}</template>
            <template #content>
              <Hook0Input v-model="dateAttenuation" type="datetime-local"></Hook0Input>
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine type="full-width">
            <template #label>
              <Hook0Stack direction="column" gap="xs">
                {{ t('serviceTokens.customDatalogClaims') }}
                <Hook0HelpText tone="emphasis">
                  {{ t('serviceTokens.customDatalogClaimsHelp') }}
                </Hook0HelpText>
              </Hook0Stack>
            </template>
            <template #content>
              <textarea
                v-model="customDatalogClaims"
                class="attenuation-form__datalog-textarea"
                :placeholder="t('serviceTokens.customDatalogClaimsPlaceholder')"
                rows="4"
                spellcheck="false"
              ></textarea>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>
        <Hook0CardFooter>
          <Hook0Button
            variant="secondary"
            type="button"
            data-test="service-token-back-button"
            @click="emit('cancel')"
          >
            {{ t('common.cancel') }}
          </Hook0Button>
          <Hook0Button variant="ghost" type="button" @click="previewToken">
            {{ t('serviceTokens.tokenPreview') }}
          </Hook0Button>
          <Hook0Button variant="primary" submit>
            {{ t('serviceTokens.generateAttenuated') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Form>

      <!-- Token Preview (Advanced mode only, before generation) -->
      <Hook0CardContent v-if="tokenPreviewBlocks.length > 0 && !attenuatedBiscuit">
        <Hook0CardContentLine type="full-width">
          <template #label>
            {{ t('serviceTokens.tokenPreview') }}
          </template>
          <template #content>
            <TokenPreviewTabs :blocks="tokenPreviewTabBlocks" :raw="tokenPreviewRaw" />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
    </template>

    <!-- Generated Token Result -->
    <Hook0CardContent v-if="attenuatedBiscuit">
      <Hook0CardContentLine type="full-width">
        <template #content>
          <Hook0Stack direction="column" gap="md">
            <Hook0Alert type="warning">
              <template #description>
                {{ t('serviceTokens.tokenDisplayWarning') }}
              </template>
            </Hook0Alert>

            <template v-if="attenuationMode === 'advanced'">
              <TokenPreviewTabs :blocks="tokenPreviewTabBlocks" :raw="tokenPreviewRaw" />
            </template>

            <template v-else>
              <Hook0Code :code="attenuatedBiscuit.toBase64()"></Hook0Code>
            </template>
          </Hook0Stack>
        </template>
      </Hook0CardContentLine>
    </Hook0CardContent>
  </Hook0Card>
</template>

<style scoped>
.mode-toggle {
  display: inline-flex;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.mode-toggle__option {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: var(--color-bg-primary);
  border: none;
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.mode-toggle__option:first-child {
  border-right: 1px solid var(--color-border);
}

.mode-toggle__option:hover {
  background-color: var(--color-bg-secondary);
}

.mode-toggle__option:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.mode-toggle__option--active {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.mode-toggle__option--active:hover {
  background-color: var(--color-primary-light);
}

.attenuation-form__datalog-textarea {
  display: block;
  width: 100%;
  padding: 0.75rem;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  line-height: 1.5;
  color: var(--color-text-primary);
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
  resize: vertical;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.attenuation-form__datalog-textarea::placeholder {
  color: var(--color-text-muted);
  opacity: 1;
}

.attenuation-form__datalog-textarea:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow:
    0 0 0 1px var(--color-primary),
    var(--shadow-sm);
}
</style>
