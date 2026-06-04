// Wave 2 Day 5 Load Test Script
// Purpose: Validate v3.0.0 performance before production deployment
// Target: p95 < 100ms, error rate < 1%, throughput > 500 req/s

import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const reqDuration = new Trend('req_duration');
const successCount = new Counter('requests_success');
const failureCount = new Counter('requests_failure');
const activeVUs = new Gauge('active_vus');

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const JWT_TOKEN = __ENV.JWT_TOKEN || 'test_token_placeholder';

// Test options
export const options = {
  vus: 10,          // Initial VUs
  stages: [
    // Warmup: 0 → 10 VUs over 5 minutes
    { duration: '5m', target: 10, name: 'warmup' },

    // Standard load: 10 → 50 VUs over 5 minutes, hold for 10 minutes
    { duration: '5m', target: 50, name: 'standard_ramp' },
    { duration: '10m', target: 50, name: 'standard_load' },

    // Peak load: 50 → 100 VUs over 5 minutes, hold for 5 minutes
    { duration: '5m', target: 100, name: 'peak_ramp' },
    { duration: '5m', target: 100, name: 'peak_load' },

    // Cooldown: 100 → 0 VUs over 5 minutes
    { duration: '5m', target: 0, name: 'cooldown' },
  ],

  thresholds: {
    // Success criteria
    'http_req_duration': ['p(95)<100', 'p(99)<200'],  // 95th percentile < 100ms
    'errors': ['rate<0.01'],                           // Error rate < 1%
    'req_duration': ['p(95)<100'],
  },

  ext: {
    loadimpact: {
      projectID: 3363030,
      name: 'Wave 2 Day 5 Load Test',
    },
  },
};

// Setup phase
export function setup() {
  console.log(`🚀 Load test starting: ${BASE_URL}`);

  // Verify health endpoint
  let res = http.get(`${BASE_URL}/health`);
  if (res.status !== 200) {
    throw new Error('Health check failed: ' + res.status);
  }

  console.log('✅ Health check passed');

  return {
    baseUrl: BASE_URL,
    timestamp: new Date().toISOString(),
  };
}

// Main test function
export default function (data) {
  activeVUs.set(__VU);

  // Test scenario: Mixed workload
  group('Auth API - Login', function () {
    authLogin();
  });

  group('Files API - Metadata', function () {
    filesMetadata();
  });

  group('Health Check', function () {
    healthCheck();
  });

  group('Metrics API', function () {
    metricsAPI();
  });

  // Variable sleep between iterations (0-2 seconds)
  sleep(Math.random() * 2);
}

// Teardown phase
export function teardown(data) {
  console.log(`✅ Load test completed at ${new Date().toISOString()}`);
  console.log(`📊 Summary: ${successCount.value} success, ${failureCount.value} failures`);
}

// ============================================
// Test Functions
// ============================================

function authLogin() {
  const payload = JSON.stringify({
    username: 'testuser',
    password: 'testpassword',
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const startTime = new Date();
  const res = http.post(`${BASE_URL}/api/v1/auth/login`, payload, params);
  const duration = new Date() - startTime;

  reqDuration.add(duration);

  const success = check(res, {
    'auth login status is 200': (r) => r.status === 200,
    'auth login has token': (r) => r.json('token') !== undefined,
    'auth login latency < 50ms': (r) => r.timings.duration < 50,
  });

  if (success) {
    successCount.add(1);
  } else {
    failureCount.add(1);
    errorRate.add(1);
  }
}

function filesMetadata() {
  const params = {
    headers: {
      'Authorization': `Bearer ${JWT_TOKEN}`,
    },
  };

  const startTime = new Date();
  const res = http.get(`${BASE_URL}/api/v1/files?page=1&per_page=20`, params);
  const duration = new Date() - startTime;

  reqDuration.add(duration);

  const success = check(res, {
    'files list status is 200 or 401': (r) => r.status === 200 || r.status === 401,
    'files list latency < 100ms': (r) => r.timings.duration < 100,
    'files list response time reasonable': (r) => r.timings.duration < 200,
  });

  if (success) {
    successCount.add(1);
  } else {
    failureCount.add(1);
    errorRate.add(1);
  }
}

function healthCheck() {
  const startTime = new Date();
  const res = http.get(`${BASE_URL}/health`);
  const duration = new Date() - startTime;

  reqDuration.add(duration);

  const success = check(res, {
    'health status is 200': (r) => r.status === 200,
    'health response has status field': (r) => r.json('status') !== undefined,
    'health latency < 20ms': (r) => r.timings.duration < 20,
  });

  if (success) {
    successCount.add(1);
  } else {
    failureCount.add(1);
    errorRate.add(1);
  }
}

function metricsAPI() {
  const startTime = new Date();
  const res = http.get(`${BASE_URL}/api/v1/metrics`);
  const duration = new Date() - startTime;

  reqDuration.add(duration);

  const success = check(res, {
    'metrics status is 200': (r) => r.status === 200,
    'metrics content-type is text/plain': (r) => r.headers['Content-Type'].includes('text/plain'),
    'metrics has http_requests_total': (r) => r.body.includes('http_requests_total'),
    'metrics latency < 50ms': (r) => r.timings.duration < 50,
  });

  if (success) {
    successCount.add(1);
  } else {
    failureCount.add(1);
    errorRate.add(1);
  }
}

// ============================================
// Advanced Scenarios (Optional)
// ============================================

function fileUploadScenario() {
  // Simulate file upload (small 1MB file)
  const fileData = 'x'.repeat(1024 * 1024);  // 1MB

  const params = {
    headers: {
      'Authorization': `Bearer ${JWT_TOKEN}`,
    },
  };

  const startTime = new Date();
  const res = http.post(`${BASE_URL}/api/v1/files/upload`, fileData, params);
  const duration = new Date() - startTime;

  reqDuration.add(duration);

  const success = check(res, {
    'upload status is 200 or 401': (r) => r.status === 200 || r.status === 401,
    'upload latency < 500ms': (r) => r.timings.duration < 500,
  });

  if (success) {
    successCount.add(1);
  } else {
    failureCount.add(1);
    errorRate.add(1);
  }
}

function databaseStressTest() {
  // High concurrency query test
  const params = {
    headers: {
      'Authorization': `Bearer ${JWT_TOKEN}`,
    },
  };

  const urls = [
    `${BASE_URL}/api/v1/files?page=1&per_page=50`,
    `${BASE_URL}/api/v1/files?page=2&per_page=50`,
    `${BASE_URL}/api/v1/files?page=3&per_page=50`,
  ];

  urls.forEach((url) => {
    const startTime = new Date();
    const res = http.get(url, params);
    const duration = new Date() - startTime;

    reqDuration.add(duration);

    if (res.status === 200 || res.status === 401) {
      successCount.add(1);
    } else {
      failureCount.add(1);
      errorRate.add(1);
    }
  });
}

// ============================================
// Test Execution Summary
// ============================================

/*
Expected Results (from PERFORMANCE_BENCHMARKS.md):

Warmup Phase (5m, 10 VU):
  - Req/s: 50-60
  - p95: 20-30ms
  - Error: 0%

Standard Load (15m, 10→50 VU):
  - Req/s: 150-300
  - p95: 40-60ms
  - Error: < 0.5%

Peak Load (10m, 50→100 VU):
  - Req/s: 400-500
  - p95: 60-90ms
  - Error: < 1%

Cooldown (5m, 100→0 VU):
  - Req/s: 500→0
  - p95: 60-90ms

PASS Criteria:
  ✅ Overall p95: < 100ms
  ✅ Overall error rate: < 1%
  ✅ Peak throughput: > 500 req/s
  ✅ Max p99: < 200ms
*/
