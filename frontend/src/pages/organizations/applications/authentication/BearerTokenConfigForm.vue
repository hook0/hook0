<script setup lang="ts">
import { computed } from 'vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import { BearerTokenConfig } from './AuthenticationService';

interface Props {
  modelValue: BearerTokenConfig;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);

// Computed properties for v-model binding
const config = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const token = computed({
  get: () => config.value.token,
  set: (value) => {
    config.value = { ...config.value, token: value };
  },
});

const headerName = computed({
  get: () => config.value.header_name || 'Authorization',
  set: (value) => {
    config.value = { ...config.value, header_name: value };
  },
});

const prefix = computed({
  get: () => config.value.prefix !== undefined ? config.value.prefix : 'Bearer',
  set: (value) => {
    config.value = { ...config.value, prefix: value };
  },
});
</script>

<template>
  <div class="space-y-4">
    <!-- Bearer Token -->
    <Hook0CardContentLine>
      <template #label>Bearer Token</template>
      <template #content>
        <Hook0Input
          v-model="token"
          type="text"
          placeholder="env://API_TOKEN or enter directly"
          required
        >
          <template #helpText>
            The bearer token to use for authentication. Use env://VARIABLE_NAME to reference an environment variable
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Header Name -->
    <Hook0CardContentLine>
      <template #label>Header Name</template>
      <template #content>
        <Hook0Input
          v-model="headerName"
          type="text"
          placeholder="Authorization"
        >
          <template #helpText>
            The HTTP header name to use for the token (default: Authorization)
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Token Prefix -->
    <Hook0CardContentLine>
      <template #label>Token Prefix</template>
      <template #content>
        <Hook0Input
          v-model="prefix"
          type="text"
          placeholder="Bearer"
        >
          <template #helpText>
            The prefix to add before the token (default: Bearer). Leave empty for no prefix
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Example -->
    <div class="mt-4 p-4 bg-gray-50 rounded-lg">
      <p class="text-sm font-semibold text-gray-700 mb-2">Example Header:</p>
      <code class="text-sm bg-gray-100 px-2 py-1 rounded">
        {{ headerName || 'Authorization' }}: {{ prefix ? prefix + ' ' : '' }}&lt;token-value&gt;
      </code>
    </div>
  </div>
</template>