import http, { UUID } from '@/http';
import type { components } from '@/types';
import { useAuthStore } from '@/stores/auth';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];

export type Application = definitions['Application'];
export type ApplicationInfo = definitions['ApplicationInfo'];
export type ApplicationPost = definitions['ApplicationPost'];

export function create(application: ApplicationPost): Promise<Application> {
  return unwrapResponse(http.post<Application>('/applications', application)).then(
    (application) => {
      // we force the user token refresh so that the org/app selector refreshes its options
      return useAuthStore()
        .refresh()
        .then(() => application);
    }
  );
}

export function list(organization_id: UUID): Promise<Array<Application>> {
  return unwrapResponse(
    http.get<Array<Application>>('/applications', {
      params: {
        organization_id: organization_id,
      },
    })
  );
}

export function get(application_id: UUID): Promise<ApplicationInfo> {
  return unwrapResponse(http.get<ApplicationInfo>(`/applications/${application_id}`));
}

export function update(application_id: UUID, application: ApplicationPost): Promise<Application> {
  return unwrapResponse(http.put<Application>(`/applications/${application_id}`, application)).then(
    (application) => {
      // we force the user token refresh so that the org/app selector refreshes its options
      return useAuthStore()
        .refresh()
        .then(() => application);
    }
  );
}

export function remove(application_id: UUID): Promise<void> {
  return unwrapResponse(http.delete<void>(`/applications/${application_id}`, {})).then(
    (application) => {
      // we force the user token refresh so that the org/app selector refreshes its options
      return useAuthStore()
        .refresh()
        .then(() => application);
    }
  );
}
