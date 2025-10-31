<template>
  <div>
    <Hook0Card>
      <Hook0CardHeader>
        <h2 class="text-2xl font-bold">Custom Retry Schedules</h2>
        <Hook0Button @click="createNewSchedule" class="ml-auto">
          <Hook0Icon name="plus" class="mr-2" />
          New Retry Schedule
        </Hook0Button>
      </Hook0CardHeader>
      
      <Hook0CardContent>
        <div v-if="loading" class="flex justify-center py-8">
          <Hook0Loader />
        </div>
        
        <div v-else-if="error" class="text-red-500 p-4 bg-red-50 rounded">
          <Hook0Icon name="alert-circle" class="inline mr-2" />
          {{ error }}
        </div>
        
        <div v-else-if="retrySchedules.length === 0" class="text-center py-8 text-gray-500">
          <Hook0Icon name="calendar-off" class="w-16 h-16 mx-auto mb-4 opacity-50" />
          <p class="text-lg">No custom retry schedules configured</p>
          <p class="mt-2">Create your first retry schedule to customize webhook delivery behavior</p>
        </div>
        
        <ag-grid-vue
          v-else
          class="ag-theme-alpine w-full"
          :style="{ height: '400px' }"
          :columnDefs="columnDefs"
          :rowData="retrySchedules"
          :defaultColDef="defaultColDef"
          @cell-clicked="onCellClicked"
        />
      </Hook0CardContent>
    </Hook0Card>
    
    <!-- Feature Information Card -->
    <Hook0Card class="mt-6">
      <Hook0CardHeader>
        <h3 class="text-lg font-semibold">About Custom Retry Schedules</h3>
      </Hook0CardHeader>
      <Hook0CardContent>
        <div class="space-y-3 text-sm text-gray-600">
          <p>
            Custom retry schedules allow you to define exactly how and when webhook deliveries should be retried after failures.
          </p>
          <ul class="list-disc list-inside space-y-1 ml-4">
            <li>Configure retry intervals to match your system's requirements</li>
            <li>Set maximum retry attempts to control resource usage</li>
            <li>Apply different schedules to different subscriptions</li>
            <li>Optimize for real-time or batch processing scenarios</li>
          </ul>
          <div class="bg-blue-50 p-3 rounded mt-4">
            <Hook0Icon name="info" class="inline mr-2 text-blue-600" />
            <span class="text-blue-800">Default retry schedule: 5s, 30s, 2m, 5m, 15m, 30m, 1h</span>
          </div>
        </div>
      </Hook0CardContent>
    </Hook0Card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { AgGridVue } from 'ag-grid-vue3';
import type { ColDef, CellClickedEvent } from 'ag-grid-community';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import { RetryScheduleService } from './RetryScheduleService';

const route = useRoute();
const router = useRouter();
const organizationId = route.params.organizationId as string;
const applicationId = route.params.applicationId as string;

const loading = ref(true);
const error = ref<string | null>(null);
const retrySchedules = ref<any[]>([]);

const columnDefs: ColDef[] = [
  {
    field: 'name',
    headerName: 'Schedule Name',
    flex: 1,
    minWidth: 200,
  },
  {
    field: 'retry_intervals',
    headerName: 'Retry Intervals',
    flex: 2,
    minWidth: 300,
    valueFormatter: (params) => {
      if (!params.value) return '';
      return params.value.join(', ');
    },
  },
  {
    field: 'max_attempts',
    headerName: 'Max Attempts',
    width: 120,
  },
  {
    field: 'is_active',
    headerName: 'Status',
    width: 100,
    cellRenderer: (params: any) => {
      const status = params.value ? 'Active' : 'Inactive';
      const color = params.value ? 'green' : 'gray';
      return `<span class="px-2 py-1 text-xs rounded-full bg-${color}-100 text-${color}-800">${status}</span>`;
    },
  },
  {
    field: 'subscription_count',
    headerName: 'Subscriptions',
    width: 120,
    valueFormatter: (params) => params.value || 0,
  },
  {
    field: 'created_at',
    headerName: 'Created',
    width: 150,
    valueFormatter: (params) => {
      if (!params.value) return '';
      return new Date(params.value).toLocaleDateString();
    },
  },
  {
    headerName: 'Actions',
    width: 100,
    cellRenderer: () => {
      return '<button class="text-blue-600 hover:text-blue-800">Edit</button>';
    },
  },
];

const defaultColDef: ColDef = {
  sortable: true,
  resizable: true,
};

const loadRetrySchedules = async () => {
  loading.value = true;
  error.value = null;
  
  try {
    const schedules = await RetryScheduleService.listSchedules(organizationId, applicationId);
    retrySchedules.value = schedules;
  } catch (err: any) {
    error.value = err.message || 'Failed to load retry schedules';
  } finally {
    loading.value = false;
  }
};

const createNewSchedule = () => {
  router.push({
    name: 'retry-schedules-new',
    params: { organizationId, applicationId },
  });
};

const onCellClicked = (event: CellClickedEvent) => {
  if (event.column?.getColId() === 'actions' || event.column?.getColId() === 'name') {
    router.push({
      name: 'retry-schedules-edit',
      params: { 
        organizationId, 
        applicationId,
        scheduleId: event.data.retry_schedule_id,
      },
    });
  }
};

onMounted(() => {
  loadRetrySchedules();
});
</script>

<style>
@import 'ag-grid-community/styles/ag-grid.css';
@import 'ag-grid-community/styles/ag-theme-alpine.css';
</style>