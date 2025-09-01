<script setup lang="ts">
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-floating-promises */
import { ref, computed, onMounted } from 'vue';
import { UUID } from '@/http';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import { Hook0SelectSingleOption } from '@/components/Hook0Select';
import { push } from 'notivue';

import OAuth2ConfigForm from './OAuth2ConfigForm.vue';
import BearerTokenConfigForm from './BearerTokenConfigForm.vue';
import CertificateConfigForm from './CertificateConfigForm.vue';
import BasicAuthConfigForm from './BasicAuthConfigForm.vue';

import * as AuthenticationService from './AuthenticationService';

interface Props {
  applicationId?: UUID;
  subscriptionId?: UUID;
  isNew?: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits(['saved', 'deleted', 'cancelled']);

// State
const isLoading = ref(false);
const isSaving = ref(false);
const currentConfig = ref<AuthenticationService.AuthenticationConfigResponse | null>(null);

// Form state
const authenticationType = ref<AuthenticationService.AuthenticationType>('bearer');
const authConfig = ref<any>({});

// Authentication type options
const authTypeOptions: Hook0SelectSingleOption[] = [
  { value: 'oauth2', label: 'OAuth 2.0' },
  { value: 'bearer', label: 'Bearer Token' },
  { value: 'certificate', label: 'Client Certificate' },
  { value: 'basic', label: 'Basic Authentication' },
];

// Computed
const isSubscriptionConfig = computed(() => !!props.subscriptionId);
const configTitle = computed(() => {
  if (isSubscriptionConfig.value) {
    return props.isNew
      ? 'Configure Subscription Authentication'
      : 'Edit Subscription Authentication';
  }
  return props.isNew ? 'Configure Default Authentication' : 'Edit Default Authentication';
});

const configSubtitle = computed(() => {
  if (isSubscriptionConfig.value) {
    return 'Override the default authentication for this specific subscription';
  }
  return 'Set the default authentication for all subscriptions in this application';
});

// Methods
async function loadCurrentConfig() {
  isLoading.value = true;
  try {
    if (props.subscriptionId) {
      currentConfig.value = await AuthenticationService.getSubscriptionAuthentication(
        props.subscriptionId
      );
    } else if (props.applicationId) {
      currentConfig.value = await AuthenticationService.getApplicationAuthentication(
        props.applicationId
      );
    }

    if (currentConfig.value) {
      authenticationType.value = currentConfig.value.auth_type;
      authConfig.value = { ...currentConfig.value.config };
    }
  } catch (error: any) {
    push.error({
      title: 'Failed to load authentication',
      message: error.detail || 'Could not load authentication configuration',
      duration: 5000,
    });
  } finally {
    isLoading.value = false;
  }
}

async function save() {
  // Validate configuration
  const errors = AuthenticationService.validateAuthenticationConfig(
    authenticationType.value,
    authConfig.value as Record<string, unknown>
  );
  if (errors.length > 0) {
    push.warning({
      title: 'Validation Error',
      message: errors.join(', '),
      duration: 5000,
    });
    return;
  }

  isSaving.value = true;
  try {
    const configRequest: AuthenticationService.AuthenticationConfigRequest = {
      type: authenticationType.value,
      config: authConfig.value,
    };

    let response: AuthenticationService.AuthenticationConfigResponse;
    if (props.subscriptionId) {
      response = await AuthenticationService.configureSubscriptionAuthentication(
        props.subscriptionId,
        configRequest
      );
    } else if (props.applicationId) {
      response = await AuthenticationService.configureApplicationAuthentication(
        props.applicationId,
        configRequest
      );
    } else {
      throw new Error('No application or subscription ID provided');
    }

    push.success({
      title: 'Authentication configured',
      message: 'Authentication settings have been saved successfully',
      duration: 5000,
    });

    emit('saved', response);
  } catch (error: any) {
    push.error({
      title: 'Failed to save authentication',
      message: error.detail || 'Could not save authentication configuration',
      duration: 5000,
    });
  } finally {
    isSaving.value = false;
  }
}

async function deleteConfig() {
  if (!confirm('Are you sure you want to delete this authentication configuration?')) {
    return;
  }

  isSaving.value = true;
  try {
    if (props.subscriptionId) {
      await AuthenticationService.deleteSubscriptionAuthentication(props.subscriptionId);
    } else if (props.applicationId) {
      await AuthenticationService.deleteApplicationAuthentication(props.applicationId);
    }

    push.success({
      title: 'Authentication deleted',
      message: 'Authentication configuration has been removed',
      duration: 5000,
    });

    emit('deleted');
  } catch (error: any) {
    push.error({
      title: 'Failed to delete authentication',
      message: error.detail || 'Could not delete authentication configuration',
      duration: 5000,
    });
  } finally {
    isSaving.value = false;
  }
}

function cancel() {
  emit('cancelled');
}

// Handle authentication type change
function onAuthTypeChange(newType: string | null | undefined) {
  if (!newType) return;
  authenticationType.value = newType as AuthenticationService.AuthenticationType;
  // Reset config when type changes
  authConfig.value = getDefaultConfig(newType as AuthenticationService.AuthenticationType);
}

function getDefaultConfig(type: AuthenticationService.AuthenticationType): any {
  switch (type) {
    case 'oauth2':
      return {
        grant_type: 'client_credentials',
        client_id: '',
        client_secret: '',
        token_endpoint: '',
        scopes: [],
        token_refresh_threshold: 300,
        custom_headers: {},
      };
    case 'bearer':
      return {
        token: '',
        header_name: 'Authorization',
        prefix: 'Bearer',
      };
    case 'certificate':
      return {
        client_cert: '',
        client_key: '',
        ca_cert: '',
        verify_hostname: true,
        mtls: true,
      };
    case 'basic':
      return {
        username: '',
        password: '',
      };
    default:
      return {};
  }
}

// Lifecycle
onMounted(() => {
  if (!props.isNew) {
    loadCurrentConfig();
  } else {
    authConfig.value = getDefaultConfig(authenticationType.value);
  }
});
</script>

<template>
  <div>
    <Hook0Loader v-if="isLoading" />
    <Hook0Card v-else>
      <Hook0CardHeader>
        <template #header>{{ configTitle }}</template>
        <template #subtitle>{{ configSubtitle }}</template>
      </Hook0CardHeader>

      <Hook0CardContent>
        <!-- Authentication Type Selection -->
        <Hook0CardContentLine>
          <template #label>Authentication Type</template>
          <template #content>
            <Hook0Select
              :model-value="authenticationType"
              :options="authTypeOptions"
              placeholder="Select authentication type"
              @update:model-value="onAuthTypeChange"
            />
          </template>
        </Hook0CardContentLine>

        <!-- Dynamic Configuration Form based on selected type -->
        <div v-if="authenticationType" class="mt-4">
          <OAuth2ConfigForm v-if="authenticationType === 'oauth2'" v-model="authConfig" />
          <BearerTokenConfigForm v-else-if="authenticationType === 'bearer'" v-model="authConfig" />
          <CertificateConfigForm
            v-else-if="authenticationType === 'certificate'"
            v-model="authConfig"
          />
          <BasicAuthConfigForm v-else-if="authenticationType === 'basic'" v-model="authConfig" />
        </div>

        <!-- Info about secret storage -->
        <Hook0Alert type="warning" class="mt-4">
          <template #title>Secret Storage</template>
          <template #content>
            <p>Secrets can be stored in two ways:</p>
            <ul class="list-disc ml-5 mt-2">
              <li>
                <strong>Environment Variable:</strong> Use <code>env://VARIABLE_NAME</code> to
                reference an environment variable
              </li>
              <li>
                <strong>Encrypted:</strong> Enter the value directly and it will be encrypted and
                stored securely in the database
              </li>
            </ul>
          </template>
        </Hook0Alert>
      </Hook0CardContent>

      <Hook0CardFooter>
        <div class="flex justify-between">
          <div>
            <Hook0Button
              v-if="!props.isNew && currentConfig"
              variant="danger"
              :disabled="isSaving"
              @click="deleteConfig"
            >
              <Hook0Icon name="trash" />
              Delete Configuration
            </Hook0Button>
          </div>
          <div class="flex gap-2">
            <Hook0Button variant="secondary" :disabled="isSaving" @click="cancel">
              Cancel
            </Hook0Button>
            <Hook0Button variant="primary" :disabled="isSaving" @click="save">
              <Hook0Icon v-if="isSaving" name="spinner" class="animate-spin" />
              <Hook0Icon v-else name="save" />
              {{ props.isNew ? 'Create' : 'Update' }} Configuration
            </Hook0Button>
          </div>
        </div>
      </Hook0CardFooter>
    </Hook0Card>
  </div>
</template>