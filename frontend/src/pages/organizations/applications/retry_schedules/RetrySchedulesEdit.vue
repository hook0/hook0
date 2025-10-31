<template>
  <div>
    <Hook0Card>
      <Hook0CardHeader>
        <h2 class="text-2xl font-bold">
          {{ isEditMode ? 'Edit Retry Schedule' : 'Create Retry Schedule' }}
        </h2>
      </Hook0CardHeader>
      
      <Hook0CardContent>
        <form @submit.prevent="handleSubmit" class="space-y-6">
          <!-- Schedule Name -->
          <div>
            <label for="name" class="block text-sm font-medium text-gray-700 mb-2">
              Schedule Name
            </label>
            <Hook0Input
              id="name"
              v-model="schedule.name"
              type="text"
              placeholder="e.g., Aggressive Retry, Conservative Retry"
              required
              class="w-full"
            />
            <p class="mt-1 text-sm text-gray-500">
              A descriptive name for this retry schedule
            </p>
          </div>
          
          <!-- Retry Intervals -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              Retry Intervals
            </label>
            <div class="space-y-2">
              <div
                v-for="(interval, index) in schedule.retry_intervals"
                :key="index"
                class="flex items-center space-x-2"
              >
                <Hook0Input
                  v-model="schedule.retry_intervals[index]"
                  type="text"
                  placeholder="e.g., 5s, 2m, 1h"
                  pattern="^\d+[smh]$"
                  required
                  class="flex-1"
                />
                <Hook0Button
                  type="button"
                  variant="outline"
                  @click="removeInterval(index)"
                  :disabled="schedule.retry_intervals.length === 1"
                >
                  <Hook0Icon name="trash-2" class="w-4 h-4" />
                </Hook0Button>
              </div>
              <Hook0Button
                type="button"
                variant="outline"
                @click="addInterval"
                class="w-full"
              >
                <Hook0Icon name="plus" class="mr-2" />
                Add Interval
              </Hook0Button>
            </div>
            <p class="mt-2 text-sm text-gray-500">
              Specify intervals between retry attempts (e.g., 5s, 2m, 1h)
            </p>
          </div>
          
          <!-- Max Attempts -->
          <div>
            <label for="maxAttempts" class="block text-sm font-medium text-gray-700 mb-2">
              Maximum Attempts
            </label>
            <Hook0Input
              id="maxAttempts"
              v-model.number="schedule.max_attempts"
              type="number"
              min="1"
              max="100"
              required
              class="w-48"
            />
            <p class="mt-1 text-sm text-gray-500">
              Maximum number of retry attempts (1-100)
            </p>
          </div>
          
          <!-- Active Status -->
          <div class="flex items-center space-x-3">
            <input
              id="isActive"
              v-model="schedule.is_active"
              type="checkbox"
              class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <label for="isActive" class="text-sm font-medium text-gray-700">
              Schedule is active
            </label>
          </div>
          
          <!-- Preview -->
          <div class="bg-gray-50 p-4 rounded-lg">
            <h3 class="text-sm font-medium text-gray-700 mb-2">Preview</h3>
            <div class="text-sm text-gray-600">
              <p>
                <strong>Total retry time:</strong> 
                {{ calculateTotalTime() }}
              </p>
              <p class="mt-1">
                <strong>Retry pattern:</strong>
                <span class="font-mono">{{ schedule.retry_intervals.join(' → ') }}</span>
              </p>
              <p class="mt-1">
                <strong>Attempts:</strong> 
                Initial attempt + {{ Math.min(schedule.max_attempts, schedule.retry_intervals.length) }} retries
              </p>
            </div>
          </div>
          
          <!-- Error Display -->
          <div v-if="errors.length > 0" class="bg-red-50 p-4 rounded-lg">
            <div class="flex">
              <Hook0Icon name="alert-circle" class="text-red-400 mr-2" />
              <div>
                <h3 class="text-sm font-medium text-red-800">Validation Errors</h3>
                <ul class="mt-2 text-sm text-red-600 list-disc list-inside">
                  <li v-for="error in errors" :key="error">{{ error }}</li>
                </ul>
              </div>
            </div>
          </div>
          
          <!-- Actions -->
          <div class="flex justify-between pt-4 border-t">
            <Hook0Button
              type="button"
              variant="outline"
              @click="handleCancel"
            >
              Cancel
            </Hook0Button>
            <div class="space-x-3">
              <Hook0Button
                v-if="isEditMode"
                type="button"
                variant="danger"
                @click="handleDelete"
              >
                Delete Schedule
              </Hook0Button>
              <Hook0Button
                type="submit"
                :disabled="loading"
              >
                <Hook0Loader v-if="loading" class="mr-2 w-4 h-4" />
                {{ isEditMode ? 'Update Schedule' : 'Create Schedule' }}
              </Hook0Button>
            </div>
          </div>
        </form>
      </Hook0CardContent>
    </Hook0Card>
    
    <!-- Affected Subscriptions (Edit Mode Only) -->
    <Hook0Card v-if="isEditMode && subscriptions.length > 0" class="mt-6">
      <Hook0CardHeader>
        <h3 class="text-lg font-semibold">Affected Subscriptions</h3>
      </Hook0CardHeader>
      <Hook0CardContent>
        <p class="text-sm text-gray-600 mb-4">
          The following subscriptions are using this retry schedule:
        </p>
        <Hook0List>
          <Hook0ListItem
            v-for="subscription in subscriptions"
            :key="subscription.subscription_id"
          >
            <div class="flex items-center justify-between">
              <div>
                <span class="font-medium">{{ subscription.name }}</span>
                <span class="text-gray-500 ml-2">{{ subscription.url }}</span>
              </div>
              <router-link
                :to="{
                  name: 'subscriptions-edit',
                  params: {
                    organizationId,
                    applicationId,
                    subscriptionId: subscription.subscription_id,
                  },
                }"
                class="text-blue-600 hover:text-blue-800"
              >
                View →
              </router-link>
            </div>
          </Hook0ListItem>
        </Hook0List>
      </Hook0CardContent>
    </Hook0Card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0List from '@/components/Hook0List.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import { RetryScheduleService } from './RetryScheduleService';
import type { CreateRetryScheduleInput } from './RetryScheduleService';

const route = useRoute();
const router = useRouter();
const organizationId = route.params.organizationId as string;
const applicationId = route.params.applicationId as string;
const scheduleId = route.params.scheduleId as string | undefined;

const isEditMode = computed(() => !!scheduleId);
const loading = ref(false);
const errors = ref<string[]>([]);
const subscriptions = ref<any[]>([]);

const schedule = ref<CreateRetryScheduleInput>({
  name: '',
  retry_intervals: ['5s', '30s', '2m', '5m', '15m'],
  max_attempts: 7,
  is_active: true,
});

const addInterval = () => {
  schedule.value.retry_intervals.push('30s');
};

const removeInterval = (index: number) => {
  if (schedule.value.retry_intervals.length > 1) {
    schedule.value.retry_intervals.splice(index, 1);
  }
};

const calculateTotalTime = () => {
  try {
    let totalSeconds = 0;
    const attempts = Math.min(schedule.value.max_attempts, schedule.value.retry_intervals.length);
    
    for (let i = 0; i < attempts; i++) {
      totalSeconds += RetryScheduleService.parseIntervalToSeconds(schedule.value.retry_intervals[i]);
    }
    
    return RetryScheduleService.formatInterval(totalSeconds);
  } catch {
    return 'Invalid intervals';
  }
};

const validateSchedule = (): boolean => {
  errors.value = RetryScheduleService.validateSchedule(schedule.value);
  return errors.value.length === 0;
};

const handleSubmit = async () => {
  if (!validateSchedule()) {
    return;
  }
  
  loading.value = true;
  errors.value = [];
  
  try {
    if (isEditMode.value) {
      await RetryScheduleService.updateSchedule(
        organizationId,
        applicationId,
        scheduleId!,
        schedule.value
      );
    } else {
      await RetryScheduleService.createSchedule(
        organizationId,
        applicationId,
        schedule.value
      );
    }
    
    router.push({
      name: 'retry-schedules',
      params: { organizationId, applicationId },
    });
  } catch (error: any) {
    errors.value = [error.message || 'Failed to save retry schedule'];
  } finally {
    loading.value = false;
  }
};

const handleDelete = async () => {
  if (!confirm('Are you sure you want to delete this retry schedule?')) {
    return;
  }
  
  loading.value = true;
  
  try {
    await RetryScheduleService.deleteSchedule(
      organizationId,
      applicationId,
      scheduleId!
    );
    
    router.push({
      name: 'retry-schedules',
      params: { organizationId, applicationId },
    });
  } catch (error: any) {
    errors.value = [error.message || 'Failed to delete retry schedule'];
  } finally {
    loading.value = false;
  }
};

const handleCancel = () => {
  router.push({
    name: 'retry-schedules',
    params: { organizationId, applicationId },
  });
};

const loadSchedule = async () => {
  if (!isEditMode.value) return;
  
  loading.value = true;
  
  try {
    const existingSchedule = await RetryScheduleService.getSchedule(
      organizationId,
      applicationId,
      scheduleId!
    );
    
    schedule.value = {
      name: existingSchedule.name,
      retry_intervals: existingSchedule.retry_intervals,
      max_attempts: existingSchedule.max_attempts,
      is_active: existingSchedule.is_active,
    };
    
    // Load subscriptions using this schedule
    subscriptions.value = await RetryScheduleService.getScheduleSubscriptions(
      organizationId,
      applicationId,
      scheduleId!
    );
  } catch (error: any) {
    errors.value = [error.message || 'Failed to load retry schedule'];
  } finally {
    loading.value = false;
  }
};

onMounted(() => {
  loadSchedule();
});
</script>