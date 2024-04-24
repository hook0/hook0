import http from 'k6/http';
import { check } from 'k6';

export default function (base_url, auth_token, application_id, event_id, subscription_id) {
    let res = http.get(
        `${base_url}api/v1/request_attempts/?application_id=${application_id}&event_id=${event_id}&subscription_id=${subscription_id}`,
        {
            headers: {
                Authorization: auth_token,
                accept: 'application/json',
            },
        },
    );

    console.log(res.status)
    console.log(res.body)

    if(!check(res, {
        'List request attempts': (r) => r.status === 200 && r.body && r.body.includes('status'),
    })) return null;

    return JSON.parse(res.body).status;
}