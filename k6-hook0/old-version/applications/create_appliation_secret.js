import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, applicationId, masterApiKey) {
    let url = `${baseUrl}/api/v1/application_secrets/`
    let payload = JSON.stringify({
        application_id: applicationId,
        name: 'test_k6'
    })

    let params = {
        headers: {
            'Authorization': masterApiKey,
            'accept': 'application/json',
            'content-type': 'application/json'
        }
    }

    let res = http.post(url, payload, params)
    if(!check(res, {
        'Secret key created': (r) => r.status === 201 && r.body && r.body.includes('token')
    })) {
        return null;
    }

    return JSON.parse(res.body).token
}