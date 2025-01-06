import http from 'k6/http';

export default function (baseUrl, applicationId, serviceToken) {
    const url = `${baseUrl}api/v1/applications/${applicationId}`;
    const params = {
        headers: {
            Authorization: `Bearer ${serviceToken}`,
            accept: 'application/json',
            'content-type': 'application/json',
        },
    };
    const response = http.request('DELETE', url, null, params);
    if (response.status !== 204) {
        console.error('Failed to delete application:', response.status, response.body);
    }
}