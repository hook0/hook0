<script setup lang="ts">
import { computed } from 'vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import { Hook0SelectSingleOption } from '@/components/Hook0Select';
import { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import { OAuth2Config } from './AuthenticationService';

interface Props {
  modelValue: OAuth2Config;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);

// Options
const grantTypeOptions: Hook0SelectSingleOption[] = [
  { value: 'client_credentials', label: 'Client Credentials' },
  // Authorization Code and Password grant types coming soon
];

// Computed properties for v-model binding
const config = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const clientId = computed({
  get: () => config.value.client_id,
  set: (value) => {
    config.value = { ...config.value, client_id: value };
  },
});

const clientSecret = computed({
  get: () => config.value.client_secret,
  set: (value) => {
    config.value = { ...config.value, client_secret: value };
  },
});

const tokenEndpoint = computed({
  get: () => config.value.token_endpoint,
  set: (value) => {
    config.value = { ...config.value, token_endpoint: value };
  },
});

const grantType = computed({
  get: () => config.value.grant_type,
  set: (value) => {
    config.value = { ...config.value, grant_type: value };
  },
});

const tokenRefreshThreshold = computed({
  get: () => config.value.token_refresh_threshold?.toString() || '300',
  set: (value) => {
    config.value = { ...config.value, token_refresh_threshold: parseInt(value) || 300 };
  },
});

const scopes = computed({
  get: () => (config.value.scopes || []).join(' '),
  set: (value) => {
    config.value = { 
      ...config.value, 
      scopes: value ? value.split(/\s+/).filter(s => s.length > 0) : [] 
    };
  },
});

const customHeaders = computed({
  get: () => {
    const headers = config.value.custom_headers || {};
    return Object.entries(headers).map(([key, value]) => ({ key, value }));
  },
  set: (pairs: Hook0KeyValueKeyValuePair[]) => {
    const headers: Record<string, string> = {};
    pairs.forEach(pair => {
      if (pair.key) {
        headers[pair.key] = pair.value;
      }
    });
    config.value = { ...config.value, custom_headers: headers };
  },
});
</script>

<template>
  <div class="space-y-4">
    <!-- Grant Type -->
    <Hook0CardContentLine>
      <template #label>Grant Type</template>
      <template #content>
        <Hook0Select
          v-model="grantType"
          :options="grantTypeOptions"
          placeholder="Select OAuth2 grant type"
        />
        <span class="text-sm text-gray-500 mt-1">
          The OAuth2 flow to use for authentication
        </span>
      </template>
    </Hook0CardContentLine>

    <!-- Client ID -->
    <Hook0CardContentLine>
      <template #label>Client ID</template>
      <template #content>
        <Hook0Input
          v-model="clientId"
          type="text"
          placeholder="your-client-id"
          required
        >
          <template #helpText>
            The OAuth2 client ID provided by the authorization server
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Client Secret -->
    <Hook0CardContentLine>
      <template #label>Client Secret</template>
      <template #content>
        <Hook0Input
          v-model="clientSecret"
          type="text"
          placeholder="env://OAUTH_CLIENT_SECRET or enter directly"
          required
        >
          <template #helpText>
            The OAuth2 client secret. Use env://VARIABLE_NAME to reference an environment variable
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Token Endpoint -->
    <Hook0CardContentLine>
      <template #label>Token Endpoint</template>
      <template #content>
        <Hook0Input
          v-model="tokenEndpoint"
          type="url"
          placeholder="https://auth.example.com/oauth/token"
          required
        >
          <template #helpText>
            The URL to obtain OAuth2 access tokens
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Scopes -->
    <Hook0CardContentLine>
      <template #label>Scopes (Optional)</template>
      <template #content>
        <Hook0Input
          v-model="scopes"
          type="text"
          placeholder="read write admin"
        >
          <template #helpText>
            Space-separated list of OAuth2 scopes to request
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Token Refresh Threshold -->
    <Hook0CardContentLine>
      <template #label>Token Refresh Threshold</template>
      <template #content>
        <Hook0Input
          v-model="tokenRefreshThreshold"
          type="number"
          placeholder="300"
          min="60"
          max="3600"
        >
          <template #helpText>
            Seconds before token expiration to trigger automatic refresh (default: 300)
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Custom Headers -->
    <Hook0CardContentLine>
      <template #label>Custom Headers (Optional)</template>
      <template #content>
        <Hook0KeyValue
          :value="customHeaders"
          @update:model-value="customHeaders = $event"
          key-placeholder="Header-Name"
          value-placeholder="Header-Value"
        />
        <span class="text-sm text-gray-500 mt-1">
          Additional headers to include in token requests
        </span>
      </template>
    </Hook0CardContentLine>
  </div>
</template>