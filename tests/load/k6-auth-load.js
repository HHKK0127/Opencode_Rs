import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages: [
    { duration: '2m', target: 100 },   // ランプアップ
    { duration: '5m', target: 1000 },  // ピーク負荷
    { duration: '2m', target: 0 },      // ランプダウン
  ],
  thresholds: {
    http_req_duration: ['p(95)<100'],    // p95 < 100ms
    http_req_failed: ['rate<0.001'],   // エラー率 < 0.1%
    errors: ['rate<0.001'],
  },
};

const BASE_URL = 'http://localhost:8080/api/v1';

export default function () {
  // ヘルスチェック
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'health status is 200': (r) => r.status === 200,
    'health response time < 50ms': (r) => r.timings.duration < 50,
  }) || errorRate.add(1);

  // 認証フロー（10%のユーザー）
  if (Math.random() < 0.1) {
    const loginRes = http.post(`${BASE_URL}/auth/login`, JSON.stringify({
      username: `testuser_${__VU}`,
      password: 'testpassword',
    }), {
      headers: { 'Content-Type': 'application/json' },
    });

    check(loginRes, {
      'login status is 200': (r) => r.status === 200,
      'login response time < 200ms': (r) => r.timings.duration < 200,
      'token received': (r) => r.json('token') !== undefined,
    }) || errorRate.add(1);

    if (loginRes.status === 200) {
      const token = loginRes.json('token');

      // 保護エンドポイントアクセス
      const protectedRes = http.get(`${BASE_URL}/users`, {
        headers: { 'Authorization': `Bearer ${token}` },
      });

      check(protectedRes, {
        'protected endpoint status is 200': (r) => r.status === 200,
        'protected response time < 150ms': (r) => r.timings.duration < 150,
      }) || errorRate.add(1);
    }
  }

  sleep(1);
}
