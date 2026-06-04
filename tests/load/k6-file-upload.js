import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 50 },
    { duration: '3m', target: 200 },
    { duration: '1m', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],    // ファイルアップロードは許容範囲広め
  },
};

const BASE_URL = 'http://localhost:8080/api/v1';

export function setup() {
  // ログインしてトークン取得
  const loginRes = http.post(`${BASE_URL}/auth/login`, JSON.stringify({
    username: 'testuser',
    password: 'testpassword',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  if (loginRes.status !== 200) {
    throw new Error('Failed to obtain auth token for setup');
  }

  return { token: loginRes.json('token') };
}

export default function (data) {
  const binFile = open('./fixtures/test-file.bin', 'b');

  const res = http.post(`${BASE_URL}/files/upload`, {
    file: http.file(binFile, 'test-file.bin'),
  }, {
    headers: { 'Authorization': `Bearer ${data.token}` },
  });

  check(res, {
    'upload status is 200': (r) => r.status === 200,
    'upload response time < 500ms': (r) => r.timings.duration < 500,
  });
}
