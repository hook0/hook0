import http from 'k6/http';
import { check } from 'k6';

export default function(baseUrl, organizationId, masterApiKey) {
    const url = `${baseUrl}api/v1/applications/`;
    const payload = JSON.stringify({
        name: 'test_k6',
        organization_id: organizationId, // Ensure this value is not undefined or null
    });
    const params = {
        headers: {
            'Authorization': `Bearer ${masterApiKey}`,
            'accept': 'application/json',
            'content-type': 'application/json',
        },
    };
    const res = http.post(url, payload, params);
    if(!check(res, {
        'Application created': (r) => r.status === 201 && r.body && r.body.includes('organization_id') && r.body.includes('name') && r.body.includes('application_id'),
    })) {
        return null;
    }

    return JSON.parse(res.body).application_id;
}