<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { UUID } from '@/http';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { push } from 'notivue';
import * as AuthenticationService from './AuthenticationService';

interface Props {
  subscriptionId?: UUID;
  applicationId?: UUID;
  limit?: number;
}

const props = withDefaults(defineProps<Props>(), {
  limit: 50,
});

// State
const isLoading = ref(false);
const auditLogs = ref<AuthenticationService.AuthenticationAuditLog[]>([]);
const offset = ref(0);
const hasMore = ref(true);

// Computed
const tableHeaders = computed(() => [
  { key: 'status', label: 'Status', width: '80px' },
  { key: 'created_at', label: 'Timestamp', width: '180px' },
  { key: 'authentication_type', label: 'Auth Type', width: '120px' },
  { key: 'subscription_id', label: 'Subscription', width: '200px' },
  { key: 'error_message', label: 'Error Message' },
]);

const formattedLogs = computed(() => 
  auditLogs.value.map(log => ({
    ...log,
    status_icon: log.is_success ? 'check-circle' : 'times-circle',
    status_color: log.is_success ? 'text-green-600' : 'text-red-600',
    auth_type_formatted: formatAuthType(log.authentication_type),
  }))
);

// Methods
function formatAuthType(type: string): string {
  switch (type) {
    case 'oauth2': return 'OAuth 2.0';
    case 'bearer': return 'Bearer Token';
    case 'certificate': return 'Certificate';
    case 'basic': return 'Basic Auth';
    default: return type;
  }
}

async function loadAuditLogs(append = false) {
  isLoading.value = true;
  try {
    const logs = await AuthenticationService.getAuthenticationAuditLogs({
      subscription_id: props.subscriptionId,
      limit: props.limit,
      offset: append ? offset.value : 0,
    });

    if (append) {
      auditLogs.value = [...auditLogs.value, ...logs];
    } else {
      auditLogs.value = logs;
    }

    hasMore.value = logs.length === props.limit;
    offset.value = auditLogs.value.length;
  } catch (error: any) {
    push.error({
      title: 'Failed to load audit logs',
      message: error.detail || 'Could not load authentication audit logs',
      duration: 5000,
    });
  } finally {
    isLoading.value = false;
  }
}

async function loadMore() {
  if (!hasMore.value || isLoading.value) return;
  await loadAuditLogs(true);
}

function refresh() {
  offset.value = 0;
  loadAuditLogs(false);
}

// Lifecycle
onMounted(() => {
  loadAuditLogs();
});
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Authentication Audit Log</template>
      <template #subtitle>
        View authentication attempts and their status
      </template>
      <template #actions>
        <Hook0Button variant="secondary" size="sm" @click="refresh" :disabled="isLoading">
          <Hook0Icon name="refresh" :class="{ 'animate-spin': isLoading }" />
          Refresh
        </Hook0Button>
      </template>
    </Hook0CardHeader>

    <Hook0CardContent>
      <Hook0Loader v-if="isLoading && auditLogs.length === 0" />
      
      <Hook0Alert v-else-if="auditLogs.length === 0" type="warning">
        <template #title>No authentication logs</template>
        <template #content>
          Authentication attempts will appear here once webhooks with authentication are sent.
        </template>
      </Hook0Alert>

      <div v-else>
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th 
                v-for="header in tableHeaders" 
                :key="header.key"
                :style="{ width: header.width }"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {{ header.label }}
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            <tr v-for="log in formattedLogs" :key="log.authentication_audit_log_id">
              <!-- Status -->
              <td class="px-6 py-4 whitespace-nowrap">
                <Hook0Icon 
                  :name="log.status_icon" 
                  :class="log.status_color"
                />
              </td>
              
              <!-- Timestamp -->
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {{ new Date(log.created_at).toLocaleString() }}
              </td>
              
              <!-- Auth Type -->
              <td class="px-6 py-4 whitespace-nowrap">
                <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800">
                  {{ log.auth_type_formatted }}
                </span>
              </td>
              
              <!-- Subscription -->
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                <code v-if="log.subscription_id" class="text-xs">
                  {{ log.subscription_id }}
                </code>
                <span v-else class="text-gray-400">-</span>
              </td>
              
              <!-- Error Message -->
              <td class="px-6 py-4 text-sm text-gray-900">
                <span v-if="log.error_message" class="text-red-600">
                  {{ log.error_message }}
                </span>
                <span v-else-if="log.is_success" class="text-green-600">
                  Success
                </span>
                <span v-else class="text-gray-400">-</span>
              </td>
            </tr>
          </tbody>
        </table>

        <!-- Load More Button -->
        <div v-if="hasMore" class="mt-4 text-center">
          <Hook0Button 
            variant="secondary" 
            @click="loadMore" 
            :disabled="isLoading"
          >
            <Hook0Icon v-if="isLoading" name="spinner" class="animate-spin" />
            <Hook0Icon v-else name="arrow-down" />
            Load More
          </Hook0Button>
        </div>
      </div>
    </Hook0CardContent>
  </Hook0Card>
</template>