import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

export default function (auth_token, base_url, application_id, event_type, labels) {
    let url = base_url + 'api/v1/event/';
    let headers = {
        'Authorization': auth_token,
        'accept': 'application/json',
        'content-type': 'application/json'
    };
    let payload = {
        "labels": labels,
        "application_id": application_id,
        "event_id": uuidv4(),
        "event_type": event_type,
        "occurred_at": "2022-11-04T16:12:58Z",
        "payload_content_type": "application/json",
        "payload": "{\"test_k6\": true}",
    };

    let res = http.post(url, JSON.stringify(payload), {headers: headers});

    if(!check(res, {
        'Ingest event': (r) => r.status === 201 && r.body && r.body.includes('event_id') && r.body.includes('received_at'),
    })) {
        return null;
    }

    return JSON.parse(res.body).event_id;
}