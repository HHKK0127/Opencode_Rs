import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '5m', target: 100 },
    { duration: '15m', target: 100 },
    { duration: '5m', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500'],
  },
};

export default function () {
  const base_url = __ENV.BASE_URL || 'http://localhost:8080';
  
  const login_res = http.post(`${base_url}/api/v1/auth/login`, {
    username: 'testuser',
    password: 'testpassword',
  });
  
  const token = login_res.json('token');
  
  // ファイル検索（キャッシング対象）
  const search_res = http.get(`${base_url}/api/v1/files/search?q=test`, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  
  check(search_res, {
    'search status': (r) => r.status === 200,
  });
  
  sleep(1);
}
