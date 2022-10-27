import {AxiosResponse} from 'axios';
import http, {UUID} from '@/http';
import type {components} from '@/types';

type definitions = components['schemas'];

export type ApplicationSecret = definitions['ApplicationSecret'];
export type ApplicationSecretPost = definitions['ApplicationSecretPost'];

export function create(application_secret: ApplicationSecretPost): Promise<ApplicationSecret> {
    return http.post('/application_secrets', application_secret).then((res: AxiosResponse<ApplicationSecret>) => res.data);
}

export function remove(application_id: string, application_secret_token: string): Promise<void> {
    return http.delete(`/application_secrets/${application_secret_token}`, {
        params: {
            application_id
        }
    }).then((res: AxiosResponse<void>) => res.data);
}

export function update(application_secret_token: string, application_secret: ApplicationSecretPost): Promise<ApplicationSecret> {
    return http
        .put(`/application_secrets/${application_secret_token}`, application_secret)
        .then((res: AxiosResponse<ApplicationSecret>) => res.data);
}

export function list(application_id: UUID): Promise<Array<ApplicationSecret>> {
    return http.get('/application_secrets', {
        params: {
            application_id: application_id,
        },
    }).then((res: AxiosResponse<Array<ApplicationSecret>>) => res.data);
}
