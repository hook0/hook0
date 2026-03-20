<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';
import { addDays, addYears, isBefore } from 'date-fns';
import { Biscuit } from '@biscuit-auth/biscuit-wasm';
import { Lock, Copy, ShieldCheck, Zap } from 'lucide-vue-next';

import { useServiceTokenDetail } from './useServiceTokenQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import {
  attenuateBiscuit,
  attenuateBiscuitWithDatalog,
  parseBiscuitFromBase64,
  getBiscuitBlocks,
} from '@/utils/biscuit_auth';
import type { BiscuitBlockInfo } from '@/utils/biscuit_auth';
import { trySyncCall } from '@/utils/result';
import { routes } from '@/routes';
import { useTracking } from '@/composables/useTracking';
import { useRouteIds } from '@/composables/useRouteIds';

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
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Checkbox from '@/components/Hook0Checkbox.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Tabs from '@/components/Hook0Tabs.vue';

const { t } = useI18n();
const router = useRouter();
const { trackEvent } = useTracking();
const { organizationId, serviceTokenId } = useRouteIds();

// Queries
const {
  data: serviceToken,
  isLoading: tokenLoading,
  error: tokenError,
  refetch: refetchToken,
} = useServiceTokenDetail(serviceTokenId, organizationId);

const {
  data: rawApplications,
  isLoading: appsLoading,
  error: appsError,
  refetch: refetchApps,
} = useApplicationList(organizationId);

const { data: instanceConfig } = useInstanceConfig();

const applicationOptions = computed(() => [
  { label: '', value: '' },
  ...(rawApplications.value ?? []).map((a) => ({ label: a.name, value: a.application_id })),
]);

// MCP Configuration example
const mcpConfigExample = computed(() => {
  const token = serviceToken.value?.biscuit ?? t('serviceTokens.tokenPlaceholder');
  return JSON.stringify(
    {
      mcpServers: {
        hook0: {
          command: 'hook0-mcp',
          env: {
            HOOK0_API_TOKEN: token,
          },
        },
      },
    },
    null,
    2
  );
});

// AI Assistants tabs
const aiTab = ref('claude');
const aiTabs = [
  { id: 'claude', label: 'Claude' },
  { id: 'chatgpt', label: 'ChatGPT' },
  { id: 'generic', label: 'Generic MCP' },
];

const chatgptConfigExample = computed(() => {
  const token = serviceToken.value?.biscuit ?? t('serviceTokens.tokenPlaceholder');
  return JSON.stringify(
    {
      mcpServers: {
        hook0: {
          type: 'stdio',
          command: 'hook0-mcp',
          env: {
            HOOK0_API_TOKEN: token,
          },
        },
      },
    },
    null,
    2
  );
});

const genericMcpConfigExample = computed(() => {
  const token = serviceToken.value?.biscuit ?? t('serviceTokens.tokenPlaceholder');
  return `HOOK0_API_TOKEN=${token}\nhook0-mcp`;
});

function copyToken() {
  if (!serviceToken.value) return;
  navigator.clipboard.writeText(serviceToken.value.biscuit).then(
    () => {
      toast.success(t('common.copied'), { description: t('common.codeCopied') });
    },
    () => {
      toast.error(t('common.error'), { description: t('common.clipboardCopyError') });
    }
  );
}

function copyConfig() {
  const config =
    aiTab.value === 'claude'
      ? mcpConfigExample.value
      : aiTab.value === 'chatgpt'
        ? chatgptConfigExample.value
        : genericMcpConfigExample.value;
  navigator.clipboard.writeText(config).then(
    () => {
      toast.success(t('common.copied'), { description: t('common.codeCopied') });
    },
    () => {
      toast.error(t('common.error'), { description: t('common.clipboardCopyError') });
    }
  );
}

// Mode toggle: 'simple' or 'advanced'
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
  if (selectedExpiryPreset.value === 'none') {
    return null;
  }
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

// Update token preview when attenuated token changes (advanced mode)
watch(attenuatedBiscuit, (biscuit) => {
  if (biscuit) {
    tokenPreviewBlocks.value = getBiscuitBlocks(biscuit);
    tokenPreviewRaw.value = biscuit.toBase64();
  } else {
    tokenPreviewBlocks.value = [];
    tokenPreviewRaw.value = '';
  }
});

// Reset form when switching modes
watch(attenuationMode, () => {
  attenuatedBiscuit.value = null;
  selectedApplicationId.value = null;
  // Reset simple mode
  selectedExpiryPreset.value = '30d';
  customExpiryDate.value = null;
  // Reset advanced mode
  isDateExpirationAttenuation.value = false;
  dateAttenuation.value = null;
  customDatalogClaims.value = '';
});

function cancel() {
  void router.push({
    name: routes.ServicesTokenList,
    params: { organization_id: organizationId.value },
  });
}

/**
 * Shared validation and submission logic for both simple and advanced modes.
 * @param attenuateFn - The mode-specific function that produces the attenuated Biscuit.
 * @param trackLabel - The tracking label ('simple' or 'advanced').
 */
function validateAndSubmit(attenuateFn: () => Biscuit, trackLabel: string): void {
  const publicKey = instanceConfig.value?.biscuit_public_key;
  if (!publicKey) {
    toast.error(t('common.somethingWentWrong'), {
      description: t('serviceTokens.publicKeyError'),
      duration: 5000,
    });
    return;
  }

  if (!serviceToken.value) {
    toast.error(t('common.error'), {
      description: t('serviceTokens.invalidToken'),
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

function submitSimple() {
  const publicKey = instanceConfig.value?.biscuit_public_key;
  const expiryDate = computeExpiryDate();

  if (!selectedApplicationId.value && !expiryDate) {
    toast.error(t('common.error'), {
      description: t('serviceTokens.invalidForm'),
      duration: 5000,
    });
    return;
  }

  if (expiryDate && isBefore(expiryDate, new Date())) {
    toast.error(t('common.error'), {
      description: t('serviceTokens.invalidExpirationDate'),
      duration: 5000,
    });
    return;
  }

  validateAndSubmit(
    () =>
      attenuateBiscuit(
        serviceToken.value!.biscuit,
        selectedApplicationId.value,
        expiryDate,
        publicKey!
      ),
    'simple'
  );
}

function submitAdvanced() {
  const publicKey = instanceConfig.value?.biscuit_public_key;
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

  if (expiry && isBefore(expiry, new Date())) {
    toast.error(t('common.error'), {
      description: t('serviceTokens.invalidExpirationDate'),
      duration: 5000,
    });
    return;
  }

  validateAndSubmit(
    () =>
      attenuateBiscuitWithDatalog(
        serviceToken.value!.biscuit,
        selectedApplicationId.value,
        expiry,
        customDatalogClaims.value,
        publicKey!
      ),
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
  const publicKey = instanceConfig.value?.biscuit_public_key;
  if (!publicKey || !serviceToken.value) {
    return;
  }

  const result = trySyncCall(() => parseBiscuitFromBase64(serviceToken.value.biscuit, publicKey));

  if (!result.ok) {
    // Preview parsing failure is non-critical; leave preview empty
    return;
  }

  tokenPreviewBlocks.value = getBiscuitBlocks(result.value);
  tokenPreviewRaw.value = serviceToken.value.biscuit;
}
</script>

<template>
  <Hook0PageLayout
    :title="
      serviceToken
        ? t('serviceTokens.titleSingle', { name: serviceToken.name })
        : t('serviceTokens.title')
    "
  >
    <!-- Loading skeleton -->
    <Hook0Card v-if="tokenLoading">
      <Hook0CardHeader>
        <template #header>{{ t('serviceTokens.title') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0Stack direction="column" gap="md">
          <Hook0Skeleton size="hero" />
          <Hook0Skeleton size="block" />
          <Hook0Skeleton size="heading" />
        </Hook0Stack>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Error state -->
    <Hook0ErrorCard v-else-if="tokenError" :error="tokenError" @retry="refetchToken()" />

    <!-- Data loaded -->
    <template v-else-if="serviceToken">
      <Hook0Stack direction="column" gap="lg">
        <!-- Service Token Card -->
        <Hook0Card data-test="service-token-detail-card">
          <Hook0CardContent>
            <Hook0Stack direction="column" gap="md" class="token-section">
              <!-- Token value box -->
              <div class="token-box">
                <Lock :size="16" class="token-box__icon" aria-hidden="true" />
                <span class="token-box__value" data-test="service-token-value">{{
                  serviceToken.biscuit
                }}</span>
                <Hook0Button variant="primary" size="sm" type="button" @click="copyToken">
                  <Copy :size="14" aria-hidden="true" />
                  {{ t('common.copy') }}
                </Hook0Button>
              </div>

              <!-- Warning -->
              <Hook0Alert type="warning">
                <template #description>
                  {{ t('serviceTokens.tokenWarningFull') }}
                </template>
              </Hook0Alert>
            </Hook0Stack>
          </Hook0CardContent>
        </Hook0Card>

        <!-- AI Assistants Integration -->
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>{{ t('serviceTokens.aiIntegrationTitle') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <div class="ai-config-section">
              <Hook0Tabs v-model="aiTab" :tabs="aiTabs">
                <template #claude>
                  <Hook0Stack direction="column" gap="md">
                    <span class="ai-config-section__hint">{{
                      t('serviceTokens.addToConfig', { file: 'claude_desktop_config.json' })
                    }}</span>
                    <pre class="config-code-block"><code>{{ mcpConfigExample }}</code></pre>
                  </Hook0Stack>
                </template>
                <template #chatgpt>
                  <Hook0Stack direction="column" gap="md">
                    <span class="ai-config-section__hint">{{
                      t('serviceTokens.addToConfig', { file: 'chatgpt_config.json' })
                    }}</span>
                    <pre class="config-code-block"><code>{{ chatgptConfigExample }}</code></pre>
                  </Hook0Stack>
                </template>
                <template #generic>
                  <Hook0Stack direction="column" gap="md">
                    <span class="ai-config-section__hint">{{
                      t('serviceTokens.genericMcpHint')
                    }}</span>
                    <pre class="config-code-block"><code>{{ genericMcpConfigExample }}</code></pre>
                  </Hook0Stack>
                </template>
              </Hook0Tabs>
              <div class="ai-config-section__footer">
                <Hook0Button variant="primary" size="sm" type="button" @click="copyConfig">
                  {{ t('serviceTokens.copyConfig') }}
                </Hook0Button>
              </div>
            </div>
          </Hook0CardContent>
        </Hook0Card>

        <!-- Token Attenuation & Permissions -->
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
                    :class="{
                      'mode-toggle__option--active': attenuationMode === 'simple',
                    }"
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
                    :class="{
                      'mode-toggle__option--active': attenuationMode === 'advanced',
                    }"
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
          <Hook0ErrorCard v-else-if="appsError" :error="appsError" @retry="refetchApps()" />

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
                  @click="cancel"
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
                  @click="cancel"
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

                  <!-- Decoded/Raw tabs for advanced mode -->
                  <template v-if="attenuationMode === 'advanced'">
                    <TokenPreviewTabs :blocks="tokenPreviewTabBlocks" :raw="tokenPreviewRaw" />
                  </template>

                  <!-- Simple mode: just the raw token -->
                  <template v-else>
                    <Hook0Code :code="attenuatedBiscuit.toBase64()"></Hook0Code>
                  </template>
                </Hook0Stack>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
        </Hook0Card>
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
/* Mode Toggle */
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

/* Datalog Textarea */
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

/* Token section */
.token-section {
  padding: 0.5rem 1.25rem 1.25rem;
}

/* Token value box */
.token-box {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background-color: var(--color-bg-tertiary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
}

.token-box__icon {
  flex-shrink: 0;
  color: var(--color-text-muted);
}

.token-box__value {
  flex: 1;
  min-width: 0;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  line-height: 1.5;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  word-break: break-all;
}

/* AI Config section */
.ai-config-section {
  padding: 0 1.25rem 1.25rem;
}

.ai-config-section__hint {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.ai-config-section__footer {
  margin-top: 1rem;
}

/* Light code block for config */
.config-code-block {
  margin: 0;
  padding: 1rem;
  background-color: var(--color-bg-tertiary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  line-height: 1.7;
  color: var(--color-text-primary);
  overflow-x: auto;
  white-space: pre;
}
</style>
