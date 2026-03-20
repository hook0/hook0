<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Lock } from 'lucide-vue-next';

import { useServiceTokenDetail } from './useServiceTokenQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import { useClipboardCopy } from '@/composables/useClipboardCopy';
import { routes } from '@/routes';
import { useRouteIds } from '@/composables/useRouteIds';

import ServiceTokenAiConfig from './ServiceTokenAiConfig.vue';
import ServiceTokenAttenuationForm from './ServiceTokenAttenuationForm.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';

const { t } = useI18n();
const router = useRouter();
const clipboardCopy = useClipboardCopy();
const { organizationId, serviceTokenId } = useRouteIds();

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

const aiTab = ref('claude');

function copyToken() {
  if (!serviceToken.value) return;
  clipboardCopy(serviceToken.value.biscuit);
}

function cancel() {
  void router.push({
    name: routes.ServicesTokenList,
    params: { organization_id: organizationId.value },
  });
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
              <Hook0Alert type="warning">
                <template #description>
                  {{ t('serviceTokens.tokenWarningFull') }}
                </template>
              </Hook0Alert>

              <div class="token-box">
                <Lock :size="16" class="token-box__icon" aria-hidden="true" />
                <span class="token-box__value" data-test="service-token-value">{{
                  serviceToken.biscuit
                }}</span>
                <Hook0Button variant="primary" size="sm" type="button" @click="copyToken">
                  {{ t('common.copy') }}
                </Hook0Button>
              </div>
            </Hook0Stack>
          </Hook0CardContent>
        </Hook0Card>

        <!-- AI Assistants Integration -->
        <ServiceTokenAiConfig v-model="aiTab" :token="serviceToken.biscuit" />

        <!-- Token Attenuation & Permissions -->
        <ServiceTokenAttenuationForm
          :biscuit-token="serviceToken.biscuit"
          :public-key="instanceConfig?.biscuit_public_key"
          :application-options="applicationOptions"
          :apps-loading="appsLoading"
          :apps-error="appsError ?? null"
          @cancel="cancel"
          @retry-apps="refetchApps()"
        />
      </Hook0Stack>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.token-section {
  padding: 0.5rem 1.25rem 1.25rem;
}

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
  word-break: break-all;
}
</style>
