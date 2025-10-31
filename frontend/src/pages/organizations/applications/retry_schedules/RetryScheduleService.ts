import { http } from '@/http';

export interface RetrySchedule {
  retry_schedule_id: string;
  organization_id: string;
  application_id: string;
  name: string;
  retry_intervals: string[];
  max_attempts: number;
  is_active: boolean;
  subscription_count?: number;
  created_at: string;
  updated_at: string;
}

export interface CreateRetryScheduleInput {
  name: string;
  retry_intervals: string[];
  max_attempts: number;
  is_active: boolean;
}

export interface UpdateRetryScheduleInput {
  name?: string;
  retry_intervals?: string[];
  max_attempts?: number;
  is_active?: boolean;
}

export class RetryScheduleService {
  static async listSchedules(
    organizationId: string,
    applicationId: string
  ): Promise<RetrySchedule[]> {
    const response = await http.get(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/retry-schedules`
    );
    return response.data;
  }

  static async getSchedule(
    organizationId: string,
    applicationId: string,
    scheduleId: string
  ): Promise<RetrySchedule> {
    const response = await http.get(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/retry-schedules/${scheduleId}`
    );
    return response.data;
  }

  static async createSchedule(
    organizationId: string,
    applicationId: string,
    input: CreateRetryScheduleInput
  ): Promise<RetrySchedule> {
    const response = await http.post(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/retry-schedules`,
      input
    );
    return response.data;
  }

  static async updateSchedule(
    organizationId: string,
    applicationId: string,
    scheduleId: string,
    input: UpdateRetryScheduleInput
  ): Promise<RetrySchedule> {
    const response = await http.put(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/retry-schedules/${scheduleId}`,
      input
    );
    return response.data;
  }

  static async deleteSchedule(
    organizationId: string,
    applicationId: string,
    scheduleId: string
  ): Promise<void> {
    await http.delete(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/retry-schedules/${scheduleId}`
    );
  }

  static async getScheduleSubscriptions(
    organizationId: string,
    applicationId: string,
    scheduleId: string
  ): Promise<any[]> {
    const response = await http.get(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/retry-schedules/${scheduleId}/subscriptions`
    );
    return response.data;
  }

  static parseIntervals(intervalsString: string): string[] {
    // Parse intervals from a string like "5s, 30s, 2m, 5m, 15m"
    return intervalsString
      .split(',')
      .map(s => s.trim())
      .filter(s => s.length > 0);
  }

  static formatInterval(seconds: number): string {
    if (seconds < 60) {
      return `${seconds}s`;
    } else if (seconds < 3600) {
      const minutes = Math.floor(seconds / 60);
      const remainingSeconds = seconds % 60;
      return remainingSeconds > 0 ? `${minutes}m ${remainingSeconds}s` : `${minutes}m`;
    } else {
      const hours = Math.floor(seconds / 3600);
      const remainingMinutes = Math.floor((seconds % 3600) / 60);
      return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
    }
  }

  static parseIntervalToSeconds(interval: string): number {
    const match = interval.match(/^(\d+)([smh])$/);
    if (!match) {
      throw new Error(`Invalid interval format: ${interval}`);
    }
    
    const value = parseInt(match[1], 10);
    const unit = match[2];
    
    switch (unit) {
      case 's':
        return value;
      case 'm':
        return value * 60;
      case 'h':
        return value * 3600;
      default:
        throw new Error(`Unknown interval unit: ${unit}`);
    }
  }

  static validateSchedule(schedule: CreateRetryScheduleInput): string[] {
    const errors: string[] = [];
    
    if (!schedule.name || schedule.name.trim().length === 0) {
      errors.push('Schedule name is required');
    }
    
    if (!schedule.retry_intervals || schedule.retry_intervals.length === 0) {
      errors.push('At least one retry interval is required');
    } else {
      // Validate interval format
      for (const interval of schedule.retry_intervals) {
        if (!interval.match(/^\d+[smh]$/)) {
          errors.push(`Invalid interval format: ${interval}. Use format like "5s", "2m", or "1h"`);
        }
      }
    }
    
    if (schedule.max_attempts < 1) {
      errors.push('Maximum attempts must be at least 1');
    }
    
    if (schedule.max_attempts > 100) {
      errors.push('Maximum attempts cannot exceed 100');
    }
    
    return errors;
  }
}