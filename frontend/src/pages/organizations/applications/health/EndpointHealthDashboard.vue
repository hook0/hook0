<template>
  <div>
    <!-- Health Overview Cards -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
      <Hook0Card>
        <Hook0CardContent>
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-500">Healthy Endpoints</p>
              <p class="text-2xl font-bold text-green-600">{{ healthStats.healthy }}</p>
            </div>
            <Hook0Icon name="check-circle" class="w-8 h-8 text-green-500" />
          </div>
        </Hook0CardContent>
      </Hook0Card>
      
      <Hook0Card>
        <Hook0CardContent>
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-500">Warning</p>
              <p class="text-2xl font-bold text-yellow-600">{{ healthStats.warning }}</p>
            </div>
            <Hook0Icon name="alert-triangle" class="w-8 h-8 text-yellow-500" />
          </div>
        </Hook0CardContent>
      </Hook0Card>
      
      <Hook0Card>
        <Hook0CardContent>
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-500">Failed</p>
              <p class="text-2xl font-bold text-red-600">{{ healthStats.failed }}</p>
            </div>
            <Hook0Icon name="x-circle" class="w-8 h-8 text-red-500" />
          </div>
        </Hook0CardContent>
      </Hook0Card>
      
      <Hook0Card>
        <Hook0CardContent>
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-500">Disabled</p>
              <p class="text-2xl font-bold text-gray-600">{{ healthStats.disabled }}</p>
            </div>
            <Hook0Icon name="pause-circle" class="w-8 h-8 text-gray-500" />
          </div>
        </Hook0CardContent>
      </Hook0Card>
    </div>
    
    <!-- Endpoint Health Table -->
    <Hook0Card>
      <Hook0CardHeader>
        <h2 class="text-2xl font-bold">Endpoint Health Monitoring</h2>
        <div class="ml-auto flex items-center space-x-3">
          <Hook0Select
            v-model="filterStatus"
            :options="statusOptions"
            placeholder="All Statuses"
            class="w-40"
          />
          <Hook0Button @click="refreshData">
            <Hook0Icon name="refresh-cw" class="mr-2" />
            Refresh
          </Hook0Button>
        </div>
      </Hook0CardHeader>
      
      <Hook0CardContent>
        <div v-if="loading" class="flex justify-center py-8">
          <Hook0Loader />
        </div>
        
        <div v-else-if="error" class="text-red-500 p-4 bg-red-50 rounded">
          <Hook0Icon name="alert-circle" class="inline mr-2" />
          {{ error }}
        </div>
        
        <div v-else-if="endpoints.length === 0" class="text-center py-8 text-gray-500">
          <Hook0Icon name="activity" class="w-16 h-16 mx-auto mb-4 opacity-50" />
          <p class="text-lg">No endpoints found</p>
          <p class="mt-2">Create subscriptions to start monitoring endpoint health</p>
        </div>
        
        <ag-grid-vue
          v-else
          class="ag-theme-alpine w-full"
          :style="{ height: '500px' }"
          :columnDefs="columnDefs"
          :rowData="filteredEndpoints"
          :defaultColDef="defaultColDef"
          @cell-clicked="onCellClicked"
        />
      </Hook0CardContent>
    </Hook0Card>
    
    <!-- Auto-Recovery Settings -->
    <Hook0Card class="mt-6">
      <Hook0CardHeader>
        <h3 class="text-lg font-semibold">Auto-Recovery Settings</h3>
      </Hook0CardHeader>
      <Hook0CardContent>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <label class="flex items-center space-x-3">
              <input
                v-model="autoRecoverySettings.enabled"
                type="checkbox"
                class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <span class="text-sm font-medium text-gray-700">
                Enable automatic endpoint recovery
              </span>
            </label>
            <p class="ml-7 mt-1 text-sm text-gray-500">
              Automatically attempt to recover failed endpoints
            </p>
          </div>
          
          <div>
            <label class="flex items-center space-x-3">
              <input
                v-model="autoRecoverySettings.notifications"
                type="checkbox"
                class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <span class="text-sm font-medium text-gray-700">
                Send notifications on status changes
              </span>
            </label>
            <p class="ml-7 mt-1 text-sm text-gray-500">
              Receive alerts when endpoint health changes
            </p>
          </div>
        </div>
        
        <div class="mt-4 pt-4 border-t">
          <Hook0Button @click="saveAutoRecoverySettings" :disabled="savingSettings">
            <Hook0Loader v-if="savingSettings" class="mr-2 w-4 h-4" />
            Save Settings
          </Hook0Button>
        </div>
      </Hook0CardContent>
    </Hook0Card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { AgGridVue } from 'ag-grid-vue3';
import type { ColDef, CellClickedEvent } from 'ag-grid-community';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import { EndpointHealthService } from './EndpointHealthService';

const route = useRoute();
const router = useRouter();
const organizationId = route.params.organizationId as string;
const applicationId = route.params.applicationId as string;

const loading = ref(true);
const error = ref<string | null>(null);
const endpoints = ref<any[]>([]);
const filterStatus = ref<string>('');
const savingSettings = ref(false);

const autoRecoverySettings = ref({
  enabled: true,
  notifications: true,
});

const healthStats = computed(() => {
  return {
    healthy: endpoints.value.filter(e => e.status === 'healthy').length,
    warning: endpoints.value.filter(e => e.status === 'warning').length,
    failed: endpoints.value.filter(e => e.status === 'failed').length,
    disabled: endpoints.value.filter(e => e.status === 'disabled').length,
  };
});

const filteredEndpoints = computed(() => {
  if (!filterStatus.value) {
    return endpoints.value;
  }
  return endpoints.value.filter(e => e.status === filterStatus.value);
});

const statusOptions = [
  { value: '', label: 'All Statuses' },
  { value: 'healthy', label: 'Healthy' },
  { value: 'warning', label: 'Warning' },
  { value: 'failed', label: 'Failed' },
  { value: 'disabled', label: 'Disabled' },
];

const columnDefs: ColDef[] = [
  {
    field: 'status',
    headerName: 'Status',
    width: 100,
    cellRenderer: (params: any) => {
      const statusColors: Record<string, string> = {
        healthy: 'green',
        warning: 'yellow',
        failed: 'red',
        disabled: 'gray',
      };
      const color = statusColors[params.value] || 'gray';
      const icon = params.value === 'healthy' ? 'check' : 
                   params.value === 'warning' ? 'alert-triangle' :
                   params.value === 'failed' ? 'x' : 'pause';
      return `
        <div class="flex items-center">
          <span class="w-2 h-2 bg-${color}-500 rounded-full mr-2"></span>
          <span class="capitalize">${params.value}</span>
        </div>
      `;
    },
  },
  {
    field: 'subscription_name',
    headerName: 'Subscription',
    flex: 1,
    minWidth: 200,
  },
  {
    field: 'endpoint_url',
    headerName: 'Endpoint URL',
    flex: 2,
    minWidth: 300,
  },
  {
    field: 'success_rate',
    headerName: 'Success Rate (24h)',
    width: 150,
    valueFormatter: (params) => {
      if (params.value === null || params.value === undefined) return 'N/A';
      return `${(params.value * 100).toFixed(1)}%`;
    },
    cellClassRules: {
      'text-green-600': (params: any) => params.value >= 0.99,
      'text-yellow-600': (params: any) => params.value >= 0.95 && params.value < 0.99,
      'text-red-600': (params: any) => params.value < 0.95,
    },
  },
  {
    field: 'avg_response_time',
    headerName: 'Avg Response Time',
    width: 150,
    valueFormatter: (params) => {
      if (!params.value) return 'N/A';
      return `${params.value}ms`;
    },
  },
  {
    field: 'total_attempts',
    headerName: 'Total Attempts (24h)',
    width: 150,
    valueFormatter: (params) => params.value?.toLocaleString() || '0',
  },
  {
    field: 'failed_attempts',
    headerName: 'Failed (24h)',
    width: 120,
    valueFormatter: (params) => params.value?.toLocaleString() || '0',
    cellClassRules: {
      'text-red-600': (params: any) => params.value > 0,
    },
  },
  {
    field: 'last_failure',
    headerName: 'Last Failure',
    width: 150,
    valueFormatter: (params) => {
      if (!params.value) return 'Never';
      const date = new Date(params.value);
      const now = new Date();
      const diff = now.getTime() - date.getTime();
      const hours = Math.floor(diff / (1000 * 60 * 60));
      if (hours < 1) {
        const minutes = Math.floor(diff / (1000 * 60));
        return `${minutes}m ago`;
      } else if (hours < 24) {
        return `${hours}h ago`;
      } else {
        const days = Math.floor(hours / 24);
        return `${days}d ago`;
      }
    },
  },
  {
    headerName: 'Actions',
    width: 120,
    cellRenderer: (params: any) => {
      const actions = [];
      if (params.data.status === 'disabled') {
        actions.push('<button class="text-green-600 hover:text-green-800 mr-2">Enable</button>');
      } else {
        actions.push('<button class="text-yellow-600 hover:text-yellow-800 mr-2">Disable</button>');
      }
      actions.push('<button class="text-blue-600 hover:text-blue-800">Details</button>');
      return actions.join('');
    },
  },
];

const defaultColDef: ColDef = {
  sortable: true,
  resizable: true,
};

const loadEndpointHealth = async () => {
  loading.value = true;
  error.value = null;
  
  try {
    const healthData = await EndpointHealthService.getEndpointHealth(
      organizationId,
      applicationId
    );
    endpoints.value = healthData;
  } catch (err: any) {
    error.value = err.message || 'Failed to load endpoint health data';
  } finally {
    loading.value = false;
  }
};

const refreshData = () => {
  loadEndpointHealth();
};

const onCellClicked = (event: CellClickedEvent) => {
  const column = event.column?.getColId();
  
  if (column === 'actions') {
    const buttonText = (event.event?.target as HTMLElement)?.textContent;
    
    if (buttonText === 'Enable' || buttonText === 'Disable') {
      toggleEndpointStatus(event.data);
    } else if (buttonText === 'Details') {
      router.push({
        name: 'subscription-health-details',
        params: {
          organizationId,
          applicationId,
          subscriptionId: event.data.subscription_id,
        },
      });
    }
  } else if (column === 'subscription_name') {
    router.push({
      name: 'subscriptions-edit',
      params: {
        organizationId,
        applicationId,
        subscriptionId: event.data.subscription_id,
      },
    });
  }
};

const toggleEndpointStatus = async (endpoint: any) => {
  try {
    const newStatus = endpoint.status === 'disabled' ? 'enabled' : 'disabled';
    await EndpointHealthService.updateEndpointStatus(
      organizationId,
      applicationId,
      endpoint.subscription_id,
      newStatus
    );
    await loadEndpointHealth();
  } catch (err: any) {
    error.value = err.message || 'Failed to update endpoint status';
  }
};

const saveAutoRecoverySettings = async () => {
  savingSettings.value = true;
  
  try {
    await EndpointHealthService.updateAutoRecoverySettings(
      organizationId,
      applicationId,
      autoRecoverySettings.value
    );
  } catch (err: any) {
    error.value = err.message || 'Failed to save auto-recovery settings';
  } finally {
    savingSettings.value = false;
  }
};

// Auto-refresh every 30 seconds
let refreshInterval: number | undefined;

onMounted(() => {
  loadEndpointHealth();
  refreshInterval = window.setInterval(loadEndpointHealth, 30000);
});

onUnmounted(() => {
  if (refreshInterval) {
    window.clearInterval(refreshInterval);
  }
});
</script>

<style>
@import 'ag-grid-community/styles/ag-grid.css';
@import 'ag-grid-community/styles/ag-theme-alpine.css';
</style>