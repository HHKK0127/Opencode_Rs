// Authentication Helper for Wave 4 Load Tests
// Handles login, token management, and session creation

import http from 'k6/http';
import { check } from 'k6';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

/**
 * Login and get JWT token
 * @returns {string} JWT token
 */
export function login(username = 'testuser', password = 'testpassword') {
  const payload = JSON.stringify({
    username: username,
    password: password,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(`${BASE_URL}/api/v1/auth/login`, payload, params);

  check(res, {
    'login status is 200': (r) => r.status === 200,
    'login response has token': (r) => r.json('token') !== null,
  });

  if (res.status === 200) {
    return res.json('token');
  }

  throw new Error(`Login failed: ${res.status}`);
}

/**
 * Get authenticated request headers
 * @param {string} token JWT token
 * @returns {Object} Headers object
 */
export function getAuthHeaders(token) {
  return {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  };
}

/**
 * Refresh JWT token
 * @param {string} token Current JWT token
 * @returns {string} New JWT token
 */
export function refreshToken(token) {
  const params = {
    headers: getAuthHeaders(token),
  };

  const res = http.post(`${BASE_URL}/api/v1/auth/refresh`, null, params);

  check(res, {
    'refresh status is 200': (r) => r.status === 200,
    'refresh response has token': (r) => r.json('token') !== null,
  });

  if (res.status === 200) {
    return res.json('token');
  }

  throw new Error(`Token refresh failed: ${res.status}`);
}

/**
 * Logout (invalidate session)
 * @param {string} token JWT token
 * @returns {boolean} Success
 */
export function logout(token) {
  const params = {
    headers: getAuthHeaders(token),
  };

  const res = http.post(`${BASE_URL}/api/v1/auth/logout`, null, params);

  check(res, {
    'logout status is 200': (r) => r.status === 200,
  });

  return res.status === 200;
}

/**
 * Validate session
 * @param {string} token JWT token
 * @returns {Object} Session data
 */
export function validateSession(token) {
  const params = {
    headers: getAuthHeaders(token),
  };

  const res = http.post(`${BASE_URL}/api/v1/sessions/validate`, null, params);

  check(res, {
    'validate session status is 200': (r) => r.status === 200,
    'validate session response has user_id': (r) => r.json('user_id') !== null,
  });

  if (res.status === 200) {
    return res.json();
  }

  throw new Error(`Session validation failed: ${res.status}`);
}

/**
 * Get session info
 * @param {string} token JWT token
 * @returns {Object} Session info
 */
export function getSessionInfo(token) {
  const params = {
    headers: getAuthHeaders(token),
  };

  const res = http.get(`${BASE_URL}/api/v1/sessions/info`, params);

  check(res, {
    'get session info status is 200': (r) => r.status === 200,
    'session info has remaining_ttl': (r) => r.json('remaining_ttl_seconds') !== null,
  });

  if (res.status === 200) {
    return res.json();
  }

  throw new Error(`Get session info failed: ${res.status}`);
}
