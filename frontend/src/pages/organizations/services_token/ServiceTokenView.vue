<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { push } from 'notivue';
import { isBefore } from 'date-fns';
import { Biscuit } from '@biscuit-auth/biscuit-wasm';
import { Bot, BookOpen } from 'lucide-vue-next';

import { useServiceTokenDetail } from './useServiceTokenQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import { attenuateBiscuit } from '@/utils/biscuit_auth';
import { routes } from '@/routes';
import { useTracking } from '@/composables/useTracking';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
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
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Form from '@/components/Hook0Form.vue';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { trackEvent } = useTracking();

const organizationId = computed(() => route.params.organization_id as string);
const serviceTokenId = computed(() => route.params.service_token_id as string);

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
  const token = serviceToken.value?.biscuit ?? 'YOUR_TOKEN_HERE';
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

// Form state
const selectedApplicationId = ref<string | null>(null);
const isDateExpirationAttenuation = ref(false);
const dateAttenuation = ref<string | null>(null);
const attenuatedBiscuit = ref<Biscuit | null>(null);

function cancel() {
  void router.push({
    name: routes.ServicesTokenList,
    params: { organization_id: organizationId.value },
  });
}

function submit() {
  const publicKey = instanceConfig.value?.biscuit_public_key;
  if (!publicKey) {
    push.error({
      title: t('common.somethingWentWrong'),
      message: t('serviceTokens.publicKeyError'),
      duration: 5000,
    });
    return;
  }

  if (!serviceToken.value) {
    push.error({
      title: t('common.error'),
      message: t('serviceTokens.invalidToken'),
      duration: 5000,
    });
    return;
  }

  if (!selectedApplicationId.value && !dateAttenuation.value) {
    push.error({
      title: t('common.error'),
      message: t('serviceTokens.invalidForm'),
      duration: 5000,
    });
    return;
  }

  if (dateAttenuation.value && isBefore(dateAttenuation.value, new Date())) {
    push.error({
      title: t('common.error'),
      message: t('serviceTokens.invalidExpirationDate'),
      duration: 5000,
    });
    return;
  }

  try {
    attenuatedBiscuit.value = attenuateBiscuit(
      serviceToken.value.biscuit,
      selectedApplicationId.value,
      dateAttenuation.value ? new Date(dateAttenuation.value) : null,
      publicKey
    );
    trackEvent('service-token', 'attenuate', 'success');
    push.success({
      title: t('common.success'),
      message: t('serviceTokens.tokenGenerated'),
      duration: 5000,
    });
  } catch (e) {
    push.error({
      title: t('common.somethingWentWrong'),
      message: e instanceof Error ? e.message : t('serviceTokens.tokenGenerationError'),
      duration: 5000,
    });
  }
}
</script>

<template>
  <Hook0PageLayout :title="t('serviceTokens.title')">
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
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>
              <Hook0Stack direction="row" align="center" gap="sm">
                {{ t('serviceTokens.title') }}
                <Hook0HelpText tone="neutral">{{ serviceToken.name }}</Hook0HelpText>
              </Hook0Stack>
            </template>
            <template #subtitle>
              {{ t('serviceTokens.viewDescription') }}
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0CardContentLine>
              <template #label>
                <Hook0Stack direction="column" gap="sm">
                  {{ t('serviceTokens.title') }}
                  <Hook0HelpText tone="warning">
                    {{ t('serviceTokens.tokenWarning') }}
                  </Hook0HelpText>
                  <Hook0HelpText tone="emphasis">
                    {{ t('serviceTokens.dontShare') }}
                  </Hook0HelpText>
                </Hook0Stack>
              </template>
              <template #content>
                <Hook0Code :code="serviceToken.biscuit"></Hook0Code>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
        </Hook0Card>

        <!-- AI Assistant Configuration -->
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>
              <Hook0Stack direction="row" align="center" gap="sm">
                <Hook0IconBadge variant="primary">
                  <Bot :size="18" aria-hidden="true" />
                </Hook0IconBadge>
                {{ t('serviceTokens.useWithAI') }}
              </Hook0Stack>
            </template>
            <template #subtitle>
              {{ t('serviceTokens.aiConnectDescription') }}
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0CardContentLines>
              <Hook0CardContentLine type="full-width">
                <template #content>
                  <Hook0Stack direction="column" gap="md">
                    <Hook0Stack direction="column" gap="sm">
                      <Hook0HelpText tone="emphasis">
                        {{ t('serviceTokens.claudeDesktopConfig') }}
                      </Hook0HelpText>
                      <Hook0HelpText tone="neutral">
                        {{
                          t('serviceTokens.addToConfig', {
                            file: 'claude_desktop_config.json',
                          })
                        }}
                      </Hook0HelpText>
                      <Hook0Code :code="mcpConfigExample" language="json"></Hook0Code>
                    </Hook0Stack>
                    <Hook0Alert type="info">
                      <template #description>
                        <Hook0Stack direction="row" align="start" gap="sm">
                          <BookOpen :size="16" aria-hidden="true" />
                          <Hook0Stack direction="column" gap="none">
                            {{ t('serviceTokens.detailedSetupInstructions') }}
                            <Hook0Button
                              variant="link"
                              href="https://documentation.hook0.com/reference/mcp-for-ia-assistant"
                              target="_blank"
                            >
                              {{ t('serviceTokens.mcpIntegrationGuide') }}
                            </Hook0Button>
                          </Hook0Stack>
                        </Hook0Stack>
                      </template>
                    </Hook0Alert>
                  </Hook0Stack>
                </template>
              </Hook0CardContentLine>
            </Hook0CardContentLines>
          </Hook0CardContent>
        </Hook0Card>

        <!-- Attenuation Card -->
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>{{ t('serviceTokens.attenuateTitle') }}</template>
            <template #subtitle>
              {{ t('serviceTokens.attenuateDescription') }}
            </template>
          </Hook0CardHeader>

          <!-- Loading applications -->
          <Hook0CardContent v-if="appsLoading">
            <Hook0Stack direction="column" gap="md">
              <Hook0Skeleton size="hero" />
              <Hook0Skeleton size="heading" />
            </Hook0Stack>
          </Hook0CardContent>

          <!-- Error loading applications -->
          <Hook0ErrorCard v-else-if="appsError" :error="appsError" @retry="refetchApps()" />

          <!-- Attenuation form -->
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
              </Hook0CardContent>
              <Hook0CardFooter>
                <Hook0Button variant="secondary" type="button" @click="cancel">
                  {{ t('common.cancel') }}
                </Hook0Button>
                <Hook0Button variant="primary" submit>
                  {{ t('serviceTokens.generate') }}
                </Hook0Button>
              </Hook0CardFooter>
            </Hook0Form>
          </template>

          <Hook0CardContent v-if="attenuatedBiscuit">
            <Hook0CardContentLines>
              <Hook0Code :code="attenuatedBiscuit.toBase64()"></Hook0Code>
            </Hook0CardContentLines>
          </Hook0CardContent>
        </Hook0Card>
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
/* Component uses Hook0* components exclusively - no custom styles needed */
</style>
