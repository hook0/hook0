import http, { UUID } from '@/http';

export type AuthenticationType = 'oauth2' | 'bearer' | 'certificate' | 'basic' | 'custom';

export type OAuth2GrantType = 'client_credentials' | 'authorization_code' | 'password';

export interface OAuth2Config {
  grant_type: OAuth2GrantType;
  client_id: string;
  client_secret: string; // env://VARIABLE_NAME or encrypted value
  token_endpoint: string;
  scopes?: string[];
  token_refresh_threshold?: number;
  custom_headers?: Record<string, string>;
}

export interface BearerTokenConfig {
  token: string; // env://VARIABLE_NAME or encrypted value
  header_name?: string;
  prefix?: string;
}

export interface CertificateConfig {
  client_cert: string; // env://VARIABLE_NAME or encrypted value
  client_key: string; // env://VARIABLE_NAME or encrypted value
  ca_cert?: string; // env://VARIABLE_NAME or encrypted value
  verify_hostname?: boolean;
  mtls?: boolean;
}

export interface BasicAuthConfig {
  username: string;
  password: string; // env://VARIABLE_NAME or encrypted value
}

export interface CustomAuthConfig {
  headers: Record<string, string>;
  query_params?: Record<string, string>;
}

export type AuthenticationConfig = 
  | OAuth2Config 
  | BearerTokenConfig 
  | CertificateConfig 
  | BasicAuthConfig 
  | CustomAuthConfig;

export interface AuthenticationConfigRequest {
  type: AuthenticationType;
  config: AuthenticationConfig;
}

export interface AuthenticationConfigResponse {
  authentication_config_id: UUID;
  application_id: UUID;
  subscription_id?: UUID;
  auth_type: AuthenticationType;
  config: AuthenticationConfig;
  is_active: boolean;
}

export interface AuthenticationAuditLog {
  authentication_audit_log_id: UUID;
  subscription_id?: UUID;
  request_attempt_id?: UUID;
  authentication_type: string;
  is_success: boolean;
  error_message?: string;
  metadata?: Record<string, any>;
  created_at: string;
}

// Application authentication
export async function getApplicationAuthentication(application_id: UUID): Promise<AuthenticationConfigResponse | null> {
  try {
    const response = await http.get<AuthenticationConfigResponse>(`/applications/${application_id}/authentication`);
    return response.data;
  } catch (error: any) {
    if (error.status === 404) {
      return null;
    }
    throw error;
  }
}

export async function configureApplicationAuthentication(
  application_id: UUID,
  config: AuthenticationConfigRequest
): Promise<AuthenticationConfigResponse> {
  const response = await http.put<AuthenticationConfigResponse>(`/applications/${application_id}/authentication`, config);
  return response.data;
}

export async function deleteApplicationAuthentication(application_id: UUID): Promise<void> {
  await http.delete(`/applications/${application_id}/authentication`);
}

// Subscription authentication
export async function getSubscriptionAuthentication(subscription_id: UUID): Promise<AuthenticationConfigResponse | null> {
  try {
    const response = await http.get<AuthenticationConfigResponse>(`/subscriptions/${subscription_id}/authentication`);
    return response.data;
  } catch (error: any) {
    if (error.status === 404) {
      return null;
    }
    throw error;
  }
}

export async function configureSubscriptionAuthentication(
  subscription_id: UUID,
  config: AuthenticationConfigRequest
): Promise<AuthenticationConfigResponse> {
  const response = await http.put<AuthenticationConfigResponse>(`/subscriptions/${subscription_id}/authentication`, config);
  return response.data;
}

export async function deleteSubscriptionAuthentication(subscription_id: UUID): Promise<void> {
  await http.delete(`/subscriptions/${subscription_id}/authentication`);
}

// Audit logs
export async function getAuthenticationAuditLogs(
  filters?: {
    subscription_id?: UUID;
    limit?: number;
    offset?: number;
  }
): Promise<AuthenticationAuditLog[]> {
  const params = new URLSearchParams();
  if (filters?.subscription_id) params.append('subscription_id', filters.subscription_id);
  if (filters?.limit) params.append('limit', filters.limit.toString());
  if (filters?.offset) params.append('offset', filters.offset.toString());

  const response = await http.get<AuthenticationAuditLog[]>(`/authentication/audit-logs?${params.toString()}`);
  return response.data;
}

// Helper functions
export function isSecretReference(value: string): boolean {
  return value.startsWith('env://') || value.startsWith('encrypted://');
}

export function getSecretDisplayValue(value: string): string {
  if (value.startsWith('env://')) {
    return `Environment Variable: ${value.substring(6)}`;
  }
  if (value.startsWith('encrypted://')) {
    return `Encrypted Secret: ${value.substring(12)}`;
  }
  return value;
}

export function validateAuthenticationConfig(type: AuthenticationType, config: any): string[] {
  const errors: string[] = [];

  switch (type) {
    case 'oauth2':
      if (!config.grant_type) errors.push('Grant type is required');
      if (!config.client_id) errors.push('Client ID is required');
      if (!config.client_secret) errors.push('Client secret is required');
      if (!config.token_endpoint) errors.push('Token endpoint is required');
      if (config.token_endpoint && !isValidUrl(config.token_endpoint)) {
        errors.push('Token endpoint must be a valid URL');
      }
      break;

    case 'bearer':
      if (!config.token) errors.push('Token is required');
      break;

    case 'certificate':
      if (!config.client_cert) errors.push('Client certificate is required');
      if (!config.client_key) errors.push('Client key is required');
      break;

    case 'basic':
      if (!config.username) errors.push('Username is required');
      if (!config.password) errors.push('Password is required');
      break;

    case 'custom':
      if (!config.headers || Object.keys(config.headers).length === 0) {
        errors.push('At least one header is required');
      }
      break;
  }

  return errors;
}

function isValidUrl(url: string): boolean {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}