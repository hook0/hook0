<script setup lang="ts">
import { computed } from 'vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import { BasicAuthConfig } from './AuthenticationService';

interface Props {
  modelValue: BasicAuthConfig;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);

// Computed properties for v-model binding
const config = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const username = computed({
  get: () => config.value.username,
  set: (value) => {
    config.value = { ...config.value, username: value };
  },
});

const password = computed({
  get: () => config.value.password,
  set: (value) => {
    config.value = { ...config.value, password: value };
  },
});
</script>

<template>
  <div class="space-y-4">
    <!-- Username -->
    <Hook0CardContentLine>
      <template #label>Username</template>
      <template #content>
        <Hook0Input
          v-model="username"
          type="text"
          placeholder="api-user"
          required
        >
          <template #helpText>
            The username for Basic authentication
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Password -->
    <Hook0CardContentLine>
      <template #label>Password</template>
      <template #content>
        <Hook0Input
          v-model="password"
          type="text"
          placeholder="env://API_PASSWORD or enter directly"
          required
        >
          <template #helpText>
            The password for Basic authentication. Use env://VARIABLE_NAME to reference an environment variable
          </template>
        </Hook0Input>
      </template>
    </Hook0CardContentLine>

    <!-- Example -->
    <div class="mt-4 p-4 bg-gray-50 rounded-lg">
      <p class="text-sm font-semibold text-gray-700 mb-2">How it works:</p>
      <p class="text-sm text-gray-600 mb-2">
        Basic authentication combines the username and password with a colon, encodes them in Base64, 
        and sends them in the Authorization header.
      </p>
      <code class="text-sm bg-gray-100 px-2 py-1 rounded block">
        Authorization: Basic &lt;base64(username:password)&gt;
      </code>
    </div>

    <!-- Security Warning -->
    <div class="mt-4 p-4 bg-amber-50 border border-amber-200 rounded-lg">
      <p class="text-sm text-amber-800">
        <strong>Security Warning:</strong> Basic authentication transmits credentials in each request. 
        Always use HTTPS and consider using more secure authentication methods like OAuth2 when possible.
      </p>
    </div>
  </div>
</template>