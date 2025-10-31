import { http } from '@/http';

export interface EndpointHealth {
  subscription_id: string;
  subscription_name: string;
  endpoint_url: string;
  status: 'healthy' | 'warning' | 'failed' | 'disabled';
  success_rate: number;
  avg_response_time: number;
  total_attempts: number;
  failed_attempts: number;
  last_failure: string | null;
  last_success: string | null;
  consecutive_failures: number;
  disabled_at: string | null;
  disabled_reason: string | null;
}

export interface EndpointHealthDetails {
  subscription_id: string;
  subscription_name: string;
  endpoint_url: string;
  current_status: 'healthy' | 'warning' | 'failed' | 'disabled';
  health_metrics: {
    success_rate_1h: number;
    success_rate_24h: number;
    success_rate_7d: number;
    avg_response_time_1h: number;
    avg_response_time_24h: number;
    avg_response_time_7d: number;
    total_attempts_1h: number;
    total_attempts_24h: number;
    total_attempts_7d: number;
  };
  recent_failures: Array<{
    timestamp: string;
    status_code: number;
    error_message: string;
    retry_count: number;
  }>;
  status_history: Array<{
    timestamp: string;
    old_status: string;
    new_status: string;
    reason: string;
  }>;
}

export interface AutoRecoverySettings {
  enabled: boolean;
  notifications: boolean;
  recovery_threshold?: number;
  disable_threshold?: number;
  monitoring_window?: string;
}

export interface HealthAlert {
  alert_id: string;
  subscription_id: string;
  alert_type: 'endpoint_disabled' | 'high_failure_rate' | 'slow_response' | 'endpoint_recovered';
  severity: 'info' | 'warning' | 'critical';
  message: string;
  details: Record<string, any>;
  created_at: string;
  acknowledged: boolean;
  acknowledged_by?: string;
  acknowledged_at?: string;
}

export class EndpointHealthService {
  static async getEndpointHealth(
    organizationId: string,
    applicationId: string
  ): Promise<EndpointHealth[]> {
    const response = await http.get(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/endpoints`
    );
    return response.data;
  }

  static async getEndpointHealthDetails(
    organizationId: string,
    applicationId: string,
    subscriptionId: string
  ): Promise<EndpointHealthDetails> {
    const response = await http.get(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/endpoints/${subscriptionId}`
    );
    return response.data;
  }

  static async updateEndpointStatus(
    organizationId: string,
    applicationId: string,
    subscriptionId: string,
    status: 'enabled' | 'disabled'
  ): Promise<void> {
    await http.post(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/endpoints/${subscriptionId}/status`,
      { status }
    );
  }

  static async triggerHealthCheck(
    organizationId: string,
    applicationId: string,
    subscriptionId: string
  ): Promise<{ healthy: boolean; response_time: number; error?: string }> {
    const response = await http.post(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/endpoints/${subscriptionId}/check`
    );
    return response.data;
  }

  static async getHealthAlerts(
    organizationId: string,
    applicationId: string,
    options?: {
      acknowledged?: boolean;
      severity?: 'info' | 'warning' | 'critical';
      limit?: number;
    }
  ): Promise<HealthAlert[]> {
    const params = new URLSearchParams();
    if (options?.acknowledged !== undefined) {
      params.append('acknowledged', String(options.acknowledged));
    }
    if (options?.severity) {
      params.append('severity', options.severity);
    }
    if (options?.limit) {
      params.append('limit', String(options.limit));
    }

    const response = await http.get(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/alerts?${params}`
    );
    return response.data;
  }

  static async acknowledgeAlert(
    organizationId: string,
    applicationId: string,
    alertId: string
  ): Promise<void> {
    await http.post(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/alerts/${alertId}/acknowledge`
    );
  }

  static async getAutoRecoverySettings(
    organizationId: string,
    applicationId: string
  ): Promise<AutoRecoverySettings> {
    const response = await http.get(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/auto-recovery`
    );
    return response.data;
  }

  static async updateAutoRecoverySettings(
    organizationId: string,
    applicationId: string,
    settings: AutoRecoverySettings
  ): Promise<void> {
    await http.put(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/auto-recovery`,
      settings
    );
  }

  static async getHealthMetrics(
    organizationId: string,
    applicationId: string,
    timeRange: '1h' | '24h' | '7d' | '30d' = '24h'
  ): Promise<{
    overall_success_rate: number;
    avg_response_time: number;
    total_endpoints: number;
    healthy_endpoints: number;
    warning_endpoints: number;
    failed_endpoints: number;
    disabled_endpoints: number;
    total_attempts: number;
    failed_attempts: number;
    time_series: Array<{
      timestamp: string;
      success_rate: number;
      avg_response_time: number;
      attempts: number;
      failures: number;
    }>;
  }> {
    const response = await http.get(
      `/api/v1/organizations/${organizationId}/applications/${applicationId}/health/metrics?range=${timeRange}`
    );
    return response.data;
  }

  static calculateHealthStatus(successRate: number, responseTime: number): 'healthy' | 'warning' | 'failed' {
    if (successRate < 0.95 || responseTime > 5000) {
      return 'failed';
    } else if (successRate < 0.99 || responseTime > 2000) {
      return 'warning';
    }
    return 'healthy';
  }

  static formatResponseTime(ms: number): string {
    if (ms < 1000) {
      return `${Math.round(ms)}ms`;
    } else if (ms < 60000) {
      return `${(ms / 1000).toFixed(1)}s`;
    } else {
      return `${(ms / 60000).toFixed(1)}m`;
    }
  }

  static getStatusColor(status: string): string {
    const colors: Record<string, string> = {
      healthy: 'green',
      warning: 'yellow',
      failed: 'red',
      disabled: 'gray',
    };
    return colors[status] || 'gray';
  }

  static getStatusIcon(status: string): string {
    const icons: Record<string, string> = {
      healthy: 'check-circle',
      warning: 'alert-triangle',
      failed: 'x-circle',
      disabled: 'pause-circle',
    };
    return icons[status] || 'help-circle';
  }
}