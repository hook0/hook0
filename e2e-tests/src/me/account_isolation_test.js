import http from 'k6/http';
import { check } from 'k6';

/**
 * Black-box test: Verify that a user cannot delete or affect another user's account.
 *
 * This test validates account isolation by:
 * 1. Using two different user accounts
 * 2. Having User A request account deletion
 * 3. Verifying User B's account is completely unaffected
 *
 * The test expects:
 * - TEST_USER_EMAIL / TEST_USER_PASSWORD (User A)
 * - TEST_USER_EMAIL_2 / TEST_USER_PASSWORD_2 (User B)
 */

function register(baseUrl, email, password, firstName = 'E2E', lastName = 'Test') {
  const url = `${baseUrl}api/v1/registrations`;

  const payload = JSON.stringify({
    email: email,
    password: password,
    first_name: firstName,
    last_name: lastName,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, payload, params);
  // 201 = created, 409 = already exists (both are OK for our tests)
  if (res.status !== 201 && res.status !== 409) {
    console.warn('register response:', res.status, res.body);
    return false;
  }

  return true;
}

function login(baseUrl, email, password) {
  const url = `${baseUrl}api/v1/auth/login`;

  const payload = JSON.stringify({
    email: email,
    password: password,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, payload, params);
  if (
    !check(res, {
      'Login succeeded': (r) => r.status === 201 && r.body && r.body.includes('access_token'),
    })
  ) {
    console.warn('login response:', res.status, res.body);
    return null;
  }

  return JSON.parse(res.body);
}

function getDeletionStatus(baseUrl, accessToken) {
  const url = `${baseUrl}api/v1/account/deletion-status`;

  const params = {
    headers: {
      Authorization: `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.get(url, params);
  if (
    !check(res, {
      'Get deletion status succeeded': (r) =>
        r.status === 200 && r.body && r.body.includes('deletion_requested'),
    })
  ) {
    console.warn('get_deletion_status response:', res.status, res.body);
    return null;
  }

  return JSON.parse(res.body);
}

function requestDeletion(baseUrl, accessToken) {
  const url = `${baseUrl}api/v1/account`;

  const params = {
    headers: {
      Authorization: `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.del(url, null, params);
  if (
    !check(res, {
      'Request deletion succeeded': (r) => r.status === 204,
    })
  ) {
    console.warn('request_deletion response:', res.status, res.body);
    return false;
  }

  return true;
}

function cancelDeletion(baseUrl, accessToken) {
  const url = `${baseUrl}api/v1/account/cancel-deletion`;

  const params = {
    headers: {
      Authorization: `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, null, params);
  if (
    !check(res, {
      'Cancel deletion succeeded': (r) => r.status === 204,
    })
  ) {
    console.warn('cancel_deletion response:', res.status, res.body);
    return false;
  }

  return true;
}

export default function (config) {
  const h = config.apiOrigin;

  try {
    // 0. Register both test users (if not already existing)
    console.log('Step 0: Registering test users...');
    if (!register(h, config.testUserEmail, config.testUserPassword, 'E2E-A', 'TestUser')) {
      throw new Error('Failed to register User A');
    }
    if (!register(h, config.testUserEmail2, config.testUserPassword2, 'E2E-B', 'TestUser')) {
      throw new Error('Failed to register User B');
    }

    // 1. Login as both users
    console.log('Step 1: Logging in as User A...');
    const loginA = login(h, config.testUserEmail, config.testUserPassword);
    if (!loginA || !loginA.access_token) {
      throw new Error('Failed to login as User A');
    }
    const tokenA = loginA.access_token;

    console.log('Step 2: Logging in as User B...');
    const loginB = login(h, config.testUserEmail2, config.testUserPassword2);
    if (!loginB || !loginB.access_token) {
      throw new Error('Failed to login as User B');
    }
    const tokenB = loginB.access_token;

    // 2. Verify both users have no pending deletion
    console.log('Step 3: Verifying both users have no pending deletion...');
    const statusA_initial = getDeletionStatus(h, tokenA);
    if (statusA_initial === null) {
      throw new Error('Failed to get initial deletion status for User A');
    }
    if (statusA_initial.deletion_requested !== false) {
      throw new Error('User A should not have pending deletion initially');
    }

    const statusB_initial = getDeletionStatus(h, tokenB);
    if (statusB_initial === null) {
      throw new Error('Failed to get initial deletion status for User B');
    }
    if (statusB_initial.deletion_requested !== false) {
      throw new Error('User B should not have pending deletion initially');
    }

    // 3. User A requests deletion
    console.log('Step 4: User A requests account deletion...');
    const deleteResultA = requestDeletion(h, tokenA);
    if (!deleteResultA) {
      throw new Error('Failed to request deletion for User A');
    }

    // 4. Verify User A has pending deletion
    console.log('Step 5: Verifying User A has pending deletion...');
    const statusA_after = getDeletionStatus(h, tokenA);
    if (statusA_after === null) {
      throw new Error('Failed to get deletion status for User A after request');
    }
    if (statusA_after.deletion_requested !== true) {
      throw new Error('User A should have pending deletion after request');
    }

    // 5. CRITICAL: Verify User B is NOT affected
    console.log('Step 6: CRITICAL - Verifying User B is NOT affected...');
    const statusB_after = getDeletionStatus(h, tokenB);
    if (statusB_after === null) {
      throw new Error('Failed to get deletion status for User B after User A deletion request');
    }
    if (statusB_after.deletion_requested !== false) {
      throw new Error(
        'SECURITY VIOLATION: User B was affected by User A deletion request! Account isolation is broken!'
      );
    }

    // 6. Cleanup: Cancel User A's deletion
    console.log('Step 7: Cleanup - cancelling User A deletion...');
    const cancelResult = cancelDeletion(h, tokenA);
    if (!cancelResult) {
      throw new Error('Failed to cancel deletion for User A');
    }

    // 7. Final verification
    console.log('Step 8: Final verification...');
    const statusA_final = getDeletionStatus(h, tokenA);
    if (statusA_final === null || statusA_final.deletion_requested !== false) {
      throw new Error('Failed to verify User A deletion was cancelled');
    }

    const statusB_final = getDeletionStatus(h, tokenB);
    if (statusB_final === null || statusB_final.deletion_requested !== false) {
      throw new Error('User B state is incorrect at end of test');
    }

    console.log('✓ Account isolation test passed: User accounts are properly isolated');
  } catch (error) {
    console.error('Account isolation test FAILED:', error.message);
    throw error;
  }
}
