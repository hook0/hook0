import http, { UUID } from '@/http';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type ApplicationSecret = definitions['ApplicationSecret'];
export type ApplicationSecretPost = definitions['ApplicationSecretPost'];

export function create(application_secret: ApplicationSecretPost): Promise<ApplicationSecret> {
  return unwrapResponse(http.post<ApplicationSecret>('/application_secrets', application_secret));
}

export function remove(application_id: string, application_secret_token: string): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/application_secrets/${application_secret_token}`, {
      params: {
        application_id,
      },
    })
  );
}

export function update(
  application_secret_token: string,
  application_secret: ApplicationSecretPost
): Promise<ApplicationSecret> {
  return unwrapResponse(
    http.put<ApplicationSecret>(
      `/application_secrets/${application_secret_token}`,
      application_secret
    )
  );
}

export function list(application_id: UUID): Promise<Array<ApplicationSecret>> {
  return unwrapResponse(
    http.get<Array<ApplicationSecret>>('/application_secrets', {
      params: {
        application_id: application_id,
      },
    })
  );
}
