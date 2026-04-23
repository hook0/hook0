import http from 'k6/http';
import { check } from 'k6';

function headers(serviceToken) {
  return {
    headers: {
      Authorization: `Bearer ${serviceToken}`,
      'Content-Type': 'application/json',
    },
  };
}

export function createExponential(baseUrl, organizationId, serviceToken, name) {
  const url = `${baseUrl}api/v1/retry_schedules`;
  const payload = JSON.stringify({
    organization_id: organizationId,
    strategy: 'exponential_increasing',
    name,
    max_retries: 5,
    base_delay_secs: 60,
    wait_factor: 2,
  });
  const res = http.post(url, payload, headers(serviceToken));
  check(res, {
    'Retry schedule (exp) created': (response) =>
      response.status === 201 && response.body && response.body.includes('retry_schedule_id'),
  });
  return res.status === 201 ? JSON.parse(res.body) : null;
}

export function createLinear(baseUrl, organizationId, serviceToken, name) {
  const url = `${baseUrl}api/v1/retry_schedules`;
  const payload = JSON.stringify({
    organization_id: organizationId,
    strategy: 'linear',
    name,
    max_retries: 3,
    delay_secs: 60,
  });
  const res = http.post(url, payload, headers(serviceToken));
  check(res, {
    'Retry schedule (linear) created': (response) =>
      response.status === 201 && response.body && response.body.includes('retry_schedule_id'),
  });
  return res.status === 201 ? JSON.parse(res.body) : null;
}

export function createCustom(baseUrl, organizationId, serviceToken, name) {
  const url = `${baseUrl}api/v1/retry_schedules`;
  const payload = JSON.stringify({
    organization_id: organizationId,
    strategy: 'custom',
    name,
    custom_intervals_secs: [30, 60, 120],
  });
  const res = http.post(url, payload, headers(serviceToken));
  check(res, {
    'Retry schedule (custom) created': (response) =>
      response.status === 201 && response.body && response.body.includes('retry_schedule_id'),
  });
  return res.status === 201 ? JSON.parse(res.body) : null;
}

export function listSchedules(baseUrl, organizationId, serviceToken) {
  const url = `${baseUrl}api/v1/retry_schedules?organization_id=${organizationId}`;
  const res = http.get(url, headers(serviceToken));
  check(res, { 'Retry schedule list ok': (response) => response.status === 200 });
  return res.status === 200 ? JSON.parse(res.body) : null;
}

export function getSchedule(baseUrl, serviceToken, scheduleId) {
  const url = `${baseUrl}api/v1/retry_schedules/${scheduleId}`;
  const res = http.get(url, headers(serviceToken));
  check(res, { 'Retry schedule get ok': (response) => response.status === 200 });
  return res.status === 200 ? JSON.parse(res.body) : null;
}

export function updateSchedule(baseUrl, serviceToken, scheduleId, name) {
  const url = `${baseUrl}api/v1/retry_schedules/${scheduleId}`;
  const payload = JSON.stringify({
    strategy: 'linear',
    name,
    max_retries: 10,
    delay: 120,
  });
  const res = http.put(url, payload, headers(serviceToken));
  check(res, { 'Retry schedule updated': (response) => response.status === 200 });
  return res.status === 200 ? JSON.parse(res.body) : null;
}

export function deleteSchedule(baseUrl, serviceToken, scheduleId) {
  const url = `${baseUrl}api/v1/retry_schedules/${scheduleId}`;
  const res = http.del(url, null, headers(serviceToken));
  check(res, { 'Retry schedule deleted': (response) => response.status === 204 });
  return res.status === 204;
}

export function rejectsTooManyRetries(baseUrl, organizationId, serviceToken) {
  const url = `${baseUrl}api/v1/retry_schedules`;
  const payload = JSON.stringify({
    organization_id: organizationId,
    strategy: 'linear',
    name: 'too-many-retries',
    max_retries: 16,
    delay: 60,
  });
  const res = http.post(url, payload, headers(serviceToken));
  check(res, { 'Over-cap max_retries rejected': (response) => response.status === 400 });
}

export function rejectsTooShortDelay(baseUrl, organizationId, serviceToken) {
  const url = `${baseUrl}api/v1/retry_schedules`;
  const payload = JSON.stringify({
    organization_id: organizationId,
    strategy: 'linear',
    name: 'zero-delay',
    max_retries: 3,
    delay: 0,
  });
  const res = http.post(url, payload, headers(serviceToken));
  check(res, { 'Zero delay rejected': (response) => response.status === 400 });
}

export function rejectsTotalOverCap(baseUrl, organizationId, serviceToken) {
  const url = `${baseUrl}api/v1/retry_schedules`;
  const payload = JSON.stringify({
    organization_id: organizationId,
    strategy: 'custom',
    name: 'total-too-long',
    intervals: [604800, 604800, 604800],
  });
  const res = http.post(url, payload, headers(serviceToken));
  check(res, { 'Total duration over 7d rejected': (response) => response.status === 400 });
}

export function rejectsDuplicateName(baseUrl, organizationId, serviceToken, existingName) {
  const url = `${baseUrl}api/v1/retry_schedules`;
  const payload = JSON.stringify({
    organization_id: organizationId,
    strategy: 'linear',
    name: existingName,
    max_retries: 3,
    delay: 60,
  });
  const res = http.post(url, payload, headers(serviceToken));
  check(res, { 'Duplicate name rejected': (response) => response.status === 409 });
}

export function assignToSubscription(
  baseUrl,
  serviceToken,
  subscriptionId,
  subscriptionBody,
  scheduleId
) {
  const url = `${baseUrl}api/v1/subscriptions/${subscriptionId}`;
  const payload = JSON.stringify({
    ...subscriptionBody,
    retry_schedule_id: scheduleId,
  });
  const res = http.put(url, payload, headers(serviceToken));
  check(res, {
    'Assign retry schedule ok': (response) =>
      response.status === 200 && response.body.includes('retry_schedule_id'),
  });
  return res.status === 200 ? JSON.parse(res.body) : null;
}

export function rejectsCrossOrgAssign(baseUrl, serviceToken, subscriptionId, subscriptionBody) {
  const url = `${baseUrl}api/v1/subscriptions/${subscriptionId}`;
  // Fake uuid: a schedule id that can't exist in this org
  const payload = JSON.stringify({
    ...subscriptionBody,
    retry_schedule_id: '00000000-0000-0000-0000-000000000000',
  });
  const res = http.put(url, payload, headers(serviceToken));
  check(res, { 'Cross-org retry_schedule_id rejected': (response) => response.status === 404 });
}
