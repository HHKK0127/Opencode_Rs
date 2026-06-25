import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 50 },
    { duration: '3m', target: 50 },
    { duration: '1m', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500'],
    'http_req_failed': ['rate<0.1'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const JSON_HEADERS = { 'Content-Type': 'application/json' };

export default function () {
  // ログイン (JSON形式)
  const login_res = http.post(
    `${BASE_URL}/api/v1/auth/login`,
    JSON.stringify({ username: 'testuser', password: 'testpassword' }),
    { headers: JSON_HEADERS }
  );

  const loginOk = check(login_res, {
    'login status 200': (r) => r.status === 200,
  });

  if (!loginOk) { sleep(1); return; }

  const token = login_res.json('token');
  const authHeaders = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`,
  };

  // ファイルリスト繰り返し取得（キャッシング効果測定）
  for (let i = 0; i < 3; i++) {
    const list_res = http.get(`${BASE_URL}/api/v1/files?page=1&per_page=20`, {
      headers: authHeaders,
    });
    check(list_res, {
      'list status 200': (r) => r.status === 200,
    });
  }

  sleep(1);
}
