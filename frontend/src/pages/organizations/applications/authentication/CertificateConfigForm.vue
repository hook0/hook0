<script setup lang="ts">
import { computed } from 'vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import { CertificateConfig } from './AuthenticationService';

interface Props {
  modelValue: CertificateConfig;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);

// Computed properties for v-model binding
const config = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const clientCert = computed({
  get: () => config.value.client_cert,
  set: (value) => {
    config.value = { ...config.value, client_cert: value };
  },
});

const clientKey = computed({
  get: () => config.value.client_key,
  set: (value) => {
    config.value = { ...config.value, client_key: value };
  },
});

const caCert = computed({
  get: () => config.value.ca_cert || '',
  set: (value) => {
    config.value = { ...config.value, ca_cert: value || undefined };
  },
});

const verifyHostname = computed({
  get: () => config.value.verify_hostname !== false,
  set: (value) => {
    config.value = { ...config.value, verify_hostname: value };
  },
});

const mtls = computed({
  get: () => config.value.mtls !== false,
  set: (value) => {
    config.value = { ...config.value, mtls: value };
  },
});
</script>

<template>
  <div class="space-y-4">
    <!-- Client Certificate -->
    <Hook0CardContentLine>
      <template #label>Client Certificate</template>
      <template #content>
        <Hook0Input
          v-model="clientCert"
          type="text"
          placeholder="env://CLIENT_CERT_PEM or enter PEM content"
          required
        >
          <template #helpText>
            The client certificate in PEM format. Use env://VARIABLE_NAME to reference an
            environment variable
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Client Key -->
    <Hook0CardContentLine>
      <template #label>Client Private Key</template>
      <template #content>
        <Hook0Input
          v-model="clientKey"
          type="text"
          placeholder="env://CLIENT_KEY_PEM or enter PEM content"
          required
        >
          <template #helpText>
            The client private key in PEM format. Use env://VARIABLE_NAME to reference an
            environment variable
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- CA Certificate -->
    <Hook0CardContentLine>
      <template #label>CA Certificate (Optional)</template>
      <template #content>
        <Hook0Input
          v-model="caCert"
          type="text"
          placeholder="env://CA_CERT_PEM or enter PEM content"
        >
          <template #helpText>
            The Certificate Authority certificate for server verification. Leave empty to use system
            CA bundle
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Verify Hostname -->
    <Hook0CardContentLine>
      <template #label>Security Options</template>
      <template #content>
        <div class="space-y-2">
          <label class="flex items-center">
            <input
              v-model="verifyHostname"
              type="checkbox"
              class="mr-2 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span class="text-sm">Verify hostname in server certificate</span>
          </label>
          <label class="flex items-center">
            <input
              v-model="mtls"
              type="checkbox"
              class="mr-2 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span class="text-sm">Enable mutual TLS (mTLS)</span>
          </label>
        </div>
        <span class="text-sm text-gray-500 mt-2 block">
          Mutual TLS provides two-way authentication between client and server
        </span>
      </template>
    </Hook0CardContentLine>

    <!-- Warning -->
    <div class="mt-4 p-4 bg-amber-50 border border-amber-200 rounded-lg">
      <p class="text-sm text-amber-800">
        <strong>Security Note:</strong> Store certificates and keys securely. Use environment
        variables for production deployments to avoid exposing sensitive data in configuration.
      </p>
    </div>
  </div>
</template>