import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

/**
 * @description Updates a retry schedule: name change, strategy change, invalid cross-fields, duplicate name.
 * Returns the updated schedule object on success, or null on failure.
 * @example
 *   const updated = update_retry_schedule(baseUrl, serviceToken, organizationId, scheduleId)
 *   // updated => { retry_schedule_id, name, strategy, ... }
 */
export default function (baseUrl, service_token, organization_id, schedule_id) {
  const url = `${baseUrl}api/v1/retry_schedules/${schedule_id}?organization_id=${organization_id}`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  // --- Fetch current state ---
  const current_res = http.get(url, params);
  if (current_res.status !== 200) {
    console.warn('Failed to fetch schedule before update:', current_res.status, current_res.body);
    return null;
  }
  const current = JSON.parse(current_res.body);

  // --- Update name ---
  const new_name = 'k6_updated_' + uuidv4();
  let res = http.put(
    url,
    JSON.stringify({
      name: new_name,
      strategy: current.strategy,
      max_retries: current.max_retries,
      custom_intervals: current.custom_intervals,
      linear_delay: current.linear_delay,
      increasing_base_delay: current.increasing_base_delay,
      increasing_wait_factor: current.increasing_wait_factor,
    }),
    params
  );
  if (
    !check(res, {
      'Update name returns 200': (r) => r.status === 200,
      'Update name changed': (r) => {
        const body = JSON.parse(r.body);
        return body.name === new_name;
      },
      'Update updated_at changed': (r) => {
        const body = JSON.parse(r.body);
        return body.updated_at !== current.updated_at;
      },
    })
  ) {
    console.warn('Update name failed:', res.status, res.body);
    return null;
  }

  // --- Change strategy increasing -> linear ---
  res = http.put(
    url,
    JSON.stringify({
      name: new_name,
      strategy: 'linear',
      max_retries: 3,
      linear_delay: 120,
    }),
    params
  );
  if (
    !check(res, {
      'Change to linear returns 200': (r) => r.status === 200,
      'Strategy changed to linear': (r) => {
        const body = JSON.parse(r.body);
        return body.strategy === 'linear' && body.linear_delay === 120;
      },
    })
  ) {
    console.warn('Change strategy failed:', res.status, res.body);
    return null;
  }

  // --- Invalid cross-fields: linear with custom_intervals ---
  res = http.put(
    url,
    JSON.stringify({
      name: new_name,
      strategy: 'linear',
      max_retries: 3,
      linear_delay: 120,
      custom_intervals: [10, 20, 30],
    }),
    params
  );
  check(res, {
    'Invalid cross-fields returns 400 or 422': (r) => r.status === 400 || r.status === 422,
  });

  // --- Update to duplicate name (create a temporary schedule first) ---
  const dup_name = 'k6_dup_target_' + uuidv4();
  const create_url = `${baseUrl}api/v1/retry_schedules/`;
  const dup_res = http.post(
    create_url,
    JSON.stringify({
      organization_id,
      name: dup_name,
      strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
      max_retries: 3,
    }),
    params
  );
  if (dup_res.status === 201) {
    res = http.put(
      url,
      JSON.stringify({
        name: dup_name,
        strategy: 'linear',
        max_retries: 3,
        linear_delay: 120,
      }),
      params
    );
    check(res, {
      'Update to duplicate name returns 409 or 422': (r) => r.status === 409 || r.status === 422,
    });

    // Clean up temporary schedule
    const dup_schedule_id = JSON.parse(dup_res.body).retry_schedule_id;
    http.del(
      `${baseUrl}api/v1/retry_schedules/${dup_schedule_id}?organization_id=${organization_id}`,
      null,
      params
    );
  }

  return JSON.parse(http.get(url, params).body);
}
